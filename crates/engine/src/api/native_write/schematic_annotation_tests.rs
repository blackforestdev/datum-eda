//! Actual native library Symbols, bound/unbound ownership and canonical Proposal history.
use super::*;
use crate::api::native_write::{
    BatchComposer, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
        NativeProposalResponse,
    },
};
use crate::substrate::{CommitSource, ProjectResolver, ProposalStatus, electrical_test_partitions};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "annotation fixture")
}
fn request(model: &DesignModel, root: &Path, action: Action) -> NativeProposalRequest {
    NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    }
}
fn act(model: &mut DesignModel, root: &Path, action: Action) -> NativeProposalResponse {
    proposal::execute(model, &request(model, root, action)).unwrap()
}
fn verb(model: &mut DesignModel, root: &Path, name: &str, params: Value) -> Uuid {
    let write = (find_native_write_verb(name).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
fn ci_semantics(model: &DesignModel, symbol: Uuid, ci: Uuid) -> Value {
    assert_eq!(
        model.component_instances[&ci].placed_symbol_refs,
        vec![symbol]
    );
    let mut payload = raw(model, ci)["component_instance"].clone();
    assert_eq!(
        payload["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    payload["object_revision"] = Value::Null;
    payload["placed_symbol_refs"][0]["object_revision"] = Value::Null;
    payload
}
pub(crate) fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, Vec<Uuid>, Vec<Uuid>) {
    let (root, mut model, first, part) = fixture(name);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let sheet = snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find(|s| s.symbols.contains_key(&first))
        .unwrap()
        .uuid;
    let mut ids = vec![first];
    for (reference, x, y) in [("U2", 0, 0), ("U3", 30_000_000, 40_000_000)] {
        ids.push(verb(&mut model, &root, "datum.schematic.place_symbol", json!({"sheet":sheet,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":reference,"value":"OPAMP","x_nm":x,"y_nm":y})));
    }
    let mut cis = vec![];
    for symbol in &ids[..2] {
        let id = Uuid::new_v4();
        act(
            &mut model,
            &root,
            Action::CreateAssignment {
                proposal_id: id,
                symbol: *symbol,
                instance_path: vec![],
                part,
                gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
            },
        );
        cis.push(
            act(&mut model, &root, Action::AcceptApply { proposal_id: id })
                .assignment
                .unwrap()
                .component_instance,
        );
    }
    (root, model, sheet, ids, cis)
}
pub(crate) fn annotate(model: &mut DesignModel, root: &Path, sheet: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    act(
        model,
        root,
        Action::CreateAnnotation {
            proposal_id: id,
            sheet,
            instance_path: vec![],
        },
    );
    act(model, root, Action::AcceptApply { proposal_id: id });
    id
}
#[test]
fn native_annotation_complete_sheet_swaps_raw_binding_history_and_reopen() {
    let (root, mut model, sheet, ids, cis) = setup("annotation-swaps-history");
    for ci in &cis {
        let previous = raw(&model, *ci)["component_instance"].clone();
        let mut next = previous.clone();
        next["object_revision"] = json!(model.objects[ci].object_revision.0 + 1);
        for binding in next["library_bindings"]
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            binding["provenance"] = json!({"opaque": ["retain", 7]});
        }
        let write = BatchComposer::compose(&model, provenance())
            .push_op(Operation::SetComponentInstance {
                component_instance_id: *ci,
                previous_component_instance: previous,
                component_instance: next,
            })
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let before = raw(&model, ids[0]);
    let bindings: Vec<_> = ids
        .iter()
        .zip(&cis)
        .map(|(s, c)| ci_semantics(&model, *s, *c))
        .collect();
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let identities = model.electrical_identities.clone();
    let summary = preview(&model, sheet, &[]).unwrap();
    assert_eq!(summary.symbol_count, 3);
    assert_eq!(summary.changes.len(), 2);
    assert!(
        summary
            .changes
            .iter()
            .any(|c| c.symbol == ids[0] && c.previous_reference == "U1" && c.reference == "U2")
    );
    assert!(
        summary
            .changes
            .iter()
            .any(|c| c.symbol == ids[1] && c.previous_reference == "U2" && c.reference == "U1")
    );
    let rejected = Uuid::new_v4();
    let draft = act(
        &mut model,
        &root,
        Action::CreateAnnotation {
            proposal_id: rejected,
            sheet,
            instance_path: vec![],
        },
    );
    assert_eq!(draft.annotation.as_ref(), Some(&summary));
    assert!(draft.assignment.is_none());
    assert!(model.proposals[&rejected].batch.operations.iter().any(
        |op| matches!(op, Operation::GuardObjectRevision{object_id,..} if *object_id==ids[2])
    ));
    assert_eq!(raw(&model, ids[0]), before);
    assert_eq!(
        act(
            &mut model,
            &root,
            Action::Reject {
                proposal_id: rejected
            }
        )
        .status,
        ProposalStatus::Rejected
    );
    assert_eq!(raw(&model, ids[0]), before);
    let id = annotate(&mut model, &root, sheet);
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = raw(&model, ids[0]);
    let mut expected = before.clone();
    expected["symbols"][ids[0].to_string()]["reference"] = json!("U2");
    expected["symbols"][ids[1].to_string()]["reference"] = json!("U1");
    assert_eq!(
        after, expected,
        "Only references changed; fields, pins and raw source remain exact"
    );
    assert_eq!(
        act(&mut model, &root, Action::Preview { proposal_id: id }).annotation,
        Some(summary)
    );
    for ((symbol, ci), binding) in ids.iter().zip(&cis).zip(&bindings) {
        assert_eq!(ci_semantics(&model, *symbol, *ci), *binding);
    }
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    assert_eq!(model.electrical_identities, identities);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, ids[0]), before);
    for ((symbol, ci), binding) in ids.iter().zip(&cis).zip(&bindings) {
        assert_eq!(ci_semantics(&model, *symbol, *ci), *binding);
    }
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, ids[0]), after);
    for ((symbol, ci), binding) in ids.iter().zip(&cis).zip(&bindings) {
        assert_eq!(ci_semantics(&model, *symbol, *ci), *binding);
    }
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    let count = model.journal.len();
    assert!(preview(&model, sheet, &[]).unwrap().changes.is_empty());
    let request = request(
        &model,
        &root,
        Action::CreateAnnotation {
            proposal_id: Uuid::new_v4(),
            sheet,
            instance_path: vec![],
        },
    );
    assert!(
        proposal::execute(&mut model, &request)
            .unwrap_err()
            .to_string()
            .contains("already annotated")
    );
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}
#[path = "schematic_annotation_refusal_tests.rs"]
mod refusals;

#[path = "schematic_annotation_planning_tests.rs"]
mod planning;
