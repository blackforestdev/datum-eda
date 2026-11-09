//! Package correction composes explicit defaults and never adopts old users.
use super::{tests::*, *};
use crate::api::native_write::{
    commit_prepared,
    registry::find_native_write_verb,
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
    let p =
        library::build_pool_library_write(model, provenance(), Some("pool"), vec![spec]).unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn new_package(model: &mut DesignModel, root: &Path, old: Uuid, omit_pads: bool) -> Uuid {
    let id = Uuid::new_v4();
    let mut package = raw(model, "packages", old);
    package["uuid"] = json!(id);
    package["name"] = json!("Existing alternate Package");
    for field in ["terminals", "pads"] {
        if let Some(rows) = package[field].as_object() {
            package[field] = Value::Object(
                rows.values()
                    .map(|row| {
                        let id = Uuid::new_v4();
                        let mut row = row.clone();
                        row["uuid"] = json!(id);
                        (id.to_string(), row)
                    })
                    .collect(),
            );
        }
    }
    if omit_pads {
        package.as_object_mut().unwrap().remove("pads");
    }
    store(model, root, "packages", id, package, true);
    id
}
#[test]
fn saved_part_package_composes_footprint_and_clear_preserving_raw_guards_and_cold_history() {
    let (root, mut model, entity, package, other, foot) = fixture();
    let part = Uuid::new_v4();
    let terminal = Uuid::new_v4();
    let mut descriptor = raw(&model, "packages", other);
    descriptor["terminals"] =
        json!({terminal.to_string():{"uuid":terminal,"name":"1","role":"lead"}});
    store(&mut model, &root, "packages", other, descriptor, false);
    let second = Uuid::new_v4();
    store(
        &mut model,
        &root,
        "footprints",
        second,
        super::super::super::library_footprint::pool_footprint_payload(
            second,
            other,
            "Alternate land",
        ),
        true,
    );
    let p = prepare(&model, &root, part, fields(entity, package, Some(foot))).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut original = raw(&model, "parts", part);
    original["package"] = json!(package.simple().to_string());
    original["tags"] = json!(["z", "a", "z"]);
    original["custom"] = json!({"ordered":[3,1,3]});
    original
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    store(&mut model, &root, "parts", part, original.clone(), false);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(edit(&model, &root, part, json!({"package":other})).is_err());
    let p = edit(
        &model,
        &root,
        part,
        json!({"package":other,"default_footprint":second}),
    )
    .unwrap();
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert_eq!(raw(&model, "parts", part), original);
    for id in [other, second, entity] {
        assert!(p.batch.operations.iter().any(
            |op| matches!(op,Operation::GuardObjectRevision{object_id,expected_object_revision}
            if *object_id==id && *expected_object_revision==model.objects[&id].object_revision)
        ));
    }
    let mut changed = raw(&model, "packages", other);
    changed["name"] = json!("Concurrent descriptor name");
    store(&mut model, &root, "packages", other, changed.clone(), false);
    let after = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut model, &root, p).is_err());
    assert_eq!(std::fs::read(&journal).unwrap(), after);
    let p = edit(
        &model,
        &root,
        part,
        json!({"package":other,"default_footprint":second}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut selected = original.clone();
    selected["package"] = json!(other);
    selected["default_footprint"] = json!(second);
    assert_eq!(raw(&model, "parts", part), selected);
    assert_eq!(raw(&model, "packages", other), changed);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "parts", part), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "parts", part), selected);
    let p = edit(
        &model,
        &root,
        part,
        json!({"package":package,"default_footprint":null}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut cleared = selected.clone();
    cleared["package"] = json!(package);
    cleared.as_object_mut().unwrap().remove("default_footprint");
    assert_eq!(raw(&model, "parts", part), cleared);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "parts", part), selected);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "parts", part), cleared);
    for fields in [
        json!({"package":entity}),
        json!({"package":Uuid::new_v4()}),
        json!({"package":null}),
        json!({"package":""}),
        json!({"package":false}),
    ] {
        assert!(edit(&model, &root, part, fields).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_part_package_untouched_null_or_absent_footprint_never_becomes_an_inferred_default() {
    let (root, mut model, entity, package, other, _) = fixture();
    for absent in [false, true] {
        let part = Uuid::new_v4();
        let p = prepare(&model, &root, part, fields(entity, package, None)).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut before = raw(&model, "parts", part);
        if absent {
            before.as_object_mut().unwrap().remove("default_footprint");
        }
        store(&mut model, &root, "parts", part, before.clone(), false);
        let p = edit(&model, &root, part, json!({"package":other})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut expected = before.clone();
        expected["package"] = json!(other);
        assert_eq!(raw(&model, "parts", part), expected);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, "parts", part), before);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, "parts", part), expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_part_package_preserves_old_multigate_edits_and_new_explicit_declared_part() {
    use crate::api::native_write::schematic_delete_proposal::NativeProposalAction as Action;
    let (root, mut model, first, req) = gate_fixture::setup("part-package-old-multigate");
    let ci = req.component_instance;
    let options = gates::options(&model, first, &[]).unwrap();
    let part = options.part;
    let original = raw(&model, "parts", part);
    let old_package = serde_json::from_value(original["package"].clone()).unwrap();
    let package = new_package(&mut model, &root, old_package, false);
    gate_fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    gate_fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let before = gate_fixture::raw(&model, first);
    let owner = gate_fixture::raw(&model, ci);
    let ids = model.electrical_identities.clone();
    let p = edit(&model, &root, part, json!({"package":package})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for n in 0..3 {
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(gate_fixture::raw(&model, first), before);
        assert_eq!(gate_fixture::raw(&model, ci), owner);
        assert_eq!(model.electrical_identities, ids);
        assert_eq!(
            gates::options(&model, first, &[]).unwrap().entity,
            options.entity
        );
        if n == 0 {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        if n == 1 {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        }
    }
    let second = model.component_instances[&ci]
        .placed_symbol_refs
        .iter()
        .copied()
        .find(|id| *id != first)
        .unwrap();
    for params in [
        json!({"symbol":second,"instance_path":[],"x_nm":61_000_000,"y_nm":31_000_000,"rotation_deg":90,"mirrored":true}),
        json!({"symbols":[first,second],"instance_path":[],"reference":{"x":0,"y":0},"x_nm":5_000_000,"y_nm":5_000_000,"rotation_deg":90,"mirrored":false}),
    ] {
        let p = (find_native_write_verb("datum.schematic.move_symbol")
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
        commit_prepared(&mut model, &root, p).unwrap();
        let moved = gate_fixture::raw(&model, first);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(gate_fixture::raw(&model, first), moved);
        assert_eq!(
            gate_fixture::raw(&model, ci)["component_instance"]["part_ref"],
            owner["component_instance"]["part_ref"]
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(
                    part,
                    "parts",
                    crate::substrate::ObjectRevision(0)
                )
                .unwrap()["package"],
            json!(old_package)
        );
    }
    let current = raw(&model, "parts", part);
    assert_eq!(current["package"], json!(package));
    let sheet = gate_fixture::raw(&model, first);
    let original_symbol = &sheet["symbols"][first.to_string()];
    let gate = serde_json::from_value(original_symbol["gate"].clone()).unwrap();
    let library_symbol = serde_json::from_value(original_symbol["lib_id"].clone()).unwrap();
    let selection = PartGateSelection::capture(&model, part, gate, library_symbol).unwrap();
    let current_revision = selection.part_revision;
    let p=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"sheet":req.sheet,"instance_path":[],"library_symbol":library_symbol,"explicit_part":selection,"reference":"U9","value":"Current Package choice","x_nm":120_000_000,"y_nm":90_000_000})).unwrap();
    let fresh = p.primary_object_id.unwrap();
    let old_owner = gate_fixture::raw(&model, ci);
    commit_prepared(&mut model, &root, p).unwrap();
    let new_ci = model
        .component_instances
        .values()
        .find(|ci| ci.placed_symbol_refs.contains(&fresh))
        .unwrap()
        .id;
    let declared = gate_fixture::raw(&model, new_ci)["component_instance"]["part_ref"].clone();
    assert_eq!(declared["object_revision"], json!(current_revision));
    assert_eq!(
        model
            .materialized_pool_object_revision(
                part,
                "parts",
                crate::substrate::ObjectRevision(current_revision)
            )
            .unwrap()["package"],
        json!(package)
    );
    assert_eq!(gate_fixture::raw(&model, ci), old_owner);
    let placed = gate_fixture::raw(&model, fresh);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(gate_fixture::raw(&model, first), sheet);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(gate_fixture::raw(&model, fresh), placed);
    assert_eq!(gate_fixture::raw(&model, ci), old_owner);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_part_package_legacy_absent_or_empty_pad_membership_and_duplicate_owned_ids_refuse() {
    let (root, mut model, first, _) = gate_fixture::setup("part-package-legacy-refusal");
    let options = gates::options(&model, first, &[]).unwrap();
    let part = options.part;
    let mut before = raw(&model, "parts", part);
    let package = serde_json::from_value(before["package"].clone()).unwrap();
    let mut descriptor = raw(&model, "packages", package);
    let collision = Uuid::new_v4();
    let mut duplicate = descriptor.clone();
    duplicate["uuid"] = json!(collision);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "packages", collision),
            object: duplicate,
        }],
    );
    assert!(
        p.and_then(|p| commit_prepared(&mut model, &root, p))
            .is_err()
    );
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    let pad = Uuid::new_v4();
    let padstack = model
        .objects
        .values()
        .find(|o| o.domain == "pool" && o.kind == "padstacks")
        .unwrap()
        .object_id;
    descriptor["pads"] = json!({pad.to_string():{"uuid":pad,"name":"Legacy","position":{"x":0,"y":0},"padstack":padstack,"layer":0}});
    store(&mut model, &root, "packages", package, descriptor, false);
    let entity = raw(&model, "entities", options.entity);
    let row = entity["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    let gate: Uuid = serde_json::from_value(row["uuid"].clone()).unwrap();
    let unit = serde_json::from_value(row["unit"].clone()).unwrap();
    let logical = raw(&model, "units", unit);
    let pin = logical["pins"].as_object().unwrap().keys().next().unwrap();
    before["pad_map"] = json!({pad.to_string():{"gate":gate,"pin":pin}});
    store(&mut model, &root, "parts", part, before.clone(), false);
    for absent in [false, true] {
        let other = new_package(&mut model, &root, package, absent);
        if !absent {
            let mut p = raw(&model, "packages", other);
            p["pads"] = json!({});
            store(&mut model, &root, "packages", other, p, false);
        }
        let bytes = std::fs::read(&journal).unwrap();
        assert!(
            edit(
                &model,
                &root,
                part,
                json!({"package":other,"default_footprint":null,"default_pin_pad_map":null})
            )
            .unwrap_err()
            .to_string()
            .contains("legacy")
        );
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, "parts", part), before);
    }
    std::fs::remove_dir_all(root).unwrap();
}
