use crate::error::GeoError;
use crate::point::GeoPoint;
use serde::{Deserialize, Serialize};

/// Represents an axis-aligned bounding box defined by [west, south, east, north]
/// using EPSG:4326 coordinate reference system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    west: f64,
    south: f64,
    east: f64,
    north: f64,
}

impl BoundingBox {
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Result<Self, GeoError> {
        if !(-180.0..=180.0).contains(&west) {
            return Err(GeoError::InvalidLongitude(west));
        }
        if !(-180.0..=180.0).contains(&east) {
            return Err(GeoError::InvalidLongitude(east));
        }
        if !(-90.0..=90.0).contains(&south) {
            return Err(GeoError::InvalidLatitude(south));
        }
        if !(-90.0..=90.0).contains(&north) {
            return Err(GeoError::InvalidLatitude(north));
        }
        if south > north {
            return Err(GeoError::InvalidBoundingBoxLatitude { south, north });
        }
        if west > east {
            return Err(GeoError::InvalidBoundingBoxLongitude { west, east });
        }

        Ok(Self {
            west,
            south,
            east,
            north,
        })
    }

    #[inline]
    pub fn west(&self) -> f64 {
        self.west
    }

    #[inline]
    pub fn south(&self) -> f64 {
        self.south
    }

    #[inline]
    pub fn east(&self) -> f64 {
        self.east
    }

    #[inline]
    pub fn north(&self) -> f64 {
        self.north
    }

    pub fn contains(&self, point: &GeoPoint) -> bool {
        point.longitude() >= self.west
            && point.longitude() <= self.east
            && point.latitude() >= self.south
            && point.latitude() <= self.north
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bbox() {
        let bbox = BoundingBox::new(51.0, 35.0, 52.0, 36.0).unwrap();
        assert_eq!(bbox.west(), 51.0);
        assert_eq!(bbox.south(), 35.0);
        assert_eq!(bbox.east(), 52.0);
        assert_eq!(bbox.north(), 36.0);

        let pt_inside = GeoPoint::new(51.5, 35.5).unwrap();
        let pt_outside = GeoPoint::new(53.0, 35.5).unwrap();
        assert!(bbox.contains(&pt_inside));
        assert!(!bbox.contains(&pt_outside));
    }

    #[test]
    fn test_invalid_bounds() {
        assert!(BoundingBox::new(51.0, 36.0, 52.0, 35.0).is_err());
        assert!(BoundingBox::new(52.0, 35.0, 51.0, 36.0).is_err());
    }
}