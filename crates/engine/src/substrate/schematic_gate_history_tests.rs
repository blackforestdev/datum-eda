use super::*;
use crate::api::native_write::{
    schematic_delete_proposal::NativeProposalAction as Action,
    schematic_gate_placement::{
        preview,
        tests::{action, provenance, setup},
    },
};
#[test]
fn native_gate_history_rejects_unrecorded_membership_library_and_epoch_changes() {
    let (root, mut model, _, req) = setup("gate-history-tamper");
    let new_symbol = preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let tx = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    let target = prior.last().unwrap();
    assert!(matches(tx, target, prior));
    for control in 0..4 {
        let mut altered = tx.clone();
        for op in &mut altered.operations {
            if let Operation::SetComponentInstance {
                component_instance, ..
            } = op
            {
                match control {
                    0 => {
                        component_instance["placed_symbol_refs"][0]["object_revision"] = json!(999)
                    }
                    1 => component_instance["part_ref"]["object_id"] = json!(Uuid::new_v4()),
                    2 => component_instance["extension"] = json!("tampered"),
                    _ => component_instance["placed_symbol_refs"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"object_id":new_symbol,"object_revision":0})),
                }
            }
        }
        assert!(!matches(&altered, target, prior));
    }
    // Even an exactly mirrored operation/inverse pair cannot turn the narrow
    // membership history family into an arbitrary semantic library rewrite.
    let mut altered = target.clone();
    for op in &mut altered.operations {
        if let Operation::SetComponentInstance {
            component_instance, ..
        } = op
        {
            component_instance["part_ref"]["object_id"] = json!(Uuid::nil());
        }
    }
    for op in &mut altered.inverse_operations {
        if let Operation::SetComponentInstance {
            previous_component_instance,
            ..
        } = op
        {
            previous_component_instance["part_ref"]["object_id"] = json!(Uuid::nil());
        }
    }
    assert!(normalized(&altered, prior).is_none());
    std::fs::remove_dir_all(root).unwrap();
}
