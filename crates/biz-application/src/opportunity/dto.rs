use biz_domain::opportunity::{OpportunityType, RemoteScope, Salary, WorkplaceType};
use biz_domain::taxonomy::ExperienceLevel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOpportunityCommand {
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub category_id: Uuid,
    pub occupation_id: Option<Uuid>,
    pub opportunity_type: OpportunityType,
    pub workplace_type: WorkplaceType,
    pub remote_scope: Option<RemoteScope>,
    pub experience_level: ExperienceLevel,
    pub salary: Salary,
    pub location_ids: Vec<Uuid>,
    pub skill_ids: Vec<Uuid>,
}