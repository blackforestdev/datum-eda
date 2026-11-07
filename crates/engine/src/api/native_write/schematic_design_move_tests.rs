//! Consecutive local edits use ordinary canonical history, without hidden drafts.
use super::super::{
    design_clipboard_edit::ExistingTransformContactPolicy, registry::find_native_write_verb,
};
use super::*;
use crate::substrate::TransactionKind;
use serde_json::Value;
fn manual() -> WriteProvenance {
    WriteProvenance::new("datum-gui", CommitSource::Manual, "visible local edit")
}
fn move_build(
    model: &DesignModel,
    root: &Path,
    source: CommitSource,
    params: Value,
) -> Result<PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        WriteProvenance::new("datum-gui", source, "manual visible edit"),
        params,
    )
}
fn direct(sheet: Uuid, edit: DesignEditIntent) -> Value {
    json!({"sheet":sheet,"instance_path":[],"design_edit":edit})
}
fn snapshot(
    model: &DesignModel,
    sheet: Uuid,
    sentinel: Uuid,
) -> (
    Value,
    std::collections::BTreeMap<Uuid, crate::substrate::ElectricalIdentityRecord>,
) {
    (
        source_sheet_payload(model, sheet, sentinel).unwrap(),
        model.electrical_identities.clone(),
    )
}
#[test]
fn manual_consecutive_endpoint_transform_paste_each_one_transaction_and_exact_cold_history() {
    let (root, mut model, sheet, wire, junction, sentinel) = fixture(false);
    let count = model.journal.len();
    let mut states = vec![snapshot(&model, sheet, sentinel)];
    let endpoint=move_build(&model,&root,CommitSource::Manual,json!({"object":wire,"instance_path":[],"x_nm":2000,"y_nm":0,"wire_edit":{"handle":{"kind":"endpoint","end":"to"},"routing":"free","edit_seed":Uuid::new_v4()}})).unwrap();
    persist(&mut model, &root, endpoint);
    states.push(snapshot(&model, sheet, sentinel));
    let edit = DesignEditIntent::TransformExisting {
        targets: targets(wire, junction),
        reference: Point::zero(),
        placement: ClipboardPlacement {
            position: Point::new(2731, 5019),
            rotation_deg: 90,
            mirrored: true,
        },
        contact_policy: ExistingTransformContactPolicy::PreserveFixedContacts,
    };
    let transform = move_build(&model, &root, CommitSource::Manual, direct(sheet, edit)).unwrap();
    persist(&mut model, &root, transform);
    states.push(snapshot(&model, sheet, sentinel));
    let bundle = DesignClipboardBundle::capture(
        &model,
        sheet,
        &[],
        &targets(wire, junction),
        Point::new(2731, 5019),
    )
    .unwrap();
    let seed = Uuid::new_v4();
    let edit = DesignEditIntent::PlaceBundle {
        bundle,
        placement: ClipboardPlacement {
            position: Point::new(20000, 20000),
            rotation_deg: 0,
            mirrored: false,
        },
        edit_seed: seed,
        contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
    };
    let mut paste = move_build(&model, &root, CommitSource::Manual, direct(sheet, edit)).unwrap();
    assert_eq!(paste.batch.batch_id, seed);
    // Same final native.write envelope batch identity, assigned by the daemon.
    paste.batch.batch_id = seed;
    persist(&mut model, &root, paste);
    states.push(snapshot(&model, sheet, sentinel));
    assert_eq!(model.journal.len(), count + 3);
    assert!(model.proposals.is_empty());
    for tx in &model.journal[count..] {
        assert_eq!(tx.transaction_kind, TransactionKind::Normal);
        assert!(!tx.operations.iter().any(|op| matches!(
            op,
            Operation::CreateProposalMetadata { .. }
                | Operation::SetProposalMetadata { .. }
                | Operation::DeleteProposalMetadata { .. }
        )));
    }
    for expected in states[..3].iter().rev() {
        model.commit_journal_undo(&root, manual().into()).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(&snapshot(&model, sheet, sentinel), expected);
    }
    for expected in &states[1..] {
        model.commit_journal_redo(&root, manual().into()).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(&snapshot(&model, sheet, sentinel), expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manual_design_move_rejects_automation_cut_mixed_shapes_and_stale_write() {
    let (root, mut model, sheet, wire, junction, sentinel) = fixture(false);
    let before = snapshot(&model, sheet, sentinel);
    let edit = DesignEditIntent::TransformExisting {
        targets: targets(wire, junction),
        reference: Point::zero(),
        placement: ClipboardPlacement {
            position: Point::new(2731, 5019),
            rotation_deg: 90,
            mirrored: false,
        },
        contact_policy: ExistingTransformContactPolicy::PreserveFixedContacts,
    };
    let params = direct(sheet, edit.clone());
    for source in [
        CommitSource::Assistant,
        CommitSource::Tool,
        CommitSource::Cli,
        CommitSource::Test,
    ] {
        assert!(move_build(&model, &root, source, params.clone()).is_err());
    }
    let count = model.journal.len();
    let mut mixed = params.clone();
    mixed["object"] = json!(wire);
    mixed["x_nm"] = json!(0);
    assert!(move_build(&model, &root, CommitSource::Manual, mixed).is_err());
    let mut wrongpath = params.clone();
    wrongpath["instance_path"] = json!([Uuid::new_v4()]);
    assert!(move_build(&model, &root, CommitSource::Manual, wrongpath).is_err());
    assert!(
        move_build(
            &model,
            &root,
            CommitSource::Manual,
            direct(
                sheet,
                DesignEditIntent::Cut {
                    targets: targets(wire, junction)
                }
            )
        )
        .is_err()
    );
    assert_eq!(model.journal.len(), count);
    assert_eq!(snapshot(&model, sheet, sentinel), before);
    let stale = move_build(&model, &root, CommitSource::Manual, params).unwrap();
    let fresh = Uuid::new_v4();
    let write = BatchComposer::compose(&model, manual())
        .push_op(create_wire(
            sheet,
            fresh,
            Point::new(30000, 0),
            Point::new(31000, 0),
            false,
        ))
        .finish()
        .unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    persist(&mut model, &root, write);
    let current = snapshot(&model, sheet, sentinel);
    let count = model.journal.len();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(model.journal.len(), count);
    assert_eq!(snapshot(&model, sheet, sentinel), current);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manual_first_paste_birth_uses_retained_final_request_batch_identity() {
    let (source_root, source, sheet, wire, junction, _) = fixture(false);
    let bundle = DesignClipboardBundle::capture(
        &source,
        sheet,
        &[],
        &targets(wire, junction),
        Point::zero(),
    )
    .unwrap();
    let (root, mut model, _, target) = topology_tests::fixture("manual_first_paste");
    let seed = Uuid::new_v4();
    let edit = DesignEditIntent::PlaceBundle {
        bundle,
        placement: ClipboardPlacement {
            position: Point::new(2731, 5019),
            rotation_deg: 0,
            mirrored: false,
        },
        edit_seed: seed,
        contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
    };
    let params = direct(target, edit);
    let first = move_build(&model, &root, CommitSource::Manual, params.clone()).unwrap();
    let second = move_build(&model, &root, CommitSource::Manual, params).unwrap();
    assert_eq!(
        first.batch, second.batch,
        "physical and first semantic allocations are stable before commit"
    );
    assert_eq!(first.batch.batch_id, seed);
    let allocations: Vec<_> = first
        .batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::CreateElectricalIdentity { record } => Some(record.id),
            _ => None,
        })
        .collect();
    assert_eq!(
        allocations,
        vec![Uuid::new_v5(&seed, b"datum:net-allocation:0")]
    );
    let count = model.journal.len();
    persist(&mut model, &root, first);
    assert_eq!(model.journal.len(), count + 1);
    assert!(model.proposals.is_empty());
    assert!(
        allocations
            .iter()
            .all(|id| model.electrical_identities.contains_key(id))
    );
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(source_root).unwrap();
}
