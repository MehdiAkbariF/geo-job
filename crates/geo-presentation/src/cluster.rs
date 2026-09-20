use crate::feature::{GeoJsonGeometry, MapFeature};
use geo_query::LocationCluster;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterProperties {
    pub cluster: bool,
    pub point_count: i64,
    pub bbox: [f64; 4], // [west, south, east, north]
}

pub type ClusterFeature = MapFeature<ClusterProperties>;

impl From<LocationCluster> for ClusterFeature {
    fn from(c: LocationCluster) -> Self {
        let props = ClusterProperties {
            cluster: true,
            point_count: c.count,
            bbox: [
                c.bounds.west(),
                c.bounds.south(),
                c.bounds.east(),
                c.bounds.north(),
            ],
        };

        MapFeature {
            r#type: "Feature",
            id: None,
            geometry: GeoJsonGeometry::Point {
                coordinates: c.center.to_coordinates(),
            },
            properties: props,
        }
    }
}