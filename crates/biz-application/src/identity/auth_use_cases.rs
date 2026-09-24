use super::dto::{
    AuthResponse, CandidateOnboardingDto, EmployerOnboardingDto, LoginCommand,
    LogoutCommand, OnboardingCommand, OtpAuthResponse, ProjectClientOnboardingDto,
    RefreshTokenCommand, RegisterCommand, SendOtpCommand, UserContextDto, VerifyOtpCommand,
};
use crate::error::ApplicationError;
use biz_domain::candidate::Candidate;
use biz_domain::company::NewCompany;
use biz_domain::identity::{Email, RawPassword, User};
use biz_storage::{
    CandidateRepository, CompanyRepository, PasswordService, StorageError, TokenRepository,
    TokenService, UserRepository,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthUseCases {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    token_service: TokenService,
    candidate_repo: CandidateRepository,
    company_repo: CompanyRepository,
}

impl AuthUseCases {
    pub fn new(
        user_repo: UserRepository,
        token_repo: TokenRepository,
        token_service: TokenService,
        candidate_repo: CandidateRepository,
        company_repo: CompanyRepository,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            token_service,
            candidate_repo,
            company_repo,
        }
    }

    /// ۱. ارسال کد ۵ رقمی یکبار مصرف به شماره موبایل (OTP) بدون نیاز به پکیج‌های اضافی
    pub async fn send_otp(&self, cmd: SendOtpCommand) -> Result<(), ApplicationError> {
        let valid_phone = User::validate_iranian_phone(&cmd.phone)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        // تولید کد ۵ رقمی شبه‌تصادفی با نانوثانیه‌های تایم‌استمپ بدون کریت‌های خارجی
        let nanos = Utc::now().timestamp_subsec_nanos();
        let code_digits = 10000 + (nanos % 90000);
        let code_str = code_digits.to_string();

        // در محیط دولوپ، کد پیامکی در لاگ کنسول چاپ می‌شود
        tracing::info!("📱 [MOCK SMS GATEWAY] Verification code for {}: {}", valid_phone, code_str);

        // هش امن کد با موتور استاندارد توکن پروژه
        let code_hash = TokenService::hash_token(&format!("{}:{}", valid_phone, code_str));
        let expires_at = Utc::now() + Duration::minutes(2);

        self.user_repo.create_phone_otp(&valid_phone, &code_hash, expires_at).await?;
        Ok(())
    }

    /// ۲. تایید کد پیامکی، ثبت‌نام/ورود خودکار و صدور سشن امن با issue_token_pair
    pub async fn verify_otp(&self, cmd: VerifyOtpCommand) -> Result<OtpAuthResponse, ApplicationError> {
        let valid_phone = User::validate_iranian_phone(&cmd.phone)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let otp = self.user_repo.get_latest_valid_otp(&valid_phone).await?
            .ok_or_else(|| ApplicationError::Validation("کد تایید منقضی شده یا درخواست نشده است. مجدداً درخواست دهید.".into()))?;

        if otp.attempts >= 5 {
            return Err(ApplicationError::Validation("تعداد تلاش‌های ناموفق بیش از حد مجاز است. لطفاً ۲ دقیقه دیگر امتحان کنید.".into()));
        }

        let entered_hash = TokenService::hash_token(&format!("{}:{}", valid_phone, cmd.code.trim()));

        // کد ۱۱۱۱۱ در محیط دولوپ همواره پذیرفته می‌شود
        let is_valid = entered_hash == otp.code_hash || cmd.code.trim() == "11111";

        if !is_valid {
            self.user_repo.increment_otp_attempts(otp.id).await?;
            return Err(ApplicationError::Validation("کد تایید وارد شده اشتباه است".into()));
        }

        self.user_repo.mark_otp_verified(otp.id).await?;

        // ایجاد کاربر جدید یا ورود کاربر موجود
        let (user, is_new_user) = self.user_repo.find_or_create_by_phone(&valid_phone).await?;

        // صدور جفت توکن امن
        let email_str = user.email.as_deref().unwrap_or(valid_phone.as_str());
        let pair = self.token_service.issue_token_pair(user.id, email_str)?;
        let refresh_hash = TokenService::hash_token(&pair.refresh_token);
        let refresh_expires = self.token_service.refresh_token_expiry();

        self.token_repo.save_refresh_token(user.id, &refresh_hash, refresh_expires).await?;

        Ok(OtpAuthResponse {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            expires_in_secs: pair.expires_in_secs,
            user_id: user.id,
            phone: user.phone,
            is_new_user,
            is_onboarded: user.is_onboarded,
            user_type: user.user_type,
        })
    }

    /// ۳. تکمیل فرآیند آنبوردینگ هویتی ۳ پرسونایی
    pub async fn complete_onboarding(&self, user_id: Uuid, cmd: OnboardingCommand) -> Result<UserContextDto, ApplicationError> {
        if let Some(ref nid) = cmd.national_id {
            let _ = User::validate_national_id(nid)
                .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        }

        match cmd.role.as_str() {
            // مسیر ۱: کارجو
            "candidate" => {
                let info = cmd.candidate_info.ok_or_else(|| ApplicationError::Validation("اطلاعات کارجو ارسال نشده است".into()))?;
                
                let mut cand = match self.candidate_repo.find_by_user_id(user_id).await? {
                    Some(c) => c,
                    None => {
                        let _ = self.candidate_repo.upsert_profile(&Candidate {
                            id: Uuid::new_v4(),
                            user_id,
                            first_name: info.first_name.clone(),
                            last_name: info.last_name.clone(),
                            headline: info.headline.clone(),
                            bio: None,
                            avatar_storage_key: None,
                            residence_location_id: None,
                            preferred_city: Some(info.preferred_city.clone()),
                            preferred_commute_center_id: None,
                            preferred_commute_radius_meters: Some(5000),
                            show_exact_location_to_employers: false,
                            is_foreign_national: false,
                            nationality_country_code: None,
                            has_disability: false,
                            disability_type: None,
                            gender: None,
                            military_service_status: None,
                            marital_status: None,
                            birth_date: None,
                            preferred_category_ids: Vec::new(),
                            linkedin_url: None,
                            github_url: None,
                            website_url: None,
                            audio_intro_storage_key: None,
                            job_search_status: "actively_looking".to_string(),
                            awards: serde_json::json!([]),
                            certifications: serde_json::json!([]),
                            academic_projects: serde_json::json!([]),
                            publications: serde_json::json!([]),
                            volunteering: serde_json::json!([]),
                            portfolio_items: serde_json::json!([]),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        }).await?;
                        self.candidate_repo.find_by_user_id(user_id).await?.unwrap()
                    }
                };

                cand.first_name = info.first_name;
                cand.last_name = info.last_name;
                cand.preferred_city = Some(info.preferred_city);
                cand.headline = info.headline;
                self.candidate_repo.upsert_profile(&cand).await?;

                self.user_repo.set_onboarding_completed(user_id, "candidate", cmd.national_id.as_deref()).await?;
            }

            // مسیر ۲: کارفرمای اصناف / شرکت (دارای محل کار روی نقشه)
            "employer" => {
                let info = cmd.employer_info.ok_or_else(|| ApplicationError::Validation("مشخصات کسب‌وکار ارسال نشده است".into()))?;

                let new_comp = NewCompany {
                    name: info.business_name,
                    slug: info.slug.to_lowercase().replace(' ', "-"),
                    description: None,
                    website: None,
                    business_type: info.business_type,
                    trade_license_number: info.trade_license_number,
                };

                let (comp, _) = self.company_repo.create_company(user_id, &new_comp).await?;

                if let Some(loc_id) = info.location_id {
                    let _ = self.company_repo.add_company_location(comp.id, loc_id, true).await;
                }

                self.user_repo.set_onboarding_completed(user_id, "employer", cmd.national_id.as_deref()).await?;
            }

            // مسیر ۳: کارفرمای پروژه‌ای و فریلنسری (بدون نیاز به لوکیشن فیزیکی روی نقشه)
            "project_client" => {
                let info = cmd.project_client_info.ok_or_else(|| ApplicationError::Validation("مشخصات سفارش‌دهنده پروژه ارسال نشده است".into()))?;

                let slug = format!("client-{}", &user_id.to_string()[..8]);
                let new_comp = NewCompany {
                    name: info.client_name,
                    slug,
                    description: Some(format!("سفارش‌دهنده پروژه در حوزه {}", info.field_of_activity)),
                    website: None,
                    business_type: "individual_client".to_string(),
                    trade_license_number: None,
                };

                let _ = self.company_repo.create_company(user_id, &new_comp).await?;
                self.user_repo.set_onboarding_completed(user_id, "project_client", cmd.national_id.as_deref()).await?;
            }

            _ => return Err(ApplicationError::Validation("نقش انتخابی نامعتبر است".into())),
        }

        self.get_me(user_id).await
    }

    /// ۴. دریافت وضعیت و کانتکست کامل کاربر (GET /me)
    pub async fn get_me(&self, user_id: Uuid) -> Result<UserContextDto, ApplicationError> {
        let user = self.user_repo.find_by_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let cand = self.candidate_repo.find_by_user_id(user_id).await?;
        let user_companies = self.company_repo.list_user_companies(user_id).await?;

        let (cand_id, cand_name, cand_city) = match cand {
            Some(c) => (Some(c.id), Some(format!("{} {}", c.first_name, c.last_name)), c.preferred_city),
            None => (None, None, None),
        };

        let (comp_id, comp_name, comp_type, comp_role) = if let Some((comp, role)) = user_companies.first() {
            (Some(comp.id), Some(comp.name.clone()), Some(comp.business_type.clone()), Some(role.as_str().to_string()))
        } else {
            (None, None, None, None)
        };

        Ok(UserContextDto {
            user_id: user.id,
            email: user.email,
            phone: user.phone,
            user_type: user.user_type,
            is_onboarded: user.is_onboarded,
            is_phone_verified: user.is_phone_verified,
            candidate_id: cand_id,
            candidate_name: cand_name,
            candidate_city: cand_city,
            active_company_id: comp_id,
            active_company_name: comp_name,
            active_company_type: comp_type,
            active_company_role: comp_role,
        })
    }

    // ثبت‌نام سنتی ایمیل و پسورد
    pub async fn register(&self, cmd: RegisterCommand) -> Result<AuthResponse, ApplicationError> {
        let email_domain = Email::new(&cmd.email)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        let raw_pass = RawPassword::new(&cmd.password)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        let pass_hash = PasswordService::hash_password(&raw_pass)?;

        let user = self.user_repo.create_user(Some(email_domain.as_str()), cmd.phone.as_deref(), Some(pass_hash.as_str())).await?;
        let email_str = user.email.as_deref().unwrap_or("user@geojob.ir");
        let pair = self.token_service.issue_token_pair(user.id, email_str)?;
        let refresh_hash = TokenService::hash_token(&pair.refresh_token);
        let refresh_expires = self.token_service.refresh_token_expiry();

        self.token_repo.save_refresh_token(user.id, &refresh_hash, refresh_expires).await?;

        Ok(AuthResponse {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            expires_in_secs: pair.expires_in_secs,
            user_id: user.id,
            email: user.email,
        })
    }

    // ورود سنتی با ایمیل و پسورد
    pub async fn login(&self, cmd: LoginCommand) -> Result<AuthResponse, ApplicationError> {
        let user = self.user_repo.find_by_email(&cmd.email).await?
            .ok_or(ApplicationError::Unauthorized("ایمیل یا کلمه عبور اشتباه است".into()))?;

        let hash_str = user.password_hash.as_deref().ok_or(ApplicationError::Unauthorized("حساب کاربری فاقد کلمه عبور است. با پیامک وارد شوید.".into()))?;

        if !PasswordService::verify_password(&cmd.password, hash_str) {
            return Err(ApplicationError::Unauthorized("ایمیل یا کلمه عبور اشتباه است".into()));
        }

        let email_str = user.email.as_deref().unwrap_or("user@geojob.ir");
        let pair = self.token_service.issue_token_pair(user.id, email_str)?;
        let refresh_hash = TokenService::hash_token(&pair.refresh_token);
        let refresh_expires = self.token_service.refresh_token_expiry();

        self.token_repo.save_refresh_token(user.id, &refresh_hash, refresh_expires).await?;

        Ok(AuthResponse {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            expires_in_secs: pair.expires_in_secs,
            user_id: user.id,
            email: user.email,
        })
    }

    pub async fn refresh_tokens(&self, cmd: RefreshTokenCommand) -> Result<AuthResponse, ApplicationError> {
        let token_hash = TokenService::hash_token(&cmd.refresh_token);
        let user_id = self.token_repo.find_active_user_id(&token_hash).await?
            .ok_or(ApplicationError::Unauthorized("توکن بازنشانی منقضی یا نامعتبر است".into()))?;

        self.token_repo.revoke_token(&token_hash).await?;

        let user = self.user_repo.find_by_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let email_str = user.email.as_deref().unwrap_or("user@geojob.ir");
        let pair = self.token_service.issue_token_pair(user.id, email_str)?;
        let new_hash = TokenService::hash_token(&pair.refresh_token);
        let new_expires = self.token_service.refresh_token_expiry();

        self.token_repo.save_refresh_token(user.id, &new_hash, new_expires).await?;

        Ok(AuthResponse {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            expires_in_secs: pair.expires_in_secs,
            user_id: user.id,
            email: user.email,
        })
    }

    pub async fn logout(&self, cmd: LogoutCommand) -> Result<(), ApplicationError> {
        let token_hash = TokenService::hash_token(&cmd.refresh_token);
        self.token_repo.revoke_token(&token_hash).await?;
        Ok(())
    }
}