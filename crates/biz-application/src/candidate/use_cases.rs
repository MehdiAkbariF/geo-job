use super::dto::{AddExperienceCommand, CandidateProfileDto, UpdateProfileCommand};
use crate::error::ApplicationError;
use biz_storage::{CandidateRepository, StorageError};
use uuid::Uuid;

pub struct CandidateUseCases {
    candidate_repo: CandidateRepository,
}

impl CandidateUseCases {
    pub fn new(candidate_repo: CandidateRepository) -> Self {
        Self { candidate_repo }
    }

    pub async fn get_or_create_profile(
        &self,
        user_id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<CandidateProfileDto, ApplicationError> {
        let candidate = match self.candidate_repo.find_by_user_id(user_id).await? {
            Some(c) => c,
            None => {
                self.candidate_repo
                    .upsert_profile(user_id, first_name, last_name, None, None, None)
                    .await?
            }
        };

        Ok(CandidateProfileDto {
            id: candidate.id,
            first_name: candidate.first_name,
            last_name: candidate.last_name,
            headline: candidate.headline,
            bio: candidate.bio,
            preferred_city: candidate.preferred_city,
        })
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        cmd: UpdateProfileCommand,
    ) -> Result<CandidateProfileDto, ApplicationError> {
        let candidate = self
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

        Ok(CandidateProfileDto {
            id: candidate.id,
            first_name: candidate.first_name,
            last_name: candidate.last_name,
            headline: candidate.headline,
            bio: candidate.bio,
            preferred_city: candidate.preferred_city,
        })
    }

    pub async fn add_experience(
        &self,
        user_id: Uuid,
        cmd: AddExperienceCommand,
    ) -> Result<Uuid, ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        let exp_id = self
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

        Ok(exp_id)
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
}