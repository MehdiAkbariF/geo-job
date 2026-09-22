use biz_domain::opportunity::{OpportunityType, RemoteScope, Salary, WorkplaceType};
use biz_domain::taxonomy::ExperienceLevel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateOpportunityCommand {
    #[serde(default)]
    #[schema(value_type = Option<String>, format = Uuid)]
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    #[schema(value_type = String, format = Uuid)]
    pub category_id: Uuid,
    #[schema(value_type = Option<String>, format = Uuid)]
    pub occupation_id: Option<Uuid>,
    pub opportunity_type: OpportunityType,
    pub workplace_type: WorkplaceType,
    pub remote_scope: Option<RemoteScope>,
    pub experience_level: ExperienceLevel,
    pub salary: Salary,
    #[schema(value_type = Vec<String>)]
    pub location_ids: Vec<Uuid>,
    #[schema(value_type = Vec<String>)]
    pub skill_ids: Vec<Uuid>,
}