use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;
use uuid::Uuid;

use super::io::{
    copy_unknown_bytes, digest_bytes, generation_dir, next_generation_number, promote_head,
    write_generation,
};
use super::migration::MigrationTransformResult;
use super::model::{RepositoryState, StoredPreferenceValue};
use super::*;
use crate::preferences::{
    DescriptorRegistry, PreferenceKey, active_v1_registry, reserved_v1_registry,
};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "datum-preference-repository-test-{}",
            Uuid::new_v4()
        )))
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn metadata(reason: &str) -> MutationMetadata {
    MutationMetadata {
        actor: "test-user".to_owned(),
        reason: reason.to_owned(),
        writer_instance: "test-process".to_owned(),
        audit: None,
    }
}

fn repository(directory: &TestDirectory) -> PreferenceRepository {
    PreferenceRepository::new(&directory.0, active_v1_registry())
}

fn ready(repository: &PreferenceRepository) -> RepositorySnapshot {
    match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => panic!("expected ready repository, got {other:?}"),
    }
}

fn reduced_motion() -> PreferenceKey {
    PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap()
}

#[test]
fn immutable_generations_use_expected_head_and_validate_values() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    assert_eq!(initial.generation, 0);
    let initial_manifest = std::fs::read(generation_dir(&directory.0, 0).join("manifest.json"))
        .expect("initial manifest");

    let generation = repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("enable reduced motion"),
        )
        .unwrap();
    assert_eq!(generation.generation, 1);
    assert_eq!(
        ready(&repository)
            .value(PreferencePartition::User, &reduced_motion())
            .map(|stored| &stored.value),
        Some(&json!(true))
    );
    assert_eq!(
        std::fs::read(generation_dir(&directory.0, 0).join("manifest.json")).unwrap(),
        initial_manifest,
        "a successor must not rewrite its parent generation"
    );

    assert!(matches!(
        repository.commit_mutations(
            &initial,
            &[PreferenceMutation::Remove {
                partition: PreferencePartition::User,
                key: reduced_motion(),
            }],
            &metadata("stale removal"),
        ),
        Err(RepositoryError::ExpectedGenerationMismatch)
    ));
    assert!(matches!(
        repository.commit_mutations(
            &generation,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!("not-a-boolean"),
            }],
            &metadata("invalid value"),
        ),
        Err(RepositoryError::InvalidPreferenceValue(_))
    ));
    assert_eq!(ready(&repository).generation, generation);
}

#[test]
fn unknown_bytes_survive_mutation_backup_restore_and_reverse_restore() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let exact = vec![0, 255, b'{', b'\n', 17, 0, 99];
    let with_unknown = repository
        .commit_mutations(
            &initial,
            &[
                PreferenceMutation::Set {
                    partition: PreferencePartition::User,
                    key: reduced_motion(),
                    value: json!(true),
                },
                PreferenceMutation::PutUnknown(UnknownEnvelope {
                    identity: "vendor.future.setting".to_owned(),
                    provider: Some("vendor.example".to_owned()),
                    scope: "user".to_owned(),
                    source_version: "9".to_owned(),
                    required_extension: Some("vendor.future".to_owned()),
                    ordering: Some("after:vendor.base".to_owned()),
                    exact_bytes: exact.clone(),
                }),
                PreferenceMutation::PutUnknown(UnknownEnvelope {
                    identity: "vendor.future.setting-copy".to_owned(),
                    provider: Some("vendor.example".to_owned()),
                    scope: "user".to_owned(),
                    source_version: "9".to_owned(),
                    required_extension: Some("vendor.future".to_owned()),
                    ordering: None,
                    exact_bytes: exact.clone(),
                }),
            ],
            &metadata("store known and unknown"),
        )
        .unwrap();
    let snapshot = ready(&repository);
    assert_eq!(
        repository
            .read_unknown_exact(&snapshot, "vendor.future.setting")
            .unwrap(),
        exact
    );
    assert_eq!(snapshot.receipts.len(), 2);

    let backup = repository
        .create_exact_backup(&with_unknown, &metadata("backup true state"))
        .unwrap();
    let changed = repository
        .commit_mutations(
            &with_unknown,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(false),
            }],
            &metadata("disable reduced motion"),
        )
        .unwrap();
    let plan = repository.plan_restore(&changed, &backup).unwrap();
    assert!(
        plan.changed_keys
            .contains(&"user:datum.accessibility.reduced_motion".to_owned())
    );
    let restored = repository
        .commit_restore(&plan, &metadata("restore true state"))
        .unwrap();
    let snapshot = ready(&repository);
    assert_eq!(snapshot.generation, restored.generation);
    assert_eq!(
        snapshot
            .value(PreferencePartition::User, &reduced_motion())
            .map(|stored| &stored.value),
        Some(&json!(true))
    );
    assert_eq!(
        repository
            .read_unknown_exact(&snapshot, "vendor.future.setting")
            .unwrap(),
        exact
    );
    assert_eq!(snapshot.receipts.len(), 4);

    let reverse = repository
        .plan_restore(&restored.generation, &restored.displaced_backup)
        .unwrap();
    let reversed = repository
        .commit_restore(&reverse, &metadata("reverse restore"))
        .unwrap();
    assert_eq!(
        ready(&repository)
            .value(PreferencePartition::User, &reduced_motion())
            .map(|stored| &stored.value),
        Some(&json!(false))
    );
    assert!(matches!(
        repository.commit_restore(&plan, &metadata("stale preview")),
        Err(RepositoryError::RestorePreviewStale)
    ));
    assert_eq!(ready(&repository).generation, reversed.generation);
}

fn theme_v1_to_v2(value: &serde_json::Value) -> MigrationTransformResult {
    MigrationTransformResult::Value(value.clone())
}

#[test]
fn migration_plan_is_pure_moves_one_alias_and_writes_a_pre_migration_backup() {
    let directory = TestDirectory::new();
    let source_registry = reserved_v1_registry();
    let live_key = PreferenceKey::parse("datum.schematic.drawing_theme").unwrap();
    let mut descriptor = source_registry.get(&live_key).unwrap().clone();
    descriptor.schema_version = 2;
    let mut registry = DescriptorRegistry::default();
    registry.register(descriptor).unwrap();
    let repository = PreferenceRepository::new(&directory.0, registry);
    let initial = repository.initialize(&metadata("initialize")).unwrap();

    let mut state = RepositoryState::empty();
    state.user.insert(
        "datum.schematic.theme".to_owned(),
        StoredPreferenceValue {
            schema_version: 1,
            value: json!("dark"),
        },
    );
    let migration_unknown = vec![9, 0, 8, 255, 7];
    state.unknown_envelopes.insert(
        "future.theme.metadata".to_owned(),
        UnknownEnvelopeRef {
            identity: "future.theme.metadata".to_owned(),
            provider: None,
            scope: "user".to_owned(),
            source_version: "3".to_owned(),
            required_extension: None,
            ordering: None,
            payload_digest: digest_bytes(&migration_unknown),
            payload_len: migration_unknown.len() as u64,
        },
    );
    let unknown_bytes = BTreeMap::from([(
        "future.theme.metadata".to_owned(),
        migration_unknown.clone(),
    )]);
    let generation = write_generation(
        &directory.0,
        &initial.repository_id,
        1,
        Some(0),
        "old-datum",
        &state,
        &unknown_bytes,
    )
    .unwrap();
    promote_head(&directory.0, &generation).unwrap();
    match repository.inspect() {
        RepositoryStatus::MigrationRequired { issues, .. } => {
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].key, "datum.schematic.theme");
        }
        other => panic!("expected retained migration state, got {other:?}"),
    }

    let mut migrations = MigrationRegistry::default();
    migrations
        .register(PreferenceMigration {
            identity: "drawing-theme-v1-v2".to_owned(),
            key: live_key.clone(),
            from_schema_version: 1,
            to_schema_version: 2,
            reversible: true,
            transform: theme_v1_to_v2,
        })
        .unwrap();
    let before_plan = file_tree(&directory.0);
    let plan = repository.plan_migration(&generation, &migrations).unwrap();
    assert_eq!(
        file_tree(&directory.0),
        before_plan,
        "planning must not write"
    );
    assert!(plan.changes.iter().any(|change| matches!(
        change,
        MigrationChange::AliasMoved { retired_key, live_key }
            if retired_key == "datum.schematic.theme"
                && live_key == "datum.schematic.drawing_theme"
    )));
    assert!(plan.changes.iter().any(|change| matches!(
        change,
        MigrationChange::ValueMigrated { migration_identity, .. }
            if migration_identity == "drawing-theme-v1-v2"
    )));
    let (migrated, backup) = repository
        .commit_migration(&plan, &metadata("migrate drawing theme"))
        .unwrap();
    let snapshot = ready(&repository);
    assert_eq!(snapshot.generation, migrated);
    assert!(!snapshot.user.contains_key("datum.schematic.theme"));
    assert_eq!(
        snapshot
            .user
            .get("datum.schematic.drawing_theme")
            .map(|stored| stored.schema_version),
        Some(2)
    );
    assert_eq!(
        repository
            .read_unknown_exact(&snapshot, "future.theme.metadata")
            .unwrap(),
        migration_unknown
    );
    assert!(
        directory
            .0
            .join("backups")
            .join(backup.backup_id)
            .join("manifest.json")
            .is_file()
    );
}

#[test]
fn unreadable_generation_is_not_repaired_and_recovery_is_explicit() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let generation = repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("write current"),
        )
        .unwrap();
    let manifest = generation_dir(&directory.0, generation.generation).join("manifest.json");
    let mut corrupted = std::fs::read(&manifest).unwrap();
    corrupted.extend_from_slice(b"truncated-garbage");
    std::fs::write(&manifest, &corrupted).unwrap();
    let before_inspect = file_tree(&directory.0);

    let evidence = match repository.inspect() {
        RepositoryStatus::Unreadable(evidence) => evidence,
        other => panic!("expected unreadable repository, got {other:?}"),
    };
    assert_eq!(file_tree(&directory.0), before_inspect);
    assert_eq!(evidence.exact_bytes, corrupted);
    assert_eq!(evidence.recovery_candidates, vec![initial.clone()]);
    let preserved = repository
        .preserve_unreadable(&evidence, "recovery-process")
        .unwrap();
    assert_eq!(std::fs::read(&preserved).unwrap(), corrupted);
    assert_eq!(std::fs::read(&manifest).unwrap(), corrupted);

    let recovered = repository
        .restore_recovery_candidate(&evidence, &initial, &metadata("restore last good"))
        .unwrap();
    assert_eq!(recovered.generation, 2);
    assert!(ready(&repository).user.is_empty());
    assert_eq!(std::fs::read(manifest).unwrap(), corrupted);
}

#[test]
fn missing_head_with_retained_generations_is_diagnostic_not_a_new_repository() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    std::fs::remove_file(directory.0.join("head.json")).unwrap();
    let before = file_tree(&directory.0);
    let evidence = match repository.inspect() {
        RepositoryStatus::Unreadable(evidence) => evidence,
        other => panic!("expected missing-head diagnostic, got {other:?}"),
    };
    assert_eq!(before, file_tree(&directory.0));
    assert_eq!(evidence.recovery_candidates, vec![initial.clone()]);
    assert!(matches!(
        repository.initialize(&metadata("unsafe reinitialize")),
        Err(RepositoryError::ExpectedGenerationMismatch)
    ));
    let recovered = repository
        .restore_recovery_candidate(&evidence, &initial, &metadata("restore missing head"))
        .unwrap();
    assert_eq!(recovered.generation, 1);
    assert!(matches!(repository.inspect(), RepositoryStatus::Ready(_)));
}

#[test]
fn corrupt_opaque_payload_and_incomplete_backup_are_refused_without_writes() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let exact = vec![1, 3, 3, 7, 0, 255];
    let generation = repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::PutUnknown(UnknownEnvelope {
                identity: "future.binary".to_owned(),
                provider: None,
                scope: "user".to_owned(),
                source_version: "1".to_owned(),
                required_extension: None,
                ordering: None,
                exact_bytes: exact,
            })],
            &metadata("store future bytes"),
        )
        .unwrap();
    let backup = repository
        .create_exact_backup(&generation, &metadata("backup future bytes"))
        .unwrap();
    let changed = repository
        .commit_mutations(
            &generation,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("advance current head"),
        )
        .unwrap();
    let envelope = ready(&repository)
        .unknown_envelopes
        .get("future.binary")
        .unwrap()
        .clone();
    let backup_payload = super::io::opaque_path(
        &directory.0.join("backups").join(&backup.backup_id),
        &envelope,
    );
    std::fs::write(&backup_payload, b"damaged backup").unwrap();
    let before_plan = file_tree(&directory.0);
    assert!(matches!(
        repository.plan_restore(&changed, &backup),
        Err(RepositoryError::BackupIncomplete(_))
    ));
    assert_eq!(before_plan, file_tree(&directory.0));
    assert_eq!(ready(&repository).generation, changed);

    let live_payload =
        super::io::opaque_path(&generation_dir(&directory.0, changed.generation), &envelope);
    let damaged = b"damaged live payload".to_vec();
    std::fs::write(&live_payload, &damaged).unwrap();
    let before_inspect = file_tree(&directory.0);
    let evidence = match repository.inspect() {
        RepositoryStatus::Unreadable(evidence) => evidence,
        other => panic!("expected opaque-integrity diagnostic, got {other:?}"),
    };
    assert_eq!(evidence.source_path, live_payload);
    assert_eq!(evidence.exact_bytes, damaged);
    assert_eq!(before_inspect, file_tree(&directory.0));
}

#[test]
fn alias_collision_is_retained_and_refused_without_migration_side_effects() {
    let directory = TestDirectory::new();
    let source_registry = reserved_v1_registry();
    let live_key = PreferenceKey::parse("datum.schematic.drawing_theme").unwrap();
    let descriptor = source_registry.get(&live_key).unwrap().clone();
    let mut registry = DescriptorRegistry::default();
    registry.register(descriptor).unwrap();
    let repository = PreferenceRepository::new(&directory.0, registry);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let mut state = RepositoryState::empty();
    for key in ["datum.schematic.theme", "datum.schematic.drawing_theme"] {
        state.user.insert(
            key.to_owned(),
            StoredPreferenceValue {
                schema_version: 1,
                value: json!("dark"),
            },
        );
    }
    let generation = write_generation(
        &directory.0,
        &initial.repository_id,
        1,
        Some(0),
        "old-datum",
        &state,
        &BTreeMap::new(),
    )
    .unwrap();
    promote_head(&directory.0, &generation).unwrap();
    let before = file_tree(&directory.0);
    assert!(matches!(
        repository.plan_migration(&generation, &MigrationRegistry::default()),
        Err(RepositoryError::AliasCollision(alias))
            if alias == "datum.schematic.theme"
    ));
    assert_eq!(before, file_tree(&directory.0));
}

#[test]
fn unpublished_generation_and_writer_contention_never_replace_the_head() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let snapshot = ready(&repository);
    let state = snapshot.state();
    let unknown = copy_unknown_bytes(&directory.0, &snapshot).unwrap();
    let orphan = write_generation(
        &directory.0,
        &initial.repository_id,
        1,
        Some(0),
        "interrupted-process",
        &state,
        &unknown,
    )
    .unwrap();
    assert_eq!(ready(&repository).generation, initial);
    assert_eq!(next_generation_number(&directory.0), 2);

    let held_lease = super::io::WriterLease::acquire(&directory.0, "other-process").unwrap();
    assert!(matches!(
        repository.commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("contended write"),
        ),
        Err(RepositoryError::WriterLeaseUnavailable)
    ));
    drop(held_lease);
    let committed = repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("write after interruption"),
        )
        .unwrap();
    assert_eq!(committed.generation, 2);
    assert!(generation_dir(&directory.0, orphan.generation).is_dir());
    assert!(directory.0.join("writer.lock").is_file());
}

#[test]
fn initial_mutation_is_the_only_content_of_generation_zero() {
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let generation = repository
        .initialize_with_mutations(
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion(),
                value: json!(true),
            }],
            &metadata("first write"),
        )
        .unwrap();
    assert_eq!(generation.generation, 0);
    let snapshot = ready(&repository);
    assert_eq!(snapshot.generation, generation);
    assert_eq!(
        snapshot
            .value(PreferencePartition::User, &reduced_motion())
            .map(|stored| &stored.value),
        Some(&json!(true))
    );
    assert_eq!(snapshot.receipts.len(), 1);
    assert_eq!(snapshot.receipts[0].operation, "CommitPreferenceMutation");
}

fn file_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut result);
    }
    result
}
