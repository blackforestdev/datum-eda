//! Saved Entity correction is a future-use choice, never a placed component migration.
use super::{tests::*, *};
use crate::api::native_write::{
    commit_prepared,
    registry::find_native_write_verb,
    schematic_delete_proposal::{DeleteClass, DeleteTarget, NativeProposalAction as Action},
    schematic_gate_placement::{self as gates, tests as gate_fixture},
    schematic_library::part_placement::PartGateSelection,
};
use crate::substrate::{DesignModel, Operation, ProjectResolver};
use std::path::Path;

fn edit(
    model: &DesignModel,
    root: &Path,
    part: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":part,"pool_path":"pool","fields":fields}),
    )
}
fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(
        model,
        &PoolLibraryObjectTarget::new("pool", kind, id).relative_path,
        id,
    )
    .unwrap()
}
fn store(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, object: Value, create: bool) {
    let target = PoolLibraryObjectTarget::new("pool", kind, id);
    let spec = if create {
        PoolLibraryOperationSpec::Create { target, object }
    } else {
        PoolLibraryOperationSpec::Set { target, object }
    };
    let write =
        library::build_pool_library_write(model, provenance(), Some("pool"), vec![spec]).unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn replacement(model: &mut DesignModel, root: &Path, original: Uuid) -> (Uuid, Vec<Uuid>) {
    let entity = Uuid::new_v4();
    let mut next = raw(model, "entities", original);
    next["uuid"] = json!(entity);
    next["name"] = json!("Explicit replacement Entity");
    let mut ids = Vec::new();
    next["gates"] = Value::Object(
        next["gates"]
            .as_object()
            .unwrap()
            .values()
            .map(|row| {
                let id = Uuid::new_v4();
                ids.push(id);
                let mut row = row.clone();
                row["uuid"] = json!(id);
                (id.to_string(), row)
            })
            .collect(),
    );
    store(model, root, "entities", entity, next, true);
    (entity, ids)
}

#[test]
fn saved_part_entity_changes_only_requested_raw_leaves_and_guards_consumed_sources() {
    let (root, mut model, entity, package, _, foot) = fixture();
    let part = Uuid::new_v4();
    let p = prepare(&model, &root, part, fields(entity, package, Some(foot))).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let (other, _) = replacement(&mut model, &root, entity);
    let mut original = raw(&model, "parts", part);
    original["entity"] = json!(entity.simple().to_string());
    original["custom"] = json!({"ordered":[3,1,3]});
    original["tags"] = json!(["z", "a", "z"]);
    original
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    store(&mut model, &root, "parts", part, original.clone(), false);
    let journal = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal).unwrap();
    let write = edit(
        &model,
        &root,
        part,
        json!({"entity":other,"default_footprint":null}),
    )
    .unwrap();
    assert_eq!(std::fs::read(&journal).unwrap(), before);
    assert_eq!(raw(&model, "parts", part), original);
    let candidate = raw(&model, "entities", other);
    let gate = candidate["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    for id in [
        other,
        package,
        serde_json::from_value(gate["unit"].clone()).unwrap(),
        serde_json::from_value(gate["symbol"].clone()).unwrap(),
    ] {
        assert!(write.batch.operations.iter().any(|op| matches!(op,
            Operation::GuardObjectRevision { object_id, expected_object_revision }
                if *object_id == id && *expected_object_revision == model.objects[&id].object_revision)));
    }
    let mut competing = candidate.clone();
    competing["description"] = json!("Changed after preview");
    store(&mut model, &root, "entities", other, competing, false);
    let after_competing = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(std::fs::read(&journal).unwrap(), after_competing);
    assert_eq!(raw(&model, "parts", part), original);
    let write = edit(
        &model,
        &root,
        part,
        json!({"entity":other,"default_footprint":null}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut expected = original.clone();
    expected["entity"] = json!(other);
    expected
        .as_object_mut()
        .unwrap()
        .remove("default_footprint");
    assert_eq!(raw(&model, "parts", part), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "parts", part), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "parts", part), expected);
    for fields in [
        json!({"entity":package}),
        json!({"entity":Uuid::new_v4()}),
        json!({"entity":null}),
        json!({"entity":""}),
        json!({"entity":false}),
    ] {
        assert!(edit(&model, &root, part, fields).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saved_part_entity_preserves_old_multigate_edits_remaining_gate_and_new_current_selection() {
    let (root, mut model, first, mut req) = gate_fixture::setup("part-entity-old-multigate");
    let ci = req.component_instance;
    let old = gates::options(&model, first, &[]).unwrap();
    let old_entity = old.entity;
    let part = old.part;
    // A real third Gate remains available on the old component after relinking.
    let mut entity = raw(&model, "entities", old_entity);
    let third = Uuid::new_v4();
    let mut row = entity["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    row["uuid"] = json!(third);
    row["name"] = json!("Remaining old gate");
    entity["gates"][third.to_string()] = row;
    store(&mut model, &root, "entities", old_entity, entity, false);
    let second = gates::preview(&model, &req).unwrap().symbol.uuid;
    gate_fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    gate_fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let (other, new_gates) = replacement(&mut model, &root, old_entity);
    let before_sheet = gate_fixture::raw(&model, first);
    let before_ci = gate_fixture::raw(&model, ci);
    let before_part = raw(&model, "parts", part);
    let identities = model.electrical_identities.clone();
    let write = edit(&model, &root, part, json!({"entity":other})).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    for history in 0..3 {
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(gate_fixture::raw(&model, first), before_sheet);
        assert_eq!(gate_fixture::raw(&model, ci), before_ci);
        assert_eq!(model.electrical_identities, identities);
        let options = gates::options(&model, first, &[]).unwrap();
        assert_eq!(options.entity, old_entity);
        assert!(options.choices.iter().any(|g| g.gate == third));
        if history == 0 {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        if history == 1 {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        }
    }
    assert_eq!(raw(&model, "parts", part)["entity"], json!(other));
    assert_ne!(before_part["entity"], json!(other));
    let choice = gates::options(&model, first, &[])
        .unwrap()
        .choices
        .into_iter()
        .find(|g| g.gate == third)
        .unwrap();
    req.proposal_id = Uuid::new_v4();
    req.reference = "U3".into();
    req.gate = third;
    req.library_symbol = choice.library_symbol;
    req.x_nm += 20_000_000;
    let third_symbol = gates::preview(&model, &req).unwrap().symbol.uuid;
    gate_fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    gate_fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    assert_eq!(model.component_instances[&ci].placed_symbol_refs.len(), 3);
    for params in [
        json!({"symbol":second,"instance_path":[],"x_nm":61_000_000,"y_nm":31_000_000,"rotation_deg":90,"mirrored":true}),
        json!({"symbols":[first,second,third_symbol],"instance_path":[],"reference":{"x":0,"y":0},"x_nm":5_000_000,"y_nm":5_000_000,"rotation_deg":90,"mirrored":false}),
    ] {
        let write = (find_native_write_verb("datum.schematic.move_symbol")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let moved = gate_fixture::raw(&model, first);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(gate_fixture::raw(&model, first), moved);
        assert_eq!(model.component_instances[&ci].part_ref, Some(part));
        assert_eq!(
            gate_fixture::raw(&model, ci)["component_instance"]["part_ref"],
            before_ci["component_instance"]["part_ref"]
        );
    }
    let delete = Action::CreateDelete {
        proposal_id: Uuid::new_v4(),
        sheet: req.sheet,
        instance_path: vec![],
        targets: vec![DeleteTarget {
            class: DeleteClass::Symbol,
            id: second,
        }],
    };
    let draft = gate_fixture::action(&mut model, &root, delete);
    gate_fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(model.component_instances[&ci].placed_symbol_refs.len(), 2);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        gates::options(&model, first, &[]).unwrap().entity,
        old_entity
    );
    let new_entity = raw(&model, "entities", other);
    let new_row = &new_entity["gates"][new_gates[0].to_string()];
    let symbol = serde_json::from_value(new_row["symbol"].clone()).unwrap();
    let selection = PartGateSelection::capture(&model, part, new_gates[0], symbol).unwrap();
    assert_eq!(selection.entity, other);
    assert!(PartGateSelection::capture(&model, part, req.gate, symbol).is_err());
    let before_new = gate_fixture::raw(&model, first);
    let old_ci = gate_fixture::raw(&model, ci);
    let write = (find_native_write_verb("datum.schematic.place_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"sheet":req.sheet,"instance_path":[],"library_symbol":symbol,
            "explicit_part":selection,"reference":"U9","value":"Explicit current Entity",
            "x_nm":120_000_000,"y_nm":90_000_000}),
    )
    .unwrap();
    for id in [part, other, selection.unit, symbol] {
        assert!(write.batch.operations.iter().any(|op| matches!(op,
            Operation::GuardObjectRevision { object_id, expected_object_revision }
                if *object_id==id && *expected_object_revision==model.objects[&id].object_revision)));
    }
    let placed = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let after_new = gate_fixture::raw(&model, placed);
    assert_eq!(
        after_new["symbols"][placed.to_string()]["entity"],
        json!(other)
    );
    assert_eq!(
        after_new["symbols"][placed.to_string()]["gate"],
        json!(new_gates[0])
    );
    for (id, old) in before_new["symbols"].as_object().unwrap() {
        assert_eq!(after_new["symbols"][id], *old);
    }
    assert_eq!(gate_fixture::raw(&model, ci), old_ci);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(gate_fixture::raw(&model, first), before_new);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(gate_fixture::raw(&model, placed), after_new);
    assert_eq!(gate_fixture::raw(&model, ci), old_ci);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saved_part_entity_refuses_legacy_gate_mapping_and_canonical_duplicate_gate_identity() {
    let (root, mut model, first, _) = gate_fixture::setup("part-entity-legacy-refusal");
    let options = gates::options(&model, first, &[]).unwrap();
    let part = options.part;
    let original_entity = raw(&model, "entities", options.entity);
    let duplicate = Uuid::new_v4();
    let mut collision = original_entity.clone();
    collision["uuid"] = json!(duplicate);
    let journal = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal).unwrap();
    let candidate = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "entities", duplicate),
            object: collision,
        }],
    );
    assert!(
        candidate
            .and_then(|p| commit_prepared(&mut model, &root, p))
            .is_err()
    );
    assert_eq!(std::fs::read(&journal).unwrap(), before);
    assert!(
        !root
            .join(format!("pool/entities/{duplicate}.json"))
            .exists()
    );
    let (other, _) = replacement(&mut model, &root, options.entity);
    let mut original = raw(&model, "parts", part);
    let package: Uuid = serde_json::from_value(original["package"].clone()).unwrap();
    let mut package_raw = raw(&model, "packages", package);
    let pad = Uuid::new_v4();
    let padstack = model
        .objects
        .values()
        .find(|o| o.domain == "pool" && o.kind == "padstacks")
        .unwrap()
        .object_id;
    package_raw["pads"] = json!({pad.to_string():{"uuid":pad,"name":"Legacy pad","position":{"x":0,"y":0},"padstack":padstack,"layer":0}});
    store(&mut model, &root, "packages", package, package_raw, false);
    let placed = gate_fixture::raw(&model, first);
    let gate: Uuid =
        serde_json::from_value(placed["symbols"][first.to_string()]["gate"].clone()).unwrap();
    let unit: Uuid =
        serde_json::from_value(original_entity["gates"][gate.to_string()]["unit"].clone()).unwrap();
    let logical = raw(&model, "units", unit);
    let pin = logical["pins"].as_object().unwrap().keys().next().unwrap();
    original["pad_map"] = json!({pad.to_string():{"gate":gate,"pin":pin}});
    store(&mut model, &root, "parts", part, original.clone(), false);
    let before = std::fs::read(&journal).unwrap();
    for fields in [
        json!({"entity":other}),
        json!({"entity":other,"default_pin_pad_map":null}),
    ] {
        let error = edit(&model, &root, part, fields).unwrap_err();
        assert!(error.to_string().contains("gate"), "{error}");
        assert_eq!(raw(&model, "parts", part), original);
        assert_eq!(std::fs::read(&journal).unwrap(), before);
    }
    std::fs::remove_dir_all(root).unwrap();
}
