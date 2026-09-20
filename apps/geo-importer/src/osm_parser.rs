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

/// Reads an OSM PBF file block-by-block in a streaming fashion.
/// Only extracts named places, amenities, offices, and commercial points.
pub fn stream_osm_pbf<F>(pbf_path: &Path, mut on_batch: F, batch_size: usize) -> Result<usize, ImporterError>
where
    F: FnMut(Vec<ExtractedOsmPoint>) -> Result<(), ImporterError>,
{
    let reader = ElementReader::from_path(pbf_path)?;
    let mut batch = Vec::with_capacity(batch_size);
    let mut total_extracted = 0;

    reader.for_each(|element| {
        if let Element::DenseNode(node) = element {
            let mut tags_map = HashMap::new();
            let mut name = None;
            let mut category = None;

            for (key, val) in node.tags() {
                tags_map.insert(key.to_string(), val.to_string());
                if key == "name" || key == "name:fa" {
                    name = Some(val.to_string());
                } else if key == "place" || key == "amenity" || key == "office" || key == "shop" {
                    category = Some(val.to_string());
                }
            }

            // Only capture points that have identifying geographic context
            if let (Some(cat), Ok(pt)) = (category, GeoPoint::new(node.lon(), node.lat())) {
                batch.push(ExtractedOsmPoint {
                    osm_id: node.id,
                    point: pt,
                    name,
                    category: cat,
                    tags: tags_map,
                });

                if batch.len() >= batch_size {
                    total_extracted += batch.len();
                    let current_batch = std::mem::replace(&mut batch, Vec::with_capacity(batch_size));
                    let _ = on_batch(current_batch);
                }
            }
        }
    })?;

    if !batch.is_empty() {
        total_extracted += batch.len();
        on_batch(batch)?;
    }

    Ok(total_extracted)
}