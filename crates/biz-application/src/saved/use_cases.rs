use super::dto::SaveSearchCommand;
use crate::error::ApplicationError;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult};
use biz_domain::saved::SavedSearch;
use biz_storage::{CandidateRepository, SavedRepository, StorageError};
use uuid::Uuid;

pub struct SavedUseCases {
    saved_repo: SavedRepository,
    candidate_repo: CandidateRepository,
}

impl SavedUseCases {
    pub fn new(saved_repo: SavedRepository, candidate_repo: CandidateRepository) -> Self {
        Self { saved_repo, candidate_repo }
    }

    pub async fn save_opportunity(&self, user_id: Uuid, opportunity_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        self.saved_repo.save_opportunity(candidate.id, opportunity_id).await?;
        Ok(())
    }

    pub async fn remove_saved_opportunity(&self, user_id: Uuid, opportunity_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        self.saved_repo.remove_saved_opportunity(candidate.id, opportunity_id).await?;
        Ok(())
    }

    pub async fn list_saved_opportunities(&self, user_id: Uuid) -> Result<Vec<OpportunitySearchResult>, ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let items = self.saved_repo.list_saved_opportunities(candidate.id).await?;
        Ok(items)
    }

    pub async fn save_company(&self, user_id: Uuid, company_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        self.saved_repo.save_company(candidate.id, company_id).await?;
        Ok(())
    }

    pub async fn remove_saved_company(&self, user_id: Uuid, company_id: Uuid) -> Result<(), ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        self.saved_repo.remove_saved_company(candidate.id, company_id).await?;
        Ok(())
    }

    pub async fn list_saved_companies(&self, user_id: Uuid) -> Result<Vec<CompanySummary>, ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let items = self.saved_repo.list_saved_companies(candidate.id).await?;
        Ok(items)
    }

    pub async fn save_search(&self, user_id: Uuid, cmd: SaveSearchCommand) -> Result<Uuid, ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let id = self.saved_repo.create_saved_search(candidate.id, &cmd.title, &cmd.criteria).await?;
        Ok(id)
    }

    pub async fn list_saved_searches(&self, user_id: Uuid) -> Result<Vec<SavedSearch>, ApplicationError> {
        let candidate = self.candidate_repo.find_by_user_id(user_id).await?.ok_or(StorageError::UserNotFound)?;
        let searches = self.saved_repo.list_saved_searches(candidate.id).await?;
        Ok(searches)
    }
}