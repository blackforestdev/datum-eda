//! Saved definition field edits through the canonical Symbol writer.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol fields")
}
fn fixture(name: &str, fields: Value) -> (PathBuf, DesignModel, Uuid, Uuid) {
    let root = crate::api::native_write::test_support::temp_project_root(name);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: name.into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut m = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Field test Symbol");
    graphic["fields"] = fields;
    graphic["default_refdes_prefix"] = Value::Null;
    graphic["vendor"] = json!([3, 1, 3]);
    let p = library::build_pool_library_write(
        &m,
        provenance(),
        Some("pool"),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit),
                object: library::pool_unit_payload(unit, "Logical", "Maker"),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: graphic,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    (root, m, unit, symbol)
}
fn raw(m: &DesignModel, id: Uuid) -> Value {
    library::pool_library_object_payload(m, &format!("pool/symbols/{id}.json"), id).unwrap()
}
fn edit(
    m: &DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build(
        &NativeWriteContext {
            model: m,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: kind.into(),
            object_id: id,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}
#[test]
fn saved_symbol_field_order_metadata_prefix_and_history() {
    let row =
        json!({"key":"MPN","value":" old ","position":{"x":0,"y":1,"vendor":true},"unknown":[3,1]});
    let sibling = json!({"key":"Note","value":"","position":null});
    let (root, mut model, _, symbol) = fixture("symbol_field_history", json!([row, row, sibling]));
    let before = raw(&model, symbol);
    let mut edited = row.clone();
    edited["value"] = json!("  revised Ω  ");
    edited["position"]["x"] = json!(2000000);
    let added = json!({"key":"MPN","value":"","visible":true});
    let request = json!({
        "symbol_field_removals":[{"index":0,"previous":row}],
        "symbol_field_edits":[{"index":1,"previous":row,"field":edited}],
        "symbol_field_additions":[added],
        "default_refdes_prefix":" QΩ "
    });
    let prepared = edit(&model, &root, "symbols", symbol, request.clone()).unwrap();
    let stale = edit(&model, &root, "symbols", symbol, request).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = before.clone();
    expected["fields"] = json!([edited, sibling, added]);
    expected["default_refdes_prefix"] = json!(" QΩ ");
    assert_eq!(raw(&model, symbol), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, symbol), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        raw(&ProjectResolver::new(&root).resolve().unwrap(), symbol),
        expected
    );

    let empty = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"default_refdes_prefix":""}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, empty).unwrap();
    assert_eq!(raw(&model, symbol)["default_refdes_prefix"], "");
    let mut cleared_field = edited.clone();
    cleared_field.as_object_mut().unwrap().remove("position");
    let clear = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"default_refdes_prefix":null,
            "symbol_field_edits":[{"index":0,"previous":edited,"field":cleared_field}]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, clear).unwrap();
    expected
        .as_object_mut()
        .unwrap()
        .remove("default_refdes_prefix");
    expected["fields"][0] = cleared_field;
    assert_eq!(
        raw(&ProjectResolver::new(&root).resolve().unwrap(), symbol),
        expected
    );
}

#[test]
fn saved_symbol_field_stale_overlapping_invalid_and_foreign_intent_refused() {
    let row = json!({"key":"MPN","value":"old","position":{"x":0,"y":1,"vendor":true}});
    let (root, model, unit, symbol) = fixture("symbol_field_refusals", json!([row]));
    let before = raw(&model, symbol);
    let mut changed = row.clone();
    changed["visible"] = json!(true);
    let edit_row = json!({"index":0,"previous":row,"field":changed});
    for request in [
        json!({"symbol_field_edits":[edit_row,edit_row]}),
        json!({"symbol_field_edits":[edit_row],"symbol_field_removals":[{"index":0,"previous":row}]}),
        json!({"symbol_field_removals":[{"index":0,"previous":{}}]}),
        json!({"symbol_field_edits":[{"index":0,"previous":row,"field":{"key":"MPN","value":"new","position":null}}]}),
        json!({"symbol_field_edits":[{"index":0,"previous":row,"field":{"key":"MPN","value":"new","position":{"x":0,"y":1}}}]}),
        json!({"symbol_field_additions":[{"key":"MPN","value":"","uuid":Uuid::new_v4()}]}),
        json!({"symbol_field_additions":[{"key":"MPN","value":1}]}),
        json!({"default_refdes_prefix":17}),
    ] {
        assert!(
            edit(&model, &root, "symbols", symbol, request.clone()).is_err(),
            "{request}"
        );
        assert_eq!(raw(&model, symbol), before);
    }
    assert!(
        edit(
            &model,
            &root,
            "units",
            unit,
            json!({"default_refdes_prefix":"R"})
        )
        .is_err()
    );
}
