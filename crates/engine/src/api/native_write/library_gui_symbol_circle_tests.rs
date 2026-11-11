//! Saved Circle rows use canonical guarded writes and preserve raw siblings.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol circles")
}
fn circle() -> Value {
    json!({"Circle":{"center":{"x":0,"y":0,"vendor":[3,1,3]},"radius":1000000,"width":100000,"unknown":true}})
}
pub(super) fn fixture(name: &str, drawings: Value) -> (PathBuf, DesignModel, Uuid, Uuid) {
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
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Circles");
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
pub(super) fn raw(m: &DesignModel, id: Uuid) -> Value {
    library::pool_library_object_payload(m, &format!("pool/symbols/{id}.json"), id).unwrap()
}
pub(super) fn edit(
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
fn circle_lifecycle_preserves_duplicate_identity_raw_center_and_canonical_history() {
    let line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1,"vendor":true}});
    let (root, mut m, _, symbol) = fixture("circle_history", json!([circle(), line, circle()]));
    let before = raw(&m, symbol);
    let others: Vec<_> = m
        .source_shards
        .iter()
        .filter(|s| s.shard_id != m.objects[&symbol].source_shard_id)
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect();
    let mut changed = circle();
    changed["Circle"]["center"]["x"] = json!(-2000000);
    changed["Circle"]["radius"] = json!(2500000);
    let added = json!({"Circle":{"center":{"x":1000000,"y":0},"radius":500000,"width":120000}});
    let text = json!({"Text":{"text":"READY","position":{"x":0,"y":3000000},"rotation":0}});
    let p = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({
        "circle_edits":[{"index":2,"previous":circle(),"circle":changed}],
        "circle_removals":[{"index":0,"previous":circle()}],"graphic_additions":[added,text]}),
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let mut expected = before.clone();
    expected["drawings"] = json!([line, changed, added, text]);
    assert_eq!(raw(&m, symbol), expected);
    m.commit_journal_undo(&root, provenance().into()).unwrap();
    assert_eq!(raw(&m, symbol), before);
    m.commit_journal_redo(&root, provenance().into()).unwrap();
    assert_eq!(
        raw(&ProjectResolver::new(&root).resolve().unwrap(), symbol),
        expected
    );
    for (path, bytes) in others {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
}

#[test]
fn circle_refusals_preserve_source_journal_and_captured_preimages() {
    let (root, m, _, symbol) = fixture("circle_refusal", json!([circle()]));
    let before = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let mut changed = circle();
    changed["Circle"]["radius"] = json!(2000000);
    let mut metadata = changed.clone();
    metadata["Circle"]["center"]
        .as_object_mut()
        .unwrap()
        .remove("vendor");
    let change = json!({"index":0,"previous":circle(),"circle":changed});
    let mut requests = vec![
        json!({"circle_edits":[change.clone(),change.clone()]}),
        json!({"circle_edits":[change],"circle_removals":[{"index":0,"previous":circle()}]}),
        json!({"circle_edits":[{"index":0,"previous":circle(),"circle":metadata}]}),
        json!({"circle_removals":[{"index":1,"previous":circle()}]}),
        json!({"circle_removals":[{"index":0,"previous":changed}]}),
        json!({"line_additions":[circle()]}),
    ];
    for (key, value) in [
        ("radius", json!(0)),
        ("radius", json!(-1)),
        ("width", json!(0)),
        ("width", json!(-1)),
    ] {
        let mut invalid = circle();
        invalid["Circle"][key] = value;
        requests.push(json!({"graphic_additions":[invalid]}));
        requests.push(json!({"circle_edits":[{"index":0,"previous":circle(),"circle":invalid}]}));
    }
    for axis in ["x", "y"] {
        for value in [i64::MAX, i64::MIN] {
            let mut invalid = circle();
            invalid["Circle"]["center"][axis] = json!(value);
            requests.push(json!({"graphic_additions":[invalid]}));
            requests
                .push(json!({"circle_edits":[{"index":0,"previous":circle(),"circle":invalid}]}));
        }
    }
    for fields in requests {
        assert!(
            edit(&m, &root, "symbols", symbol, fields.clone()).is_err(),
            "{fields}"
        );
    }
    assert_eq!(
        raw(&ProjectResolver::new(&root).resolve().unwrap(), symbol),
        before
    );
    assert_eq!(
        std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
        journal
    );
}

#[test]
fn new_circle_uses_typed_fields_and_stale_preparation_cannot_overwrite() {
    let (root, mut m, _, symbol) = fixture("circle_typed", json!([]));
    let (row_id, point_id) = (Uuid::new_v4(), Uuid::new_v4());
    let mut added = circle();
    added["Circle"]["uuid"] = json!(row_id);
    added["Circle"]["center"]["uuid"] = json!(point_id);
    let p = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"graphic_additions":[added]}),
    )
    .unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let typed =
        serde_json::to_value(serde_json::from_value::<crate::pool::Primitive>(circle()).unwrap())
            .unwrap();
    assert_eq!(raw(&m, symbol)["drawings"], json!([typed]));
    assert!(!m.objects.contains_key(&row_id));
    assert!(!m.objects.contains_key(&point_id));
    let stale = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"circle_removals":[{"index":0,"previous":typed}]}),
    )
    .unwrap();
    let p = edit(&m, &root, "symbols", symbol, json!({"name":"Concurrent"})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let before = raw(&m, symbol);
    assert!(commit_prepared(&mut m, &root, stale).is_err());
    assert_eq!(
        raw(&ProjectResolver::new(&root).resolve().unwrap(), symbol),
        before
    );
}

#[test]
fn untouched_legacy_circle_is_not_subject_to_new_geometry_admission() {
    let mut legacy = circle();
    legacy["Circle"]["radius"] = json!(0);
    legacy["Circle"]["width"] = json!(-1);
    let (root, m, _, symbol) = fixture("circle_legacy", json!([legacy]));
    assert!(edit(&m, &root, "symbols", symbol, json!({"name":"Metadata"})).is_ok());
    assert!(
        edit(
            &m,
            &root,
            "symbols",
            symbol,
            json!({"circle_edits":[{"index":0,"previous":legacy,"circle":legacy}]})
        )
        .is_ok()
    );
    let mut changed = legacy.clone();
    changed["Circle"]["center"]["x"] = json!(1);
    assert!(
        edit(
            &m,
            &root,
            "symbols",
            symbol,
            json!({"circle_edits":[{"index":0,"previous":legacy,"circle":changed}]})
        )
        .is_err()
    );
}
