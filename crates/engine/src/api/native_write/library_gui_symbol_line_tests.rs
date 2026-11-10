//! Ordered mixed raw graphics, atomic sparse intent and canonical nested history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol Lines")
}
fn line(x: i64) -> Value {
    json!({"Line":{"from":{"x":0,"y":0,"metadata":[3,1,3]},"to":{"x":x,"y":2000000,"vendor":null},"width":100000,"unknown":{"retain":true}}})
}
fn fixture(name: &str, drawings: Value) -> (PathBuf, DesignModel, Uuid, Uuid) {
    let root = crate::api::native_write::test_support::temp_project_root(name);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: name.into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Lines");
    graphic["drawings"] = drawings;
    graphic["vendor"] = json!([3, 1, 3]);
    let p = library::build_pool_library_write(
        &model,
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
    commit_prepared(&mut model, &root, p).unwrap();
    (root, model, unit, symbol)
}
fn raw(m: &DesignModel, symbol: Uuid) -> Value {
    library::pool_library_object_payload(m, &format!("pool/symbols/{symbol}.json"), symbol).unwrap()
}
fn others(m: &DesignModel, symbol: Uuid) -> BTreeMap<String, Value> {
    m.source_shards
        .iter()
        .filter(|s| s.shard_id != m.objects[&symbol].source_shard_id)
        .map(|s| {
            (
                s.relative_path.clone(),
                m.materialized_source_shard_value_by_relative_path(&s.relative_path)
                    .unwrap(),
            )
        })
        .collect()
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
fn original_later_duplicate_row_composes_rectangle_remove_add_history_and_cold_losslessly() {
    let rect = json!({"Rect":{"min":{"x":-2000000,"y":-1000000,"vendor":true},"max":{"x":2000000,"y":1000000},"width":100000}});
    let (root, mut m, _, symbol) = fixture(
        "line_raw_history",
        json!([{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},line(1000000),rect,line(1000000)]),
    );
    let before = raw(&m, symbol);
    let untouched = others(&m, symbol);
    let revisions: BTreeMap<_, _> = m
        .objects
        .iter()
        .filter(|(id, _)| **id != symbol)
        .map(|(id, o)| (*id, o.object_revision))
        .collect();
    let mut changed = line(9000000);
    changed["Line"]["width"] = json!(200000);
    let mut posed = before["drawings"].clone();
    posed[2]["Rect"]["min"]["x"] = json!(-3000000);
    let added = serde_json::to_value(
        serde_json::from_value::<crate::pool::Primitive>(line(3000000)).unwrap(),
    )
    .unwrap();
    let fields = json!({"name":"Corrected","drawings":posed,"line_edits":[{"index":3,"previous":before["drawings"][3],"line":changed}],"line_removals":[{"index":1,"previous":before["drawings"][1]}],"line_additions":[added]});
    let mut expected = before.clone();
    expected["name"] = json!("Corrected");
    expected["drawings"] = json!([before["drawings"][0], posed[2], changed, added]);
    let p = edit(&m, &root, "symbols", symbol, fields).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    assert_eq!(raw(&m, symbol), expected);
    m.commit_journal_undo(&root, provenance().into()).unwrap();
    assert_eq!(raw(&m, symbol), before);
    m.commit_journal_redo(&root, provenance().into()).unwrap();
    assert_eq!(raw(&m, symbol), expected);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, symbol), expected);
    assert_eq!(others(&cold, symbol), untouched);
    assert!(
        revisions
            .iter()
            .all(|(id, revision)| cold.objects[id].object_revision == *revision)
    );
}
#[test]
fn conflicting_non_line_missing_preimage_and_new_numeric_failures_are_atomic() {
    let (root, m, unit, symbol) = fixture(
        "line_refusals",
        json!([{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},line(1000000)]),
    );
    let before = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let old = &before["drawings"][1];
    let change = json!({"index":1,"previous":old,"line":line(3000000)});
    let mut lost = line(3000000);
    lost["Line"]["from"]
        .as_object_mut()
        .unwrap()
        .remove("metadata");
    let mut zero = line(1000000);
    zero["Line"]["width"] = json!(0);
    let mut coincident = line(1000000);
    coincident["Line"]["to"] = coincident["Line"]["from"].clone();
    let cases = vec![
        json!({"line_edits":[change.clone(),change.clone()]}),
        json!({"line_edits":[change],"line_removals":[{"index":1,"previous":old}]}),
        json!({"line_removals":[{"index":0,"previous":before["drawings"][0]}]}),
        json!({"line_removals":[{"index":9,"previous":old}]}),
        json!({"line_removals":[{"index":1,"previous":line(999)}]}),
        json!({"line_edits":[{"index":1,"previous":old,"line":lost}]}),
        json!({"line_edits":[{"index":1,"previous":old,"line":before["drawings"][0]}]}),
        json!({"line_additions":[zero]}),
        json!({"line_additions":[coincident]}),
        json!({"line_additions":[before["drawings"][0]]}),
    ];
    for fields in cases {
        assert!(
            edit(&m, &root, "symbols", symbol, fields.clone()).is_err(),
            "{fields}"
        );
        assert_eq!(raw(&m, symbol), before);
        assert_eq!(
            std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
            journal
        );
    }
    assert!(
        edit(
            &m,
            &root,
            "units",
            unit,
            json!({"line_additions":[line(1000000)]})
        )
        .is_err()
    );
}
#[test]
fn empty_graphics_append_and_existing_schema_numeric_boundary_remain_distinct() {
    let (root, mut m, _, symbol) = fixture("line_empty", json!([]));
    let p = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"line_additions":[line(1000000)]}),
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let before = raw(&m, symbol);
    let mut changed = before["drawings"][0].clone();
    changed["Line"]["to"]["x"] = changed["Line"]["from"]["x"].clone();
    changed["Line"]["to"]["y"] = changed["Line"]["from"]["y"].clone();
    changed["Line"]["width"] = json!(-1);
    let p = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"line_edits":[{"index":0,"previous":before["drawings"][0],"line":changed}]}),
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    assert_eq!(raw(&m, symbol)["drawings"][0], changed);
    let stale = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"line_removals":[{"index":0,"previous":changed}]}),
    )
    .unwrap();
    let p = edit(&m, &root, "symbols", symbol, json!({"name":"Concurrent"})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let current = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    assert!(commit_prepared(&mut m, &root, stale).is_err());
    assert_eq!(raw(&m, symbol), current);
    assert_eq!(
        std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
        journal
    );
}

#[test]
fn new_line_extensions_cannot_mint_persisted_nested_graphic_identities() {
    let (root, mut model, _, symbol) = fixture("line_new_identity", json!([]));
    let (line_id, point_id) = (Uuid::new_v4(), Uuid::new_v4());
    let mut addition = line(1000000);
    addition["Line"]["uuid"] = json!(line_id);
    addition["Line"]["from"]["uuid"] = json!(point_id);
    let expected = serde_json::to_value(
        serde_json::from_value::<crate::pool::Primitive>(addition.clone()).unwrap(),
    )
    .unwrap();
    let before = others(&model, symbol);
    let prepared = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"line_additions":[addition]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(raw(&model, symbol)["drawings"], json!([expected]));
    assert!(!model.objects.contains_key(&line_id) && !model.objects.contains_key(&point_id));
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert!(!cold.objects.contains_key(&line_id) && !cold.objects.contains_key(&point_id));
    assert_eq!(others(&cold, symbol), before);
}
