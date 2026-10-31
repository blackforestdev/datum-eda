//! Migration and alias-resolution tests for the preference repository.
//!
//! Extracted from `tests.rs` at the PM022 ceiling crossing (real ownership: the
//! migration/alias family moves here as its own module). Shared fixtures
//! (`TestDirectory`, `metadata`, `repository`, `ready`, `file_tree`) stay in `tests`
//! as `pub(super)` and are imported here -- the same shape as the backup-restore /
//! downgrade-preservation split.

use std::collections::BTreeMap;

use serde_json::json;

use super::io::{digest_bytes, promote_head, write_generation};
use super::migration::MigrationTransformResult;
use super::model::{RepositoryState, StoredPreferenceValue};
use super::tests::{TestDirectory, file_tree, metadata, ready, repository};
use super::*;
use crate::preferences::{DescriptorRegistry, PreferenceKey, reserved_v1_registry};

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
        Some(&initial),
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
        Some(&initial),
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
fn bare_declared_alias_migrates_to_its_live_identity() {
    // Regression for dat-preferences-bare-alias-migration-mv1k: the active catalog
    // declares the BARE alias `console_duration` for datum.console.feedback_duration.
    // Detection reports MigrationRequired; before the migrate_partition fix, execution
    // rejected the bare spelling as UnknownPreferenceKey instead of migrating it
    // (GP-C04 §6: every declared alias migrates to its one live identity).
    let directory = TestDirectory::new();
    let repository = repository(&directory); // active_v1_registry declares the bare alias
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let mut state = RepositoryState::empty();
    state.user.insert(
        "console_duration".to_owned(),
        StoredPreferenceValue {
            schema_version: 1,
            value: json!("10s"),
        },
    );
    let generation = write_generation(
        &directory.0,
        &initial.repository_id,
        1,
        Some(&initial),
        "old-datum",
        &state,
        &BTreeMap::new(),
    )
    .unwrap();
    promote_head(&directory.0, &generation).unwrap();

    // Detection recognizes the bare alias as requiring migration.
    match repository.inspect() {
        RepositoryStatus::MigrationRequired { issues, .. } => {
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].key, "console_duration");
        }
        other => panic!("expected migration-required, got {other:?}"),
    }

    // Execution now migrates the bare alias to its one live identity, carrying the value,
    // and planning stays pure.
    let before_plan = file_tree(&directory.0);
    let plan = repository
        .plan_migration(&generation, &MigrationRegistry::default())
        .unwrap();
    assert_eq!(
        file_tree(&directory.0),
        before_plan,
        "planning must not write"
    );
    assert!(plan.changes.iter().any(|change| matches!(
        change,
        MigrationChange::AliasMoved { retired_key, live_key }
            if retired_key == "console_duration"
                && live_key == "datum.console.feedback_duration"
    )));
    let (migrated, _backup) = repository
        .commit_migration(&plan, &metadata("migrate console duration"))
        .unwrap();
    let snapshot = ready(&repository);
    assert_eq!(snapshot.generation, migrated);
    assert!(!snapshot.user.contains_key("console_duration"));
    assert_eq!(
        snapshot
            .user
            .get("datum.console.feedback_duration")
            .map(|stored| &stored.value),
        Some(&json!("10s")),
    );
}
#[test]
fn bare_declared_alias_and_live_key_collide_and_are_refused() {
    // Regression companion: a store holding BOTH the bare alias and its live key is an
    // ambiguous identity; migration refuses the whole operation without writing.
    let directory = TestDirectory::new();
    let repository = repository(&directory);
    let initial = repository.initialize(&metadata("initialize")).unwrap();
    let mut state = RepositoryState::empty();
    for key in ["console_duration", "datum.console.feedback_duration"] {
        state.user.insert(
            key.to_owned(),
            StoredPreferenceValue {
                schema_version: 1,
                value: json!("10s"),
            },
        );
    }
    let generation = write_generation(
        &directory.0,
        &initial.repository_id,
        1,
        Some(&initial),
        "old-datum",
        &state,
        &BTreeMap::new(),
    )
    .unwrap();
    promote_head(&directory.0, &generation).unwrap();
    let before = file_tree(&directory.0);
    assert!(matches!(
        repository.plan_migration(&generation, &MigrationRegistry::default()),
        Err(RepositoryError::AliasCollision(alias)) if alias == "console_duration"
    ));
    assert_eq!(
        before,
        file_tree(&directory.0),
        "a refused plan must not write"
    );
}
