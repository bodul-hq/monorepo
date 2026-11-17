//! Everything other modules may use from offer delivery: its registration.

use super::record_export_messages;
use crate::platform::Platform;
use crate::platform::bus::Bus;
use record_export_messages::prelude::register as prelude_register;

/// Registers every offer delivery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    prelude_register(bus, platform);
}
