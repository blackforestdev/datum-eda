use super::*;
use crate::{
    api::native_write::{
        commit_prepared,
        electrical::binding_fixture::{BindingFixture, fixture},
        registry::find_native_write_verb,
    },
    substrate::{CommitSource, DesignModel, ProjectResolver},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "edit custom symbol metadata")
}

fn raw(model: &DesignModel, symbol: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&symbol].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["symbols"][symbol.to_string()]
    .clone()
}

fn setup(name: &str) -> (BindingFixture, Uuid, Uuid) {
    let mut f = fixture(name);
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    let mut symbol = raw(&f.model, f.symbol.uuid);
    symbol["extension"] = json!({"future":[3,"retained",{"flag":true}]});
    symbol["pins"][0]["pin_extension"] = json!([1, 2]);
    symbol["fields"] = json!([
        {"uuid":first,"key":"Reference","value":"first","visible":true,
         "position":{"x":123,"y":-456},"field_extension":{"alignment":"custom"}},
        {"uuid":second,"key":"Reference","value":"second","visible":false,
         "position":null,"field_extension":["opaque"]}
    ]);
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            symbol,
        })
        .primary_object(f.symbol.uuid)
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    (f, first, second)
}

fn build(f: &BindingFixture, patch: Value) -> Result<PreparedWrite, EngineError> {
    let mut params = json!({"symbol":f.symbol.uuid,"instance_path":[],"field":"custom"});
    params
        .as_object_mut()
        .unwrap()
        .extend(patch.as_object().unwrap().clone());
    (find_native_write_verb("datum.schematic.set_symbol_field")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        params,
    )
}

fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, result);
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, &mut result);
    result
}

fn roundtrip(f: &mut BindingFixture, prepared: PreparedWrite, expected: &Value) {
    let before = raw(&f.model, f.symbol.uuid);
    let electrical = f.model.electrical_identities.clone();
    let components = f.model.component_instances.clone();
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
            .count(),
        1
    );
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f.model, f.symbol.uuid), *expected);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f.model, f.symbol.uuid), before);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f.model, f.symbol.uuid), *expected);
    assert_eq!(f.model.electrical_identities, electrical);
    assert_eq!(f.model.component_instances, components);
}

#[test]
fn native_custom_fields_preserve_duplicate_names_extensions_and_history() {
    let (mut f, first, second) = setup("native_custom_fields_history");
    let mut expected = raw(&f.model, f.symbol.uuid);
    let before = bytes(&f.root);
    let prepared = build(&f, json!({"field_id":first,"value":"edited"})).unwrap();
    assert_eq!(bytes(&f.root), before, "prepare/cancel does not persist");
    expected["fields"][0]["value"] = json!("edited");
    roundtrip(&mut f, prepared, &expected);

    let prepared = build(&f, json!({"field_id":second,"key":"Value","value":""})).unwrap();
    expected["fields"][1]["key"] = json!("Value");
    expected["fields"][1]["value"] = json!("");
    roundtrip(&mut f, prepared, &expected);

    let prepared = build(&f, json!({"key":"Reference","value":"new"})).unwrap();
    let created = prepared
        .batch
        .operations
        .iter()
        .find_map(|op| match op {
            Operation::SetSchematicSymbol { symbol, .. } => Some(symbol["fields"][2].clone()),
            _ => None,
        })
        .unwrap();
    let id: Uuid = serde_json::from_value(created["uuid"].clone()).unwrap();
    assert_ne!(id, first);
    assert_ne!(id, second);
    assert_eq!(
        created,
        json!({"uuid":id,"key":"Reference","value":"new",
        "visible":false,"position":null})
    );
    expected["fields"].as_array_mut().unwrap().push(created);
    roundtrip(&mut f, prepared, &expected);

    let prepared = build(&f, json!({"field_id":first,"clear":true})).unwrap();
    expected["fields"].as_array_mut().unwrap().remove(0);
    roundtrip(&mut f, prepared, &expected);
    assert_eq!(expected["fields"][0]["uuid"], json!(second));
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_custom_fields_refuse_invalid_targets_and_combinations_without_writes() {
    let (f, first, _) = setup("native_custom_fields_invalid");
    let before = bytes(&f.root);
    for patch in [
        json!({"field":"reference","key":"X","value":"Y"}),
        json!({"symbol":Uuid::new_v4(),"key":"X","value":"Y"}),
        json!({"instance_path":[Uuid::new_v4()],"key":"X","value":"Y"}),
        json!({"instance_path":null,"key":"X","value":"Y"}),
        json!({"field_id":Uuid::new_v4(),"value":"Y"}),
        json!({"field_id":Uuid::new_v4(),"clear":true}),
        json!({"field_id":first}),
        json!({"field_id":first,"value":"first"}),
        json!({"key":"X"}),
        json!({"value":"Y"}),
        json!({"clear":true}),
        json!({"field_id":first,"clear":true,"value":"Y"}),
        json!({"field_id":first,"clear":true,"key":"X"}),
        json!({"key":"","value":"Y"}),
        json!({"key":"   ","value":"Y"}),
        json!({"key":"X\nY","value":"Y"}),
        json!({"key":"X","value":"Y\rZ"}),
        json!({"key":"X","value":"Y","visible":true}),
    ] {
        assert!(build(&f, patch.clone()).is_err(), "accepted {patch}");
        assert_eq!(
            bytes(&f.root),
            before,
            "refused request wrote source/journal"
        );
    }
    // No trimming, case-folding or reserved-name handling is permitted.
    assert!(build(&f, json!({"key":" Value ","value":""})).is_ok());
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_custom_fields_refuse_duplicate_field_uuid_and_source_owner_mismatch() {
    let (mut f, first, _) = setup("native_custom_fields_ownership");
    let mut symbol = raw(&f.model, f.symbol.uuid);
    symbol["fields"][1]["uuid"] = json!(first);
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            symbol,
        })
        .primary_object(f.symbol.uuid)
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = bytes(&f.root);
    assert!(build(&f, json!({"field_id":first,"value":"refused"})).is_err());
    assert!(build(&f, json!({"key":"new","value":"refused"})).is_err());
    assert_eq!(bytes(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();

    let (mut f, first, _) = setup("native_custom_fields_source_owner");
    let before = bytes(&f.root);
    f.model
        .objects
        .get_mut(&f.symbol.uuid)
        .unwrap()
        .source_shard_id = Uuid::new_v4();
    assert!(build(&f, json!({"field_id":first,"value":"refused"})).is_err());
    assert_eq!(bytes(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_custom_fields_stale_prepared_edit_cannot_overwrite_newer_metadata() {
    let (mut f, first, second) = setup("native_custom_fields_stale");
    let stale = build(&f, json!({"field_id":first,"value":"stale"})).unwrap();
    let newer = build(&f, json!({"field_id":second,"value":"newer"})).unwrap();
    commit_prepared(&mut f.model, &f.root, newer).unwrap();
    let before = bytes(&f.root);
    assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
    assert_eq!(bytes(&f.root), before);
    let reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        raw(&reopened, f.symbol.uuid)["fields"][1]["value"],
        json!("newer")
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[path = "schematic_field_presentation_tests.rs"]
mod presentation;
