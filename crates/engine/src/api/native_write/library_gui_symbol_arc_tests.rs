//! Arc sparse intent reuses canonical Circle fixtures without adding a writer.
use super::circle_tests::{edit, fixture, raw};
use super::*;
use crate::api::native_write::commit_prepared;
use crate::substrate::{CommitSource, ProjectResolver};
struct Cleanup(std::path::PathBuf);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit saved Symbol arcs")
}
fn arc() -> Value {
    json!({"Arc":{"arc":{"center":{"x":0,"y":0,"vendor":[3,1,3]},"radius":1000000,"start_angle":3500,"end_angle":100,"vendor":"range"},"width":100000,"unknown":true}})
}
#[test]
fn arc_lifecycle_preserves_duplicate_identity_raw_geometry_and_history() {
    let line = json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1,"vendor":true}});
    let (root, mut m, _, symbol) = fixture("arc_history", json!([arc(), line, arc()]));
    let _cleanup = Cleanup(root.clone());
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
    let mut changed = arc();
    changed["Arc"]["arc"]["center"]["x"] = json!(-2000000);
    changed["Arc"]["arc"]["radius"] = json!(2500000);
    changed["Arc"]["arc"]["start_angle"] = json!(i32::MIN);
    changed["Arc"]["arc"]["end_angle"] = json!(i32::MAX);
    let added = json!({"Arc":{"arc":{"center":{"x":1000000,"y":0},"radius":500000,"start_angle":0,"end_angle":3600},"width":120000}});
    let p=edit(&m,&root,"symbols",symbol,json!({"arc_edits":[{"index":2,"previous":arc(),"arc":changed}],"arc_removals":[{"index":0,"previous":arc()}],"graphic_additions":[added]})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let mut expected = before.clone();
    expected["drawings"] = json!([line, changed, added]);
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
fn arc_refusals_preserve_full_preimages_source_and_history() {
    let (root, m, _, symbol) = fixture("arc_refusal", json!([arc()]));
    let _cleanup = Cleanup(root.clone());
    let before = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let cursor = std::fs::read(root.join(".datum/journal/cursor.json")).unwrap();
    let mut changed = arc();
    changed["Arc"]["arc"]["radius"] = json!(2000000);
    let change = json!({"index":0,"previous":arc(),"arc":changed});
    let mut metadata = changed.clone();
    metadata["Arc"]["arc"]["center"]
        .as_object_mut()
        .unwrap()
        .remove("vendor");
    let mut requests = vec![
        json!({"arc_edits":[change.clone(),change.clone()]}),
        json!({"arc_edits":[change],"arc_removals":[{"index":0,"previous":arc()}]}),
        json!({"arc_edits":[{"index":0,"previous":arc(),"arc":metadata}]}),
        json!({"arc_removals":[{"index":1,"previous":arc()}]}),
        json!({"arc_removals":[{"index":0,"previous":changed}]}),
        json!({"line_additions":[arc()]}),
    ];
    for (pointer, value) in [
        ("/Arc/arc/radius", json!(0)),
        ("/Arc/arc/radius", json!(-1)),
        ("/Arc/width", json!(0)),
        ("/Arc/width", json!(-1)),
        ("/Arc/arc/start_angle", json!(i64::from(i32::MIN) - 1)),
        ("/Arc/arc/end_angle", json!(i64::from(i32::MAX) + 1)),
        ("/Arc/arc/center/x", json!(i64::MAX)),
        ("/Arc/arc/center/y", json!(i64::MIN)),
    ] {
        let mut invalid = arc();
        *invalid.pointer_mut(pointer).unwrap() = value;
        requests.push(json!({"graphic_additions":[invalid]}));
        requests.push(json!({"arc_edits":[{"index":0,"previous":arc(),"arc":invalid}]}));
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
    assert_eq!(
        std::fs::read(root.join(".datum/journal/cursor.json")).unwrap(),
        cursor
    );
}
#[test]
fn new_arc_serializes_typed_geometry_and_stale_batch_cannot_overwrite() {
    let (root, mut m, _, symbol) = fixture("arc_typed", json!([]));
    let _cleanup = Cleanup(root.clone());
    let ids = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let mut added = arc();
    added["Arc"]["uuid"] = json!(ids[0]);
    added["Arc"]["arc"]["uuid"] = json!(ids[1]);
    added["Arc"]["arc"]["center"]["uuid"] = json!(ids[2]);
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
        serde_json::to_value(serde_json::from_value::<crate::pool::Primitive>(arc()).unwrap())
            .unwrap();
    assert_eq!(raw(&m, symbol)["drawings"], json!([typed]));
    assert!(ids.iter().all(|id| !m.objects.contains_key(id)));
    let stale = edit(
        &m,
        &root,
        "symbols",
        symbol,
        json!({"arc_removals":[{"index":0,"previous":typed}]}),
    )
    .unwrap();
    let p = edit(&m, &root, "symbols", symbol, json!({"name":"Concurrent"})).unwrap();
    commit_prepared(&mut m, &root, p).unwrap();
    let before = raw(&m, symbol);
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    assert!(commit_prepared(&mut m, &root, stale).is_err());
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
fn unchanged_legacy_arc_keeps_raw_values_without_new_geometry_gate() {
    let mut legacy = arc();
    legacy["Arc"]["arc"]["radius"] = json!(0);
    legacy["Arc"]["width"] = json!(-1);
    let (root, m, _, symbol) = fixture("arc_legacy", json!([legacy]));
    let _cleanup = Cleanup(root.clone());
    assert!(edit(&m, &root, "symbols", symbol, json!({"name":"Metadata"})).is_ok());
    assert!(
        edit(
            &m,
            &root,
            "symbols",
            symbol,
            json!({"arc_edits":[{"index":0,"previous":legacy,"arc":legacy}]})
        )
        .is_ok()
    );
    let mut changed = legacy.clone();
    changed["Arc"]["arc"]["center"]["x"] = json!(1);
    assert!(
        edit(
            &m,
            &root,
            "symbols",
            symbol,
            json!({"arc_edits":[{"index":0,"previous":legacy,"arc":changed}]})
        )
        .is_err()
    );
}
