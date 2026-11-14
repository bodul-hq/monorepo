pub use super::command::AcquireStorefrontToken;
pub use super::event::StorefrontTokenAcquired;
use super::handler;
use crate::platform::{Platform, bus::Bus};
pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(handler::Handler(platform.http.clone()));
}
