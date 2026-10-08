use super::*;
use crate::api::native_write::board_layout::source_profile::build_adopt_board_source_profile;
use crate::api::native_write::{commit_prepared, test_support};
use crate::ir::geometry::Point;
use crate::substrate::{
    CommitSource, ProjectResolver, component_pad_source::component_pad_projection,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn write(path: &Path, value: &Value) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(value).unwrap()
        ),
    )
    .unwrap();
}
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "pad-source-test",
        CommitSource::Test,
        "PM057 journaled source lifetime",
    )
}
pub(super) fn fixture() -> (
    PathBuf,
    DesignModel,
    ComponentPadMaterializationRequest,
    Uuid,
) {
    let (root, _, board_id, placed) =
        test_support::resolved_model_with_board_package("placed_pad_authority");
    let path = root.join("board/board.json");
    let mut board: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let part = Uuid::parse_str(
        board["packages"][placed.to_string()]["part"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let package = Uuid::parse_str(
        board["packages"][placed.to_string()]["package"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let pad = Uuid::new_v4();
    let stack = Uuid::new_v4();
    board["pads"] = json!({});
    board["component_pads"] = json!({});
    board["packages"][placed.to_string()]["layer"] = 1.into();
    board["stackup"] = json!({"layers":[{"id":1,"name":"F.Cu","layer_type":"Copper","thickness_nm":35000},{"id":2,"name":"B.Cu","layer_type":"Copper","thickness_nm":35000}]});
    write(&path, &board);
    let pool = root.join("pool");
    write(
        &pool.join(format!("packages/{package}.json")),
        &json!({"schema_version":1,"uuid":package,"name":"PKG","pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":123000,"y":456000},"padstack":stack,"layer":1}},"courtyard":{"vertices":[],"closed":true},"silkscreen":[],"models_3d":[],"tags":[]}),
    );
    write(
        &pool.join(format!("padstacks/{stack}.json")),
        &json!({"schema_version":1,"uuid":stack,"name":"PAD","aperture":{"circle":{"diameter_nm":600000}},"drill_nm":null,"plated":false}),
    );
    write(
        &pool.join(format!("parts/{part}.json")),
        &json!({"schema_version":1,"uuid":part,"entity":Uuid::new_v4(),"package":package,"pad_map":{},"mpn":"TEST","manufacturer":"","value":"","description":"","datasheet":"","parametric":{},"orderable_mpns":[],"tags":[],"lifecycle":"Active","base":null}),
    );
    let project = root.join("project.json");
    let mut manifest: Value = serde_json::from_slice(&std::fs::read(&project).unwrap()).unwrap();
    manifest["pools"] = json!([{"path":"pool","priority":1}]);
    write(&project, &manifest);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let adoption = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    commit_prepared(&mut model, &root, adoption).unwrap();
    let request = ComponentPadMaterializationRequest {
        placement: serde_json::from_value(board["packages"][placed.to_string()].clone()).unwrap(),
        land_pattern: PadLandPatternSource::LegacyPackage,
        component_instance: None,
        pin_pad_map: None,
        correspondences: vec![],
        adopt_existing: false,
    };
    (root, model, request, pad)
}
#[test]
fn placed_pad_source_lifetimes_replacement_correspondence_stale_pose_and_exact_history() {
    let (root, mut model, mut request, library) = fixture();
    let path = root.join("board/board.json");
    let before = std::fs::read(&path).unwrap();
    let prepared = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let report = commit_prepared(&mut model, &root, prepared).unwrap();
    let first = std::fs::read(&path).unwrap();
    let source: Value = serde_json::from_slice(&first).unwrap();
    let pad: ComponentPadSource = serde_json::from_value(
        source["component_pads"][request.placement.uuid.to_string()][0].clone(),
    )
    .unwrap();
    assert_ne!(pad.uuid, library);
    assert!(report.transaction.diff.created.contains(&pad.uuid));
    assert_eq!(model.objects[&library].domain, "pool");
    let projection = component_pad_projection(&model, &source).unwrap();
    assert_eq!(
        projection.pads[&pad.uuid].position,
        Point::new(123000, 456000)
    );
    assert_eq!(projection.pads[&pad.uuid].net, None);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, model.model_revision);
    assert_eq!(reopened.objects, model.objects);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), first);
    request.correspondences.push(PlacedPadCorrespondence {
        placed_pad_id: pad.uuid,
        target_library_pad: source::capture_pad(
            &model,
            &pad.source_binding.as_ref().unwrap().land_pattern,
            library,
        )
        .unwrap(),
    });
    let rematerialize =
        build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    let report = commit_prepared(&mut model, &root, rematerialize).unwrap();
    assert!(report.transaction.diff.modified.contains(&pad.uuid));
    assert!(!report.transaction.diff.created.contains(&pad.uuid));
    assert!(!report.transaction.diff.deleted.contains(&pad.uuid));
    let move_write = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Position {
            x: 1000000,
            y: 2000000,
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, move_write).unwrap();
    let rotate = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    commit_prepared(&mut model, &root, rotate).unwrap();
    let flip = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Side { layer: 2 },
    )
    .unwrap();
    commit_prepared(&mut model, &root, flip).unwrap();
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = component_pad_projection(&model, &source).unwrap();
    assert_eq!(
        projection.pads[&pad.uuid].position,
        Point::new(544000, 1877000)
    );
    assert_eq!(projection.pads[&pad.uuid].layer, 2);
    let set_part = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Part {
            part_id: Uuid::new_v4(),
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, set_part).unwrap();
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = component_pad_projection(&model, &source).unwrap();
    assert_eq!(
        projection.statuses[&pad.uuid],
        source::ComponentPadMaterializationStatus::Stale
    );
    assert_eq!(projection.pads[&pad.uuid].net, None);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn placed_pad_replacement_and_two_placements_account_for_all_ids_and_refuse_partial_source() {
    let (root, mut model, request, _) = fixture();
    let path = root.join("board/board.json");
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let first = std::fs::read(&path).unwrap();
    let source: Value = serde_json::from_slice(&first).unwrap();
    let first_pad: ComponentPadSource = serde_json::from_value(
        source["component_pads"][request.placement.uuid.to_string()][0].clone(),
    )
    .unwrap();
    let mut other = request.clone();
    other.placement.uuid = Uuid::new_v4();
    other.placement.reference = "U2".into();
    other.placement.position = Point::new(2000000, 0);
    let payload = prepare_component_pad_materialization(&model, &other, &json!({})).unwrap();
    let write = super::super::build_place_board_package(
        &model,
        provenance(),
        &BoardPackagePlacement {
            package: other.placement.clone(),
            materialized: payload,
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let other_pad: ComponentPadSource = serde_json::from_value(
        source["component_pads"][other.placement.uuid.to_string()][0].clone(),
    )
    .unwrap();
    assert_ne!(first_pad.uuid, other_pad.uuid);
    let before = std::fs::read(&path).unwrap();
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    let after = std::fs::read(&path).unwrap();
    assert!(report.transaction.diff.deleted.contains(&first_pad.uuid));
    assert!(!model.objects.contains_key(&first_pad.uuid));
    assert!(model.objects.contains_key(&other_pad.uuid));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(model.objects.contains_key(&first_pad.uuid));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), after);
    let mut forged =
        build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    for op in &mut forged.batch.operations {
        if let Operation::SetBoardPackagePackage { materialized, .. } = op {
            materialized["component_pads"] = json!([]);
            materialized["pad_source_transition"]["allocations"] = json!({});
        }
    }
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, forged).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), after);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    // Deletion retires actual children; compensation restores their identities,
    // while a new placement cannot reuse the retired placement's incarnation.
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let key = request.placement.uuid.to_string();
    let stored_package = board["packages"][&key].clone();
    let materialized = Value::Object(
        board
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(name, values)| {
                name.starts_with("component_")
                    .then(|| values.get(&key).map(|value| (name.clone(), value.clone())))
                    .flatten()
            })
            .collect(),
    );
    let retired: Uuid =
        serde_json::from_value(board["component_pads"][&key][0]["uuid"].clone()).unwrap();
    let write = super::super::build_delete_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        stored_package,
        materialized,
    )
    .unwrap();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    assert!(report.transaction.diff.deleted.contains(&retired));
    assert!(!model.objects.contains_key(&retired));
    let deleted = std::fs::read(&path).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), after);
    assert!(model.objects.contains_key(&retired));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), deleted);
    let payload = prepare_component_pad_materialization(&model, &request, &json!({})).unwrap();
    let write = super::super::build_place_board_package(
        &model,
        provenance(),
        &BoardPackagePlacement {
            package: request.placement.clone(),
            materialized: payload,
        },
    )
    .unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    let mut replacement = request.clone();
    replacement.placement.uuid = Uuid::new_v4();
    let payload = prepare_component_pad_materialization(&model, &replacement, &json!({})).unwrap();
    let fresh: Uuid = serde_json::from_value(payload["component_pads"][0]["uuid"].clone()).unwrap();
    assert_ne!(fresh, retired);
    let write = super::super::build_place_board_package(
        &model,
        provenance(),
        &BoardPackagePlacement {
            package: replacement.placement,
            materialized: payload,
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn placed_pad_explicit_legacy_adoption_recovers_library_owner_and_exact_inverse() {
    let (root, mut model, mut request, library) = fixture();
    let path = root.join("board/board.json");
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let stack: Uuid = serde_json::from_value(
        model
            .materialized_source_shard_value_by_relative_path(&format!(
                "pool/packages/{}.json",
                request.placement.package
            ))
            .unwrap()["pads"][library.to_string()]["padstack"]
            .clone(),
    )
    .unwrap();
    let old = json!({"uuid":library,"name":"1","position":{"x":123000,"y":456000},"padstack":stack,"layer":1,"drill_nm":null,"shape":"circle","diameter_nm":600000,"width_nm":0,"height_nm":0});
    let write = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Package {
            package_ref_id: request.placement.package,
            previous_materialized: json!({}),
            materialized: json!({"component_pads":[old]}),
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(model.objects[&library].domain, "board");
    let before = std::fs::read(&path).unwrap();
    request.adopt_existing = true;
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let after = std::fs::read(&path).unwrap();
    assert_eq!(model.objects[&library].domain, "pool");
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), after);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    assert_eq!(source["schema_version"], 4);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn placed_pad_library_history_preserves_survivors_but_refuses_uuid_rebirth() {
    use crate::api::native_write::library::{
        PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
        build_pool_library_write,
    };
    let (root, mut model, mut request, library) = fixture();
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let key = request.placement.uuid.to_string();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let first: ComponentPadSource =
        serde_json::from_value(board["component_pads"][&key][0].clone()).unwrap();
    let path = format!("pool/packages/{}.json", request.placement.package);
    let mut package = model
        .materialized_source_shard_value_by_relative_path(&path)
        .unwrap();
    package["pads"][library.to_string()]["position"]["x"] = 789000.into();
    let target = Target::new("pool", "packages", request.placement.package);
    let write = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![Spec::Set {
            target: target.clone(),
            object: package.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    request.correspondences = vec![PlacedPadCorrespondence {
        placed_pad_id: first.uuid,
        target_library_pad: capture_component_library_pad(
            &model,
            request.placement.package,
            &request.land_pattern,
            library,
        )
        .unwrap(),
    }];
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        board["component_pads"][&key][0]["uuid"],
        first.uuid.to_string()
    );
    assert_eq!(board["component_pads"][&key][0]["position"]["x"], 789000);
    let removed = package["pads"]
        .as_object_mut()
        .unwrap()
        .remove(&library.to_string())
        .unwrap();
    let write = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![Spec::Set {
            target: target.clone(),
            object: package.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    package["pads"][library.to_string()] = removed;
    let write = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![Spec::Set {
            target,
            object: package,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    request.correspondences[0].target_library_pad = capture_component_library_pad(
        &model,
        request.placement.package,
        &request.land_pattern,
        library,
    )
    .unwrap();
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    assert!(build_materialize_component_pad_sources(&model, provenance(), &request).is_err());
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    request.correspondences.clear();
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_ne!(
        board["component_pads"][&key][0]["uuid"],
        first.uuid.to_string()
    );
    assert!(!model.objects.contains_key(&first.uuid));
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, model.objects);
    assert_eq!(reopened.model_revision, model.model_revision);
    std::fs::remove_dir_all(root).unwrap();
}
