use super::dto::{
    AddEducationCommand, AddExperienceCommand, AddLanguageCommand, AddReferenceCommand,
    AddResumeCommand, CandidatePreferencesDto, CandidateProfileDto, EducationDto, ExperienceDto,
    LanguageDto, ReferenceDto, ResumeDto, SearchTalentsRequest, SendInvitationCommand,
    SetSkillsCommand, SkillDto, TrackedApplicationDto, UpdateProfileCommand,
};
use crate::error::ApplicationError;
use biz_domain::candidate::{
    Candidate, CandidateEducation, CandidateExperience, CandidateLanguage, CandidateReference,
    CandidateResume, TalentSearchResult,
};
use biz_storage::{ApplicationRepository, CandidateRepository, CompanyRepository, OpportunityRepository, StorageError};
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CandidateUseCases {
    candidate_repo: CandidateRepository,
    app_repo: ApplicationRepository,
    company_repo: Option<CompanyRepository>,
    opp_repo: Option<OpportunityRepository>,
}

impl CandidateUseCases {
    pub fn new(candidate_repo: CandidateRepository, app_repo: ApplicationRepository) -> Self {
        Self {
            candidate_repo,
            app_repo,
            company_repo: None,
            opp_repo: None,
        }
    }

    pub fn with_employer_repos(
        mut self,
        company_repo: CompanyRepository,
        opp_repo: OpportunityRepository,
    ) -> Self {
        self.company_repo = Some(company_repo);
        self.opp_repo = Some(opp_repo);
        self
    }

    /// هلپر اختصاصی جهت تضمین وجود پروفایل کارجو و جلوگیری ریشه‌ای از خطای UserNotFound در درخواست‌های همزمان
    async fn ensure_candidate(&self, user_id: Uuid) -> Result<Candidate, ApplicationError> {
        if let Some(c) = self.candidate_repo.find_by_user_id(user_id).await? {
            return Ok(c);
        }

        let new_cand = Candidate {
            id: Uuid::new_v4(),
            user_id,
            first_name: "کاربر".to_string(),
            last_name: "جدید".to_string(),
            headline: None,
            bio: None,
            avatar_storage_key: None,
            residence_location_id: None,
            preferred_city: None,
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
        };

        Ok(self.candidate_repo.upsert_profile(&new_cand).await?)
    }

    pub async fn get_full_profile(&self, user_id: Uuid) -> Result<CandidateProfileDto, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;

        let skills_with_names = self.candidate_repo.get_skills_with_names(candidate.id).await?;
        let exps = self.candidate_repo.list_experiences(candidate.id).await?;
        let edus = self.candidate_repo.list_educations(candidate.id).await?;
        let langs = self.candidate_repo.list_languages(candidate.id).await?;
        let refs = self.candidate_repo.list_references(candidate.id).await?;
        let resumes = self.candidate_repo.list_resumes(candidate.id).await?;

        let experiences = exps
            .into_iter()
            .map(|e| ExperienceDto {
                id: e.id,
                title: e.title,
                company_name: e.company_name,
                activity_field: e.activity_field,
                seniority_level: e.seniority_level,
                company_industry: e.company_industry,
                country: e.country,
                city: e.city,
                start_month: e.start_month,
                start_year: e.start_year,
                end_month: e.end_month,
                end_year: e.end_year,
                is_current: e.is_current,
                achievements: e.achievements,
            })
            .collect();

        let educations = edus
            .into_iter()
            .map(|e| EducationDto {
                id: e.id,
                institution: e.institution,
                degree_level: e.degree_level,
                field_of_study: e.field_of_study,
                gpa: e.gpa,
                start_year: e.start_year,
                end_year: e.end_year,
                is_current: e.is_current,
            })
            .collect();

        let languages = langs
            .into_iter()
            .map(|l| LanguageDto {
                id: l.id,
                language_name: l.language_name,
                proficiency_level: l.proficiency_level,
            })
            .collect();

        let references = refs
            .into_iter()
            .map(|r| ReferenceDto {
                id: r.id,
                full_name: r.full_name,
                organization_name: r.organization_name,
                job_title: r.job_title,
                relationship_type: r.relationship_type,
                start_year: r.start_year,
                end_year: r.end_year,
                is_still_colleagues: r.is_still_colleagues,
                phone: r.phone,
            })
            .collect();

        let resume_dtos = resumes
            .into_iter()
            .map(|r| ResumeDto {
                id: r.id,
                storage_key: r.storage_key,
                filename: r.filename,
                mime_type: r.mime_type,
                file_size: r.file_size,
                created_at: r.created_at,
            })
            .collect();

        let skills = skills_with_names
            .into_iter()
            .map(|(id, name)| SkillDto { id, name })
            .collect();

        Ok(CandidateProfileDto {
            id: candidate.id,
            first_name: candidate.first_name,
            last_name: candidate.last_name,
            headline: candidate.headline,
            bio: candidate.bio,
            preferred_city: candidate.preferred_city,
            residence_location_id: candidate.residence_location_id,
            preferred_commute_radius_meters: candidate.preferred_commute_radius_meters,
            show_exact_location_to_employers: candidate.show_exact_location_to_employers,
            is_foreign_national: candidate.is_foreign_national,
            nationality_country_code: candidate.nationality_country_code,
            has_disability: candidate.has_disability,
            disability_type: candidate.disability_type,
            gender: candidate.gender,
            military_service_status: candidate.military_service_status,
            marital_status: candidate.marital_status,
            birth_date: candidate.birth_date,
            preferred_category_ids: candidate.preferred_category_ids,
            linkedin_url: candidate.linkedin_url,
            github_url: candidate.github_url,
            website_url: candidate.website_url,
            audio_intro_storage_key: candidate.audio_intro_storage_key,
            job_search_status: candidate.job_search_status,
            awards: candidate.awards,
            certifications: candidate.certifications,
            academic_projects: candidate.academic_projects,
            publications: candidate.publications,
            volunteering: candidate.volunteering,
            portfolio_items: candidate.portfolio_items,
            skills,
            experiences,
            educations,
            languages,
            references,
            resumes: resume_dtos,
        })
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        cmd: UpdateProfileCommand,
    ) -> Result<CandidateProfileDto, ApplicationError> {
        let mut candidate = self.ensure_candidate(user_id).await?;

        candidate.first_name = cmd.first_name;
        candidate.last_name = cmd.last_name;
        candidate.headline = cmd.headline;
        candidate.bio = cmd.bio;
        candidate.preferred_city = cmd.preferred_city;
        candidate.residence_location_id = cmd.residence_location_id;
        candidate.preferred_commute_radius_meters = cmd.preferred_commute_radius_meters;
        if let Some(show) = cmd.show_exact_location_to_employers { candidate.show_exact_location_to_employers = show; }
        if let Some(foreign) = cmd.is_foreign_national { candidate.is_foreign_national = foreign; }
        candidate.nationality_country_code = cmd.nationality_country_code;
        if let Some(dis) = cmd.has_disability { candidate.has_disability = dis; }
        candidate.disability_type = cmd.disability_type;
        candidate.gender = cmd.gender;
        candidate.military_service_status = cmd.military_service_status;
        candidate.marital_status = cmd.marital_status;
        candidate.birth_date = cmd.birth_date;
        if let Some(cats) = cmd.preferred_category_ids { candidate.preferred_category_ids = cats; }
        candidate.linkedin_url = cmd.linkedin_url;
        candidate.github_url = cmd.github_url;
        candidate.website_url = cmd.website_url;
        candidate.audio_intro_storage_key = cmd.audio_intro_storage_key;
        if let Some(status) = cmd.job_search_status { candidate.job_search_status = status; }
        if let Some(awards) = cmd.awards { candidate.awards = awards; }
        if let Some(certs) = cmd.certifications { candidate.certifications = certs; }
        if let Some(projs) = cmd.academic_projects { candidate.academic_projects = projs; }
        if let Some(pubs) = cmd.publications { candidate.publications = pubs; }
        if let Some(vols) = cmd.volunteering { candidate.volunteering = vols; }
        if let Some(ports) = cmd.portfolio_items { candidate.portfolio_items = ports; }

        let _ = self.candidate_repo.upsert_profile(&candidate).await?;
        self.get_full_profile(user_id).await
    }

    pub async fn search_talents_for_employer(
        &self,
        _employer_user_id: Uuid,
        req: SearchTalentsRequest,
    ) -> Result<Vec<TalentSearchResult>, ApplicationError> {
        let skill_ids: Vec<Uuid> = req.skill_ids
            .map(|raw| raw.split(',').filter_map(|s| Uuid::parse_str(s.trim()).ok()).collect())
            .unwrap_or_default();

        let bbox = match req.bbox {
            Some(ref b) => {
                let parts: Vec<&str> = b.split(',').collect();
                if parts.len() == 4 {
                    let w: f64 = parts[0].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let s: f64 = parts[1].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let e: f64 = parts[2].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let n: f64 = parts[3].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    Some(geo_types::BoundingBox::new(w, s, e, n).map_err(|e| ApplicationError::Validation(e.to_string()))?)
                } else { None }
            }
            None => None,
        };

        let point = match (req.lon, req.lat) {
            (Some(lon), Some(lat)) => Some(geo_types::GeoPoint::new(lon, lat).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            _ => None,
        };

        let results = self.candidate_repo.search_talents(
            req.q.as_deref(),
            &skill_ids,
            req.city.as_deref(),
            req.actively_looking_only.unwrap_or(false),
            bbox.as_ref(),
            point.as_ref(),
            req.radius_meters,
            req.opportunity_id,
            req.limit.unwrap_or(60),
        ).await?;

        Ok(results)
    }

    pub async fn get_matched_talents_for_opportunity(
        &self,
        _employer_user_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<Vec<TalentSearchResult>, ApplicationError> {
        let opp_repo = self.opp_repo.as_ref().ok_or_else(|| ApplicationError::Validation("Missing repo".into()))?;
        let _opp = opp_repo.find_by_id(opportunity_id).await?.ok_or(StorageError::OpportunityNotFound)?;

        let opp_coords = opp_repo.get_first_location_coords(opportunity_id).await?;
        let center = match opp_coords {
            Some((lon, lat)) => Some(geo_types::GeoPoint::new(lon, lat).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            None => None,
        };

        let results = self.candidate_repo.search_talents(
            None,
            &[],
            None,
            false,
            None,
            center.as_ref(),
            Some(25000.0),
            Some(opportunity_id),
            40,
        ).await?;

        Ok(results)
    }

    pub async fn send_job_invitation(
        &self,
        employer_user_id: Uuid,
        candidate_id: Uuid,
        cmd: SendInvitationCommand,
    ) -> Result<Uuid, ApplicationError> {
        let opp_repo = self.opp_repo.as_ref().ok_or_else(|| ApplicationError::Validation("Missing repo".into()))?;
        let company_repo = self.company_repo.as_ref().ok_or_else(|| ApplicationError::Validation("Missing repo".into()))?;

        let opp = opp_repo.find_by_id(cmd.opportunity_id).await?.ok_or(StorageError::OpportunityNotFound)?;
        let role = company_repo.get_user_role(opp.company_id, employer_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("Not authorized for this company".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized("Insufficient permissions to invite candidates".into()));
        }

        let inv_id = self.candidate_repo.send_invitation(
            cmd.opportunity_id,
            candidate_id,
            opp.company_id,
            employer_user_id,
            cmd.message.as_deref(),
        ).await?;

        Ok(inv_id)
    }

    pub async fn add_experience(&self, user_id: Uuid, cmd: AddExperienceCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let exp = CandidateExperience {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            title: cmd.title,
            company_name: cmd.company_name,
            activity_field: cmd.activity_field,
            seniority_level: cmd.seniority_level,
            company_industry: cmd.company_industry,
            country: cmd.country,
            city: cmd.city,
            start_month: cmd.start_month,
            start_year: cmd.start_year,
            end_month: cmd.end_month,
            end_year: cmd.end_year,
            is_current: cmd.is_current,
            achievements: cmd.achievements,
            created_at: Utc::now(),
        };
        Ok(self.candidate_repo.add_experience(&exp).await?)
    }

    pub async fn delete_experience(&self, user_id: Uuid, exp_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.delete_experience(candidate.id, exp_id).await?;
        Ok(())
    }

    pub async fn add_education(&self, user_id: Uuid, cmd: AddEducationCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let edu = CandidateEducation {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            institution: cmd.institution,
            degree_level: cmd.degree_level,
            field_of_study: cmd.field_of_study,
            gpa: cmd.gpa,
            start_year: cmd.start_year,
            end_year: cmd.end_year,
            is_current: cmd.is_current,
            created_at: Utc::now(),
        };
        Ok(self.candidate_repo.add_education(&edu).await?)
    }

    pub async fn delete_education(&self, user_id: Uuid, edu_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.delete_education(candidate.id, edu_id).await?;
        Ok(())
    }

    pub async fn add_language(&self, user_id: Uuid, cmd: AddLanguageCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let lang = CandidateLanguage {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            language_name: cmd.language_name,
            proficiency_level: cmd.proficiency_level,
            created_at: Utc::now(),
        };
        Ok(self.candidate_repo.add_language(&lang).await?)
    }

    pub async fn delete_language(&self, user_id: Uuid, lang_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.delete_language(candidate.id, lang_id).await?;
        Ok(())
    }

    pub async fn add_reference(&self, user_id: Uuid, cmd: AddReferenceCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let r = CandidateReference {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            full_name: cmd.full_name,
            organization_name: cmd.organization_name,
            job_title: cmd.job_title,
            relationship_type: cmd.relationship_type,
            start_year: cmd.start_year,
            end_year: cmd.end_year,
            is_still_colleagues: cmd.is_still_colleagues,
            phone: cmd.phone,
            created_at: Utc::now(),
        };
        Ok(self.candidate_repo.add_reference(&r).await?)
    }

    pub async fn delete_reference(&self, user_id: Uuid, ref_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.delete_reference(candidate.id, ref_id).await?;
        Ok(())
    }

    pub async fn add_resume(&self, user_id: Uuid, cmd: AddResumeCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let res = CandidateResume {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            storage_key: cmd.storage_key,
            filename: cmd.filename,
            mime_type: cmd.mime_type,
            file_size: cmd.file_size,
            created_at: Utc::now(),
        };
        Ok(self.candidate_repo.add_resume(&res).await?)
    }

    pub async fn delete_resume(&self, user_id: Uuid, resume_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.delete_resume(candidate.id, resume_id).await?;
        Ok(())
    }

    pub async fn set_skills(&self, user_id: Uuid, skill_ids: &[Uuid]) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.set_skills(candidate.id, skill_ids).await?;
        Ok(())
    }

    pub async fn get_preferences(&self, user_id: Uuid) -> Result<CandidatePreferencesDto, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let prefs = self.candidate_repo.get_preferences(candidate.id).await?;
        match prefs {
            Some(p) => Ok(CandidatePreferencesDto {
                preferred_workplace_types: p.preferred_workplace_types,
                preferred_opportunity_types: p.preferred_opportunity_types,
                expected_salary_min: p.expected_salary_min,
                salary_currency: p.salary_currency,
                remote_only: p.remote_only,
            }),
            None => Ok(CandidatePreferencesDto {
                preferred_workplace_types: vec![],
                preferred_opportunity_types: vec![],
                expected_salary_min: None,
                salary_currency: "IRR".to_string(),
                remote_only: false,
            }),
        }
    }

    pub async fn set_preferences(
        &self,
        user_id: Uuid,
        dto: CandidatePreferencesDto,
    ) -> Result<(), ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        self.candidate_repo.upsert_preferences(
            candidate.id,
            &dto.preferred_workplace_types,
            &dto.preferred_opportunity_types,
            dto.expected_salary_min,
            &dto.salary_currency,
            dto.remote_only,
        ).await?;
        Ok(())
    }

    pub async fn list_my_applications(&self, user_id: Uuid) -> Result<Vec<TrackedApplicationDto>, ApplicationError> {
        let candidate = self.ensure_candidate(user_id).await?;
        let apps = self.app_repo.list_by_candidate(candidate.id).await?;
        Ok(apps
            .into_iter()
            .map(|a| TrackedApplicationDto {
                application_id: a.id,
                opportunity_id: a.opportunity_id,
                status: a.status.as_str().to_string(),
                created_at: a.created_at,
            })
            .collect())
    }
}