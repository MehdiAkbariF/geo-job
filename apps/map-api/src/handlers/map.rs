use crate::dto::{BBoxQueryParams, NearbyQueryParams};
use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use geo_presentation::{ClusterFeature, FeatureCollection, LocationProperties, MapFeature};
use geo_query::{bbox_query::BBoxSearchOptions, cluster_locations_in_bbox, find_locations_in_bbox, find_locations_within_radius};
use serde_json::Value;

pub async fn search_viewport(
    State(state): State<AppState>,
    Query(params): Query<BBoxQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    let bbox = params.parse_bbox()?;
    let zoom = params.zoom.unwrap_or(12);

    if zoom < 11 {
        let grid_size = match zoom {
            0..=5 => 1.0,
            6..=8 => 0.25,
            _ => 0.08,
        };

        let clusters = cluster_locations_in_bbox(&state.pool, &bbox, grid_size).await?;
        let cluster_features: Vec<ClusterFeature> = clusters.into_iter().map(ClusterFeature::from).collect();
        let collection = FeatureCollection::new(cluster_features);
        Ok(Json(serde_json::to_value(collection).unwrap()))
    } else {
        let options = BBoxSearchOptions {
            limit: params.limit,
            source: params.source,
        };
        let locations = find_locations_in_bbox(&state.pool, &bbox, options).await?;
        let features: Vec<MapFeature<LocationProperties>> = locations.into_iter().map(MapFeature::from).collect();
        let collection = FeatureCollection::new(features);
        Ok(Json(serde_json::to_value(collection).unwrap()))
    }
}

pub async fn search_nearby(
    State(state): State<AppState>,
    Query(params): Query<NearbyQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    let (center, radius) = params.parse_point_and_radius()?;
    let results = find_locations_within_radius(&state.pool, &center, &radius, params.limit).await?;

    let features: Vec<Value> = results
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": item.location.point.to_coordinates()
                },
                "properties": {
                    "id": item.location.id,
                    "address_summary": item.location.address_summary,
                    "distance_meters": item.distance.as_meters()
                }
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "type": "FeatureCollection",
        "features": features
    })))
}