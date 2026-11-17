//! Everything other modules may use from offer delivery: its registration.

use record_export_messages::prelude::register as record_export_messages_register;

use super::record_export_messages;
use crate::platform::Platform;
use crate::platform::bus::Bus;

/// Registers every offer delivery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    record_export_messages_register(bus, platform);
}
