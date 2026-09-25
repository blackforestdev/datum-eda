use super::*;
use std::sync::Arc;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = temp_project_root("read-cache");
        write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn project_read_cache_reuses_only_identical_content_not_mtime_or_length() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let mut cache = ProjectReadCache::default();
    let first = cache.resolve(root).unwrap();
    assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);
    assert_eq!(*first, ProjectResolver::new(root).resolve().unwrap());
    assert!(Arc::ptr_eq(&first, &cache.resolve(root).unwrap()));

    let path = root.join("project.json");
    let metadata = std::fs::metadata(&path).unwrap();
    let original = std::fs::read_to_string(&path).unwrap();
    let changed = original.replace("resolver-test", "external-edit");
    assert_eq!(changed.len(), original.len());
    std::fs::write(&path, changed).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(metadata.modified().unwrap()))
        .unwrap();
    let second = cache.resolve(root).unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(second.project.name, "external-edit");
    assert_eq!(*second, ProjectResolver::new(root).resolve().unwrap());
    assert!(Arc::ptr_eq(&second, &cache.resolve(root).unwrap()));
}

#[test]
fn project_read_cache_preserves_added_removed_and_corrupt_input_diagnostics() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let mut cache = ProjectReadCache::default();
    let first = cache.resolve(root).unwrap();
    let path = root.join(".datum/journal/transactions.jsonl");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "not a transaction\n").unwrap();
    let corrupt = cache.resolve(root).unwrap();
    assert!(!Arc::ptr_eq(&first, &corrupt));
    assert_eq!(*corrupt, ProjectResolver::new(root).resolve().unwrap());
    assert!(
        corrupt
            .diagnostics
            .iter()
            .any(|d| d.code == "journal_parse_error")
    );
    assert!(!Arc::ptr_eq(&corrupt, &cache.resolve(root).unwrap()));
    std::fs::remove_file(path).unwrap();
    let repaired = cache.resolve(root).unwrap();
    assert_eq!(*repaired, ProjectResolver::new(root).resolve().unwrap());
    assert!(repaired.diagnostics.is_empty());
    std::fs::remove_file(root.join("board/board.json")).unwrap();
    assert_eq!(
        *cache.resolve(root).unwrap(),
        ProjectResolver::new(root).resolve().unwrap()
    );
    std::fs::write(root.join("project.json"), "{").unwrap();
    assert!(cache.resolve(root).is_err());
}

#[test]
fn project_read_cache_bypasses_external_pools_symlinks_and_pending_recovery() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let mut cache = ProjectReadCache::default();
    let assert_bypass = |cache: &mut ProjectReadCache| {
        let a = cache.resolve(root).unwrap();
        let b = cache.resolve(root).unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
        assert_eq!(*b, ProjectResolver::new(root).resolve().unwrap());
    };
    let link = root.join("unrelated-link");
    std::os::unix::fs::symlink(root.join("board/board.json"), &link).unwrap();
    assert_bypass(&mut cache);
    std::fs::remove_file(link).unwrap();
    let stage = root.join(".datum/revision/v1/stage/incomplete");
    std::fs::create_dir_all(&stage).unwrap();
    assert_bypass(&mut cache);
    std::fs::remove_dir(stage).unwrap();
    let lock = root.join(".datum/revision/write.lock");
    std::fs::write(&lock, "pending").unwrap();
    assert_bypass(&mut cache);
    std::fs::remove_file(lock).unwrap();
    let path = root.join("project.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    manifest["pools"] = serde_json::json!([{"path": "../external-pool"}]);
    write_json(&path, manifest);
    assert_bypass(&mut cache);
}

#[test]
fn project_read_cache_switches_roots_and_revalidates_journal_cursor() {
    let first = Fixture::new();
    let second = Fixture::new();
    let mut cache = ProjectReadCache::default();
    let a = cache.resolve(&first.0).unwrap();
    let b = cache.resolve(&second.0).unwrap();
    assert_ne!(a.project.project_id, b.project.project_id);
    assert_eq!(*b, ProjectResolver::new(&second.0).resolve().unwrap());
    let a2 = cache.resolve(&first.0).unwrap();
    assert!(!Arc::ptr_eq(&a, &a2));
    let path = first.0.join(".datum/journal/cursor.json");
    write_json(&path, serde_json::json!({"applied_transaction_count": 99}));
    let changed = cache.resolve(&first.0).unwrap();
    assert!(!Arc::ptr_eq(&a2, &changed));
    assert_eq!(*changed, ProjectResolver::new(&first.0).resolve().unwrap());
}

#[test]
fn project_read_cache_invalidates_replaced_identical_bytes_and_releases_old_model() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let mut cache = ProjectReadCache::default();
    let first = cache.resolve(root).unwrap();
    let old = Arc::downgrade(&first);
    let path = root.join("project.json");
    let metadata = std::fs::metadata(&path).unwrap();
    let original = std::fs::read(&path).unwrap();
    let replacement = root.join("replacement.json");
    std::fs::write(&replacement, original).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&replacement)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(metadata.modified().unwrap()))
        .unwrap();
    std::fs::rename(replacement, path).unwrap();
    let second = cache.resolve(root).unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(*first, *second);
    drop(first);
    assert!(old.upgrade().is_none());
    assert!(Arc::ptr_eq(&second, &cache.resolve(root).unwrap()));
}

#[test]
fn project_read_cache_preserves_join_diagnostics_and_validated_journal_reuse() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let package_id = Uuid::new_v4();
    write_project_with_board_package(root, Uuid::new_v4(), Uuid::new_v4(), package_id);
    let mut cache = ProjectReadCache::default();
    let first = cache.resolve(root).unwrap();
    assert!(
        first
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_unmatched_package")
    );
    assert_eq!(*first, ProjectResolver::new(root).resolve().unwrap());
    assert!(Arc::ptr_eq(&first, &cache.resolve(root).unwrap()));
    let mut writer = ProjectResolver::new(root).resolve().unwrap();
    writer
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(writer.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "external writer invalidates validated snapshot".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .unwrap();
    let changed = cache.resolve(root).unwrap();
    assert!(!Arc::ptr_eq(&first, &changed));
    assert_eq!(*changed, ProjectResolver::new(root).resolve().unwrap());
    assert_eq!(changed.journal.len(), 1);
    assert!(Arc::ptr_eq(&changed, &cache.resolve(root).unwrap()));
}

#[test]
fn project_read_cache_bypasses_input_retention_overflow_without_rejecting_project() {
    let fixture = Fixture::new();
    let root = &fixture.0;
    let mut cache = ProjectReadCache::default();
    let first = cache.resolve(root).unwrap();
    let extra = root.join("large-unrelated-input");
    std::fs::File::create(&extra)
        .unwrap()
        .set_len(8 * 1024 * 1024)
        .unwrap();
    let a = cache.resolve(root).unwrap();
    let b = cache.resolve(root).unwrap();
    assert!(!Arc::ptr_eq(&first, &a));
    assert!(!Arc::ptr_eq(&a, &b));
    assert_eq!(*a, *b);
    assert_eq!(*a, ProjectResolver::new(root).resolve().unwrap());
    std::fs::remove_file(extra).unwrap();
    let small = cache.resolve(root).unwrap();
    assert!(Arc::ptr_eq(&small, &cache.resolve(root).unwrap()));
}
