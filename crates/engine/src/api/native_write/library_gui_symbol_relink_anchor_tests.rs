//! Raw anchor correction retains geometry, logical-pin identity and canonical history.
use super::*;
use crate::api::native_write::{commit_prepared, registry::find_native_write_verb};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Correct anchor logical pin")
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
fn others(model: &DesignModel, symbol: Uuid) -> BTreeMap<String, Value> {
    model
        .source_shards
        .iter()
        .filter(|s| s.shard_id != model.objects[&symbol].source_shard_id)
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
fn edit(
    model: &DesignModel,
    root: &Path,
    symbol: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":fields}),
    )
}
fn anchor(pin: Uuid, x: i64) -> Value {
    json!({"pin":pin,"position":{"x":x,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}})
}
struct Fixture {
    root: PathBuf,
    model: DesignModel,
    unit: Uuid,
    symbol: Uuid,
    pins: [Uuid; 4],
    old: Uuid,
    target: Uuid,
    other: Uuid,
}
fn fixture(name: &str, numeric_target: bool) -> Fixture {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture(name);
    let [unit, symbol, b, c, d] = std::array::from_fn(|_| Uuid::new_v4());
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let mut logical = library::pool_unit_payload(unit, "Retained", "Maker");
    logical["pins"] = json!({
        a.simple().to_string():{"uuid":a,"name":"Input","direction":"Input","swap_group":0,"alternates":[]},
        b.simple().to_string().to_uppercase():{"uuid":b,"name":"Output","direction":"Output","swap_group":7,"alternates":[],"custom":[3,1]},
        c.to_string():{"uuid":c,"name":"Supply","direction":"PowerIn","swap_group":0,"alternates":[]},
        d.to_string():{"uuid":d,"name":"Spare","direction":"Passive","swap_group":0,"alternates":[]}});
    let (old, target, other) = if numeric_target { (b, a, c) } else { (a, c, b) };
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Mixed body");
    graphic["drawings"] = json!([{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},{"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}}]);
    graphic["fields"] =
        json!([{"key":"same","value":"first","unknown":3},{"key":"same","value":"second"}]);
    graphic["pin_anchors"] = json!([{"pin":old.simple().to_string(),"position":{"x":-2000000,"y":0,"custom":true},"orientation":"Right","length_nm":1000000,"custom":[3,1]},anchor(other,2000000)]);
    graphic["custom"] = json!([3, 1, 3]);
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit),
                object: logical,
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: graphic,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(model.objects[&a].kind, "pins");
    Fixture {
        root,
        model,
        unit,
        symbol,
        pins: [a, b, c, d],
        old,
        target,
        other,
    }
}

#[test]
fn correction_composes_pose_addition_raw_spelling_history_and_cold_without_unit_edits() {
    for numeric_target in [false, true] {
        let mut f = fixture(
            &format!("anchor-correct-raw-{numeric_target}"),
            numeric_target,
        );
        let original = raw(&f.model, f.symbol);
        let unchanged = others(&f.model, f.symbol);
        let counters: BTreeMap<_, _> = f
            .pins
            .iter()
            .map(|id| (*id, f.model.objects[id].object_revision))
            .collect();
        let mut poses = original["pin_anchors"].clone();
        poses[1]["position"]["x"] = json!(3000000);
        let new = anchor(f.pins[3], 0);
        let fields = json!({"name":"Corrected","pin_anchors":poses,"anchor_relinks":{f.old.simple().to_string():f.target},"anchor_additions":[new.clone()]});
        let journal = crate::substrate::transaction_journal_path(&f.root);
        let bytes = std::fs::read(&journal).unwrap();
        let prepared = edit(&f.model, &f.root, f.symbol, fields).unwrap();
        assert_eq!(raw(&f.model, f.symbol), original);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        for id in [f.symbol, f.unit, f.old, f.target] {
            assert!(prepared.batch.operations.iter().any(|op|matches!(op,crate::substrate::Operation::GuardObjectRevision{object_id,..}if *object_id==id)));
        }
        let mut expected = original.clone();
        expected["name"] = json!("Corrected");
        expected["pin_anchors"] = poses;
        expected["pin_anchors"][0]["pin"] = json!(f.target);
        expected["pin_anchors"].as_array_mut().unwrap().push(new);
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
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
                3 => f.model = ProjectResolver::new(&f.root).resolve().unwrap(),
                _ => {}
            }
            assert_eq!(
                raw(&f.model, f.symbol),
                if phase == 1 {
                    original.clone()
                } else {
                    expected.clone()
                }
            );
            assert_eq!(others(&f.model, f.symbol), unchanged);
            for (id, rev) in &counters {
                assert_eq!(f.model.objects[id].object_revision, *rev);
            }
        }
        let pins = crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
            &f.model,
            Some(&f.symbol.to_string()),
        )
        .unwrap();
        assert_eq!(pins[0].library_pin, Some(f.target));
        assert_eq!(pins[0].position.x, -2000000);
    }
}

#[test]
fn occupied_foreign_nil_alias_conflicts_and_invalid_other_pose_refuse_without_writes() {
    let f = fixture("anchor-correct-refusals", false);
    let before = raw(&f.model, f.symbol);
    let journal = crate::substrate::transaction_journal_path(&f.root);
    let bytes = std::fs::read(&journal).unwrap();
    let mut invalid_pose = before["pin_anchors"].clone();
    invalid_pose[1]["position"]["x"] = json!("invalid retained pose");
    for fields in [
        json!({"anchor_relinks":{}}),
        json!({"anchor_relinks":[]}),
        json!({"anchor_relinks":{f.old.to_string():Uuid::nil()}}),
        json!({"anchor_relinks":{f.old.to_string():Uuid::new_v4()}}),
        json!({"anchor_relinks":{Uuid::new_v4().to_string():f.target}}),
        json!({"anchor_relinks":{f.old.to_string():f.other}}),
        json!({"anchor_relinks":{f.old.to_string():f.target,f.old.simple().to_string():f.target}}),
        json!({"anchor_relinks":{f.old.to_string():f.target},"anchor_removals":[f.old]}),
        json!({"anchor_relinks":{f.old.to_string():f.target},"anchor_additions":[anchor(f.old,0)]}),
        json!({"anchor_relinks":{f.old.to_string():f.target},"anchor_additions":[anchor(f.target,0)]}),
        json!({"anchor_relinks":{f.old.to_string():f.target,f.other.to_string():f.target}}),
        json!({"anchor_relinks":{f.old.to_string():f.other,f.other.to_string():f.old}}),
        json!({"anchor_relinks":{f.old.to_string():f.target},"pin_anchors":invalid_pose}),
        json!({"anchor_relinks":{f.old.to_string():f.target},"unit":Uuid::new_v4()}),
    ] {
        assert!(
            edit(&f.model, &f.root, f.symbol, fields.clone()).is_err(),
            "{fields}"
        );
        assert_eq!(raw(&f.model, f.symbol), before);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    let same = edit(
        &f.model,
        &f.root,
        f.symbol,
        json!({"anchor_relinks":{f.old.to_string():f.old}}),
    )
    .unwrap();
    assert!(same.batch.operations.iter().any(|op|matches!(op,crate::substrate::Operation::SetPoolLibraryObject{object,..}if *object==before)),"same-target keeps original raw reference spelling");
}

#[test]
fn actual_unit_change_after_prepare_invalidates_correction_atomically() {
    let mut f = fixture("anchor-correct-stale", false);
    let before = raw(&f.model, f.symbol);
    let correction = edit(
        &f.model,
        &f.root,
        f.symbol,
        json!({"anchor_relinks":{f.old.to_string():f.target}}),
    )
    .unwrap();
    let change=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext{model:&f.model,project_root:&f.root},provenance(),json!({"kind":"units","object_id":f.unit,"pool_path":"pool","fields":{"name":"New current Unit metadata"}})).unwrap();
    commit_prepared(&mut f.model, &f.root, change).unwrap();
    let journal = crate::substrate::transaction_journal_path(&f.root);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut f.model, &f.root, correction).is_err());
    assert_eq!(raw(&f.model, f.symbol), before);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
}

#[test]
fn actual_old_placed_pins_stay_frozen_and_new_symbol_placement_uses_corrected_reference() {
    let (root, mut model, placed, _) =
        crate::api::native_write::schematic_assignment::tests::fixture(
            "anchor-correct-real-placement",
        );
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    let sheet = source
        .sheets
        .values()
        .find(|sheet| sheet.symbols.contains_key(&placed))
        .unwrap();
    let sheet_id = sheet.uuid;
    let symbol: Uuid = sheet.symbols[&placed]
        .lib_id
        .as_ref()
        .unwrap()
        .parse()
        .unwrap();
    let before = raw(&model, symbol);
    let unit: Uuid = serde_json::from_value(before["unit"].clone()).unwrap();
    let old: Uuid = serde_json::from_value(before["pin_anchors"][0]["pin"].clone()).unwrap();
    let target = Uuid::new_v4();
    let addition=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"pin_additions":{target.to_string():{"name":"Corrected supply","electrical_type":"PowerIn"}}}})).unwrap();
    commit_prepared(&mut model, &root, addition).unwrap();
    let old_symbols: BTreeMap<_, _> = source
        .sheets
        .values()
        .flat_map(|sheet| sheet.symbols.iter())
        .map(|(id, symbol)| (*id, symbol.clone()))
        .collect();
    let frozen = model
        .materialized_placed_symbol_definition(placed, symbol)
        .unwrap()
        .unwrap();
    let unchanged = others(&model, symbol);
    let mut expected = before.clone();
    expected["pin_anchors"][0]["pin"] = json!(target);
    let prepared = edit(
        &model,
        &root,
        symbol,
        json!({"anchor_relinks":{old.to_string():target}}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    for phase in 0..4 {
        match phase {
            1 => {
                model
                    .commit_journal_undo(&root, provenance().into())
                    .unwrap();
            }
            2 => {
                model
                    .commit_journal_redo(&root, provenance().into())
                    .unwrap();
            }
            3 => model = ProjectResolver::new(&root).resolve().unwrap(),
            _ => {}
        }
        assert_eq!(
            raw(&model, symbol),
            if phase == 1 {
                before.clone()
            } else {
                expected.clone()
            }
        );
        assert_eq!(others(&model, symbol), unchanged);
        assert_eq!(
            model
                .materialized_placed_symbol_definition(placed, symbol)
                .unwrap(),
            Some(frozen.clone())
        );
    }
    // Consumer phase is a separate, explicit canonical placement. It does not
    // rewrite the actual prior placed pin records or their source definition.
    let prepared=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"sheet":sheet_id,"library_symbol":symbol,"reference":"U91","value":"Corrected consumer","x_nm":20000000,"y_nm":10000000})).unwrap();
    let added = prepared.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    let next = &source.sheets[&sheet_id].symbols[&added];
    let pin = next
        .pins
        .iter()
        .find(|pin| pin.library_pin == Some(target))
        .unwrap();
    assert_eq!(pin.name, "Corrected supply");
    assert_eq!(
        pin.electrical_type,
        crate::schematic::PinElectricalType::PowerIn
    );
    assert_eq!(
        pin.position.x - next.position.x,
        before["pin_anchors"][0]["position"]["x"].as_i64().unwrap()
    );
    for (id, old_symbol) in &old_symbols {
        assert_eq!(
            source
                .sheets
                .values()
                .find_map(|sheet| sheet.symbols.get(id)),
            Some(old_symbol)
        );
    }
    assert_eq!(
        model
            .materialized_placed_symbol_definition(placed, symbol)
            .unwrap(),
        Some(frozen)
    );
}
