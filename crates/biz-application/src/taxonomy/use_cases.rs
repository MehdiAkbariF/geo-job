use crate::error::ApplicationError;
use biz_domain::taxonomy::{Category, Industry, Occupation, Skill};
use biz_storage::TaxonomyRepository;

#[derive(Clone)]
pub struct TaxonomyUseCases {
    repo: TaxonomyRepository,
}

impl TaxonomyUseCases {
    pub fn new(repo: TaxonomyRepository) -> Self {
        Self { repo }
    }

    pub async fn list_categories(&self) -> Result<Vec<Category>, ApplicationError> {
        Ok(self.repo.list_categories().await?)
    }

    pub async fn list_industries(&self) -> Result<Vec<Industry>, ApplicationError> {
        Ok(self.repo.list_industries().await?)
    }

    pub async fn list_occupations(&self) -> Result<Vec<Occupation>, ApplicationError> {
        Ok(self.repo.list_occupations().await?)
    }

    pub async fn resolve_skill(&self, term: &str) -> Result<Option<Skill>, ApplicationError> {
        Ok(self.repo.resolve_skill(term).await?)
    }
}