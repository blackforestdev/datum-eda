use super::*;
use crate::api::native_write::{
    WriteProvenance,
    schematic_delete_proposal::{
        self as proposal, DeleteClass, DeleteTarget, NativeProposalAction as Action,
        NativeProposalRequest, bound_tests::setup,
    },
};
#[test]
fn native_bound_delete_history_rejects_semantic_epoch_and_order_tampering() {
    let (root, mut model, sheet, symbol, _, _) = setup("bound-delete-history-tamper");
    let proposal_id = Uuid::new_v4();
    for action in [
        Action::CreateDelete {
            proposal_id,
            sheet,
            instance_path: vec![],
            targets: vec![DeleteTarget {
                class: DeleteClass::Symbol,
                id: symbol,
            }],
        },
        Action::AcceptApply { proposal_id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(&mut model, &request).unwrap();
    }
    model
        .commit_journal_undo(
            &root,
            WriteProvenance::new(
                "test",
                crate::substrate::CommitSource::Manual,
                "delete history",
            )
            .into(),
        )
        .unwrap();
    model = crate::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let tx = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    let target = prior.last().unwrap();
    assert!(matches(tx, target, prior));
    for control in 0..5 {
        let mut altered = tx.clone();
        for op in &mut altered.operations {
            match op {
                Operation::CreateComponentInstance {
                    component_instance, ..
                } => match control {
                    0 => {
                        component_instance["placed_symbol_refs"][0]["object_revision"] = json!(999)
                    }
                    1 => component_instance["part_ref"]["object_id"] = json!(Uuid::new_v4()),
                    2 => component_instance["object_revision"] = json!(999),
                    _ => {}
                },
                Operation::CreateSchematicSymbol { symbol, .. } if control == 3 => {
                    symbol["value"] = json!("unrecorded edit")
                }
                _ => {}
            }
        }
        if control == 4 {
            altered.operations.reverse();
        }
        assert!(!matches(&altered, target, prior));
    }
    let mut altered = target.clone();
    altered.inverse_operations.pop();
    assert!(normalized(&altered, prior).is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_bound_delete_restoration_orders_only_required_source_births() {
    let (sheet, a, b, ci, unrelated) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let binding = Operation::CreateComponentInstance {
        component_instance_id: ci,
        component_instance: json!({"placed_symbol_refs":[{"object_id":a},{"object_id":b}]}),
    };
    let first = Operation::CreateSchematicSymbol {
        sheet_id: sheet,
        symbol_id: a,
        symbol: json!({"uuid":a}),
    };
    let second = Operation::CreateSchematicSymbol {
        sheet_id: sheet,
        symbol_id: b,
        symbol: json!({"uuid":b}),
    };
    let trailing = Operation::BumpObjectRevision {
        object_id: unrelated,
    };
    let mut ops = vec![
        binding.clone(),
        first.clone(),
        second.clone(),
        trailing.clone(),
    ];
    order_restoration(&mut ops);
    assert_eq!(ops, vec![first, second, binding, trailing]);
    let stable = ops.clone();
    order_restoration(&mut ops);
    assert_eq!(ops, stable);
}
