//! Geographic coordinates representation

use serde::{Deserialize, Serialize};

/// Geographic coordinates using latitude and longitude.
///
/// Represents a point on Earth's surface using the WGS84 coordinate system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}
