//! Persistent logical history over actual assigned fields and pose writes.
use super::*;
use crate::api::native_write::{
    BatchComposer, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use serde_json::{Value, json};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "repeated history")
}
fn source(model: &DesignModel, symbol: Uuid) -> Value {
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
fn binding(model: &DesignModel, symbol: Uuid, id: Uuid, part: Uuid) -> Value {
    assert_eq!(model.component_instances[&id].part_ref, Some(part));
    assert_eq!(
        model.component_instances[&id].placed_symbol_refs,
        vec![symbol]
    );
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    let mut raw = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    assert_eq!(
        raw["object_revision"],
        json!(model.objects[&id].object_revision.0)
    );
    assert_eq!(
        raw["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    raw["object_revision"] = Value::Null;
    raw["placed_symbol_refs"][0]["object_revision"] = Value::Null;
    raw
}
fn native(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    let prepared = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    commit_prepared(model, root, prepared)
        .unwrap()
        .transaction
        .transaction_id
}
#[test]
fn journal_multistep_assigned_fields_pose_reopen_and_branch() {
    let (root, mut model, symbol, part) = fixture("multistep-assigned");
    let proposal_id = Uuid::new_v4();
    let mut ci = None;
    for action in [
        Action::CreateAssignment {
            proposal_id,
            symbol,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
        Action::AcceptApply { proposal_id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        ci = proposal::execute(&mut model, &request)
            .unwrap()
            .assignment
            .map(|s| s.component_instance);
    }
    let ci = ci.unwrap();
    let original_binding = binding(&model, symbol, ci, part);
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let first = source(&model, symbol)["fields"][0]["uuid"].clone();
    let mut snapshots = vec![source(&model, symbol)];
    let mut ids = vec![];
    for (verb, params) in [
        (
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbol,"instance_path":[],"reference":"U9","value":"A"}),
        ),
        (
            "datum.schematic.move_symbol",
            json!({"symbol":symbol,"instance_path":[],"x_nm":12_000_000,"y_nm":21_000_000,"rotation_deg":90}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":first,"value":"C","visible":true,"position":{"x":15_000_000,"y":25_000_000}}),
        ),
    ] {
        ids.push(native(&mut model, &root, verb, params));
        snapshots.push(source(&model, symbol));
    }
    let prefix = std::fs::read(transaction_journal_path(&root)).unwrap();
    let mut undo_ids = vec![];
    for index in (0..3).rev() {
        assert_eq!(
            model
                .journal_history_targets()
                .unwrap()
                .undo
                .unwrap()
                .transaction_id,
            ids[index]
        );
        let report = model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        undo_ids.push(report.transaction.transaction_id);
        assert_eq!(report.transaction.undo_of, Some(ids[index]));
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model, symbol), snapshots[index]);
        assert_eq!(binding(&model, symbol, ci, part), original_binding);
        assert_eq!(
            model.journal_cursor.applied_transaction_count,
            model.journal.len()
        );
        assert_eq!(model.journal_history_availability().unwrap(), (true, true));
    }
    // Proposal review metadata remains a real history step; no design-only skipping.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(!model.component_instances.contains_key(&ci));
    let unassigned = source(&model, symbol);
    let before_metadata = model.model_revision.clone();
    let metadata = model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(
        metadata
            .transaction
            .operations
            .iter()
            .all(|op| matches!(op, Operation::SetProposalMetadata { .. }))
    );
    assert_eq!(model.model_revision, before_metadata);
    assert_eq!(source(&model, symbol), unassigned);
    model = ProjectResolver::new(&root).resolve().unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(source(&model, symbol), unassigned);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, symbol), snapshots[0]);
    assert_eq!(binding(&model, symbol, ci, part), original_binding);
    for index in 0..3 {
        let report = model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        assert_eq!(report.transaction.redo_of, Some(undo_ids[2 - index]));
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model, symbol), snapshots[index + 1]);
        assert_eq!(binding(&model, symbol, ci, part), original_binding);
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    }
    assert!(
        std::fs::read(transaction_journal_path(&root))
            .unwrap()
            .starts_with(&prefix)
    );
    assert_eq!(model.journal_history_availability().unwrap(), (true, false));
    // A target payload cannot be forged in the caller while retaining its ID/tip.
    let mut forged = model.clone();
    for op in &mut forged.journal.last_mut().unwrap().inverse_operations {
        if let Operation::SetSchematicSymbol { symbol, .. } = op {
            symbol["reference"] = json!("FORGED");
        }
    }
    let before = std::fs::read(transaction_journal_path(&root)).unwrap();
    assert!(
        forged
            .commit_journal_undo(&root, provenance().into())
            .is_err()
    );
    assert_eq!(
        std::fs::read(transaction_journal_path(&root)).unwrap(),
        before
    );
    let mut stale = model.clone();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    native(
        &mut model,
        &root,
        "datum.schematic.set_symbol_value",
        json!({"symbol":symbol,"instance_path":[],"value":"branch"}),
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before = std::fs::read(transaction_journal_path(&root)).unwrap();
    assert!(
        stale
            .commit_journal_undo(&root, provenance().into())
            .is_err()
    );
    assert!(
        model
            .commit_journal_redo(&root, provenance().into())
            .is_err()
    );
    assert_eq!(model.journal_history_availability().unwrap(), (true, false));
    assert_eq!(
        std::fs::read(transaction_journal_path(&root)).unwrap(),
        before
    );
    assert_eq!(binding(&model, symbol, ci, part), original_binding);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn journal_multistep_noninvertible_barrier_and_wrong_stack_link() {
    let root = temp_project_root("multistep-barrier");
    let (project, board, package) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    write_project_with_board_package(&root, project, board, package);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::BumpObjectRevision { object_id: board })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(!model.journal_history_availability().unwrap().0);
    let mut ids = vec![];
    for value in ["A", "B", "C"] {
        let write = BatchComposer::compose(&model, provenance())
            .push_op(Operation::SetBoardPackageValue {
                package_id: package,
                value: value.into(),
            })
            .finish()
            .unwrap();
        ids.push(
            commit_prepared(&mut model, &root, write)
                .unwrap()
                .transaction
                .transaction_id,
        );
    }
    for _ in 0..3 {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    let before = std::fs::read(transaction_journal_path(&root)).unwrap();
    assert_eq!(model.journal_history_availability().unwrap(), (false, true));
    assert!(
        model
            .commit_journal_undo(&root, provenance().into())
            .is_err()
    );
    assert_eq!(
        std::fs::read(transaction_journal_path(&root)).unwrap(),
        before
    );
    let mut wrong = model.journal.last().unwrap().clone();
    wrong.transaction_id = Uuid::new_v4();
    wrong.undo_of = Some(ids[2]);
    assert!(
        crate::substrate::transaction_links::validate_transaction_links(
            &wrong,
            &model.journal[..model.journal.len() - 1]
        )
        .is_err()
    );
    for _ in 0..3 {
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    assert_eq!(
        read_json_value(&root.join("board/board.json")).unwrap()["packages"][package.to_string()]["value"],
        "C"
    );
    std::fs::remove_dir_all(root).unwrap();
}
