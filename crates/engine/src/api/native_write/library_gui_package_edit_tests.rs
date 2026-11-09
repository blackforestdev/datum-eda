//! Saved Package edits retain actual CI and frozen legacy-Pad users.
use super::*;
use crate::api::native_write::{
    self as native, BatchComposer,
    board_components::pad_materialization::*,
    commit_prepared,
    electrical::binding_fixture::{self, BindingFixture},
};
use crate::substrate::{
    CommitSource, DesignModel, ElectricalIdentity, ObjectRevision, Operation, ProjectResolver,
    component_pad_source::{ComponentPadMaterializationStatus, component_pad_projection},
};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "package-edit-test",
        CommitSource::Test,
        "Preserve saved Package users",
    )
}
fn edit(f: &BindingFixture, id: Uuid, fields: Value) -> Result<PreparedWrite, EngineError> {
    build(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "packages".into(),
            object_id: id,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}
fn bind(f: &mut BindingFixture, ci: Uuid, package: Uuid, revision: u64) {
    let owner = &f.model.objects[&ci];
    let shard = f
        .model
        .source_shards
        .iter()
        .find(|s| s.shard_id == owner.source_shard_id)
        .unwrap();
    let previous = f
        .model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(owner.object_revision.0 + 1);
    next["library_bindings"][Uuid::new_v4().to_string()] = json!({"binding_role":"package", "target_object_id":package,"pinned_object_revision":revision});
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
}
fn setup(with_ci: bool, footprint: bool) -> (BindingFixture, Uuid, Uuid) {
    let mut f = binding_fixture::fixture("saved_package_users");
    let (old_pad, library_pad, map, ci) = match &f.relationship.identity {
        ElectricalIdentity::NetRelationship { evidence, .. } => (
            evidence[0].board_pad.object_id,
            evidence[0].library_pad.as_ref().unwrap().object_id,
            evidence[0].pin_pad_map.object_id,
            evidence[0].component_instance.object_id,
        ),
        _ => unreachable!(),
    };
    let board = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let old: crate::board::PlacedPad =
        serde_json::from_value(board["pads"][old_pad.to_string()].clone()).unwrap();
    let placement: crate::board::PlacedPackage =
        serde_json::from_value(board["packages"][old.package.to_string()].clone()).unwrap();
    let package = placement.package;
    let raw = f
        .model
        .materialized_source_shard_value_by_relative_path(&format!("pool/packages/{package}.json"))
        .unwrap();
    let stack: Uuid =
        serde_json::from_value(raw["pads"][library_pad.to_string()]["padstack"].clone()).unwrap();
    let prepared = library::build_pool_library_write(&f.model, provenance(), Some("pool"), vec![PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new("pool", "padstacks", stack),
        object: json!({"schema_version":1,"uuid":stack,"name":"Exact","aperture":{"circle":{"diameter_nm":600000}},"drill_nm":null,"plated":false}),
    }]).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let prepared = native::board_routing::build_delete_board_pad(
        &f.model,
        provenance(),
        old.uuid,
        serde_json::to_value(old).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    if with_ci {
        bind(&mut f, ci, package, 0);
    }
    let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
        &f.model,
        provenance(),
        serde_json::from_value(board["uuid"].clone()).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let land_pattern = if footprint {
        let id = Uuid::new_v4();
        let foot_pad_id = Uuid::new_v4();
        let mut foot_pad = raw["pads"][library_pad.to_string()].clone();
        foot_pad["uuid"] = json!(foot_pad_id);
        let prepared = library::build_pool_library_write(&f.model, provenance(), Some("pool"), vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "footprints", id),
            object: json!({"schema_version":1,"uuid":id,"name":"Explicit land","package":package,"courtyard":{"vertices":[],"closed":true},"pads":{foot_pad_id.to_string():foot_pad}}),
        }]).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        PadLandPatternSource::Footprint(id)
    } else {
        PadLandPatternSource::LegacyPackage
    };
    let prepared = build_materialize_component_pad_sources(
        &f.model,
        provenance(),
        &ComponentPadMaterializationRequest {
            placement,
            land_pattern,
            component_instance: Some(ci),
            pin_pad_map: (!footprint).then_some(map),
            correspondences: vec![],
            adopt_existing: false,
        },
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    if with_ci {
        let board = f
            .model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let pad = board["component_pads"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()[0]
            .clone();
        if let ElectricalIdentity::NetRelationship { evidence, .. } = &mut f.relationship.identity {
            evidence[0].board_pad.object_id = serde_json::from_value(pad["uuid"].clone()).unwrap();
            evidence[0].board_pad.object_revision =
                f.model.objects[&evidence[0].board_pad.object_id].object_revision;
            evidence[0].component_instance.object_revision = f.model.objects[&ci].object_revision;
        }
        let prepared = native::electrical::build_adopt_pin_correspondence(
            &f.model,
            provenance(),
            f.relationship.clone(),
            &[native::electrical::PlacedPinAdoption {
                sheet_id: f.sheet,
                symbol_id: f.symbol.uuid,
                placed_pin_id: f.symbol.pins[0].uuid,
                library_pin_id: f.library_pin,
            }],
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    }
    (f, package, ci)
}
fn source(model: &DesignModel, package: Uuid) -> Value {
    model
        .materialized_source_shard_value_by_relative_path(&format!("pool/packages/{package}.json"))
        .unwrap()
}

fn metadata_history(with_ci: bool, footprint: bool) {
    let (mut f, package, ci) = setup(with_ci, footprint);
    let original = source(&f.model, package);
    let ci_before = f.model.component_instances[&ci].clone();
    assert_eq!(
        ci_before
            .library_bindings
            .values()
            .any(|b| b.binding_role == crate::substrate::LibraryBindingRole::Package),
        with_ci
    );
    let electrical_before = f.model.electrical_identities.clone();
    let map = f.model.component_instances[&ci]
        .library_bindings
        .values()
        .find(|b| b.binding_role == crate::substrate::LibraryBindingRole::PinPadMap)
        .unwrap()
        .target_object_id;
    let map_path = f.root.join(format!("pool/pin_pad_maps/{map}.json"));
    let map_before = std::fs::read(&map_path).unwrap();
    let board = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = component_pad_projection(&f.model, &board).unwrap();
    assert!(
        projection
            .statuses
            .values()
            .all(|s| *s == ComponentPadMaterializationStatus::Current)
    );
    assert!(!projection.pads.is_empty());
    if with_ci {
        assert!(projection.pads.values().any(|p| p.net.is_some()));
    }
    let scene = serde_json::to_value(
        crate::substrate::ElectricalSelectionSnapshot::capture(&f.model)
            .unwrap()
            .schematic_source()
            .unwrap(),
    )
    .unwrap();
    let board_bytes = std::fs::read(f.root.join("board/board.json")).unwrap();
    let journal = crate::substrate::transaction_journal_path(&f.root);
    let journal_before = std::fs::read(&journal).unwrap();
    for fields in [
        json!({"name":" "}),
        json!({"package_code":7}),
        json!({"pads":{}}),
        json!({"terminals":{}}),
        json!({"body_height_nm":17}),
        json!({"uuid":package}),
        json!({"approval_status":"approved"}),
        json!({"lifecycle":"Active"}),
    ] {
        assert!(edit(&f, package, fields).is_err());
    }
    let fields = json!({"name":"Corrected body", "package_family":"As entered", "package_code":"", "mounting_type":"Designer mounting"});
    let prepared = edit(&f, package, fields.clone()).unwrap();
    assert_eq!(
        std::fs::read(&journal).unwrap(),
        journal_before,
        "preview writes nothing"
    );
    let stale = prepared.clone();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
    let mut expected = original.clone();
    expected
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    assert_eq!(source(&f.model, package), expected);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(source(&f.model, package), original);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    for model in [&f.model, &cold] {
        assert_eq!(source(model, package), expected);
        assert_eq!(
            serde_json::to_value(
                crate::substrate::ElectricalSelectionSnapshot::capture(model)
                    .unwrap()
                    .schematic_source()
                    .unwrap()
            )
            .unwrap(),
            scene
        );
        assert_eq!(model.component_instances[&ci], ci_before);
        assert_eq!(model.electrical_identities, electrical_before);
        assert_eq!(
            model
                .materialized_pool_object_revision(package, "packages", ObjectRevision(0))
                .unwrap(),
            original
        );
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_"))
        );
        let after = component_pad_projection(model, &board).unwrap();
        assert_eq!(after.statuses, projection.statuses);
        assert_eq!(after.pads, projection.pads);
        assert_eq!(after.unavailable, projection.unavailable);
    }
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        board_bytes
    );
    assert_eq!(
        std::fs::read(map_path).unwrap(),
        map_before,
        "Package Set leaves the unrelated Map source intact"
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn metadata_history_preserves_frozen_legacy_users_without_package_role() {
    metadata_history(false, false);
}
#[test]
fn metadata_history_preserves_package_role_and_connected_net_users() {
    metadata_history(true, false);
}
#[test]
fn metadata_history_preserves_footprint_bound_users() {
    metadata_history(false, true);
}

#[test]
fn restored_package_counter_refuses_existing_ci_and_frozen_only_future_pins() {
    for pinned_revision in [1, 3] {
        let (mut f, _, ci) = setup(false, false);
        let id = Uuid::new_v4();
        let prepared = build(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            provenance(),
            LibraryDefinitionRequest {
                kind: "packages".into(),
                object_id: id,
                pool_path: "pool".into(),
                fields: json!({"name":"Original"}).as_object().unwrap().clone(),
            },
            true,
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        for revision in 1..=pinned_revision {
            let prepared = edit(&f, id, json!({"name":format!("Historical {revision}")})).unwrap();
            commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        }
        for _ in 0..=pinned_revision {
            f.model
                .commit_journal_undo(&f.root, provenance().into())
                .unwrap();
        }
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.model.objects[&id].object_revision.0, 0);
        assert_eq!(
            f.model
                .materialized_pool_object_revision(id, "packages", ObjectRevision(pinned_revision))
                .unwrap()["name"],
            format!("Historical {pinned_revision}")
        );
        let before = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
        // Negative input only: a future frozen pin cannot be produced by the current
        // materializer. Redirect a cloned BoardRoot; never admit or commit this state.
        for field in ["package", "land_pattern"] {
            let mut fault = f.model.clone();
            let mut board = fault
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap();
            let pad = board["component_pads"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap()
                .as_array_mut()
                .unwrap()
                .first_mut()
                .unwrap();
            let pin = &mut pad["source_binding"][field];
            pin["object_id"] = json!(id);
            pin["owner_id"] = json!(id);
            pin["object_revision"] = json!(1);
            let bytes = serde_json::to_vec(&board).unwrap();
            let path = f.root.join(format!("negative-{field}.json"));
            std::fs::write(&path, &bytes).unwrap();
            let shard = fault
                .source_shards
                .iter_mut()
                .find(|s| s.kind == SourceShardKind::BoardRoot)
                .unwrap();
            shard.path = path;
            shard.content_hash = crate::substrate::sha256_hex(&bytes);
            let prepared = build(
                &NativeWriteContext {
                    model: &fault,
                    project_root: &f.root,
                },
                provenance(),
                LibraryDefinitionRequest {
                    kind: "packages".into(),
                    object_id: id,
                    pool_path: "pool".into(),
                    fields: json!({"name":"Conflicting one"})
                        .as_object()
                        .unwrap()
                        .clone(),
                },
                false,
            );
            assert!(prepared.unwrap_err().to_string().contains("ambiguous"));
        }
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
            before
        );
        // Actual canonical CI history admits a historical pin while today's root is0.
        bind(&mut f, ci, id, pinned_revision);
        let before_ci = f.model.component_instances[&ci].clone();
        if pinned_revision == 1 {
            assert!(
                edit(&f, id, json!({"name":"Conflicting one"}))
                    .unwrap_err()
                    .to_string()
                    .contains("ambiguous")
            );
            let prepared = edit(&f, id, json!({"name":"Historical 1"})).unwrap();
            commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        } else {
            for name in ["Later C1", "Later C2"] {
                let prepared = edit(&f, id, json!({"name":name})).unwrap();
                commit_prepared(&mut f.model, &f.root, prepared).unwrap();
            }
            let journal = crate::substrate::transaction_journal_path(&f.root);
            let bytes = std::fs::read(&journal).unwrap();
            let raw = source(&f.model, id);
            let cursor = f.model.journal_cursor.clone();
            assert!(
                f.model
                    .commit_journal_undo(&f.root, provenance().into())
                    .unwrap_err()
                    .to_string()
                    .contains("retained revision ambiguous")
            );
            assert_eq!(std::fs::read(journal).unwrap(), bytes);
            assert_eq!(source(&f.model, id), raw);
            assert_eq!(f.model.journal_cursor, cursor);
        }
        let cold = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(cold.component_instances[&ci], before_ci);
        assert!(
            !cold
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_"))
        );
        std::fs::remove_dir_all(f.root).unwrap();
    }
}

#[test]
fn first_transaction_frozen_legacy_pad_keeps_journal_start_owner_basis() {
    let (root, _, _, placed) =
        native::test_support::resolved_model_with_board_package("package_initial_pin");
    let write_initial = |path: &std::path::Path, raw: &Value| {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            format!(
                "{}\n",
                crate::ir::serialization::to_json_deterministic(raw).unwrap()
            ),
        )
        .unwrap();
    };
    // Initial on-disk source fixture only. All later writes use canonical commit.
    let mut board: Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let part: Uuid =
        serde_json::from_value(board["packages"][placed.to_string()]["part"].clone()).unwrap();
    let package: Uuid =
        serde_json::from_value(board["packages"][placed.to_string()]["package"].clone()).unwrap();
    let library_pad = Uuid::new_v4();
    let stack = Uuid::new_v4();
    board["schema_version"] = json!(4);
    board["pads"] = json!({});
    board["component_pads"] = json!({});
    board["packages"][placed.to_string()]["layer"] = json!(1);
    board["stackup"] = json!({"layers":[{"id":1,"name":"F.Cu","layer_type":"Copper","thickness_nm":35000},{"id":2,"name":"B.Cu","layer_type":"Copper","thickness_nm":35000}]});
    write_initial(&root.join("board/board.json"), &board);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    manifest["pools"] = json!([{"path":"pool","priority":1}]);
    write_initial(&root.join("project.json"), &manifest);
    let mut original = library::pool_package_payload(
        package,
        "Initial body",
        json!({library_pad.to_string():{"uuid":library_pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}),
    );
    original["vendor"] = json!({"ordered":[3,1,3]});
    original["tags"] = json!(["z", "a", "z"]);
    original["body_height_nm"] = Value::Null;
    write_initial(
        &root.join(format!("pool/packages/{package}.json")),
        &original,
    );
    write_initial(
        &root.join(format!("pool/padstacks/{stack}.json")),
        &json!({"schema_version":1,"uuid":stack,"name":"Copper","aperture":{"circle":{"diameter_nm":600000}},"plated":false}),
    );
    write_initial(
        &root.join(format!("pool/parts/{part}.json")),
        &library::pool_part_payload(
            part,
            Uuid::new_v4(),
            package,
            "Initial",
            "",
            "",
            "",
            "",
            "Active",
        ),
    );
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.journal.is_empty());
    let prepared = build_materialize_component_pad_sources(
        &model,
        provenance(),
        &ComponentPadMaterializationRequest {
            placement: serde_json::from_value(board["packages"][placed.to_string()].clone())
                .unwrap(),
            land_pattern: PadLandPatternSource::LegacyPackage,
            component_instance: None,
            pin_pad_map: None,
            correspondences: vec![],
            adopt_existing: false,
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let pin = &board["component_pads"][placed.to_string()][0]["source_binding"]["library_pad"];
    assert!(pin["journal_tip"].is_null());
    let before = component_pad_projection(&model, &board).unwrap();
    assert!(
        before
            .statuses
            .values()
            .all(|s| *s == ComponentPadMaterializationStatus::Current)
    );
    let prepared = build(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "packages".into(),
            object_id: package,
            pool_path: "pool".into(),
            fields: json!({"name":"Updated description"})
                .as_object()
                .unwrap()
                .clone(),
        },
        false,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(source(&model, package), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    for model in [&model, &cold] {
        let after = component_pad_projection(model, &board).unwrap();
        assert_eq!(after.statuses, before.statuses);
        assert_eq!(after.pads, before.pads);
        let mut expected = original.clone();
        expected["name"] = json!("Updated description");
        assert_eq!(source(model, package), expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_history_retains_hash_leaf_and_structural_source_guards() {
    use crate::substrate::component_pad_source::{ComponentPadSource, check_pin};
    let (mut f, package, _) = setup(false, false);
    let board = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let pad: ComponentPadSource = serde_json::from_value(
        board["component_pads"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()[0]
            .clone(),
    )
    .unwrap();
    let binding = pad.source_binding.as_ref().unwrap();
    let prepared = edit(&f, package, json!({"name":"Metadata only"})).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    for pin in [
        &binding.package,
        &binding.land_pattern,
        &binding.library_pad,
    ] {
        check_pin(&f.model, pin).unwrap();
        for mutation in 0..5 {
            let mut bad = pin.clone();
            match mutation {
                0 => bad.owner_value_hash = "foreign".into(),
                1 => bad.record_hash = "foreign".into(),
                2 => bad.owner_id = Uuid::new_v4(),
                3 => bad.source_shard_id = Uuid::new_v4(),
                _ => bad.object_revision.0 += 1,
            }
            assert!(check_pin(&f.model, &bad).is_err(), "mutation {mutation}");
        }
    }
    let mut bad = binding.library_pad.clone();
    bad.journal_tip = Some(Uuid::new_v4());
    assert!(check_pin(&f.model, &bad).is_err());
    // The metadata exception must not hide changed parent physical inputs even
    // when the child pad's own payload, revision and incarnation are unchanged.
    let mut physical = source(&f.model, package);
    physical["body_dimensions"] = json!({"x_nm":123000});
    let prepared = library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "packages", package),
            object: physical,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    assert!(check_pin(&f.model, &binding.library_pad).is_err());
    let projection = component_pad_projection(&f.model, &board).unwrap();
    assert_eq!(
        projection.statuses[&pad.uuid],
        ComponentPadMaterializationStatus::Stale
    );
    assert_eq!(
        projection.pads[&pad.uuid].position,
        binding.frozen_geometry.position
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
