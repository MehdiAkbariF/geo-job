pub mod category;
pub mod experience_level;
pub mod industry;
pub mod occupation;
pub mod skill;

pub use category::Category;
pub use experience_level::ExperienceLevel;
pub use industry::Industry;
pub use occupation::Occupation;
pub use skill::{Skill, SkillAlias};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Country {
    pub code: String, // ISO-3166-1 alpha-3 (e.g. "IRN", "AFG")
    pub name: String, // Persian Name (e.g. "ایران", "افغانستان")
    pub name_en: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CityTaxonomy {
    pub id: Uuid,
    pub name: String,
    pub province: String,
    #[schema(value_type = [f64; 2], example = json!([51.3890, 35.7200]))]
    pub center: [f64; 2],
}