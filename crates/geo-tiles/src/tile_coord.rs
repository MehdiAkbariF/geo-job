use geo_types::{BoundingBox, GeoError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TileError {
    #[error("Invalid zoom level: {0}. Zoom must be between 0 and 22.")]
    InvalidZoom(u8),

    #[error("Tile X ({x}) out of bounds for zoom {zoom}. Maximum is {max}.")]
    TileXOutOfBounds { x: u32, zoom: u8, max: u32 },

    #[error("Tile Y ({y}) out of bounds for zoom {zoom}. Maximum is {max}.")]
    TileYOutOfBounds { y: u32, zoom: u8, max: u32 },

    #[error("Spatial error: {0}")]
    Geo(#[from] GeoError),
}

/// Represents standard Slippy Map tile coordinates: (z, x, y)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoordinate {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

impl TileCoordinate {
    pub fn new(z: u8, x: u32, y: u32) -> Result<Self, TileError> {
        if z > 22 {
            return Err(TileError::InvalidZoom(z));
        }
        let max = (1u32 << z).saturating_sub(1);
        if x > max {
            return Err(TileError::TileXOutOfBounds { x, zoom: z, max });
        }
        if y > max {
            return Err(TileError::TileYOutOfBounds { y, zoom: z, max });
        }
        Ok(Self { z, x, y })
    }

    /// Converts slippy tile coordinates to EPSG:4326 BoundingBox [west, south, east, north]
    pub fn to_bbox(&self) -> Result<BoundingBox, TileError> {
        let n = 2.0_f64.powi(self.z as i32);
        
        let west = (self.x as f64) / n * 360.0 - 180.0;
        let east = ((self.x + 1) as f64) / n * 360.0 - 180.0;

        let north_rad = ((1.0 - 2.0 * (self.y as f64) / n) * std::f64::consts::PI).sinh().atan();
        let north = north_rad.to_degrees();

        let south_rad = ((1.0 - 2.0 * ((self.y + 1) as f64) / n) * std::f64::consts::PI).sinh().atan();
        let south = south_rad.to_degrees();

        Ok(BoundingBox::new(west, south, east, north)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tile_coords() {
        let tile = TileCoordinate::new(10, 512, 512).unwrap();
        assert_eq!(tile.z, 10);
        assert_eq!(tile.x, 512);
        assert_eq!(tile.y, 512);

        let bbox = tile.to_bbox().unwrap();
        assert!(bbox.west() < bbox.east());
        assert!(bbox.south() < bbox.north());
    }

    #[test]
    fn test_out_of_bounds_tile() {
        assert!(TileCoordinate::new(25, 0, 0).is_err());
        assert!(TileCoordinate::new(2, 4, 1).is_err()); // at zoom 2, max x is 3
    }
}