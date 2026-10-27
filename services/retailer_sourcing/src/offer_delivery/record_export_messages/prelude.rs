use super::{listener, store};
use crate::platform::{Platform, bus::Bus};

pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.listen(listener::Listener::new(store));
}
