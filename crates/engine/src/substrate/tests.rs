use {super::*, crate::ir::serialization::to_json_deterministic, std::io::Write};
mod artifact_metadata;
mod artifact_metadata_validation;
mod component_instance;
mod component_side;
mod journal_hardening;
mod journal_replay_conflicts;
mod journal_replay_diagnostics;
mod journal_revision_guards;
mod pool_library;
mod private_writer_migration;
mod production_writer_migration;
mod proposal;
mod relationship;
mod schematic_sheet_writer_migration;
mod schematic_text_writer_migration;
mod schematic_writer_migration;
mod source_shard_metadata;
mod zone_fill;
fn write_json(path: &Path, value: serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent dir should create");
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&value).expect("json should serialize"),
    )
    .expect("json should write");
}
fn temp_project_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("datum_project_resolver_{name}_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).expect("temp project root should create");
    root
}
fn write_minimal_project(root: &Path, project_id: Uuid, board_id: Uuid) {
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": project_id,
            "name": "resolver-test",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json"
        }),
    );
    write_json(
        &root.join("schematic/schematic.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"schematic"),
            "sheets": {
                Uuid::new_v5(&project_id, b"sheet").to_string(): "sheets/main.json"
            },
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": []
        }),
    );
    write_json(
        &root.join("schematic/sheets/main.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"sheet"),
            "name": "Main",
            "symbols": {},
            "wires": {},
            "junctions": {},
            "labels": {},
            "buses": {},
            "bus_entries": {},
            "ports": {},
            "noconnects": {},
            "texts": {},
            "drawings": {}
        }),
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {}
        }),
    );
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "rules": []
        }),
    );
}

fn write_project_with_board_package(
    root: &Path,
    project_id: Uuid,
    board_id: Uuid,
    package_id: Uuid,
) {
    write_minimal_project(root, project_id, board_id);
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {
                package_id.to_string(): {
                    "uuid": package_id,
                    "part": Uuid::new_v5(&project_id, b"part"),
                    "package": Uuid::new_v5(&project_id, b"package"),
                    "reference": "U1",
                    "value": "OLD",
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "layer": 0,
                    "locked": false
                }
            },
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {}
        }),
    );
}

fn write_project_with_symbol_and_package(
    root: &Path,
    project_id: Uuid,
    board_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
    part_id: Uuid,
) {
    write_project_with_board_package(root, project_id, board_id, package_id);
    write_json(
        &root.join("schematic/sheets/main.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"sheet"),
            "name": "Main",
            "symbols": {
                symbol_id.to_string(): {
                    "uuid": symbol_id,
                    "part": part_id,
                    "entity": Uuid::new_v5(&project_id, b"entity"),
                    "gate": Uuid::new_v5(&project_id, b"gate"),
                    "lib_id": "test:R",
                    "reference": "U1",
                    "value": "OLD",
                    "fields": [],
                    "pins": [],
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "mirrored": false,
                    "unit_selection": null,
                    "display_mode": "LibraryDefault",
                    "pin_overrides": [],
                    "hidden_power_behavior": "SourceDefinedImplicit"
                }
            },
            "wires": {},
            "junctions": {},
            "labels": {},
            "buses": {},
            "bus_entries": {},
            "ports": {},
            "noconnects": {},
            "texts": {},
            "drawings": {}
        }),
    );
    let mut board_value =
        read_json_value(&root.join("board/board.json")).expect("board should read");
    board_value["packages"][package_id.to_string()]["part"] =
        serde_json::Value::String(part_id.to_string());
    write_json(&root.join("board/board.json"), board_value);
}

#[test]
fn resolver_produces_stable_model_revision_for_unchanged_project() {
    let root = temp_project_root("stable");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let first = ProjectResolver::new(&root)
        .resolve()
        .expect("first resolve should succeed");
    let second = ProjectResolver::new(&root)
        .resolve()
        .expect("second resolve should succeed");

    assert_eq!(first.model_revision, second.model_revision);
    assert_eq!(first.source_shards, second.source_shards);
    assert!(first.objects.contains_key(&project_id));
    assert!(first.objects.contains_key(&board_id));
}

#[test]
fn resolver_model_revision_changes_when_authored_shard_changes() {
    let root = temp_project_root("revision_changes");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let first = ProjectResolver::new(&root)
        .resolve()
        .expect("first resolve should succeed");
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "rules": [{"uuid": Uuid::new_v4(), "object_revision": 1}]
        }),
    );
    let second = ProjectResolver::new(&root)
        .resolve()
        .expect("second resolve should succeed");

    assert_ne!(first.model_revision, second.model_revision);
}

#[test]
fn resolver_reports_missing_required_shard_without_accepted_truth_panic() {
    let root = temp_project_root("missing_shard");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    std::fs::remove_file(root.join("board/board.json")).expect("board shard should remove");

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should produce diagnostics");

    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "missing_required_shard"
            && diagnostic
                .path
                .as_ref()
                .is_some_and(|path| path.ends_with("board/board.json"))
    }));
}

#[test]
fn resolver_builds_component_instance_from_matching_symbol_and_package() {
    let root = temp_project_root("component_instance_join");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    assert_eq!(model.component_instances.len(), 1);
    let instance = model
        .component_instances
        .values()
        .next()
        .expect("component instance should exist");
    assert_eq!(instance.object_revision, ObjectRevision(0));
    assert_eq!(instance.placed_symbol_refs, vec![symbol_id]);
    assert_eq!(instance.placed_package_refs, vec![package_id]);
}

#[test]
fn resolver_does_not_join_component_instance_on_reference_only() {
    let root = temp_project_root("component_instance_reference_only");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root,
        project_id,
        board_id,
        symbol_id,
        package_id,
        Uuid::new_v4(),
    );
    let mut board_value =
        read_json_value(&root.join("board/board.json")).expect("board should read");
    board_value["packages"][package_id.to_string()]["part"] =
        serde_json::Value::String(Uuid::new_v4().to_string());
    write_json(&root.join("board/board.json"), board_value);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    assert!(model.component_instances.is_empty());
}

#[test]
fn resolver_does_not_join_component_instance_on_part_only() {
    let root = temp_project_root("component_instance_part_only");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let mut board_value =
        read_json_value(&root.join("board/board.json")).expect("board should read");
    board_value["packages"][package_id.to_string()]["reference"] =
        serde_json::Value::String("U2".to_string());
    write_json(&root.join("board/board.json"), board_value);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    assert!(model.component_instances.is_empty());
}

#[test]
fn commit_bumps_object_revision_and_records_transaction_metadata() {
    let root = temp_project_root("commit_bump");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let report = model
        .commit(OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"commit-bump"),
            expected_model_revision: Some(before.clone()),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "prove commit metadata".to_string(),
            },
            operations: vec![Operation::BumpObjectRevision {
                object_id: board_id,
            }],
        })
        .expect("commit should succeed");

    assert_ne!(model.model_revision, before);
    assert_eq!(model.objects[&board_id].object_revision, ObjectRevision(1));
    assert_eq!(report.journal_len, 1);
    assert_eq!(model.journal.len(), 1);
    assert_eq!(report.transaction.before_model_revision, before);
    assert_eq!(
        report.transaction.after_model_revision,
        model.model_revision
    );
    assert_eq!(report.transaction.diff.modified, vec![board_id]);
    assert_eq!(report.transaction.provenance.source, CommitSource::Test);
}

#[test]
fn commit_rejects_stale_expected_model_revision_without_mutation() {
    let root = temp_project_root("commit_stale");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let error = model
        .commit(OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"commit-stale"),
            expected_model_revision: Some(ModelRevision("stale".to_string())),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "prove stale guard".to_string(),
            },
            operations: vec![Operation::BumpObjectRevision {
                object_id: board_id,
            }],
        })
        .expect_err("stale commit should fail");

    assert!(error.to_string().contains("model revision mismatch"));
    assert_eq!(model.model_revision, before);
    assert_eq!(model.objects[&board_id].object_revision, ObjectRevision(0));
    assert!(model.journal.is_empty());
}

#[test]
fn commit_journaled_rejects_stale_revision_without_staging_or_mutation() {
    let root = temp_project_root("commit_journaled_stale");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.clone();
    let batch_id = Uuid::new_v5(&project_id, b"journaled-stale");

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id,
                expected_model_revision: Some(ModelRevision("stale".to_string())),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove stale journaled guard".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect_err("stale journaled commit should fail");

    assert!(error.to_string().contains("model revision mismatch"));
    assert_eq!(model, before);
    assert!(!transaction_journal_path(&root).exists());
    assert!(
        !root
            .join(".datum/stage")
            .join(batch_id.to_string())
            .exists()
    );
}

#[test]
fn commit_journaled_persists_transaction_and_resolve_replays_it() {
    let root = temp_project_root("commit_journaled");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"commit-journaled"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove durable journal append".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");

    let journal_path = transaction_journal_path(&root);
    assert!(journal_path.exists());
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    assert_eq!(reopened.journal.len(), 1);
    assert_eq!(
        reopened.journal[0].transaction_id,
        report.transaction.transaction_id
    );
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(1)
    );
    assert_eq!(
        reopened.model_revision,
        report.transaction.after_model_revision
    );
}

#[test]
fn commit_journaled_sets_board_package_value_and_promotes_board_shard() {
    let root = temp_project_root("commit_set_package_value");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-package-value"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove board shard promotion".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");

    let board_value = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "NEW"
    );
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    assert_eq!(
        reopened.objects[&package_id].object_revision,
        ObjectRevision(1)
    );
    assert_eq!(
        reopened.model_revision,
        report.transaction.after_model_revision
    );
    assert_eq!(report.transaction.diff.modified, vec![package_id]);
    assert_eq!(
        report.transaction.inverse_operations,
        vec![Operation::SetBoardPackageValue {
            package_id,
            value: "OLD".to_string(),
        }]
    );
}

#[test]
fn resolver_replays_board_package_value_when_promoted_shard_is_stale() {
    let root = temp_project_root("commit_replay_value_over_stale_shard");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"replay-value-over-stale-shard"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove journal replay restores authored shard value".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal over stale shard");
    assert_eq!(
        reopened.model_revision,
        report.transaction.after_model_revision
    );
    assert_eq!(
        reopened.objects[&package_id].object_revision,
        ObjectRevision(1)
    );
}

#[test]
fn resolver_replays_duplicate_transaction_id_once() {
    let root = temp_project_root("commit_duplicate_replay");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"duplicate-replay"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove duplicate replay idempotency".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");
    let line = format!(
        "{}\n",
        to_json_deterministic(&report.transaction).expect("transaction should serialize")
    );
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(line.as_bytes())
        .expect("duplicate transaction should append");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should skip duplicate");
    assert_eq!(reopened.journal.len(), 1);
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "journal_duplicate_transaction_skipped" })
    );
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(1)
    );
}
