use super::dto::SearchOpportunitiesRequest;
use crate::error::ApplicationError;
use biz_domain::discovery::{SearchPageResult, SearchQuery, SortBy};
use biz_storage::DiscoveryRepository;
use geo_types::{BoundingBox, GeoPoint, Radius};

pub struct DiscoveryUseCases {
    discovery_repo: DiscoveryRepository,
}

impl DiscoveryUseCases {
    pub fn new(discovery_repo: DiscoveryRepository) -> Self {
        Self { discovery_repo }
    }

    pub async fn search(&self, req: SearchOpportunitiesRequest) -> Result<SearchPageResult, ApplicationError> {
        let point = match (req.lon, req.lat) {
            (Some(lon), Some(lat)) => Some(GeoPoint::new(lon, lat).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            _ => None,
        };

        let radius = match req.radius_meters {
            Some(r) => Some(Radius::from_meters(r).map_err(|e| ApplicationError::Validation(e.to_string()))?),
            None => None,
        };

        let bbox = match req.bbox {
            Some(ref b) => {
                let parts: Vec<&str> = b.split(',').collect();
                if parts.len() == 4 {
                    let w: f64 = parts[0].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let s: f64 = parts[1].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let e: f64 = parts[2].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    let n: f64 = parts[3].trim().parse().map_err(|_| ApplicationError::Validation("Invalid bbox".into()))?;
                    Some(BoundingBox::new(w, s, e, n).map_err(|e| ApplicationError::Validation(e.to_string()))?)
                } else {
                    None
                }
            }
            None => None,
        };

        let query = SearchQuery {
            text: req.q,
            category_id: req.category_id,
            occupation_id: req.occupation_id,
            company_id: req.company_id,
            opportunity_type: req.opportunity_type,
            workplace_type: req.workplace_type,
            experience_level: req.experience_level,
            salary_min: req.salary_min,
            point,
            radius,
            bbox,
            sort: SortBy::Newest,
            cursor: req.cursor,
            limit: req.limit.unwrap_or(20),
        };

        let result = self.discovery_repo.search(&query).await?;
        Ok(result)
    }
}