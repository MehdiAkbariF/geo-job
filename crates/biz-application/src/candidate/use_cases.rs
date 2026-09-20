use super::dto::{
    AddExperienceCommand, CandidatePreferencesDto, CandidateProfileDto, ExperienceDto,
    TrackedApplicationDto, UpdateProfileCommand,
};
use crate::error::ApplicationError;
use biz_storage::{ApplicationRepository, CandidateRepository, StorageError};
use uuid::Uuid;

#[derive(Clone)]
pub struct CandidateUseCases {
    candidate_repo: CandidateRepository,
    app_repo: ApplicationRepository,
}

impl CandidateUseCases {
    pub fn new(candidate_repo: CandidateRepository, app_repo: ApplicationRepository) -> Self {
        Self {
            candidate_repo,
            app_repo,
        }
    }

    pub async fn get_full_profile(&self, user_id: Uuid) -> Result<CandidateProfileDto, ApplicationError> {
        let candidate = match self.candidate_repo.find_by_user_id(user_id).await? {
            Some(c) => c,
            None => {
                self.candidate_repo
                    .upsert_profile(user_id, "کاربر", "جدید", None, None, None)
                    .await?
            }
        };

        let skills = self.candidate_repo.get_skills(candidate.id).await?;
        let exps = self.candidate_repo.list_experiences(candidate.id).await?;

        let experiences = exps
            .into_iter()
            .map(|e| ExperienceDto {
                id: e.id,
                title: e.title,
                company_name: e.company_name,
                start_date: e.start_date,
                end_date: e.end_date,
                is_current: e.is_current,
                description: e.description,
            })
            .collect();

        Ok(CandidateProfileDto {
            id: candidate.id,
            first_name: candidate.first_name,
            last_name: candidate.last_name,
            headline: candidate.headline,
            bio: candidate.bio,
            preferred_city: candidate.preferred_city,
            skills,
            experiences,
        })
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        cmd: UpdateProfileCommand,
    ) -> Result<CandidateProfileDto, ApplicationError> {
        let _ = self
            .candidate_repo
            .upsert_profile(
                user_id,
                &cmd.first_name,
                &cmd.last_name,
                cmd.headline.as_deref(),
                cmd.bio.as_deref(),
                cmd.preferred_city.as_deref(),
            )
            .await?;

        self.get_full_profile(user_id).await
    }

    pub async fn add_experience(&self, user_id: Uuid, cmd: AddExperienceCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        let id = self
            .candidate_repo
            .add_experience(
                candidate.id,
                &cmd.title,
                &cmd.company_name,
                cmd.start_date,
                cmd.end_date,
                cmd.is_current,
                cmd.description.as_deref(),
            )
            .await?;

        Ok(id)
    }

    pub async fn delete_experience(&self, user_id: Uuid, exp_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        self.candidate_repo.delete_experience(candidate.id, exp_id).await?;
        Ok(())
    }

    pub async fn set_skills(&self, user_id: Uuid, skill_ids: &[Uuid]) -> Result<(), ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        self.candidate_repo.set_skills(candidate.id, skill_ids).await?;
        Ok(())
    }

    pub async fn get_preferences(&self, user_id: Uuid) -> Result<CandidatePreferencesDto, ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

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
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        self.candidate_repo
            .upsert_preferences(
                candidate.id,
                &dto.preferred_workplace_types,
                &dto.preferred_opportunity_types,
                dto.expected_salary_min,
                &dto.salary_currency,
                dto.remote_only,
            )
            .await?;

        Ok(())
    }

    pub async fn list_my_applications(&self, user_id: Uuid) -> Result<Vec<TrackedApplicationDto>, ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

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