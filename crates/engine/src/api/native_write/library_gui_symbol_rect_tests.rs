//! Rect rows retain raw identity and metadata through the canonical journal.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol rectangles")
}
fn rect() -> Value {
    json!({"Rect":{"min":{"x":-1000000,"y":-1000000,"vendor":[3,1,3]},"max":{"x":1000000,"y":1000000},"width":100000,"unknown":true}})
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
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Rectangles");
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
fn duplicate_rect_lifecycle_preserves_mixed_rows_metadata_and_canonical_history() {
    let line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1,"extra":[3,1]}});
    let (root, mut m, _, symbol) = fixture("rect_lifecycle", json!([rect(), line, rect()]));
    let before = raw(&m, symbol);
    let sources = m
        .source_shards
        .iter()
        .filter(|s| s.shard_id != m.objects[&symbol].source_shard_id)
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let mut changed = rect();
    changed["Rect"]["max"]["y"] = json!(4000000);
    let added_line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1000000,"y":0},"width":100000}});
    let added_rect =
        serde_json::to_value(serde_json::from_value::<crate::pool::Primitive>(rect()).unwrap())
            .unwrap();
    let p=edit(&m,&root,"symbols",symbol,json!({"rect_edits":[{"index":2,"previous":before["drawings"][2],"rect":changed}],"rect_removals":[{"index":0,"previous":before["drawings"][0]}],"graphic_additions":[added_rect,added_line]})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let mut expected = before.clone();
    expected["drawings"] = json!([before["drawings"][1], changed, added_rect, added_line]);
    assert_eq!(raw(&m, symbol), expected);
    m.commit_journal_undo(&root, provenance().into()).unwrap();
    assert_eq!(raw(&m, symbol), before);
    m.commit_journal_redo(&root, provenance().into()).unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, symbol), expected);
    for (path, bytes) in sources {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
}
#[test]
fn rect_refusals_preserve_source_and_journal_and_legacy_conflicts_never_overwrite() {
    let (root, m, unit, symbol) = fixture("rect_refusals", json!([rect()]));
    let before = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let mut changed = rect();
    changed["Rect"]["max"]["y"] = json!(2000000);
    let mut legacy = before["drawings"].clone();
    legacy[0]["Rect"]["min"]["x"] = json!(-2000000);
    let change = json!({"index":0,"previous":before["drawings"][0],"rect":changed});
    let mut unknown = changed.clone();
    unknown["Rect"]["min"]
        .as_object_mut()
        .unwrap()
        .remove("vendor");
    let mut invalid = rect();
    invalid["Rect"]["width"] = json!(0);
    for fields in [
        json!({"rect_edits":[change.clone(),change.clone()]}),
        json!({"rect_edits":[change.clone()],"rect_removals":[{"index":0,"previous":rect()}]}),
        json!({"drawings":legacy,"rect_edits":[change]}),
        json!({"rect_edits":[{"index":0,"previous":rect(),"rect":unknown}]}),
        json!({"rect_edits":[{"index":0,"previous":rect(),"rect":invalid}]}),
        json!({"rect_removals":[{"index":99,"previous":rect()}]}),
        json!({"rect_removals":[{"index":0,"previous":changed}]}),
        json!({"line_removals":[{"index":0,"previous":rect()}]}),
        json!({"graphic_additions":[invalid]}),
        json!({"graphic_additions":[rect()],"line_additions":[]}),
        json!({"line_additions":[rect()]}),
        json!({"graphic_additions":[{"Polygon":{"polygon":{"vertices":[{"x":0,"y":0},{"x":1,"y":0},{"x":0,"y":1}],"closed":true},"width":1}}]}),
    ] {
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
            json!({"graphic_additions":[rect()]})
        )
        .is_err()
    );
    let mut identical = before["drawings"].clone();
    identical[0] = changed.clone();
    assert!(edit(&m,&root,"symbols",symbol,json!({"drawings":identical,"rect_edits":[{"index":0,"previous":rect(),"rect":changed}]})).is_ok());
}
#[test]
fn new_rect_extensions_cannot_create_nested_object_identities_and_stale_intent_refuses() {
    let (root, mut m, _, symbol) = fixture("rect_identity", json!([]));
    let (graphic_id, point_id) = (Uuid::new_v4(), Uuid::new_v4());
    let mut added = rect();
    added["Rect"]["uuid"] = json!(graphic_id);
    added["Rect"]["min"]["uuid"] = json!(point_id);
    let p = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[added]}),
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let expected =
        serde_json::to_value(serde_json::from_value::<crate::pool::Primitive>(rect()).unwrap())
            .unwrap();
    assert_eq!(raw(&m, symbol)["drawings"], json!([expected]));
    let stale = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"rect_removals":[{"index":0,"previous":expected}]}),
    )
    .unwrap();
    let p = edit(&m, &root, "symbols", symbol, json!({"name":"Concurrent"})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let before = raw(&m, symbol);
    assert!(commit_prepared(&mut m, &root, stale).is_err());
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, symbol), before);
    assert!(!cold.objects.contains_key(&graphic_id));
    assert!(!cold.objects.contains_key(&point_id));
}
#[test]
fn untouched_legacy_rect_admission_is_not_a_new_geometry_gate() {
    let mut legacy = rect();
    legacy["Rect"]["width"] = json!(-1);
    let (root, m, _, symbol) = fixture("rect_legacy", json!([legacy]));
    assert!(edit(&m, &root, "symbols", symbol, json!({"name":"Metadata"})).is_ok());
    assert!(
        edit(
            &m,
            &root,
            "symbols",
            symbol,
            json!({"rect_edits":[{"index":0,"previous":legacy,"rect":legacy}]})
        )
        .is_ok()
    );
}
