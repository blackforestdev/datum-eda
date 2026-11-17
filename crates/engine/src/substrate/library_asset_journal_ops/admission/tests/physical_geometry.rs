//! Public geometry workflow controls. Generated CAD here is test data only;
//! ordinary real-project GUI acceptance is owned by the PM's separate proof.
use super::*;
use crate::substrate::component_pad_source::{self as pads, ComponentPadMaterializationStatus};
use serde_json::json;
#[path = "physical_sequence.rs"]
mod sequence;

fn board(f: &Fixture) -> Value {
    f.model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()
}

fn two_users_with_copper() -> Fixture {
    let mut f = Fixture::new(false);
    assert_eq!(f.model.component_instances.len(), 1);
    let first = f
        .model
        .component_instances
        .values()
        .next()
        .unwrap()
        .placed_symbol_refs[0];
    let sheets: Vec<_> = f
        .model
        .source_shards
        .iter()
        .filter(|head| head.kind == SourceShardKind::SchematicSheet)
        .filter_map(|head| {
            let sheet = f
                .model
                .materialized_source_shard_value_by_relative_path(&head.relative_path)
                .unwrap();
            sheet["symbols"]
                .get(first.to_string())
                .is_some()
                .then_some(sheet)
        })
        .collect();
    assert_eq!(sheets.len(), 1);
    let sheet: Uuid = serde_json::from_value(sheets[0]["uuid"].clone()).unwrap();
    let part = f
        .model
        .materialized_source_shard_value_by_relative_path(&format!("pool/parts/{}.json", f.part))
        .unwrap();
    let entity: Uuid = serde_json::from_value(part["entity"].clone()).unwrap();
    let entity = f
        .model
        .materialized_source_shard_value_by_relative_path(&format!("pool/entities/{entity}.json"))
        .unwrap();
    let gates = entity["gates"].as_object().unwrap();
    assert_eq!(gates.len(), 1);
    let (gate, definition) = gates.iter().next().unwrap();
    let symbol: Uuid = serde_json::from_value(definition["symbol"].clone()).unwrap();
    let choice = native::schematic_library::part_placement::CompletePartSelection::capture(
        &f.model,
        f.part,
        gate.parse().unwrap(),
        symbol,
    )
    .unwrap();
    write(
        &f.root,
        &mut f.model,
        "datum.schematic.place_symbol",
        json!({
        "sheet":sheet,"library_symbol":symbol,"component":choice,"reference":"U2",
        "value":"Test","x_nm":10000000,"y_nm":0}),
    );
    write(
        &f.root,
        &mut f.model,
        "datum.schematic.draw_wire",
        json!({
        "sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},
            {"x":-2000000,"y":-4000000},{"x":8000000,"y":-4000000},{"x":8000000,"y":0}]}),
    );
    let board_id = serde_json::from_value(board(&f)["uuid"].clone()).unwrap();
    let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
        &f.model,
        public_provenance(),
        board_id,
    )
    .unwrap();
    native::commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let prepared = prepare_transfer(
        &f.model,
        public_provenance(),
        &TransferRequest {
            sheet,
            choices: vec![],
            edit_seed: Uuid::new_v4(),
            origin: Point::zero(),
            pitch_nm: 5000000,
            layer: 1,
        },
    )
    .unwrap();
    native::commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let packages = board(&f)["packages"].as_object().unwrap().clone();
    assert_eq!(packages.len(), 2);
    let second: Uuid = packages
        .iter()
        .find(|(_, p)| p["reference"] == "U2")
        .unwrap()
        .0
        .parse()
        .unwrap();
    for edit in [
        native::board_components::BoardPackageEdit::Position {
            x: 20000000,
            y: 10000000,
        },
        native::board_components::BoardPackageEdit::Rotation { rotation: 90 },
    ] {
        let prepared = native::board_components::build_edit_board_package(
            &f.model,
            public_provenance(),
            second,
            edit,
        )
        .unwrap();
        native::commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    }
    let current = board(&f);
    let projection = pads::component_pad_projection(&f.model, &current).unwrap();
    let pad = projection
        .pads
        .values()
        .find(|pad| pad.net.is_some())
        .expect("wired physical pad");
    let net = pad.net.unwrap();
    let start = pad.position;
    assert!(current["nets"].get(net.to_string()).is_some());
    let track = crate::board::Track::straight(
        Uuid::new_v4(),
        net,
        start,
        Point {
            x: start.x,
            y: start.y - 2000000,
        },
        200000,
        1,
    );
    let prepared =
        native::board_routing::build_place_board_track(&f.model, public_provenance(), &track)
            .unwrap();
    native::commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.baseline();
    assert_eq!(snapshot(&f.root, f.limits).lock().bindings.len(), 2);
    f
}

fn qualified_geometry(f: &Fixture) -> (BTreeMap<Uuid, Value>, BTreeMap<Uuid, Option<Uuid>>) {
    let current = board(f);
    let projection = pads::component_pad_projection(&f.model, &current).unwrap();
    assert!(
        projection.unavailable.is_empty(),
        "{:?}",
        projection.unavailable
    );
    assert!(projection.unadopted_packages.is_empty());
    assert_eq!(projection.statuses.len(), 4);
    assert!(
        projection
            .statuses
            .values()
            .all(|status| *status == ComponentPadMaterializationStatus::Current)
    );
    let geometry = current["component_pads"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|value| value.as_array().unwrap())
        .map(|pad| {
            (
                serde_json::from_value(pad["uuid"].clone()).unwrap(),
                pad["source_binding"]["frozen_geometry"].clone(),
            )
        })
        .collect();
    let nets = projection
        .pads
        .iter()
        .map(|(id, pad)| (*id, pad.net))
        .collect();
    (geometry, nets)
}

fn unchanged_design(f: &Fixture, before: &Value, schematic: &BTreeMap<PathBuf, Vec<u8>>) {
    let current = board(f);
    for field in [
        "packages",
        "tracks",
        "vias",
        "zones",
        "nets",
        "net_classes",
        "graphics",
    ] {
        assert_eq!(current[field], before[field], "retained Board {field}");
    }
    assert!(!current["tracks"].as_object().unwrap().is_empty());
    for (path, bytes) in schematic {
        assert_eq!(std::fs::read(f.root.join(path)).unwrap(), *bytes);
    }
}

fn unknown_pad_receipt_refuses(f: &Fixture) {
    use crate::substrate::library_asset_journal_ops::history::qualified_package_transaction;
    let (actual, prior) = f.model.journal.split_last().unwrap();
    let package = actual
        .operations
        .iter()
        .find_map(|operation| match operation {
            Operation::SetBoardPackagePackage { package_id, .. } => Some(*package_id),
            _ => None,
        })
        .unwrap();
    assert!(qualified_package_transaction(actual, prior, package).unwrap());
    // A malformed receipt is negative input, not an accepted model or CAD
    // fixture. Preserve its exact paired inverse so a mismatch cannot be the
    // reason the unknown raw Pad metadata is rejected.
    let mut malformed = actual.clone();
    let metadata = json!({"note":"unsupported raw pad extension","value":9007199254740993_i64});
    for operation in &mut malformed.operations {
        if let Operation::SetBoardPackagePackage {
            package_id,
            materialized,
            ..
        } = operation
            && *package_id == package
        {
            materialized["component_pads"][0]["future_metadata"] = metadata.clone();
        }
    }
    for operation in &mut malformed.inverse_operations {
        if let Operation::SetBoardPackagePackage {
            package_id,
            previous_materialized,
            ..
        } = operation
            && *package_id == package
        {
            previous_materialized["component_pads"][0]["future_metadata"] = metadata.clone();
        }
    }
    let original = malformed
        .operations
        .iter()
        .find_map(|operation| match operation {
            Operation::SetBoardPackagePackage {
                package_id,
                previous_materialized,
                materialized,
                ..
            } if *package_id == package => Some((previous_materialized, materialized)),
            _ => None,
        })
        .unwrap();
    let inverse = malformed
        .inverse_operations
        .iter()
        .find_map(|operation| match operation {
            Operation::SetBoardPackagePackage {
                package_id,
                previous_materialized,
                materialized,
                ..
            } if *package_id == package => Some((previous_materialized, materialized)),
            _ => None,
        })
        .unwrap();
    assert_eq!(original.0, inverse.1);
    assert_eq!(original.1, inverse.0);
    let before = files(&f.root);
    assert!(!matches!(
        qualified_package_transaction(&malformed, prior, package),
        Ok(true)
    ));
    assert_eq!(files(&f.root), before);
}

fn run_geometry(footprint: bool) {
    let mut f = two_users_with_copper();
    let before = board(&f);
    let (old_geometry, old_nets) = qualified_geometry(&f);
    let mut expected_geometry = old_geometry.clone();
    for pad in expected_geometry.values_mut() {
        if footprint {
            let x = pad["position"]["x"].as_i64().unwrap();
            pad["position"]["x"] = json!(x + 100000);
        } else {
            assert_eq!(pad["aperture"]["circle"]["diameter_nm"], 1000000);
            pad["aperture"]["circle"]["diameter_nm"] = json!(1200000);
        }
    }
    assert!(old_nets.values().any(Option::is_some));
    let schematic: BTreeMap<_, _> = files(&f.root)
        .into_iter()
        .filter(|(path, _)| path.starts_with("schematic"))
        .collect();
    let pool_before = files(&f.root.join("pool"));
    let lock_before = std::fs::read(f.root.join("library/lock.json")).unwrap();
    let id = if footprint { f.footprint } else { f.stack };
    let base = f.active(id);
    let mut payload = snapshot(&f.root, f.limits).assets()[&base]
        .payload()
        .value()
        .clone();
    if footprint {
        for pad in payload["pads"].as_object_mut().unwrap().values_mut() {
            let x = pad["position"]["x"].as_i64().unwrap();
            pad["position"]["x"] = json!(x + 100000);
        }
    } else {
        payload["aperture"]["circle"]["diameter_nm"] = json!(1200000);
    }
    let candidate = f.revise(&base, payload);
    assert_eq!(files(&f.root.join("pool")), pool_before);
    assert_eq!(
        std::fs::read(f.root.join("library/lock.json")).unwrap(),
        lock_before
    );
    assert_eq!(qualified_geometry(&f).0, old_geometry);
    let mut request = f.request(&candidate);
    let selection = request["request"].clone();
    let canonical = files(&f.root);
    let stage = f.root.join(".datum/stage");
    let stage_before = stage.exists().then(|| files(&stage));
    let description = native::library_assets::inspect_geometry_selection(
        &native::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        selection,
    )
    .unwrap();
    assert_eq!(description["project_id"], json!(f.model.project.project_id));
    assert_eq!(description["model_revision"], json!(f.model.model_revision));
    assert_eq!(
        description["transaction_tip"],
        json!(crate::revision::transaction_tip(&f.model.journal))
    );
    let placements = description["physical_geometry"]["placements"]
        .as_array()
        .unwrap();
    assert_eq!(placements.len(), 2);
    assert!(
        placements
            .iter()
            .all(|p| p["pads"].as_array().unwrap().len() == 2)
    );
    assert_eq!(files(&f.root), canonical);
    assert_eq!(stage.exists().then(|| files(&stage)), stage_before);
    let missing = native::library_assets::prepare_library_change(
        &native::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        public_provenance(),
        request.clone(),
    );
    assert!(
        missing.is_err(),
        "missing explicit physical intent must retain safe refusal"
    );
    request["request"]["physical_geometry"] = description["physical_geometry"].clone();
    for omission in [true, false] {
        let mut incomplete = request.clone();
        let users = incomplete["request"]["physical_geometry"]["placements"]
            .as_array_mut()
            .unwrap();
        if omission {
            users.pop();
        } else {
            users[0]["pads"].as_array_mut().unwrap().pop();
        }
        assert!(
            native::library_assets::prepare_library_change(
                &native::registry::NativeWriteContext {
                    model: &f.model,
                    project_root: &f.root
                },
                public_provenance(),
                incomplete
            )
            .is_err()
        );
        assert_eq!(files(&f.root), canonical);
    }
    let change = native::library_assets::prepare_library_change(
        &native::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        public_provenance(),
        request,
    )
    .unwrap();
    let updates = change.summary["physical_geometry_updates"]
        .as_array()
        .unwrap();
    assert_eq!(updates.len(), 2);
    assert_eq!(json!(updates), description["physical_geometry_updates"]);
    assert!(updates.iter().all(|u| {
        u["pads"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["before"] != p["after"])
    }));
    assert_eq!(files(&f.root), canonical); // dropping an unsubmitted plan is inert
    let omitted: Uuid = serde_json::from_value(updates[0]["placed_package"].clone()).unwrap();
    let mut raw = change.prepared.clone();
    raw.batch.operations.retain(|operation| {
        !matches!(operation,
        Operation::SetBoardPackagePackage { package_id, .. } if *package_id == omitted)
    });
    let revision = f.model.model_revision.clone();
    assert!(native::commit_prepared(&mut f.model, &f.root, raw).is_err());
    assert_eq!(f.model.model_revision, revision);
    assert_eq!(files(&f.root), canonical);
    native::commit_prepared(&mut f.model, &f.root, change.prepared).unwrap();
    unknown_pad_receipt_refuses(&f);
    let (adopted_geometry, adopted_nets) = qualified_geometry(&f);
    assert_eq!(adopted_geometry, expected_geometry);
    assert_eq!(
        adopted_geometry.keys().collect::<Vec<_>>(),
        old_geometry.keys().collect::<Vec<_>>()
    );
    assert_eq!(adopted_nets, old_nets);
    assert_eq!(f.active(id), candidate);
    unchanged_design(&f, &before, &schematic);
    let pool_after = files(&f.root.join("pool"));
    assert_eq!(
        pool_after.keys().collect::<Vec<_>>(),
        pool_before.keys().collect::<Vec<_>>()
    );
    let changed: Vec<_> = pool_before
        .iter()
        .filter(|(path, bytes)| pool_after.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect();
    assert_eq!(
        changed,
        vec![PathBuf::from(format!(
            "{}/{id}.json",
            if footprint { "footprints" } else { "padstacks" }
        ))]
    );
    f.model
        .commit_journal_undo(&f.root, public_provenance().into())
        .unwrap();
    assert_eq!(f.active(id), base);
    assert_eq!(qualified_geometry(&f), (old_geometry, old_nets.clone()));
    unchanged_design(&f, &before, &schematic);
    f.model
        .commit_journal_redo(&f.root, public_provenance().into())
        .unwrap();
    assert_eq!(f.active(id), candidate);
    assert_eq!(
        qualified_geometry(&f),
        (adopted_geometry.clone(), adopted_nets.clone())
    );
    unchanged_design(&f, &before, &schematic);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(id), candidate);
    assert_eq!(
        qualified_geometry(&f),
        (adopted_geometry.clone(), adopted_nets.clone())
    );
    unchanged_design(&f, &before, &schematic);
    // A genuine new library family must not break ordinary pose edits or the
    // next reviewed update through the component's retained package anchor.
    let current = board(&f);
    let second: Uuid = current["packages"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, package)| package["reference"] == "U2")
        .unwrap()
        .0
        .parse()
        .unwrap();
    for edit in [
        native::board_components::BoardPackageEdit::Position {
            x: 21000000,
            y: 11000000,
        },
        native::board_components::BoardPackageEdit::Rotation { rotation: 180 },
    ] {
        let prepared = native::board_components::build_edit_board_package(
            &f.model,
            public_provenance(),
            second,
            edit,
        )
        .unwrap();
        native::commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    }
    assert_eq!(qualified_geometry(&f), (adopted_geometry, adopted_nets));
    let moved = board(&f);
    assert_eq!(
        moved["packages"][second.to_string()]["position"],
        json!({"x":21000000,"y":11000000})
    );
    assert_eq!(moved["packages"][second.to_string()]["rotation"], 180);
    for field in ["tracks", "vias", "zones", "nets", "net_classes", "graphics"] {
        assert_eq!(
            moved[field], before[field],
            "post-update pose retains {field}"
        );
    }
    let mut next_payload = snapshot(&f.root, f.limits).assets()[&candidate]
        .payload()
        .value()
        .clone();
    if footprint {
        for pad in next_payload["pads"].as_object_mut().unwrap().values_mut() {
            let x = pad["position"]["x"].as_i64().unwrap();
            pad["position"]["x"] = json!(x + 100000);
        }
    } else {
        next_payload["aperture"]["circle"]["diameter_nm"] = json!(1300000);
    }
    let next = f.revise(&candidate, next_payload);
    let unchanged = files(&f.root);
    let next_review = native::library_assets::inspect_geometry_selection(
        &native::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        f.request(&next)["request"].clone(),
    )
    .unwrap();
    assert_eq!(
        next_review["physical_geometry"]["placements"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(files(&f.root), unchanged);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn public_shared_footprint_geometry_preserves_users_pose_copper_and_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_shared_footprint_geometry_preserves_users_pose_copper_and_history"
    );
    if !isolated(NAME) {
        run_geometry(true);
    }
}

#[test]
fn public_shared_padstack_geometry_preserves_users_pose_copper_and_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_shared_padstack_geometry_preserves_users_pose_copper_and_history"
    );
    if !isolated(NAME) {
        run_geometry(false);
    }
}
