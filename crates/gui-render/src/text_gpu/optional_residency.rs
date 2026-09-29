//! Bounded weak eviction notices for optional GPU retention, not a lifetime owner.
//! Required process admission may release idle caches in any host. Encoding and
//! submission references retain the original tracked reservations until completion.
use std::sync::{Arc, Mutex, OnceLock, Weak};

pub(crate) trait Evict: Send + Sync {
    fn evict(&self);
}
const SLOTS: usize = 16;
type Slots = [Option<Weak<dyn Evict>>; SLOTS];
fn slots() -> &'static Mutex<Slots> {
    static SLOTS: OnceLock<Mutex<Slots>> = OnceLock::new();
    SLOTS.get_or_init(|| Mutex::new(std::array::from_fn(|_| None)))
}
pub(crate) struct Registration {
    slot: usize,
    owner: Weak<dyn Evict>,
}
impl Drop for Registration {
    fn drop(&mut self) {
        let mut slots = slots().lock().unwrap();
        if slots[self.slot]
            .as_ref()
            .is_some_and(|owner| owner.ptr_eq(&self.owner))
        {
            slots[self.slot] = None;
        }
    }
}
pub(crate) fn register(owner: Arc<dyn Evict>) -> Option<Registration> {
    let owner = Arc::downgrade(&owner);
    let mut slots = slots().lock().unwrap();
    let slot = slots
        .iter()
        .position(|entry| entry.as_ref().is_none_or(|owner| owner.strong_count() == 0))?;
    slots[slot] = Some(owner.clone());
    Some(Registration { slot, owner })
}
pub(crate) fn evict_all() {
    // Never hold the registry lock while dropping resources/reservations.
    let owners = slots().lock().unwrap().clone();
    for owner in owners
        .into_iter()
        .flatten()
        .filter_map(|owner| owner.upgrade())
    {
        owner.evict();
    }
}
