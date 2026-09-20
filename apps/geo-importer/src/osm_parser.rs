use crate::error::ImporterError;
use geo_types::GeoPoint;
use osmpbf::{Element, ElementReader};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExtractedOsmPoint {
    pub osm_id: i64,
    pub point: GeoPoint,
    pub name: Option<String>,
    pub category: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedOsmRoad {
    pub osm_id: i64,
    pub name: Option<String>,
    pub highway: String,
    pub line_wkt: String,
}

/// Streams real points and road linestrings from OSM PBF file.
pub fn stream_osm_pbf_full<FPoints, FRoads>(
    pbf_path: &Path,
    mut on_points_batch: FPoints,
    mut on_roads_batch: FRoads,
    batch_size: usize,
) -> Result<(usize, usize), ImporterError>
where
    FPoints: FnMut(Vec<ExtractedOsmPoint>) -> Result<(), ImporterError>,
    FRoads: FnMut(Vec<ExtractedOsmRoad>) -> Result<(), ImporterError>,
{
    let reader = ElementReader::from_path(pbf_path)?;
    let mut node_coords: HashMap<i64, (f32, f32)> = HashMap::new();
    let mut points_batch = Vec::with_capacity(batch_size);
    let mut roads_batch = Vec::with_capacity(batch_size);

    let mut total_points = 0;
    let mut total_roads = 0;

    tracing::info!("Parsing OSM elements (Points and Highways)...");

    reader.for_each(|element| {
        match element {
            Element::DenseNode(node) => {
                let lon = node.lon();
                let lat = node.lat();
                node_coords.insert(node.id, (lon as f32, lat as f32));

                let mut tags_map = HashMap::new();
                let mut name = None;
                let mut category = None;

                for (key, val) in node.tags() {
                    tags_map.insert(key.to_string(), val.to_string());
                    if key == "name:fa" {
                        name = Some(val.to_string());
                    } else if key == "name" && name.is_none() {
                        name = Some(val.to_string());
                    } else if key == "office" || key == "amenity" || key == "shop" || key == "place" {
                        category = Some(format!("{}:{}", key, val));
                    }
                }

                if let (Some(cat), Ok(pt)) = (category, GeoPoint::new(lon, lat)) {
                    points_batch.push(ExtractedOsmPoint {
                        osm_id: node.id,
                        point: pt,
                        name,
                        category: cat,
                        tags: tags_map,
                    });

                    if points_batch.len() >= batch_size {
                        total_points += points_batch.len();
                        let batch = std::mem::replace(&mut points_batch, Vec::with_capacity(batch_size));
                        let _ = on_points_batch(batch);
                    }
                }
            }
            Element::Way(way) => {
                let mut highway = None;
                let mut name = None;

                for (k, v) in way.tags() {
                    if k == "highway" {
                        highway = Some(v.to_string());
                    } else if k == "name:fa" {
                        name = Some(v.to_string());
                    } else if k == "name" && name.is_none() {
                        name = Some(v.to_string());
                    }
                }

                if let Some(hw) = highway {
                    let mut coords_str = Vec::new();
                    for node_id in way.refs() {
                        if let Some(&(lon, lat)) = node_coords.get(&node_id) {
                            coords_str.push(format!("{:.6} {:.6}", lon, lat));
                        }
                    }

                    if coords_str.len() >= 2 {
                        let wkt = format!("LINESTRING({})", coords_str.join(", "));
                        roads_batch.push(ExtractedOsmRoad {
                            osm_id: way.id(),
                            name,
                            highway: hw,
                            line_wkt: wkt,
                        });

                        if roads_batch.len() >= batch_size {
                            total_roads += roads_batch.len();
                            let batch = std::mem::replace(&mut roads_batch, Vec::with_capacity(batch_size));
                            let _ = on_roads_batch(batch);
                        }
                    }
                }
            }
            _ => {}
        }
    })?;

    if !points_batch.is_empty() {
        total_points += points_batch.len();
        on_points_batch(points_batch)?;
    }

    if !roads_batch.is_empty() {
        total_roads += roads_batch.len();
        on_roads_batch(roads_batch)?;
    }

    Ok((total_points, total_roads))
}