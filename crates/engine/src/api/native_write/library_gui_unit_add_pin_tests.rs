//! Saved additions preserve raw logical pins and canonical nested history.
use super::*;
use crate::api::native_write::{commit_prepared, registry::find_native_write_verb};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Add logical pin")
}
fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(model, &format!("pool/{kind}/{id}.json"), id).unwrap()
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
#[test]
fn fresh_saved_pin_preserves_alias_raw_and_old_counter_through_history_cold() {
    let (root, mut model, _, part) =
        crate::api::native_write::schematic_assignment::tests::fixture("saved-pin-add-raw");
    let entity: Uuid =
        serde_json::from_value(raw(&model, "parts", part)["entity"].clone()).unwrap();
    let e = raw(&model, "entities", entity);
    let unit: Uuid = serde_json::from_value(
        e["gates"].as_object().unwrap().values().next().unwrap()["unit"].clone(),
    )
    .unwrap();
    let mut original = raw(&model, "units", unit);
    let key = original["pins"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let old: Uuid = key.parse().unwrap();
    let new = Uuid::new_v4();
    let alias = old.simple().to_string().to_uppercase();
    let row = original["pins"]
        .as_object_mut()
        .unwrap()
        .remove(&key)
        .unwrap();
    original["pins"][&alias] = row;
    original["pins"][&alias]["custom"] = json!([3, 1]);
    original["custom"] = json!(true);
    let setup = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "units", unit),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, setup).unwrap();
    let counter = model.objects[&old].object_revision;
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let fields =
        json!({"pin_additions":{new.to_string():{"name":"Enable","electrical_type":"Input"}}});
    let prepared = edit(&model, &root, "units", unit, fields.clone()).unwrap();
    assert_eq!(raw(&model, "units", unit), original);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert!(prepared.batch.operations.iter().any(|o|matches!(o,crate::substrate::Operation::GuardObjectRevision{object_id,..} if *object_id==unit)));
    let mut changed = model.clone();
    changed.objects.get_mut(&unit).unwrap().object_revision.0 += 1;
    assert!(changed.commit(prepared.batch.clone()).is_err());
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = original.clone();
    expected["pins"][new.to_string()] = json!({"uuid":new,"name":"Enable","electrical_type":"Input","direction":"Input","swap_group":0,"alternates":[]});
    assert_eq!(raw(&model, "units", unit), expected);
    assert_eq!(model.objects[&old].object_revision, counter);
    assert_eq!(
        model.objects[&new].source_shard_id,
        model.objects[&unit].source_shard_id
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "units", unit), original);
    assert!(!model.objects.contains_key(&new));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "units", unit), expected);
    assert_eq!(model.objects[&old].object_revision, counter);
    assert!(model.objects.contains_key(&new));
    let bytes = std::fs::read(&journal).unwrap();
    for rows in [
        json!({}),
        json!({old.to_string():{"name":"Collision","electrical_type":"Input"}}),
        json!({unit.to_string():{"name":"Owned","electrical_type":"Input"}}),
        json!({Uuid::nil().to_string():{"name":"Nil","electrical_type":"Input"}}),
        json!({Uuid::new_v4().to_string():{"name":"","electrical_type":"Input"}}),
        json!({Uuid::new_v4().to_string():{"name":"Invalid","electrical_type":"Unknown"}}),
        json!({Uuid::new_v4().to_string():{"name":"Forged","electrical_type":"Input","uuid":new}}),
    ] {
        assert!(edit(&model, &root, "units", unit, json!({"pin_additions":rows})).is_err());
        assert_eq!(raw(&model, "units", unit), expected);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    let duplicate = Uuid::new_v4();
    let alias_rows = json!({duplicate.to_string():{"name":"Duplicate","electrical_type":"Input"},duplicate.simple().to_string():{"name":"Alias","electrical_type":"Input"}});
    assert!(
        edit(
            &model,
            &root,
            "units",
            unit,
            json!({"pin_additions":alias_rows})
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn addition_composes_after_old_metadata_and_is_available_to_new_symbol_by_explicit_pin() {
    let (root, mut model, _, part) =
        crate::api::native_write::schematic_assignment::tests::fixture("saved-pin-add-symbol");
    let entity: Uuid =
        serde_json::from_value(raw(&model, "parts", part)["entity"].clone()).unwrap();
    let e = raw(&model, "entities", entity);
    let unit: Uuid = serde_json::from_value(
        e["gates"].as_object().unwrap().values().next().unwrap()["unit"].clone(),
    )
    .unwrap();
    let original = raw(&model, "units", unit);
    let mut pins = original["pins"].clone();
    let key = pins.as_object().unwrap().keys().next().unwrap().clone();
    pins[&key]["name"] = json!("Explicit rename");
    let new = Uuid::new_v4();
    let p=edit(&model,&root,"units",unit,json!({"pins":pins,"pin_additions":{new.to_string():{"name":"Enable","electrical_type":"Input"}}})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let current = raw(&model, "units", unit);
    assert_eq!(current["pins"][&key]["name"], json!("Explicit rename"));
    assert!(
        current["pins"]
            .as_object()
            .unwrap()
            .contains_key(&new.to_string())
    );
    let symbol = Uuid::new_v4();
    let request = json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"name":"New enabled symbol","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":new,"position":{"x":0,"y":0},"style":{"orientation":"Right","length_nm":1,"decoration":"none"}}]}});
    let p = (find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        request,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let placed = crate::api::native_write::schematic_library::materialize_pool_symbol_pins(
        &model,
        Some(&symbol.to_string()),
    )
    .unwrap();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].library_pin, Some(new));
    assert_eq!(placed[0].name, "Enable");
    std::fs::remove_dir_all(root).unwrap();
}
