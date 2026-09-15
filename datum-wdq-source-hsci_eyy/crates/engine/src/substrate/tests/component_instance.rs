use super::*;

#[test]
fn resolver_does_not_derive_component_instance_for_exact_symbol_package_match() {
    let root = temp_project_root("component_instance_exact_match");
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
        .expect("project resolves");
    assert!(model.component_instances.is_empty());
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("component_instance_"))
    );
}

#[test]
fn resolver_reports_unmatched_component_instance_symbol() {
    let root = temp_project_root("component_instance_unmatched_symbol");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
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

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    assert!(model.component_instances.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_unmatched_symbol"
            && diagnostic.message.contains(&part_id.to_string())
    }));
}

#[test]
fn resolver_reports_ambiguous_component_instance_join() {
    let root = temp_project_root("component_instance_ambiguous_join");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let extra_package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {
                package_id.to_string(): {
                    "uuid": package_id,
                    "part": part_id,
                    "package": Uuid::new_v5(&project_id, b"package-a"),
                    "reference": "U1",
                    "value": "OLD",
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "layer": 0,
                    "locked": false
                },
                extra_package_id.to_string(): {
                    "uuid": extra_package_id,
                    "part": part_id,
                    "package": Uuid::new_v5(&project_id, b"package-b"),
                    "reference": "U1",
                    "value": "OLD",
                    "position": { "x": 10, "y": 0 },
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

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    assert!(model.component_instances.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_ambiguous_join"
            && diagnostic.message.contains("1 schematic symbols")
            && diagnostic.message.contains("2 board packages")
    }));
}

#[test]
fn resolver_prefers_persisted_component_instance_over_ambiguous_reference_join() {
    let root = temp_project_root("component_instance_persisted_over_ambiguous_join");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let extra_package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {
                package_id.to_string(): {
                    "uuid": package_id,
                    "part": part_id,
                    "package": Uuid::new_v5(&project_id, b"package-a"),
                    "reference": "U1",
                    "value": "OLD",
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "layer": 0,
                    "locked": false
                },
                extra_package_id.to_string(): {
                    "uuid": extra_package_id,
                    "part": part_id,
                    "package": Uuid::new_v5(&project_id, b"package-b"),
                    "reference": "U1",
                    "value": "ALT",
                    "position": { "x": 10, "y": 0 },
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
    write_component_instance_shard(&root, component_instance_id, symbol_id, package_id);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .expect("persisted component instance should resolve");
    assert_eq!(instance.object_revision, ObjectRevision(1));
    assert_eq!(instance.authority, ComponentInstanceAuthority::Authored);
    assert_eq!(instance.placed_symbol_refs, vec![symbol_id]);
    assert_eq!(instance.placed_package_refs, vec![package_id]);
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ComponentInstance
            && shard.authority == SourceShardAuthority::AuthoredDesign
            && shard
                .relative_path
                .ends_with(&format!("{component_instance_id}.json"))
    }));
    assert!(
        model
            .objects
            .get(&component_instance_id)
            .is_some_and(|object| {
                object.domain == "component_instance"
                    && object.kind == "component_instance"
                    && object.object_revision == ObjectRevision(1)
            })
    );
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "component_instance_ambiguous_join")
    );
}

#[test]
fn resolver_rejects_component_instance_filename_mismatch() {
    let root = temp_project_root("component_instance_filename_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let shard_id = Uuid::new_v4();
    let embedded_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_component_instance_shard_at(&root, shard_id, embedded_id, symbol_id, package_id);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with diagnostics");
    assert!(!model.component_instances.contains_key(&embedded_id));
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_filename_mismatch"
            && diagnostic.message.contains(&embedded_id.to_string())
    }));
}

#[test]
fn resolver_rejects_component_instance_missing_refs() {
    let root = temp_project_root("component_instance_missing_refs");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    write_component_instance_shard(&root, component_instance_id, Uuid::new_v4(), Uuid::new_v4());

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with diagnostics");
    assert!(model.component_instances.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_unresolved_ref"
            && diagnostic
                .message
                .contains(&component_instance_id.to_string())
    }));
}

#[test]
fn persisted_component_instance_participates_in_model_revision() {
    let root = temp_project_root("component_instance_model_revision");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let derived = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before persisted shard");
    write_component_instance_shard(&root, component_instance_id, symbol_id, package_id);

    let persisted = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after persisted shard");
    assert_ne!(derived.model_revision, persisted.model_revision);
    assert!(
        persisted
            .component_instances
            .contains_key(&component_instance_id)
    );
}

#[test]
fn journaled_component_instance_rejects_duplicate_or_wrong_kind_refs() {
    let root = temp_project_root("component_instance_invalid_refs");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let component_instance_id = Uuid::new_v4();
    let duplicate_symbol_payload = serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": 0,
        "placed_symbol_refs": [
            { "object_id": symbol_id, "object_revision": 0 },
            { "object_id": symbol_id, "object_revision": 0 }
        ],
        "placed_package_refs": [{ "object_id": package_id, "object_revision": 0 }]
    });
    let duplicate_result = model.commit_journaled(
        &root,
        OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"duplicate-component-instance-ref"),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "reject duplicate component instance refs".to_string(),
            },
            operations: vec![Operation::CreateComponentInstance {
                component_instance_id,
                component_instance: duplicate_symbol_payload,
            }],
        },
    );
    assert!(
        duplicate_result
            .expect_err("duplicate ref should be rejected")
            .to_string()
            .contains("duplicate symbol ref")
    );

    let wrong_kind_payload = serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": 0,
        "placed_symbol_refs": [{ "object_id": package_id, "object_revision": 0 }],
        "placed_package_refs": [{ "object_id": symbol_id, "object_revision": 0 }]
    });
    let wrong_kind_result = model.commit_journaled(
        &root,
        OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"wrong-kind-component-instance-ref"),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "reject wrong-kind component instance refs".to_string(),
            },
            operations: vec![Operation::CreateComponentInstance {
                component_instance_id,
                component_instance: wrong_kind_payload,
            }],
        },
    );
    assert!(
        wrong_kind_result
            .expect_err("wrong-kind ref should be rejected")
            .to_string()
            .contains("must target schematic domain")
    );
}

#[test]
fn journaled_component_instance_accepts_symbol_first_instance_without_board_package() {
    let root = temp_project_root("component_instance_symbol_first");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let component_instance_id = Uuid::new_v4();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"symbol-first-component-instance"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "accept symbol-first component instance".to_string(),
                },
                operations: vec![Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance: serde_json::json!({
                        "uuid": component_instance_id,
                        "object_revision": 0,
                        "placed_symbol_refs": [{
                            "object_id": symbol_id,
                            "object_revision": 0
                        }],
                        "placed_package_refs": []
                    }),
                }],
            },
        )
        .expect("symbol-first component instance should commit");
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .expect("component instance should materialize");
    assert_eq!(instance.part_ref, None);
    assert_eq!(instance.placed_symbol_refs, vec![symbol_id]);
    assert!(instance.placed_package_refs.is_empty());

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should replay symbol-first component instance");
    assert!(
        reopened
            .component_instances
            .contains_key(&component_instance_id)
    );
}

#[test]
fn journaled_component_instance_create_set_delete_undo_redo_and_replay() {
    let root = temp_project_root("component_instance_journaled_ops");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let alternate_package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let mut board_value =
        read_json_value(&root.join("board/board.json")).expect("board should read");
    board_value["packages"][alternate_package_id.to_string()] = serde_json::json!({
        "uuid": alternate_package_id,
        "part": part_id,
        "package": Uuid::new_v5(&project_id, b"package-alt"),
        "reference": "U1",
        "value": "ALT",
        "position": { "x": 10, "y": 0 },
        "rotation": 0,
        "layer": 0,
        "locked": false
    });
    write_json(&root.join("board/board.json"), board_value);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before journaled component instance create");
    let initial_payload =
        component_instance_payload(component_instance_id, 0, symbol_id, package_id);
    let updated_payload =
        component_instance_payload(component_instance_id, 1, symbol_id, alternate_package_id);
    let component_instance_path = root.join(format!(
        ".datum/component_instances/{component_instance_id}.json"
    ));

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-component-instance"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create component instance through substrate".to_string(),
                },
                operations: vec![Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance: initial_payload.clone(),
                }],
            },
        )
        .expect("journaled component instance create should succeed");
    assert!(component_instance_path.exists());
    let created = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after component instance create");
    assert_eq!(
        created
            .component_instances
            .get(&component_instance_id)
            .map(|instance| instance.placed_package_refs.clone()),
        Some(vec![package_id])
    );

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo component instance create".to_string(),
            },
        )
        .expect("undo create should remove component instance");
    assert!(!component_instance_path.exists());
    let undone = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after component instance create undo");
    assert!(
        !undone
            .component_instances
            .contains_key(&component_instance_id)
    );

    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo component instance create".to_string(),
            },
        )
        .expect("redo create should restore component instance");
    assert!(component_instance_path.exists());

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-component-instance"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set component instance through substrate".to_string(),
                },
                operations: vec![Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance: initial_payload.clone(),
                    component_instance: updated_payload.clone(),
                }],
            },
        )
        .expect("journaled component instance set should succeed");
    let updated = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after component instance set");
    assert_eq!(
        updated
            .component_instances
            .get(&component_instance_id)
            .map(|instance| instance.placed_package_refs.clone()),
        Some(vec![alternate_package_id])
    );

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo component instance set".to_string(),
            },
        )
        .expect("undo set should restore previous component instance");
    let restored = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after component instance set undo");
    assert_eq!(
        restored
            .component_instances
            .get(&component_instance_id)
            .map(|instance| instance.placed_package_refs.clone()),
        Some(vec![package_id])
    );

    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo component instance set".to_string(),
            },
        )
        .expect("redo set should restore updated component instance");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-component-instance"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete component instance through substrate".to_string(),
                },
                operations: vec![Operation::DeleteComponentInstance {
                    component_instance_id,
                    component_instance: updated_payload.clone(),
                }],
            },
        )
        .expect("journaled component instance delete should succeed");
    assert!(!component_instance_path.exists());
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo component instance delete".to_string(),
            },
        )
        .expect("undo delete should restore component instance");
    assert!(component_instance_path.exists());

    std::fs::remove_file(&component_instance_path).expect("component instance file should delete");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves from component instance journal replay");
    assert_eq!(
        replayed
            .component_instances
            .get(&component_instance_id)
            .map(|instance| instance.placed_package_refs.clone()),
        Some(vec![alternate_package_id])
    );
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ComponentInstance
            && shard.relative_path
                == format!(".datum/component_instances/{component_instance_id}.json")
            && shard.dirty_state == SourceShardDirtyState::Missing
    }));
}

fn write_component_instance_shard(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
) {
    write_component_instance_shard_at(
        root,
        component_instance_id,
        component_instance_id,
        symbol_id,
        package_id,
    );
}

fn component_instance_payload(
    component_instance_id: Uuid,
    object_revision: u64,
    symbol_id: Uuid,
    package_id: Uuid,
) -> serde_json::Value {
    serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": object_revision,
        "placed_symbol_refs": [{
            "object_id": symbol_id,
            "object_revision": 0
        }],
        "placed_package_refs": [{
            "object_id": package_id,
            "object_revision": 0
        }]
    })
}

fn write_component_instance_shard_at(
    root: &Path,
    shard_id: Uuid,
    embedded_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
) {
    write_json(
        &root.join(format!(".datum/component_instances/{shard_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "component_instance": {
                "uuid": embedded_id,
                "object_revision": 1,
                "placed_symbol_refs": [{
                    "object_id": symbol_id,
                    "object_revision": 0
                }],
                "placed_package_refs": [{
                    "object_id": package_id,
                    "object_revision": 0
                }]
            }
        }),
    );
}
