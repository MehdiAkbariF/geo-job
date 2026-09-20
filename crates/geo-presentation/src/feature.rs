use geo_domain::Location;
use geo_types::GeoPoint;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeoJsonGeometry {
    Point { coordinates: [f64; 2] },
}

impl From<GeoPoint> for GeoJsonGeometry {
    fn from(pt: GeoPoint) -> Self {
        GeoJsonGeometry::Point {
            coordinates: pt.to_coordinates(),
        }
    }
}

/// A standard GeoJSON Feature representing a presentation entity on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapFeature<P> {
    pub r#type: &'static str,
    pub id: Option<String>,
    pub geometry: GeoJsonGeometry,
    pub properties: P,
}

impl<P> MapFeature<P> {
    pub fn new(id: Option<String>, point: GeoPoint, properties: P) -> Self {
        Self {
            r#type: "Feature",
            id,
            geometry: point.into(),
            properties,
        }
    }
}

/// A standard GeoJSON FeatureCollection container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureCollection<P> {
    pub r#type: &'static str,
    pub features: Vec<MapFeature<P>>,
}

impl<P> FeatureCollection<P> {
    pub fn new(features: Vec<MapFeature<P>>) -> Self {
        Self {
            r#type: "FeatureCollection",
            features,
        }
    }
}

/// Default presentation properties for a raw location entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationProperties {
    pub location_id: Uuid,
    pub address_summary: Option<String>,
    pub precision: String,
    pub source: String,
}

impl From<Location> for MapFeature<LocationProperties> {
    fn from(loc: Location) -> Self {
        let precision_str = format!("{:?}", loc.precision).to_lowercase();
        let props = LocationProperties {
            location_id: loc.id,
            address_summary: loc.address_summary,
            precision: precision_str,
            source: loc.source,
        };
        MapFeature::new(Some(loc.id.to_string()), loc.point, props)
    }
}