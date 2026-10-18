//! Everything other modules may use from monitoring: its registration.

use crate::platform::Platform;
use crate::platform::bus::Bus;

/// Registers every monitoring slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}
