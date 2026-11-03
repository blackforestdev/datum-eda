use super::*;
use crate::api::native_write::{
    WriteProvenance,
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, ProjectResolver};
#[test]
fn native_assignment_history_rejects_semantic_and_epoch_tampering() {
    let (root, mut model, symbol, part) = fixture("assignment-history-tamper");
    let proposal_id = Uuid::new_v4();
    let mut act = |action| {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(&mut model, &request).unwrap()
    };
    act(Action::CreateAssignment {
        proposal_id,
        symbol,
        instance_path: vec![],
        part,
        gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
    });
    act(Action::AcceptApply { proposal_id });
    model
        .commit_journal_undo(
            &root,
            WriteProvenance::new("test", CommitSource::Manual, "undo assignment").into(),
        )
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    model
        .commit_journal_redo(
            &root,
            WriteProvenance::new("test", CommitSource::Manual, "redo assignment").into(),
        )
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let tx = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    let target = prior.last().unwrap();
    assert!(matches(tx, target, prior));
    for epoch in [false, true] {
        let mut tampered = tx.clone();
        for op in &mut tampered.operations {
            if let Operation::CreateComponentInstance {
                component_instance, ..
            } = op
            {
                if epoch {
                    component_instance["placed_symbol_refs"][0]["object_revision"] = json!(999);
                } else {
                    component_instance["part_ref"]["object_id"] = json!(Uuid::new_v4());
                }
            }
        }
        assert!(!matches(&tampered, target, prior));
    }
    let mut unrelated = target.clone();
    for op in &mut unrelated.inverse_operations {
        if let Operation::SetSchematicSymbol { symbol, .. } = op {
            symbol["value"] = json!("unrelated");
        }
    }
    assert!(normalized(&unrelated, prior).is_none());
    let mut tampered = tx.clone();
    tampered.inverse_operations.pop();
    assert!(!matches(&tampered, target, prior));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_bound_fields_history_rejects_unrecorded_semantics() {
    use crate::api::native_write::{
        commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    let (root, mut model, symbol, part) = fixture("bound-fields-history-tamper");
    let id = Uuid::new_v4();
    for action in [
        Action::CreateAssignment {
            proposal_id: id,
            symbol,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
        Action::AcceptApply { proposal_id: id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(&mut model, &request).unwrap();
    }
    let provenance =
        || WriteProvenance::new("test", CommitSource::Manual, "bound property history");
    let write = (find_native_write_verb("datum.schematic.set_symbol_reference")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":symbol,"instance_path":[],"reference":"U9","value":"property edit"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let tx = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    let target = prior.last().unwrap();
    assert!(matches(tx, target, prior));
    let mut changed = tx.clone();
    for op in &mut changed.operations {
        if let Operation::SetSchematicSymbol { symbol, .. } = op {
            symbol["reference"] = json!("unrecorded");
        }
    }
    assert!(!matches(&changed, target, prior));
    for property in ["part", "position", "pins", "fields"] {
        let mut changed = target.clone();
        for op in &mut changed.inverse_operations {
            if let Operation::SetSchematicSymbol { symbol, .. } = op {
                match property {
                    "part" => symbol["part"] = json!(Uuid::new_v4()),
                    "position" => symbol["position"]["x"] = json!(42),
                    "pins" => symbol["pins"][0]["name"] = json!("different logical pin"),
                    _ => symbol["fields"][0]["extension"] = json!("lost opaque payload"),
                }
            }
        }
        assert!(normalized(&changed, prior).is_none(), "{property}");
    }
    std::fs::remove_dir_all(root).unwrap();
}
