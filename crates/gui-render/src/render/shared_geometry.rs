//! Immutable geometry with a separately owned payload allocation.
use std::{ops::Deref, sync::Arc};

/// Unlike Arc<[T]>, moving a boxed slice here does not copy its elements into
/// the reference-counted allocation. A weak owner can later observe retirement
/// without keeping the potentially large payload allocation alive.
#[derive(Debug)]
pub(crate) struct SharedGeometry<T>(Arc<Box<[T]>>, Option<Arc<crate::text_gpu::budget::Budget>>);

// Admission metadata does not change geometric equality.
impl<T: PartialEq> PartialEq for SharedGeometry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Clone for SharedGeometry<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), self.1.clone())
    }
}

impl<T> From<Vec<T>> for SharedGeometry<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<T> From<Box<[T]>> for SharedGeometry<T> {
    fn from(value: Box<[T]>) -> Self {
        Self(Arc::new(value), None)
    }
}

impl<T> Deref for SharedGeometry<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T> AsRef<[T]> for SharedGeometry<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> SharedGeometry<T> {
    pub(crate) fn for_document(value: Vec<T>, scene_id: &str) -> Self {
        Self(
            Arc::new(value.into_boxed_slice()),
            Some(super::document_gpu_budget::for_scene(scene_id)),
        )
    }

    pub(crate) fn document_budget(&self) -> Option<&Arc<crate::text_gpu::budget::Budget>> {
        self.1.as_ref()
    }

    /// Box payload and the Box slot in the Arc allocation; excludes Arc headers.
    pub(crate) fn heap_bytes(&self) -> usize {
        std::mem::size_of_val(self.as_ref()) + std::mem::size_of::<Box<[T]>>()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> std::sync::Weak<Box<[T]>> {
        Arc::downgrade(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn adoption_preserves_payload_address_and_weak_observation_does_not_pin_elements() {
        struct Element(Arc<AtomicUsize>);
        impl Drop for Element {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let payload: Box<[_]> = (0..8).map(|_| Element(drops.clone())).collect();
        let address = payload.as_ptr();
        let first = SharedGeometry::from(payload);
        assert_eq!(first.as_ptr(), address, "adoption must not copy payload");
        let observer = first.downgrade();
        let second = first.clone();
        assert!(first.ptr_eq(&second));
        drop(first);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        drop(second);
        assert!(observer.upgrade().is_none());
        assert_eq!(drops.load(Ordering::Relaxed), 8);
        // Observer survives, but owns only the Arc control block and Box slot.
        assert_eq!(observer.strong_count(), 0);
    }
}
