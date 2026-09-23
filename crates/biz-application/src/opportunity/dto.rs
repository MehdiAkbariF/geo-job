use biz_domain::opportunity::{OpportunityType, RemoteScope, Salary, WorkplaceType};
use biz_domain::taxonomy::ExperienceLevel;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
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
    pub is_urgent: Option<bool>,
    pub working_hours: Option<String>,
    pub gender_preference: Option<String>,
    pub has_insurance: Option<bool>,
    pub location_ids: Vec<Uuid>,
    pub skill_ids: Vec<Uuid>,
}