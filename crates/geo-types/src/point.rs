use crate::error::GeoError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents a validated geographic point in EPSG:4326 (WGS84).
/// Storage and serialization strictly enforce [longitude, latitude].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    longitude: f64,
    latitude: f64,
}

impl GeoPoint {
    pub fn new(longitude: f64, latitude: f64) -> Result<Self, GeoError> {
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(GeoError::InvalidLongitude(longitude));
        }
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(GeoError::InvalidLatitude(latitude));
        }
        Ok(Self { longitude, latitude })
    }

    #[inline]
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    #[inline]
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Exports coordinates in standard GeoJSON array convention: [lon, lat]
    #[inline]
    pub fn to_coordinates(&self) -> [f64; 2] {
        [self.longitude, self.latitude]
    }
}

impl Serialize for GeoPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [self.longitude, self.latitude].serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GeoPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let coords = <[f64; 2]>::deserialize(deserializer)?;
        GeoPoint::new(coords[0], coords[1]).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_geopoint() {
        let pt = GeoPoint::new(51.3890, 35.6892).unwrap();
        assert_eq!(pt.longitude(), 51.3890);
        assert_eq!(pt.latitude(), 35.6892);
        assert_eq!(pt.to_coordinates(), [51.3890, 35.6892]);
    }

    #[test]
    fn test_invalid_coordinates() {
        assert_eq!(
            GeoPoint::new(181.0, 35.0).unwrap_err(),
            GeoError::InvalidLongitude(181.0)
        );
        assert_eq!(
            GeoPoint::new(51.0, 95.0).unwrap_err(),
            GeoError::InvalidLatitude(95.0)
        );
    }

    #[test]
    fn test_serialization_contract() {
        let pt = GeoPoint::new(51.3890, 35.6892).unwrap();
        let serialized = serde_json::to_string(&pt).unwrap();
        assert_eq!(serialized, "[51.389,35.6892]");

        let deserialized: GeoPoint = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, pt);
    }
}