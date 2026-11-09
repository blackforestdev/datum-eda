//! Display-name correction preserves raw memberships and consumes guarded links.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, schematic_gate_placement as placement};
use crate::substrate::{DesignModel, ProjectResolver};
use std::{collections::BTreeMap, path::Path};
fn edit(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (super::super::super::registry::find_native_write_verb("datum.library.edit")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"entities","object_id":id,"pool_path":"pool","fields":fields}),
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
fn store(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, object: Value) {
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", kind, id),
            object,
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn retained(model: &DesignModel, entity: Uuid) -> BTreeMap<String, Value> {
    let owner = model.objects[&entity].source_shard_id;
    model
        .source_shards
        .iter()
        .filter(|s| s.shard_id != owner)
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
fn saved_gate_names_sparse_alias_patch_guards_history_and_cold_preserve_raw() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, a, b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut intent = fields(a, unit, symbol);
    intent["gates"][b.to_string()] = json!({"uuid":b,"name":"B","unit":unit,"symbol":symbol});
    let p = prepare(&model, &root, "entities", entity, intent).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut before = raw(&model, "entities", entity);
    let alias = a.simple().to_string().to_uppercase();
    let row = before["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&a.to_string())
        .unwrap();
    before["gates"][&alias] = row;
    before["gates"][&alias]["custom"] = json!([3, 1, 3]);
    before["tags"] = json!(["z", "a", "z"]);
    before["custom"] = json!({"unchanged":true});
    store(&mut model, &root, "entities", entity, before.clone());
    let request = json!({"gate_names":{a.to_string():"  Amplifier  ",b.to_string():"  Amplifier  "},"prefix":"IC"});
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let p = edit(&model, &root, entity, request.clone()).unwrap();
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert_eq!(raw(&model, "entities", entity), before);
    for id in [unit, symbol] {
        assert!(p.batch.operations.iter().any(
            |op| matches!(op,Operation::GuardObjectRevision{object_id,expected_object_revision}
            if *object_id==id && *expected_object_revision==model.objects[&id].object_revision)
        ));
    }
    let mut changed = raw(&model, "units", unit);
    changed["name"] = json!("Concurrent Unit metadata");
    store(&mut model, &root, "units", unit, changed);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut model, &root, p).is_err());
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    let p = edit(&model, &root, entity, request).unwrap();
    let stale = p.clone();
    let unchanged = retained(&model, entity);
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = before.clone();
    expected["gates"][&alias]["name"] = json!("  Amplifier  ");
    expected["gates"][b.to_string()]["name"] = json!("  Amplifier  ");
    expected["prefix"] = json!("IC");
    assert_eq!(raw(&model, "entities", entity), expected);
    assert_eq!(retained(&model, entity), unchanged);
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "entities", entity), before);
    assert_eq!(retained(&model, entity), unchanged);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "entities", entity), expected);
    assert_eq!(retained(&model, entity), unchanged);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_gate_names_invalid_missing_alias_and_topology_intents_refuse_without_writes() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, gate) = (Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(
        &model,
        &root,
        "entities",
        entity,
        fields(gate, unit, symbol),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let before = raw(&model, "entities", entity);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let others = retained(&model, entity);
    for fields in [
        json!({"gate_names":{}}),
        json!({"gate_names":null}),
        json!({"gate_names":[]}),
        json!({"gate_names":{gate.to_string():" "}}),
        json!({"gate_names":{gate.to_string():true}}),
        json!({"gate_names":{Uuid::new_v4().to_string():"Missing"}}),
        json!({"gate_names":{"malformed":"No"}}),
        json!({"gate_names":{gate.to_string():"A",gate.simple().to_string():"B"}}),
        json!({"gate_names":{gate.to_string():"Renamed"},"gates":{}}),
        json!({"gate_names":{gate.to_string():"Renamed"},"gate_units":{gate.to_string():Uuid::new_v4()}}),
    ] {
        assert!(edit(&model, &root, entity, fields).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, "entities", entity), before);
        assert_eq!(retained(&model, entity), others);
    }
    // The general Set writer can retain legacy raw aliases. The narrow saved
    // editor must refuse ambiguous parsed membership, without rewriting them.
    let mut duplicate = before.clone();
    duplicate["gates"][gate.simple().to_string()] = before["gates"][gate.to_string()].clone();
    store(&mut model, &root, "entities", entity, duplicate.clone());
    let bytes = std::fs::read(&journal).unwrap();
    assert!(
        edit(
            &model,
            &root,
            entity,
            json!({"gate_names":{gate.to_string():"Refused"}})
        )
        .is_err()
    );
    assert_eq!(raw(&model, "entities", entity), duplicate);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_gate_names_update_remaining_choice_captions_with_exact_old_component_sources() {
    let (root, mut model, first, _) = placement::tests::setup("saved-gate-names-captions");
    let options = placement::options(&model, first, &[]).unwrap();
    let entity = options.entity;
    assert!(!options.choices.is_empty());
    let before = raw(&model, "entities", entity);
    let others = retained(&model, entity);
    let names: serde_json::Map<_, _> = options
        .choices
        .iter()
        .enumerate()
        .map(|(i, g)| (g.gate.to_string(), json!(format!("Renamed gate {i}"))))
        .collect();
    let p = edit(&model, &root, entity, json!({"gate_names":names})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for choice in placement::options(&model, first, &[]).unwrap().choices {
        assert_eq!(json!(choice.gate_name), names[&choice.gate.to_string()]);
    }
    assert_eq!(retained(&model, entity), others);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "entities", entity), before);
    assert_eq!(retained(&model, entity), others);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(retained(&model, entity), others);
    for choice in placement::options(&model, first, &[]).unwrap().choices {
        assert_eq!(json!(choice.gate_name), names[&choice.gate.to_string()]);
    }
    std::fs::remove_dir_all(root).unwrap();
}
