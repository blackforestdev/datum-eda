//! Saved anchor removal preserves logical pins, ordered raw siblings and frozen users.
use super::*;
use crate::api::native_write::{
    commit_prepared, registry::find_native_write_verb, schematic_gate_placement as placement,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::{collections::BTreeMap, path::Path};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Remove saved anchor")
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
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
fn anchor(pin: Uuid, x: i64) -> Value {
    json!({"pin":pin,"position":{"x":x,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}})
}
fn make(model: &mut DesignModel, root: &Path) -> (Uuid, Uuid, [Uuid; 3]) {
    let [unit, symbol, b, c] = std::array::from_fn(|_| Uuid::new_v4());
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let mut u = library::pool_unit_payload(unit, "Retained logical pins", "Maker");
    u["pins"] = json!({a.simple().to_string():{"uuid":a,"name":"Input","direction":"Input","swap_group":0,"alternates":[]},b.to_string():{"uuid":b,"name":"Output","direction":"Output","swap_group":7,"alternates":[],"custom":[3,1]},c.to_string():{"uuid":c,"name":"Supply","direction":"PowerIn","swap_group":0,"alternates":[]}});
    let mut s = library::pool_symbol_payload(symbol, unit, "Mixed body");
    s["drawings"] = json!([{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},{"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}}]);
    s["fields"] =
        json!([{"key":"same","value":"first","unknown":3},{"key":"same","value":"second"}]);
    s["pin_anchors"] = json!([{"pin":a.simple().to_string(),"position":{"x":-2000000,"y":0,"custom":true},"orientation":"Right","length_nm":1000000,"custom":[3,1]},anchor(b,2000000)]);
    s["custom"] = json!([3, 1, 3]);
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some("pool"),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit),
                object: u,
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: s,
            },
        ],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
    (unit, symbol, [a, b, c])
}
#[test]
fn saved_symbol_remove_anchor_composes_other_pose_addition_and_raw_history_without_unit_changes() {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-remove-compose");
    let (unit, symbol, [a, b, c]) = make(&mut model, &root);
    let original = raw(&model, symbol);
    let unchanged = others(&model, symbol);
    let counters: BTreeMap<_, _> = [a, b, c]
        .into_iter()
        .map(|id| (id, model.objects[&id].object_revision))
        .collect();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let mut poses = original["pin_anchors"].clone();
    poses[0]["position"]["x"] = json!("discard invalid selected pose");
    poses[1]["position"]["x"] = json!(3000000);
    let row = anchor(c, 0);
    let p=edit(&model,&root,"symbols",symbol,json!({"pin_anchors":poses,"anchor_removals":[a.simple().to_string()],"anchor_additions":[row.clone()]})).unwrap();
    assert_eq!(raw(&model, symbol), original);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    for id in [symbol, unit, a, c] {
        assert!(p.batch.operations.iter().any(|op|matches!(op,crate::substrate::Operation::GuardObjectRevision{object_id,..}if *object_id==id)));
        let mut stale = model.clone();
        stale.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        assert!(stale.commit(p.batch.clone()).is_err());
    }
    let mut expected = original.clone();
    expected["pin_anchors"] = json!([poses[1].clone(), row]);
    commit_prepared(&mut model, &root, p).unwrap();
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
            3 => {
                model = ProjectResolver::new(&root).resolve().unwrap();
            }
            _ => {}
        }
        assert_eq!(
            raw(&model, symbol),
            if phase == 1 {
                original.clone()
            } else {
                expected.clone()
            }
        );
        assert_eq!(others(&model, symbol), unchanged);
        for (id, rev) in &counters {
            assert_eq!(model.objects[id].object_revision, *rev);
        }
        let pins = crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
            &model,
            Some(&symbol.to_string()),
        )
        .unwrap();
        assert_eq!(pins.len(), 2);
        assert_eq!(pins.iter().any(|p| p.library_pin == Some(a)), phase == 1);
        assert!(pins.iter().any(|p| p.library_pin == Some(b)));
        assert_eq!(pins.iter().any(|p| p.library_pin == Some(c)), phase != 1);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_symbol_remove_anchor_refuses_missing_duplicate_alias_samepin_wrongkind_and_malformed_source()
 {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-remove-refusal");
    let (unit, symbol, [a, _, _]) = make(&mut model, &root);
    let original = raw(&model, symbol);
    let unchanged = others(&model, symbol);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    for fields in [
        json!({"anchor_removals":[]}),
        json!({"anchor_removals":[Uuid::new_v4()]}),
        json!({"anchor_removals":[a,a.simple().to_string()]}),
        json!({"anchor_removals":{"pin":a}}),
        json!({"anchor_removals":[a],"anchor_additions":[anchor(a,0)]}),
    ] {
        assert!(edit(&model, &root, "symbols", symbol, fields).is_err());
        assert_eq!(raw(&model, symbol), original);
        assert_eq!(others(&model, symbol), unchanged);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    assert!(edit(&model, &root, "units", unit, json!({"anchor_removals":[a]})).is_err());
    for ambiguous in [true, false] {
        let mut bad = original.clone();
        if ambiguous {
            let mut alias = bad["pin_anchors"][0].clone();
            alias["pin"] = json!(a);
            bad["pin_anchors"].as_array_mut().unwrap().push(alias);
        } else {
            bad["pin_anchors"][1]["pin"] = json!(Uuid::new_v4());
        }
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            None,
            vec![PoolLibraryOperationSpec::Set {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: bad.clone(),
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let bytes = std::fs::read(&journal).unwrap();
        assert!(
            edit(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"anchor_removals":[a]})
            )
            .is_err()
        );
        assert_eq!(raw(&model, symbol), bad);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_symbol_remove_all_anchors_including_last_keeps_frozen_bound_pins_and_later_empty_placement()
 {
    let (root, mut model, first, request) = placement::tests::setup("anchor-remove-last");
    let sheet = placement::tests::raw(&model, first);
    let placed = &sheet["symbols"][first.to_string()];
    let symbol: Uuid = serde_json::from_value(placed["lib_id"].clone()).unwrap();
    let part: Uuid = serde_json::from_value(placed["part"].clone()).unwrap();
    let gate: Uuid = serde_json::from_value(placed["gate"].clone()).unwrap();
    let original = raw(&model, symbol);
    let removed: Vec<Uuid> = original["pin_anchors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| serde_json::from_value(a["pin"].clone()).unwrap())
        .collect();
    assert!(!removed.is_empty());
    let unchanged = others(&model, symbol);
    let captured = model
        .materialized_placed_symbol_definition(first, symbol)
        .unwrap()
        .unwrap();
    let binding = placement::tests::binding(&model, request.component_instance);
    let p = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"anchor_removals":removed}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut empty = original.clone();
    empty["pin_anchors"] = json!([]);
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
            3 => {
                model = ProjectResolver::new(&root).resolve().unwrap();
            }
            _ => {}
        }
        assert_eq!(
            raw(&model, symbol),
            if phase == 1 {
                original.clone()
            } else {
                empty.clone()
            }
        );
        assert_eq!(others(&model, symbol), unchanged);
        assert_eq!(placement::tests::raw(&model, first), sheet);
        assert_eq!(
            placement::tests::binding(&model, request.component_instance),
            binding
        );
        assert_eq!(
            model
                .materialized_placed_symbol_definition(first, symbol)
                .unwrap(),
            Some(captured.clone())
        );
        for pin in &removed {
            assert!(model.objects.contains_key(pin));
        }
        assert_eq!(
            crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
                &model,
                Some(&symbol.to_string())
            )
            .unwrap()
            .len(),
            if phase == 1 { removed.len() } else { 0 }
        );
    }
    let intent =
        crate::api::native_write::schematic_library::part_placement::PartGateSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
    let p=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),json!({"sheet":sheet["uuid"],"library_symbol":symbol,"explicit_part":intent,"reference":"U_EMPTY","value":"No anchors","x_nm":40000000,"y_nm":10000000})).unwrap();
    let added = p.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let actual = placement::tests::raw(&model, added);
    assert!(
        actual["symbols"][added.to_string()]["pins"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(actual["symbols"][first.to_string()], placed.clone());
    assert_eq!(
        placement::tests::binding(&model, request.component_instance),
        binding
    );
    assert_eq!(
        model
            .materialized_placed_symbol_definition(added, symbol)
            .unwrap(),
        Some(empty)
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn absent_and_empty_anchor_sources_do_not_gain_phantom_rows_and_refuse_missing_removal() {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-remove-absent");
    let (_, symbol, [a, _, _]) = make(&mut model, &root);
    for absent in [true, false] {
        let mut source = raw(&model, symbol);
        if absent {
            source.as_object_mut().unwrap().remove("pin_anchors");
        } else {
            source["pin_anchors"] = json!([]);
        }
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            None,
            vec![PoolLibraryOperationSpec::Set {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: source.clone(),
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let journal = crate::substrate::transaction_journal_path(&root);
        let bytes = std::fs::read(&journal).unwrap();
        assert!(
            edit(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"anchor_removals":[a]})
            )
            .is_err()
        );
        assert_eq!(raw(&model, symbol), source);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        let p = edit(
            &model,
            &root,
            "symbols",
            symbol,
            json!({"name":if absent{"Absent anchors"}else{"Empty anchors"}}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(
            raw(&model, symbol).get("pin_anchors"),
            source.get("pin_anchors")
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
