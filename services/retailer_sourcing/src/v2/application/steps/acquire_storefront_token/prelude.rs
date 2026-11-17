use super::Handler;
use crate::platform::{Platform, bus::Bus};

pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(Handler(platform.http.clone()));
}
