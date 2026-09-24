//! Allocation origin for synchronous renderer work, including nested helper owners.
use std::{
    cell::Cell,
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_HOST: AtomicU64 = AtomicU64::new(1);
thread_local! { static CURRENT: Cell<Option<u64>> = const { Cell::new(None) };
static CONSUMERS: Cell<u16> = const { Cell::new(0) }; }

pub(crate) struct Host(u64);
impl Host {
    pub fn new() -> Self {
        Self(
            NEXT_HOST
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("renderer identity exhausted"),
        )
    }
    pub fn id(&self) -> u64 {
        self.0
    }
    pub fn enter(&self) -> Guard {
        self.enter_for(Default::default())
    }
    pub fn enter_for(&self, consumers: crate::resource_consumers::Consumers) -> Guard {
        Guard {
            consumers: CONSUMERS.replace(consumers.bits()),
            previous: CURRENT.replace(Some(self.0)),
            _thread: PhantomData,
        }
    }
}
pub(crate) fn current() -> Option<u64> {
    CURRENT.get()
}
pub(crate) fn consumers() -> crate::resource_consumers::Consumers {
    crate::resource_consumers::Consumers::from_bits(CONSUMERS.get())
}
pub(crate) struct Guard {
    consumers: u16,
    previous: Option<u64>,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for Guard {
    fn drop(&mut self) {
        CURRENT.set(self.previous);
        CONSUMERS.set(self.consumers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_renderer_scopes_restore_on_error_without_cross_thread_leakage() {
        let first = Host::new();
        let second = Host::new();
        assert_ne!(first.id(), second.id());
        let _first = first.enter();
        assert_eq!(current(), Some(first.id()));
        assert!(std::thread::spawn(current).join().unwrap().is_none());
        let result = std::panic::catch_unwind(|| {
            let _second = second.enter();
            assert_eq!(current(), Some(second.id()));
            panic!("failed renderer construction");
        });
        assert!(result.is_err());
        assert_eq!(current(), Some(first.id()));
        drop(_first);
        assert!(current().is_none());
    }
}
