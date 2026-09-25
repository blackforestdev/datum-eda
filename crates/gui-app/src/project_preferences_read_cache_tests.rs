use super::*;
use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};

#[test]
fn cache_accounts_model_input_and_scratch_and_releases_evicted_outputs() {
    let root = std::env::temp_dir().join(format!("datum-cache-owner-{}", uuid::Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Cache ownership".to_owned(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut cache = ReadCache::default();
    let first = cache.resolve(&root).unwrap();
    let usage = cache.owner.usage();
    assert!(usage.allocator_installed);
    assert!(usage.payload_bytes > 0);
    assert!(usage.tracking_bytes > 0);
    assert!(usage.peak_payload_bytes >= usage.payload_bytes);
    assert!(accounted_bytes(&usage) <= RETAINED_BYTES);
    assert!(Arc::ptr_eq(&first, &cache.resolve(&root).unwrap()));
    let expected = (*first).clone();
    drop(first);

    // Pressure evicts only reuse: the current validated output remains identical.
    let current = cache.resolve_with_limit(&root, 0).unwrap();
    assert_eq!(*current, expected);
    let weak = Arc::downgrade(&current);
    drop(current);
    assert!(weak.upgrade().is_none());
    drop(weak);
    assert_eq!(cache.owner.usage().payload_bytes, 0);
    assert_eq!(cache.owner.usage().tracking_bytes, 0);
    assert_eq!(cache.owner.usage().allocations, 0);

    // A cold read after eviction is still authoritative; normal reuse resumes.
    let next = cache.resolve(&root).unwrap();
    assert_eq!(*next, expected);
    assert!(Arc::ptr_eq(&next, &cache.resolve(&root).unwrap()));
    drop(next);
    let owner = cache.owner.clone();
    drop(cache);
    assert_eq!(owner.usage().payload_bytes, 0);
    assert_eq!(owner.usage().tracking_bytes, 0);
    std::fs::remove_dir_all(root).unwrap();
}
