use super::dto::{GetMapPinsRequest, SearchOpportunitiesRequest};
use crate::error::ApplicationError;
use biz_domain::discovery::{
    query::SearchCursor, MapPinSummary, OpportunitySearchResult, SearchPageResult, SearchQuery, SortBy,
};
use biz_storage::{CandidateRepository, DiscoveryRepository};
use geo_types::{BoundingBox, GeoPoint, Radius};
use uuid::Uuid;

#[derive(Clone)]
pub struct DiscoveryUseCases {
    discovery_repo: DiscoveryRepository,
    candidate_repo: CandidateRepository,
}

impl DiscoveryUseCases {
    pub fn new(discovery_repo: DiscoveryRepository, candidate_repo: CandidateRepository) -> Self {
        Self {
            discovery_repo,
            candidate_repo,
        }
    }

    pub async fn search(
        &self,
        actor_user_id: Option<Uuid>,
        req: SearchOpportunitiesRequest,
    ) -> Result<SearchPageResult, ApplicationError> {
        let is_near_me = req.near_me.unwrap_or(false);

        // واکشی کانتکست رزومه کارجو برای محاسبه هوشمند Match Score
        let cand_ctx = match actor_user_id {
            Some(uid) => self.candidate_repo.get_match_context(uid).await?,
            None => None,
        };

        let mut point = match (req.lon, req.lat) {
            (Some(lon), Some(lat)) => Some(GeoPoint::new(lon, lat).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            _ => None,
        };

        let mut effective_sort = req.sort.as_deref().map(SortBy::from_str).unwrap_or(SortBy::Newest);

        if is_near_me {
            let user_id = actor_user_id.ok_or_else(|| {
                ApplicationError::Unauthorized("You must be logged in to search near your profile location".into())
            })?;

            let coords = self.candidate_repo.get_preferred_coordinates(user_id).await?;
            if let Some((cand_lon, cand_lat)) = coords {
                point = Some(GeoPoint::new(cand_lon, cand_lat).map_err(|e| ApplicationError::Validation(e.to_string()))?);
                effective_sort = SortBy::Distance;
            } else {
                return Err(ApplicationError::Validation(
                    "No preferred location set in your profile. Please set your city or address in your profile first.".into(),
                ));
            }
        }

        let radius = match req.radius_meters {
            Some(r) => Some(Radius::from_meters(r).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            None => None,
        };

        let bbox = match req.bbox {
            Some(ref b) => {
                let parts: Vec<&str> = b.split(',').collect();
                if parts.len() == 4 {
                    let w: f64 = parts[0].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox west".into()))?;
                    let s: f64 = parts[1].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox south".into()))?;
                    let e: f64 = parts[2].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox east".into()))?;
                    let n: f64 = parts[3].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox north".into()))?;
                    Some(BoundingBox::new(w, s, e, n).map_err(|e| ApplicationError::Validation(e.to_string()))?)
                } else {
                    None
                }
            }
            None => None,
        };

        let skill_ids: Vec<Uuid> = req.skill_ids
            .map(|raw| {
                raw.split(',')
                    .filter_map(|s| Uuid::parse_str(s.trim()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let cursor = req.cursor.as_deref().and_then(SearchCursor::decode);

        let query = SearchQuery {
            text: req.q,
            category_id: req.category_id,
            occupation_id: req.occupation_id,
            company_id: req.company_id,
            skill_ids,
            opportunity_type: req.opportunity_type,
            workplace_type: req.workplace_type,
            experience_level: req.experience_level,
            salary_min: req.salary_min,
            include_remote: req.include_remote.unwrap_or(false),
            city: req.city,
            point,
            radius,
            bbox,
            sort: effective_sort,
            cursor,
            limit: req.limit.unwrap_or(20),
        };

        let result = self.discovery_repo.search(&query, cand_ctx.as_ref()).await?;
        Ok(result)
    }

    /// فید اختصاصی پیشنهادات شغلی با بیشترین درصد سازگاری رزومه
    pub async fn get_recommended_opportunities(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<SearchPageResult, ApplicationError> {
        let cand_ctx = self.candidate_repo.get_match_context(user_id).await?
            .ok_or_else(|| ApplicationError::Validation("Candidate profile does not exist".into()))?;

        let query = SearchQuery {
            sort: SortBy::MatchScore,
            include_remote: true,
            limit: limit.clamp(1, 50),
            ..Default::default()
        };

        let result = self.discovery_repo.search(&query, Some(&cand_ctx)).await?;
        Ok(result)
    }

    pub async fn get_map_pins(&self, req: GetMapPinsRequest) -> Result<Vec<MapPinSummary>, ApplicationError> {
        let bbox = match req.bbox {
            Some(ref b) => {
                let parts: Vec<&str> = b.split(',').collect();
                if parts.len() == 4 {
                    let w: f64 = parts[0].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox west".into()))?;
                    let s: f64 = parts[1].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox south".into()))?;
                    let e: f64 = parts[2].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox east".into()))?;
                    let n: f64 = parts[3].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox north".into()))?;
                    Some(BoundingBox::new(w, s, e, n).map_err(|e| ApplicationError::Validation(e.to_string()))?)
                } else {
                    None
                }
            }
            None => None,
        };

        let point = match (req.lon, req.lat) {
            (Some(lon), Some(lat)) => Some(GeoPoint::new(lon, lat).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            _ => None,
        };

        let radius = match req.radius_meters {
            Some(r) => Some(Radius::from_meters(r).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            None => None,
        };

        let limit = req.limit.unwrap_or(150);
        let pins = self.discovery_repo.list_map_pins(
            bbox.as_ref(),
            point.as_ref(),
            radius.as_ref(),
            req.city.as_deref(),
            req.category_id,
            req.workplace_type.as_deref(),
            req.salary_min,
            limit,
        ).await?;

        Ok(pins)
    }

    pub async fn get_by_location(&self, location_id: Uuid) -> Result<Vec<OpportunitySearchResult>, ApplicationError> {
        let items = self.discovery_repo.list_by_location(location_id).await?;
        Ok(items)
    }
}