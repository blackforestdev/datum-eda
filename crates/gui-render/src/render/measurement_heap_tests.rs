//! Reconcile the retained scalar owner against actual scoped allocations.
use super::*;

#[test]
fn width_cache_counts_all_owned_heap_headers_through_eviction_and_release() {
    let scope = crate::cpu_alloc::Scope::new("width-cache-owned-heap");
    // Register observation metadata outside the cache-payload allocation scope.
    let mut cache = MeasurementCache::default();
    for n in 0..600 {
        let text = format!("{n} {}", "x".repeat(if n < 300 { 8 } else { 8000 }));
        scope.with(|| {
            cache.measure(&text, 13.0, TextFace::Ui, || 17.0);
        });
        let actual = scope.usage();
        assert_eq!(
            cache.retained_bytes() - std::mem::size_of::<MeasurementCache>(),
            (actual.payload_bytes + actual.tracking_bytes) as usize
        );
        assert_eq!(
            cache.text_bytes,
            cache.entries.iter().map(|e| e.0.len()).sum::<usize>()
        );
        assert!(cache.retained_bytes() <= MAX_OWNED_BYTES);
        assert!(cache.text_bytes <= MAX_MEASUREMENT_TEXT_BYTES);
    }
    drop(cache);
    assert_eq!(
        scope.usage().payload_bytes + scope.usage().tracking_bytes,
        0
    );
}
