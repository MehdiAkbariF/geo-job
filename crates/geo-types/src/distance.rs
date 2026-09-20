use crate::error::GeoError;
use serde::{Deserialize, Serialize};

/// Distance measurement strictly in meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance(f64);

impl Distance {
    #[inline]
    pub fn from_meters(meters: f64) -> Self {
        Self(meters.max(0.0))
    }

    #[inline]
    pub fn as_meters(&self) -> f64 {
        self.0
    }
}

/// Search radius measurement strictly in meters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Radius(f64);

impl Radius {
    pub fn from_meters(meters: f64) -> Result<Self, GeoError> {
        if meters <= 0.0 {
            return Err(GeoError::InvalidRadius(meters));
        }
        Ok(Self(meters))
    }

    #[inline]
    pub fn as_meters(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_and_radius() {
        let dist = Distance::from_meters(1250.5);
        assert_eq!(dist.as_meters(), 1250.5);

        let rad = Radius::from_meters(500.0).unwrap();
        assert_eq!(rad.as_meters(), 500.0);

        assert!(Radius::from_meters(-10.0).is_err());
        assert!(Radius::from_meters(0.0).is_err());
    }
}