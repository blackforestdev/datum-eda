//! Text rows retain raw identity and metadata through the canonical journal.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol text")
}
fn text(literal: &str) -> Value {
    json!({"Text":{"text":literal,"position":{"x":0,"y":1000000,"vendor":[3,1,3]},"rotation":0,"unknown":true}})
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
    let mut m = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Text graphics");
    graphic["drawings"] = drawings;
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
fn symbol_text_duplicate_rows_compose_geometry_removal_mixed_append_and_history_losslessly() {
    let line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1,"unknown":true}});
    let rect = json!({"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}});
    let (root, mut model, _, symbol) = fixture(
        "text_history",
        json!([text(" same "), line, rect, text(" same ")]),
    );
    let before = raw(&model, symbol);
    let others: Vec<_> = model
        .source_shards
        .iter()
        .filter(|s| s.shard_id != model.objects[&symbol].source_shard_id)
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect();
    let mut changed = text(" Revised ");
    changed["Text"]["position"]["x"] = json!(2000000);
    changed["Text"]["rotation"] = json!(i32::MIN);
    let mut posed = before["drawings"].clone();
    posed[2]["Rect"]["min"]["x"] = json!(-2000000);
    let added_text = serde_json::to_value(
        serde_json::from_value::<crate::pool::Primitive>(text("NEW")).unwrap(),
    )
    .unwrap();
    let added_line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}});
    let prepared=edit(&model,&root,"symbols",symbol,json!({"drawings":posed,"text_edits":[{"index":3,"previous":before["drawings"][3],"text":changed}],"text_removals":[{"index":0,"previous":before["drawings"][0]}],"graphic_additions":[text("  NEW  "),added_line]})).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut expected = before.clone();
    expected["drawings"] = json!([
        before["drawings"][1],
        posed[2],
        changed,
        added_text,
        added_line
    ]);
    assert_eq!(raw(&model, symbol), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, symbol), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, symbol), expected);
    for (path, bytes) in others {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
}
#[test]
fn symbol_text_conflicting_missing_nontext_and_invalid_new_intent_are_atomic() {
    let rect = json!({"Rect":{"min":{"x":-1,"y":-1},"max":{"x":1,"y":1},"width":1}});
    let (root, model, unit, symbol) = fixture("text_refusals", json!([rect, text("old")]));
    let before = raw(&model, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let change = json!({"index":1,"previous":text("old"),"text":text("new")});
    let mut unknown = text("new");
    unknown["Text"]["position"]
        .as_object_mut()
        .unwrap()
        .remove("vendor");
    let mut overflow = text("new");
    overflow["Text"]["rotation"] = json!(i64::from(i32::MAX) + 1);
    for fields in [
        json!({"text_edits":[change.clone(),change.clone()]}),
        json!({"text_edits":[change],"text_removals":[{"index":1,"previous":text("old")}]}),
        json!({"text_edits":[{"index":1,"previous":text("old"),"text":unknown}]}),
        json!({"text_edits":[{"index":1,"previous":text("old"),"text":overflow}]}),
        json!({"text_edits":[{"index":1,"previous":text("old"),"text":before["drawings"][0]}]}),
        json!({"text_edits":[{"index":1,"text":text("new")}]}),
        json!({"text_removals":[{"index":0,"previous":rect}]}),
        json!({"text_removals":[{"index":99,"previous":text("old")}]}),
        json!({"text_removals":[{"index":1,"previous":text("stale")}]}),
        json!({"graphic_additions":[text(" \t\n ")]}),
        json!({"line_additions":[text("NEW")]}),
    ] {
        assert!(
            edit(&model, &root, "symbols", symbol, fields.clone()).is_err(),
            "{fields}"
        );
        assert_eq!(raw(&model, symbol), before);
        assert_eq!(
            std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
            journal
        );
    }
    assert!(edit(&model, &root, "units", unit, json!({"text_edits":[]})).is_err());
}
#[test]
fn symbol_text_old_empty_literal_unicode_spelling_and_i32_angles_keep_schema_admission() {
    let (root, mut model, _, symbol) = fixture("text_schema", json!([text("")]));
    let mut previous = raw(&model, symbol)["drawings"][0].clone();
    for (literal, angle) in [("  Ω raw  ", i32::MIN), ("", i32::MAX), ("  ", -721)] {
        let mut changed = previous.clone();
        changed["Text"]["text"] = json!(literal);
        changed["Text"]["rotation"] = json!(angle);
        let prepared = edit(
            &model,
            &root,
            "symbols",
            symbol,
            json!({"text_edits":[{"index":0,"previous":previous,"text":changed}]}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        previous = changed;
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&cold, symbol)["drawings"][0], previous);
    }
}
#[test]
fn symbol_text_typed_new_rows_cannot_mint_ids_and_prepared_stale_removal_refuses() {
    let (root, mut model, _, symbol) = fixture("text_identity", json!([]));
    let (text_id, point_id) = (Uuid::new_v4(), Uuid::new_v4());
    let mut addition = text("  NEW  ");
    addition["Text"]["uuid"] = json!(text_id);
    addition["Text"]["position"]["uuid"] = json!(point_id);
    let prepared = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[addition]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let expected = serde_json::to_value(
        serde_json::from_value::<crate::pool::Primitive>(text("NEW")).unwrap(),
    )
    .unwrap();
    assert_eq!(raw(&model, symbol)["drawings"], json!([expected]));
    let stale = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"text_removals":[{"index":0,"previous":expected}]}),
    )
    .unwrap();
    let prepared = edit(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"name":"Concurrent"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let before = raw(&model, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, symbol), before);
    assert!(!cold.objects.contains_key(&text_id) && !cold.objects.contains_key(&point_id));
    assert_eq!(
        std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
        journal
    );
}
