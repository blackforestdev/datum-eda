//! Reject altered shared-member history without weakening source/owner checks.
use super::super::super::{OperationBatch, prepare};
use super::*;
use crate::api::native_write::{
    WriteProvenance,
    schematic_assignment::{options, tests::multigate::setup},
    schematic_delete_proposal::NativeProposalAction as Action,
    schematic_gate_placement::tests::action,
};
use crate::substrate::CommitSource;
use serde_json::json;

#[test]
fn native_assignment_multigate_history_refuses_semantic_and_paired_membership_tampering() {
    let (root, mut model, members, ci, alternate) = setup("assignment-multigate-history-tamper");
    let gate = options(&model, members[0], &[])
        .unwrap()
        .original
        .gate
        .unwrap();
    let draft = action(
        &mut model,
        &root,
        Action::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol: members[0],
            instance_path: vec![],
            part: alternate,
            gate,
        },
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    let target = model.journal.last().unwrap().clone();
    let batch = |target: &TransactionRecord| OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: WriteProvenance::new("test", CommitSource::Manual, "history refusal control")
            .into(),
        operations: target.inverse_operations.clone(),
    };
    assert!(prepare(&model, &target, &mut batch(&target)).unwrap());
    for case in [
        "gate",
        "entity",
        "raw",
        "paired_gate",
        "paired_entity",
        "paired_raw",
        "missing_symbol",
        "duplicate_symbol",
        "missing_member",
        "duplicate_member",
    ] {
        let mut tampered = target.clone();
        let replacement = Uuid::new_v4();
        for inverse in [false, true] {
            let ops = if inverse {
                &mut tampered.inverse_operations
            } else {
                &mut tampered.operations
            };
            if case == "missing_symbol" {
                ops.retain(|op| !matches!(op,Operation::SetSchematicSymbol{symbol_id,..} if *symbol_id==members[1]));
            } else if case == "duplicate_symbol" {
                let duplicate=ops.iter().find(|op|matches!(op,Operation::SetSchematicSymbol{symbol_id,..} if *symbol_id==members[1])).unwrap().clone();
                ops.push(duplicate);
            }
            for op in ops {
                match op {
                    Operation::SetSchematicSymbol {
                        symbol_id, symbol, ..
                    } if *symbol_id == members[1] && (inverse || case.starts_with("paired_")) => {
                        match case.trim_start_matches("paired_") {
                            "gate" => symbol["gate"] = json!(replacement),
                            "entity" => symbol["entity"] = json!(replacement),
                            "raw" => symbol["extension"] = json!("unrecorded"),
                            _ => {}
                        }
                    }
                    Operation::SetComponentInstance {
                        component_instance_id,
                        component_instance,
                        previous_component_instance,
                    } if *component_instance_id == ci
                        && matches!(case, "missing_member" | "duplicate_member") =>
                    {
                        for raw in [component_instance, previous_component_instance] {
                            let refs = raw["placed_symbol_refs"].as_array_mut().unwrap();
                            if case == "missing_member" {
                                refs.pop();
                            } else {
                                refs.push(refs[0].clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        // A paired edit must also fail when the altered in-memory journal agrees
        // with the candidate: persisted source remains the independent preimage.
        let mut candidate = model.clone();
        *candidate.journal.last_mut().unwrap() = tampered.clone();
        if case.starts_with("paired_") {
            assert!(
                prepare(&candidate, &tampered, &mut batch(&tampered)).is_err(),
                "source preimage accepted {case}"
            );
        } else {
            assert!(capture(&tampered).is_none(), "classifier accepted {case}");
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
