//! Canonical lifecycle actions reuse the shared plan and exact raw inverse.
use super::{
    commit_prepared,
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    electrical::topology_tests,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_edit::prepare_native_birth,
    schematic_symbol_payload::source_sheet_payload,
};
use crate::{
    ir::geometry::Point,
    substrate::{
        CommitSource, DesignModel, ElectricalSelectionSnapshot, Operation, ProjectResolver,
    },
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "wire lifecycle")
}
fn raw_wire(sheet: Uuid, id: Uuid, from: Point, to: Point) -> Operation {
    Operation::CreateSchematicWire {
        sheet_id: sheet,
        wire_id: id,
        wire: json!({"uuid":id,
        "from":{"x":from.x,"y":from.y,"endpoint_extension":"outer-from"},
        "to":{"x":to.x,"y":to.y,"endpoint_extension":"outer-to"},
        "vendor":{"literal":"${KEEP}","weight":7}}),
    }
}
fn setup() -> (PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let (root, mut model, _, sheet) = topology_tests::fixture("wire_lifecycle_adapter");
    let id = Uuid::new_v4();
    let sentinel = Uuid::new_v4();
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(vec![
            raw_wire(sheet, id, Point::new(0, 0), Point::new(10_000, 0)),
            raw_wire(
                sheet,
                sentinel,
                Point::new(100_000, 0),
                Point::new(110_000, 0),
            ),
        ])
        .finish()
        .unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model, sheet, id, sentinel)
}
fn build(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    action: &str,
    handle: Value,
    at: Point,
) -> Result<PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"object":id,"instance_path":[],"x_nm":at.x,"y_nm":at.y,
            "wire_edit":{"action":action,"handle":handle,"edit_seed":Uuid::new_v4()}}),
    )
}
fn persist(model: &mut DesignModel, root: &Path, prepared: PreparedWrite) {
    commit_prepared(model, root, prepared).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
}
fn segment() -> Value {
    json!({"kind":"segment","from":{"x":0,"y":0},"to":{"x":10000,"y":0}})
}
fn ids(model: &DesignModel) -> std::collections::BTreeSet<Uuid> {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .net_ids()
}

#[test]
fn canonical_split_and_vertex_coalescence_preserve_outer_payload_and_cold_history() {
    let (root, mut model, sheet, id, sentinel) = setup();
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let before_ids = ids(&model);
    let count = model.journal.len();
    let prepared = build(&model, &root, id, "split", segment(), Point::new(5_000, 0)).unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    persist(&mut model, &root, prepared);
    let split = source_sheet_payload(&model, sheet, sentinel).unwrap();
    assert_eq!(split["wires"].as_object().unwrap().len(), 3);
    assert_eq!(
        ids(&model),
        before_ids,
        "a pure source split is not a scalar-net change"
    );
    assert_eq!(
        split["wires"][sentinel.to_string()],
        before["wires"][sentinel.to_string()]
    );
    let split_identities = model.electrical_identities.clone();
    let prepared = build(
        &model,
        &root,
        id,
        "remove_vertex",
        json!({"kind":"vertex","point":{"x":5000,"y":0}}),
        Point::new(5_000, 0),
    )
    .unwrap();
    persist(&mut model, &root, prepared);
    let coalesced = source_sheet_payload(&model, sheet, sentinel).unwrap();
    assert_eq!(coalesced["wires"].as_object().unwrap().len(), 2);
    assert_eq!(
        ids(&model),
        before_ids,
        "coalescence retains the logical scalar net"
    );
    let merged = coalesced["wires"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(key, _)| *key != &sentinel.to_string())
        .unwrap()
        .1;
    assert_eq!(merged["from"], before["wires"][id.to_string()]["from"]);
    assert_eq!(merged["to"], before["wires"][id.to_string()]["to"]);
    assert_eq!(merged["vendor"], before["wires"][id.to_string()]["vendor"]);
    let coalesced_identities = model.electrical_identities.clone();
    for redo in [false, true] {
        if redo {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&model, sheet, sentinel).unwrap(),
            if redo {
                coalesced.clone()
            } else {
                split.clone()
            }
        );
        assert_eq!(
            model.electrical_identities,
            if redo {
                coalesced_identities.clone()
            } else {
                split_identities.clone()
            }
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_remove_span_is_explicit_and_undo_restores_exact_raw_and_identity() {
    let (root, mut model, sheet, id, sentinel) = setup();
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let identities = model.electrical_identities.clone();
    let count = model.journal.len();
    let prepared = build(
        &model,
        &root,
        id,
        "remove_span",
        segment(),
        Point::new(5_000, 0),
    )
    .unwrap();
    assert_eq!(model.journal.len(), count);
    persist(&mut model, &root, prepared);
    let after = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let mut expected = before.clone();
    expected["wires"]
        .as_object_mut()
        .unwrap()
        .remove(&id.to_string());
    assert_eq!(after, expected, "only the captured span is removed");
    assert_eq!(model.journal.len(), count + 1);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.electrical_identities, identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        after
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_lifecycle_rejects_wrong_handle_target_and_incompatible_raw_merge() {
    let (root, mut model, sheet, id, sentinel) = setup();
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    for (action, handle, at) in [
        ("split", segment(), Point::new(5_000, 1)),
        ("split", segment(), Point::new(0, 0)),
        (
            "remove_span",
            json!({"kind":"endpoint","end":"from"}),
            Point::new(0, 0),
        ),
        ("remove_span", segment(), Point::new(5_000, 100)),
        ("remove_vertex", segment(), Point::new(5_000, 0)),
    ] {
        assert!(build(&model, &root, id, action, handle, at).is_err());
    }
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    let prepared = build(&model, &root, id, "split", segment(), Point::new(5_000, 0)).unwrap();
    persist(&mut model, &root, prepared);
    let split = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let other = split["wires"]
        .as_object()
        .unwrap()
        .keys()
        .find(|key| **key != id.to_string() && **key != sentinel.to_string())
        .unwrap();
    let other_id = Uuid::parse_str(other).unwrap();
    let original = split["wires"][other].clone();
    let mut changed = original.clone();
    changed["vendor"]["weight"] = json!(99);
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(vec![
            Operation::DeleteSchematicWire {
                sheet_id: sheet,
                wire_id: other_id,
                wire: original,
            },
            Operation::CreateSchematicWire {
                sheet_id: sheet,
                wire_id: other_id,
                wire: changed,
            },
        ])
        .finish()
        .unwrap();
    persist(&mut model, &root, prepared);
    let incompatible = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    let error = build(
        &model,
        &root,
        id,
        "remove_vertex",
        json!({"kind":"vertex","point":{"x":5000,"y":0}}),
        Point::new(5_000, 0),
    )
    .unwrap_err();
    assert!(error.to_string().contains("metadata"));
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        incompatible
    );
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_span_cleanup_merges_only_compatible_survivors_not_deleted_branch_metadata() {
    let (root, mut model, sheet, id, sentinel) = setup();
    let original = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let prepared = build(&model, &root, id, "split", segment(), Point::new(5_000, 0)).unwrap();
    persist(&mut model, &root, prepared);
    let branch = Uuid::new_v4();
    let junction = Uuid::new_v4();
    let mut branch_op = raw_wire(
        sheet,
        branch,
        Point::new(5_000, 5_000),
        Point::new(5_000, 0),
    );
    if let Operation::CreateSchematicWire { wire, .. } = &mut branch_op {
        wire["vendor"]["weight"] = json!(99);
    }
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(vec![
            branch_op,
            Operation::PlaceSchematicMarker {
                sheet_id: sheet,
                marker_id: junction,
                marker_kind: crate::substrate::SchematicMarkerKind::Junction,
                marker: json!({"uuid":junction,"position":{"x":5000,"y":0}}),
            },
        ])
        .finish()
        .unwrap();
    persist(&mut model, &root, prepared);
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let prepared = build(
        &model,
        &root,
        branch,
        "remove_span",
        json!({"kind":"segment","from":{"x":5000,"y":5000},"to":{"x":5000,"y":0}}),
        Point::new(5_000, 2_500),
    )
    .unwrap();
    persist(&mut model, &root, prepared);
    let after = source_sheet_payload(&model, sheet, sentinel).unwrap();
    assert!(after["wires"].get(branch.to_string()).is_none());
    assert!(after["junctions"].get(junction.to_string()).is_none());
    assert_eq!(after["wires"].as_object().unwrap().len(), 2);
    let merged = after["wires"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(key, _)| *key != &sentinel.to_string())
        .unwrap()
        .1;
    for key in ["from", "to", "vendor"] {
        assert_eq!(merged[key], original["wires"][id.to_string()][key]);
    }
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        after
    );
    std::fs::remove_dir_all(root).unwrap();
}
