//! Sparse anchors preserve raw definitions, source authority and frozen placed pins.
use super::*;
use crate::api::native_write::{
    commit_prepared, registry::find_native_write_verb, schematic_gate_placement as placement,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::{collections::BTreeMap, path::Path};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Add saved anchor")
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
fn anchor(pin: Uuid, x: i64) -> Value {
    json!({"pin":pin,"position":{"x":x,"y":-2000000},"style":{"orientation":"Down","length_nm":1000000,"decoration":"none"}})
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
fn make(model: &mut DesignModel, root: &Path, pool: &str) -> (Uuid, Uuid, Uuid, Uuid) {
    let [unit, symbol, old] = std::array::from_fn(|_| Uuid::new_v4());
    // A valid compact numeric UUID is registered under parent kind `pins`.
    let new: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let mut logical = library::pool_unit_payload(unit, "Named retained Unit", "Maker");
    let alias = new.simple().to_string().to_uppercase();
    logical["pins"] = json!({old.to_string():{"uuid":old,"name":"Old","direction":"Input","swap_group":0,"alternates":[]},alias:{"uuid":new,"name":"Supply","direction":"PowerIn","swap_group":0,"alternates":[],"custom":[3,1]}});
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Mixed retained body");
    graphic["pin_anchors"] = json!([{"pin":old.simple().to_string().to_uppercase(),"position":{"x":-1000000,"y":0,"custom":true},"orientation":"Right","length_nm":500000,"custom":[3,1]}]);
    graphic["drawings"] = json!([{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000,"custom":true}},{"Line":{"from":{"x":-1000000,"y":0},"to":{"x":1000000,"y":0},"width":100000}}]);
    graphic["fields"] =
        json!([{"key":"same","value":"first","extra":9},{"key":"same","value":"second"}]);
    graphic["custom"] = json!({"retained":true});
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some(pool),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new(pool, "units", unit),
                object: logical,
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: graphic,
            },
        ],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
    (unit, symbol, old, new)
}
#[test]
fn raw_alias_append_preserves_mixed_source_and_guards_history_cold_and_current_materialization() {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-add-raw");
    let (unit, symbol, old, new) = make(&mut model, &root, "pool");
    assert_eq!(model.objects[&new].kind, "pins");
    let original = raw(&model, symbol);
    let retained = others(&model, symbol);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let row = anchor(new, 0);
    let p = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"anchor_additions":[row.clone()]}),
    )
    .unwrap();
    assert_eq!(raw(&model, symbol), original);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    for id in [unit, new, symbol] {
        assert!(p.batch.operations.iter().any(|op|matches!(op,crate::substrate::Operation::GuardObjectRevision{object_id,..} if *object_id==id)));
        let mut stale = model.clone();
        stale.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        assert!(stale.commit(p.batch.clone()).is_err());
    }
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = original.clone();
    expected["pin_anchors"].as_array_mut().unwrap().push(row);
    assert_eq!(raw(&model, symbol), expected);
    assert_eq!(others(&model, symbol), retained);
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
        assert_eq!(others(&model, symbol), retained);
        let pins = crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
            &model,
            Some(&symbol.to_string()),
        )
        .unwrap();
        assert_eq!(pins.len(), if phase == 1 { 1 } else { 2 });
        assert!(pins.iter().any(|p| p.library_pin == Some(old)));
        if phase != 1 {
            assert!(pins.iter().any(|p| p.library_pin == Some(new)
                && p.name == "Supply"
                && p.position.y == -2000000));
        }
    }
    let bytes = std::fs::read(&journal).unwrap();
    for rows in [
        json!([]),
        json!([anchor(Uuid::new_v4(), 0)]),
        json!([anchor(old, 0)]),
        json!([anchor(new, 0)]),
    ] {
        assert!(
            edit(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"anchor_additions":rows})
            )
            .is_err()
        );
        assert_eq!(raw(&model, symbol), expected);
        assert_eq!(others(&model, symbol), retained);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn read_only_unit_reference_accepts_unique_alias_but_ambiguous_source_or_membership_refuses() {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-add-reference");
    let (unit, symbol, _, new) = make(&mut model, &root, "pool/units/reference");
    assert!(super::super::super::saved_source::load(&model, unit, "units").is_err());
    let path = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&unit].source_shard_id)
        .unwrap()
        .relative_path
        .clone();
    assert!(path.starts_with("pool/units/reference/units/"));
    let before = raw(&model, symbol);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let mut ambiguous = model.clone();
    let shard = ambiguous
        .source_shards
        .iter()
        .find(|s| s.shard_id == ambiguous.objects[&unit].source_shard_id)
        .unwrap()
        .clone();
    ambiguous.source_shards.push(shard);
    assert!(
        edit(
            &ambiguous,
            &root,
            "symbols",
            symbol,
            json!({"anchor_additions":[anchor(new,0)]})
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    let p = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"anchor_additions":[anchor(new,0)]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(
        &raw(&model, symbol)["pin_anchors"][0],
        &before["pin_anchors"][0]
    );
    let mut duplicate = raw(&model, unit);
    let row = duplicate["pins"]
        .as_object()
        .unwrap()
        .values()
        .find(|p| p["uuid"] == json!(new))
        .unwrap()
        .clone();
    duplicate["pins"][new.to_string()] = row;
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(unit, "units", path.clone()),
            object: duplicate,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let bytes = std::fs::read(&journal).unwrap();
    let before = raw(&model, symbol);
    assert!(
        edit(
            &model,
            &root,
            "symbols",
            symbol,
            json!({"anchor_additions":[anchor(Uuid::new_v4(),0)]})
        )
        .is_err()
    );
    assert_eq!(raw(&model, symbol), before);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert!(
        crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
            &model,
            Some(&symbol.to_string())
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn composed_old_pose_and_new_anchor_retain_actual_bound_pin_capture_through_cold() {
    let (root, mut model, first, request) = placement::tests::setup("anchor-add-bound");
    let sheet = placement::tests::raw(&model, first);
    let symbol: Uuid =
        serde_json::from_value(sheet["symbols"][first.to_string()]["lib_id"].clone()).unwrap();
    let unit: Uuid = serde_json::from_value(raw(&model, symbol)["unit"].clone()).unwrap();
    let pin = Uuid::new_v4();
    let p = edit(
        &model,
        &root,
        "units",
        unit,
        json!({"pin_additions":{pin.to_string():{"name":"Supply","electrical_type":"PowerIn"}}}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let captured = model
        .materialized_placed_symbol_definition(first, symbol)
        .unwrap()
        .unwrap();
    let binding = placement::tests::binding(&model, request.component_instance);
    let original = raw(&model, symbol);
    let mut poses = original["pin_anchors"].clone();
    poses[0]["position"]["x"] = json!(-3000000);
    let p = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"pin_anchors":poses,"anchor_additions":[anchor(pin,0)]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(placement::tests::raw(&model, first), sheet);
    assert_eq!(
        placement::tests::binding(&model, request.component_instance),
        binding
    );
    assert_eq!(
        model
            .materialized_placed_symbol_definition(first, symbol)
            .unwrap(),
        Some(captured)
    );
    assert_eq!(raw(&model, symbol)["pin_anchors"][0], poses[0]);
    let material = crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
        &model,
        Some(&symbol.to_string()),
    )
    .unwrap();
    assert!(
        material
            .iter()
            .any(|p| p.library_pin == Some(pin) && p.name == "Supply")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_append_intent_and_wrong_class_membership_refuse_without_writes() {
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-add-refusal");
    let (unit, symbol, _, pin) = make(&mut model, &root, "pool");
    let original = raw(&model, symbol);
    let before = others(&model, symbol);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let mut negative = anchor(pin, 0);
    negative["style"]["length_nm"] = json!(-1);
    let mut invalid_pose = anchor(pin, 0);
    invalid_pose["position"]["x"] = json!("bad");
    let mut invalid_direction = anchor(pin, 0);
    invalid_direction["style"]["orientation"] = json!("Diagonal");
    for rows in [
        json!([negative]),
        json!([invalid_pose]),
        json!([invalid_direction]),
        json!([anchor(pin, 0), anchor(pin, 1)]),
        json!({"not":"array"}),
    ] {
        assert!(
            edit(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"anchor_additions":rows})
            )
            .is_err()
        );
        assert_eq!(raw(&model, symbol), original);
        assert_eq!(others(&model, symbol), before);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    assert!(
        edit(
            &model,
            &root,
            "units",
            unit,
            json!({"anchor_additions":[anchor(pin,0)]})
        )
        .is_err()
    );
    for (domain, kind) in [("pool", "units"), ("schematic", "pins")] {
        let mut fault = model.clone();
        fault.objects.get_mut(&pin).unwrap().domain = domain.into();
        fault.objects.get_mut(&pin).unwrap().kind = kind.into();
        assert!(
            edit(
                &fault,
                &root,
                "symbols",
                symbol,
                json!({"anchor_additions":[anchor(pin,0)]})
            )
            .is_err()
        );
    }
    assert_eq!(raw(&model, symbol), original);
    assert_eq!(others(&model, symbol), before);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn referenced_unit_schema_is_qualified_without_mutating_canonical_sources() {
    use sha2::{Digest, Sha256};
    let (root, mut model, _, _) =
        crate::api::native_write::schematic_assignment::tests::fixture("anchor-add-schema");
    let (unit, symbol, _, pin) = make(&mut model, &root, "pool");
    let original = raw(&model, unit);
    let before = others(&model, symbol);
    let original_symbol = raw(&model, symbol);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    for schema in [Some(json!(2)), None] {
        let mut altered = original.clone();
        if let Some(v) = schema {
            altered["schema_version"] = v;
        } else {
            altered.as_object_mut().unwrap().remove("schema_version");
        }
        // Clone-only corrupt reference: existing recovery reader reconstructs this
        // exact captured Unit from its journal, with no live source rewrite.
        let mut fault = model.clone();
        let mut changed = false;
        for tx in &mut fault.journal {
            for op in &mut tx.operations {
                if let crate::substrate::Operation::CreatePoolLibraryObject {
                    object_id,
                    object,
                    ..
                } = op
                {
                    if *object_id == unit {
                        *object = altered.clone();
                        changed = true;
                    }
                }
            }
        }
        assert!(changed);
        let shard = fault
            .source_shards
            .iter_mut()
            .find(|s| s.shard_id == model.objects[&unit].source_shard_id)
            .unwrap();
        let mut encoded = serde_json::to_vec(&altered).unwrap();
        encoded.push(b'\n');
        shard.content_hash = format!("{:x}", Sha256::digest(encoded));
        shard.path = root.join("missing-reference-fault.json");
        let path = shard.relative_path.clone();
        assert_eq!(
            fault
                .materialized_source_shard_value_by_relative_path(&path)
                .unwrap(),
            altered
        );
        let err = edit(
            &fault,
            &root,
            "symbols",
            symbol,
            json!({"anchor_additions":[anchor(pin,0)]}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("supported schema version"),
            "{err}"
        );
        assert_eq!(raw(&model, unit), original);
        assert_eq!(raw(&model, symbol), original_symbol);
        assert_eq!(others(&model, symbol), before);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    std::fs::remove_dir_all(root).unwrap();
}
