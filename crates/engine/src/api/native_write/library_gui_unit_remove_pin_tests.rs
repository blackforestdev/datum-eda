//! Unused pin deletion composes raw intent; actual references and stale commits refuse.
use super::*;
use crate::api::native_write::{commit_prepared, registry::find_native_write_verb};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Remove unused logical pin")
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let source = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&source.relative_path)
        .unwrap()
}
fn edit(
    model: &DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
    )
}
fn canonical(
    model: &mut DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    object: Value,
    create: bool,
) {
    let target = PoolLibraryObjectTarget::new("pool", kind, id);
    let operation = if create {
        PoolLibraryOperationSpec::Create { target, object }
    } else {
        PoolLibraryOperationSpec::Set { target, object }
    };
    let prepared =
        library::build_pool_library_write(model, provenance(), Some("pool"), vec![operation])
            .unwrap();
    commit_prepared(model, root, prepared).unwrap();
}
struct Fixture {
    root: PathBuf,
    model: DesignModel,
    unit: Uuid,
    symbol: Uuid,
    part: Uuid,
    gate: Uuid,
    pad: Uuid,
    pins: [Uuid; 3],
    original_placed: Uuid,
    other_entity: Uuid,
}
fn fixture(name: &str) -> Fixture {
    let (root, mut model, original_placed, original_part) =
        crate::api::native_write::schematic_assignment::tests::fixture(name);
    let package: Uuid =
        serde_json::from_value(raw(&model, original_part)["package"].clone()).unwrap();
    let other_entity =
        serde_json::from_value(raw(&model, original_part)["entity"].clone()).unwrap();
    let footprint = *model
        .objects
        .iter()
        .find(|(_, object)| object.domain == "pool" && object.kind == "footprints")
        .map(|(id, _)| id)
        .unwrap();
    let pad = raw(&model, footprint)["pads"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let [unit, symbol, entity, gate, part, b, c] = std::array::from_fn(|_| Uuid::new_v4());
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let mut logical = library::pool_unit_payload(unit, "Raw Unit", "Maker");
    logical["pins"] = json!({
        a.simple().to_string():{"uuid":a,"name":"Spare","direction":"Input","swap_group":0,"alternates":[],"custom":[3,1]},
        b.simple().to_string().to_uppercase():{"uuid":b,"name":"Used","direction":"Output","swap_group":7,"alternates":[],"custom":{"retain":true}},
        c.to_string():{"uuid":c,"name":"Spare","direction":"Passive","swap_group":0,"alternates":[]}});
    logical["custom"] = json!({"untouched":[null,9]});
    canonical(&mut model, &root, "units", unit, logical, true);
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Pin use");
    graphic["drawings"] = json!([{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}]);
    graphic["pin_anchors"] = json!([{"pin":b,"position":{"x":0,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]);
    canonical(&mut model, &root, "symbols", symbol, graphic, true);
    let entity_raw =
        library::pool_entity_payload(entity, gate, unit, symbol, "Entity", "U", "Maker", "A");
    canonical(&mut model, &root, "entities", entity, entity_raw, true);
    let mut part_raw = library::pool_part_payload(
        part, entity, package, "Part", "Maker", "Value", "", "", "Active",
    );
    part_raw["default_footprint"] = json!(footprint);
    canonical(&mut model, &root, "parts", part, part_raw, true);
    Fixture {
        root,
        model,
        unit,
        symbol,
        part,
        gate,
        pad,
        pins: [a, b, c],
        original_placed,
        other_entity,
    }
}
fn other_sources(model: &DesignModel, unit: Uuid) -> BTreeMap<String, Value> {
    model
        .source_shards
        .iter()
        .filter(|s| s.shard_id != model.objects[&unit].source_shard_id)
        .map(|s| {
            (
                s.relative_path.clone(),
                model
                    .materialized_source_shard_value_by_relative_path(&s.relative_path)
                    .unwrap(),
            )
        })
        .collect()
}

#[test]
fn unused_pin_raw_alias_composition_selected_invalid_discard_history_and_cold() {
    let mut f = fixture("unit-remove-compose");
    let [a, b, c] = f.pins;
    assert_eq!(f.model.objects[&a].kind, "pins");
    let original = raw(&f.model, f.unit);
    let others = other_sources(&f.model, f.unit);
    let mut metadata = original["pins"].clone();
    metadata[a.simple().to_string()]["name"] = json!("");
    metadata[a.simple().to_string()]["direction"] = json!("invalid discarded selected type");
    let retained_key = b.simple().to_string().to_uppercase();
    metadata[&retained_key]["name"] = json!("Edited");
    let d = Uuid::new_v4();
    let fields = json!({"name":"Edited Unit","pins":metadata,"pin_removals":[a.simple().to_string(),c],
        "pin_additions":{d.to_string():{"name":"Spare","electrical_type":"Input"}}});
    let bytes = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let p = edit(&f.model, &f.root, "units", f.unit, fields).unwrap();
    assert_eq!(raw(&f.model, f.unit), original);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
        bytes
    );
    for id in [f.unit, a, c] {
        assert!(p.batch.operations.iter().any(|op| matches!(op,crate::substrate::Operation::GuardObjectRevision{object_id,..} if *object_id==id)));
        let mut stale = f.model.clone();
        stale.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        assert!(stale.commit(p.batch.clone()).is_err());
    }
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let mut expected = original.clone();
    expected["name"] = json!("Edited Unit");
    expected["pins"]
        .as_object_mut()
        .unwrap()
        .retain(|key, _| key.parse::<Uuid>().unwrap() == b);
    expected["pins"][&retained_key]["name"] = json!("Edited");
    expected["pins"][d.to_string()] = json!({"uuid":d,"name":"Spare","direction":"Input","electrical_type":"Input","swap_group":0,"alternates":[]});
    for phase in 0..4 {
        match phase {
            1 => {
                f.model
                    .commit_journal_undo(&f.root, provenance().into())
                    .unwrap();
            }
            2 => {
                f.model
                    .commit_journal_redo(&f.root, provenance().into())
                    .unwrap();
            }
            3 => {
                f.model = ProjectResolver::new(&f.root).resolve().unwrap();
            }
            _ => {}
        }
        assert_eq!(
            raw(&f.model, f.unit),
            if phase == 1 {
                original.clone()
            } else {
                expected.clone()
            }
        );
        assert_eq!(other_sources(&f.model, f.unit), others);
        assert_eq!(f.model.objects.contains_key(&a), phase == 1);
        assert_eq!(f.model.objects.contains_key(&c), phase == 1);
    }
    // Separate addition preserves existing current-ownership semantics; no tombstones.
    let p = edit(
        &f.model,
        &f.root,
        "units",
        f.unit,
        json!({"pin_additions":{a.to_string():{"name":"Spare","electrical_type":"Input"}}}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(f.model.objects.contains_key(&a));
    assert_eq!(
        raw(&f.model, f.unit)["pins"][a.to_string()]["name"],
        json!("Spare")
    );
}

#[test]
fn last_unused_pin_removal_allows_existing_empty_unit_semantics() {
    let mut f = fixture("unit-remove-last");
    let p = edit(
        &f.model,
        &f.root,
        "symbols",
        f.symbol,
        json!({"anchor_removals":[f.pins[1]]}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let original = raw(&f.model, f.unit);
    let others = other_sources(&f.model, f.unit);
    let p = edit(
        &f.model,
        &f.root,
        "units",
        f.unit,
        json!({"pin_removals":f.pins}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    assert!(
        raw(&f.model, f.unit)["pins"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    for id in f.pins {
        assert!(!f.model.objects.contains_key(&id));
    }
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(raw(&f.model, f.unit), original);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(
        raw(&f.model, f.unit)["pins"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    assert_eq!(other_sources(&f.model, f.unit), others);
}

#[test]
fn unit_pin_invalid_removals_wrong_kind_other_dirty_and_same_id_refuse_atomically() {
    let f = fixture("unit-remove-invalid");
    let a = f.pins[0];
    let original = raw(&f.model, f.unit);
    let bytes = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let mut dirty = original["pins"].clone();
    dirty[f.pins[1].simple().to_string().to_uppercase()]["name"] = json!("");
    for fields in [
        json!({"pin_removals":[]}),
        json!({"pin_removals":[a,a.simple().to_string()]}),
        json!({"pin_removals":[Uuid::new_v4()]}),
        json!({"pin_removals":[a],"pins":dirty}),
        json!({"pin_removals":[a],"pin_additions":{a.to_string():{"name":"replacement","electrical_type":"Input"}}}),
    ] {
        assert!(edit(&f.model, &f.root, "units", f.unit, fields).is_err());
    }
    assert!(
        edit(
            &f.model,
            &f.root,
            "symbols",
            f.symbol,
            json!({"pin_removals":[a]})
        )
        .is_err()
    );
    let mut wrong = f.model.clone();
    wrong.objects.get_mut(&a).unwrap().kind = "units".into();
    assert!(
        edit(
            &wrong,
            &f.root,
            "units",
            f.unit,
            json!({"pin_removals":[a]})
        )
        .is_err()
    );
    assert_eq!(raw(&f.model, f.unit), original);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
        bytes
    );
    let error = edit(
        &f.model,
        &f.root,
        "units",
        f.unit,
        json!({"pin_removals":[f.pins[1]]}),
    )
    .unwrap_err();
    assert!(error.to_string().contains("Symbol anchor"));
}

#[test]
fn newly_committed_anchor_or_map_invalidates_prepared_unused_pin_removal() {
    for mapping in [false, true] {
        let mut f = fixture(&format!("unit-remove-stale-{mapping}"));
        let pin = f.pins[2];
        let prepared = edit(
            &f.model,
            &f.root,
            "units",
            f.unit,
            json!({"pin_removals":[pin]}),
        )
        .unwrap();
        if mapping {
            let map = Uuid::new_v4();
            let object = json!({"schema_version":1,"uuid":map,"part":f.part,"mappings":{f.pad.to_string():{"gate":f.gate,"pin":pin}},"tags":[]});
            canonical(&mut f.model, &f.root, "pin_pad_maps", map, object, true);
        } else {
            let p=edit(&f.model,&f.root,"symbols",f.symbol,json!({"anchor_additions":[{"pin":pin,"position":{"x":0,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]})).unwrap();
            commit_prepared(&mut f.model, &f.root, p).unwrap();
        }
        let unit = raw(&f.model, f.unit);
        let bytes = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
        assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
        assert_eq!(raw(&f.model, f.unit), unit);
        assert!(f.model.objects.contains_key(&pin));
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
            bytes
        );
    }
}

#[path = "library_gui_unit_pin_usage_tests.rs"]
mod usage_tests;
