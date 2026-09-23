//! Allocation origin for synchronous renderer work, including nested helper owners.
use std::{
    cell::Cell,
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_HOST: AtomicU64 = AtomicU64::new(1);
thread_local! { static CURRENT: Cell<Option<u64>> = const { Cell::new(None) }; }

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
        Guard {
            previous: CURRENT.replace(Some(self.0)),
            _thread: PhantomData,
        }
    }
}
pub(crate) fn current() -> Option<u64> {
    CURRENT.get()
}
pub(crate) struct Guard {
    previous: Option<u64>,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for Guard {
    fn drop(&mut self) {
        CURRENT.set(self.previous);
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
