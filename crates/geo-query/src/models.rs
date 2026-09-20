use geo_domain::Location;
use geo_types::{BoundingBox, Distance, GeoPoint};
use serde::{Deserialize, Serialize};

/// Location result augmented with distance relative to the query point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearbyLocation {
    pub location: Location,
    pub distance: Distance,
}

/// Represents an aggregated cluster of locations for map viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationCluster {
    pub count: i64,
    pub center: GeoPoint,
    pub bounds: BoundingBox,
}