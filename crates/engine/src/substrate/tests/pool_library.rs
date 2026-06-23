use super::*;

fn write_project_with_pool(root: &Path, project_id: Uuid, board_id: Uuid) {
    write_minimal_project(root, project_id, board_id);
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": project_id,
            "name": "pool-library-test",
            "pools": [{ "path": "pool", "priority": 1 }],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json"
        }),
    );
}

fn minimal_part(part_id: Uuid, entity_id: Uuid, package_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": part_id,
        "entity": entity_id,
        "package": package_id,
        "pad_map": {},
        "mpn": "TEST-PART",
        "manufacturer": "Datum",
        "value": "1k",
        "description": "test part",
        "datasheet": "",
        "parametric": {},
        "orderable_mpns": [],
        "tags": [],
        "lifecycle": "Active",
        "base": null,
        "behavioural_models": []
    })
}

fn spice_attachment(attachment_id: Uuid, model_id: Uuid, sha256: &str) -> serde_json::Value {
    serde_json::json!({
        "uuid": attachment_id,
        "model_uuid": model_id,
        "role": "Spice",
        "dialect": "Ngspice",
        "model_names": ["TEST_MODEL"],
        "encrypted": false,
        "encryption_scheme": null,
        "provenance": {
            "source": "vendor/test.lib",
            "vendor": "Datum",
            "fetched_at": null,
            "sha256": sha256
        },
        "format_metadata": {
            "kind": "spice",
            "ngspice_validates": true
        }
    })
}

#[test]
fn resolver_discovers_native_pool_library_directories() {
    let root = temp_project_root("pool_library_resolver");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let footprint_id = Uuid::new_v4();
    let pin_pad_map_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);
    write_json(
        &root.join(format!("pool/units/{unit_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "uuid": unit_id,
            "name": "R",
            "manufacturer": "",
            "pins": {},
            "tags": []
        }),
    );
    write_json(
        &root.join(format!("pool/symbols/{symbol_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "uuid": symbol_id,
            "name": "Resistor",
            "unit": unit_id
        }),
    );
    write_json(
        &root.join(format!("pool/footprints/{footprint_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "uuid": footprint_id,
            "name": "R_0603",
            "pads": {},
            "courtyard": { "points": [] },
            "silkscreen": [],
            "models_3d": [],
            "tags": []
        }),
    );
    write_json(
        &root.join(format!("pool/pin_pad_maps/{pin_pad_map_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "uuid": pin_pad_map_id,
            "name": "R_0603_map",
            "part": Uuid::new_v4(),
            "footprint": footprint_id,
            "mappings": {},
            "tags": []
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    for (object_id, relative_path, kind) in [
        (unit_id, format!("pool/units/{unit_id}.json"), "units"),
        (
            symbol_id,
            format!("pool/symbols/{symbol_id}.json"),
            "symbols",
        ),
        (
            footprint_id,
            format!("pool/footprints/{footprint_id}.json"),
            "footprints",
        ),
        (
            pin_pad_map_id,
            format!("pool/pin_pad_maps/{pin_pad_map_id}.json"),
            "pin_pad_maps",
        ),
    ] {
        assert!(model.source_shards.iter().any(|shard| {
            shard.kind == SourceShardKind::Pool && shard.relative_path == relative_path
        }));
        let object = model
            .objects
            .get(&object_id)
            .unwrap_or_else(|| panic!("missing pool object {object_id}"));
        assert_eq!(object.domain, "pool");
        assert_eq!(object.kind, kind);
    }
}

#[test]
fn journaled_pool_part_model_attach_detach_are_replayable_and_undoable() {
    let root = temp_project_root("pool_part_model_attach_detach");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let attachment_id = Uuid::new_v4();
    let model_id = Uuid::new_v4();
    let relative_path = format!("pool/parts/{part_id}.json");
    let attachment = spice_attachment(attachment_id, model_id, "abc123");
    write_project_with_pool(&root, project_id, board_id);
    write_json(
        &root.join(&relative_path),
        minimal_part(part_id, entity_id, package_id),
    );

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let attach_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "attach native pool part model".to_string(),
                },
                operations: vec![Operation::AttachPoolPartModel {
                    part_id,
                    relative_path: relative_path.clone(),
                    previous_attachments: vec![],
                    attachments: vec![attachment.clone()],
                }],
            },
        )
        .expect("model attachment should commit");
    assert_eq!(
        attach_report.transaction.inverse_operations,
        vec![Operation::DetachPoolPartModel {
            part_id,
            relative_path: relative_path.clone(),
            previous_attachments: vec![attachment.clone()],
            attachments: vec![],
        }]
    );
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(
        written["behavioural_models"][0]["uuid"],
        attachment_id.to_string()
    );

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    let replayed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(
        replayed["behavioural_models"],
        serde_json::json!([attachment.clone()])
    );

    drop(reopened);
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo model attachment".to_string(),
            },
        )
        .expect("model attachment should undo");
    let undone: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert!(undone["behavioural_models"].as_array().unwrap().is_empty());

    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo model attachment".to_string(),
            },
        )
        .expect("model attachment should redo");
    let redone: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(
        redone["behavioural_models"][0]["model_uuid"],
        model_id.to_string()
    );

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "detach native pool part model".to_string(),
                },
                operations: vec![Operation::DetachPoolPartModel {
                    part_id,
                    relative_path: relative_path.clone(),
                    previous_attachments: vec![attachment.clone()],
                    attachments: vec![],
                }],
            },
        )
        .expect("model detachment should commit");
    let detached: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert!(
        detached["behavioural_models"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo model detachment".to_string(),
            },
        )
        .expect("model detachment should undo");
    let restored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(
        restored["behavioural_models"],
        serde_json::json!([attachment])
    );
}

#[test]
fn journaled_pool_library_symbol_create_is_replayable_and_undoable() {
    let root = temp_project_root("pool_library_symbol_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let symbol = serde_json::json!({
        "schema_version": 1,
        "uuid": symbol_id,
        "name": "NativeSymbol",
        "unit": Uuid::new_v4()
    });
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create native pool symbol".to_string(),
                },
                operations: vec![Operation::CreatePoolLibraryObject {
                    object_id: symbol_id,
                    relative_path: format!("pool/symbols/{symbol_id}.json"),
                    object_kind: "symbols".to_string(),
                    object: symbol,
                }],
            },
        )
        .expect("symbol creation should commit");
    assert!(root.join(format!("pool/symbols/{symbol_id}.json")).exists());
    assert_eq!(model.objects.get(&symbol_id).unwrap().kind, "symbols");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(reopened.objects.get(&symbol_id).unwrap().kind, "symbols");

    let mut undo_model = reopened;
    undo_model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo native pool symbol create".to_string(),
            },
        )
        .expect("symbol creation should undo");
    assert!(!root.join(format!("pool/symbols/{symbol_id}.json")).exists());
    assert!(!undo_model.objects.contains_key(&symbol_id));
}

#[test]
fn journaled_pool_library_symbol_set_bumps_revision_and_is_undoable() {
    let root = temp_project_root("pool_library_symbol_set");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let initial_symbol = serde_json::json!({
        "schema_version": 1,
        "uuid": symbol_id,
        "name": "NativeSymbol",
        "unit": Uuid::new_v4()
    });
    let relative_path = format!("pool/symbols/{symbol_id}.json");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create native pool symbol".to_string(),
                },
                operations: vec![Operation::CreatePoolLibraryObject {
                    object_id: symbol_id,
                    relative_path: relative_path.clone(),
                    object_kind: "symbols".to_string(),
                    object: initial_symbol.clone(),
                }],
            },
        )
        .expect("symbol creation should commit");

    let replacement_symbol = serde_json::json!({
        "schema_version": 1,
        "uuid": symbol_id,
        "name": "EditedSymbol",
        "unit": Uuid::new_v4()
    });
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set native pool symbol".to_string(),
                },
                operations: vec![Operation::SetPoolLibraryObject {
                    object_id: symbol_id,
                    relative_path: relative_path.clone(),
                    object_kind: "symbols".to_string(),
                    previous_object: initial_symbol.clone(),
                    object: replacement_symbol.clone(),
                }],
            },
        )
        .expect("symbol replacement should commit");
    assert_eq!(model.objects.get(&symbol_id).unwrap().object_revision.0, 1);
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(written["name"], "EditedSymbol");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.objects.get(&symbol_id).unwrap().object_revision.0,
        1
    );

    let mut undo_model = reopened;
    undo_model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo native pool symbol set".to_string(),
            },
        )
        .expect("symbol replacement should undo");
    let restored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join(&relative_path)).unwrap()).unwrap();
    assert_eq!(restored["name"], "NativeSymbol");
}

#[test]
fn journaled_pool_library_object_rejects_invalid_path_and_identity() {
    let root = temp_project_root("pool_library_symbol_validation");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let invalid_cases = [
        (
            "../symbols/escape.json".to_string(),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": symbol_id, "name": "Escape", "unit": Uuid::new_v4() }),
            "invalid pool library object path",
        ),
        (
            format!("pool/units/{symbol_id}.json"),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": symbol_id, "name": "WrongKind", "unit": Uuid::new_v4() }),
            "does not match path",
        ),
        (
            format!("pool/symbols/{symbol_id}.json"),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": Uuid::new_v4(), "name": "WrongUuid", "unit": Uuid::new_v4() }),
            "does not match object id",
        ),
    ];

    for (relative_path, object_kind, object, expected_error) in invalid_cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".to_string(),
                        source: CommitSource::Test,
                        reason: "reject invalid native pool object".to_string(),
                    },
                    operations: vec![Operation::CreatePoolLibraryObject {
                        object_id: symbol_id,
                        relative_path,
                        object_kind,
                        object,
                    }],
                },
            )
            .expect_err("invalid pool library object should be rejected");
        assert!(
            matches!(&error, EngineError::Validation(message) if message.contains(expected_error)),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn journaled_pool_library_object_rejects_invalid_typed_payload() {
    let root = temp_project_root("pool_library_symbol_schema_validation");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let invalid_cases = [
        (
            serde_json::json!({
                "uuid": symbol_id,
                "name": "MissingSchema",
                "unit": Uuid::new_v4()
            }),
            "missing schema_version",
        ),
        (
            serde_json::json!({
                "schema_version": 2,
                "uuid": symbol_id,
                "name": "WrongSchema",
                "unit": Uuid::new_v4()
            }),
            "unsupported pool library object schema_version",
        ),
        (
            serde_json::json!({
                "schema_version": 1,
                "uuid": symbol_id,
                "name": "MissingUnit"
            }),
            "invalid pool library symbols object",
        ),
    ];

    for (object, expected_error) in invalid_cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".to_string(),
                        source: CommitSource::Test,
                        reason: "reject invalid native pool schema".to_string(),
                    },
                    operations: vec![Operation::CreatePoolLibraryObject {
                        object_id: symbol_id,
                        relative_path: format!("pool/symbols/{symbol_id}.json"),
                        object_kind: "symbols".to_string(),
                        object,
                    }],
                },
            )
            .expect_err("invalid pool library schema should be rejected");
        assert!(
            matches!(&error, EngineError::Validation(message) if message.contains(expected_error)),
            "unexpected error: {error:?}"
        );
    }
}
