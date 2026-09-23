//! Requested owned capacity including Datum allocator alignment/header bytes.
//! System allocator slack and native mappings remain separate RSS observations.
use super::*;

pub(crate) fn tracking_bytes<T>(capacity: usize) -> usize {
    if capacity == 0 || std::mem::size_of::<T>() == 0 || !installed() {
        return 0;
    }
    let layout = Layout::array::<T>(capacity).expect("live Rust capacity has a valid layout");
    extended(layout)
        .expect("live Datum allocation has a valid extended layout")
        .1
}

pub(crate) fn capacity_bytes<T>(capacity: usize) -> usize {
    capacity * std::mem::size_of::<T>() + tracking_bytes::<T>(capacity)
}

/// Observe the actual Arc allocation instead of assuming its private header.
/// Construct any nested payload before calling; it belongs to separate capacity
/// accounting. Callers cache this invariant result for their concrete type.
pub(crate) fn arc_bytes<T>(value: T) -> usize {
    let scope = Scope::new("arc-container-layout");
    let allocation = scope.with(|| Arc::new(value));
    let usage = scope.usage();
    let bytes = (usage.payload_bytes + usage.tracking_bytes) as usize;
    drop(allocation);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_capacity_matches_allocator_for_alignment_growth_and_empty_storage() {
        #[repr(align(64))]
        struct Aligned([u8; 64]);
        let scope = Scope::new("owned-capacity-proof");
        let mut bytes = scope.with(|| Vec::<u8>::with_capacity(17));
        assert_eq!(
            capacity_bytes::<u8>(bytes.capacity()) as u64,
            scope.usage().payload_bytes + scope.usage().tracking_bytes
        );
        scope.with(|| bytes.reserve_exact(100));
        assert_eq!(
            capacity_bytes::<u8>(bytes.capacity()) as u64,
            scope.usage().payload_bytes + scope.usage().tracking_bytes
        );
        drop(bytes);
        let aligned = scope.with(|| vec![Aligned([0; 64])]);
        assert_eq!(aligned[0].0[0], 0);
        assert_eq!(
            capacity_bytes::<Aligned>(aligned.capacity()) as u64,
            scope.usage().payload_bytes + scope.usage().tracking_bytes
        );
        drop(aligned);
        assert_eq!(
            scope.usage().payload_bytes + scope.usage().tracking_bytes,
            0
        );
        assert_eq!(capacity_bytes::<u8>(0), 0);
        assert_eq!(capacity_bytes::<()>(usize::MAX), 0);
    }
}
