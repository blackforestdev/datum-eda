//! Annotation through the ordinary native multi-gate and Proposal lifecycles.
use super::*;
use crate::api::native_write::{
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
    schematic_gate_placement::{
        self as gates,
        tests::{action, binding, provenance, raw, setup},
    },
};
use crate::substrate::{ProjectResolver, electrical_test_partitions};

#[test]
fn annotation_multigate_changed_subset_and_whole_owner_preserve_history() {
    for swap in [false, true] {
        let (root, mut model, first, mut req) = setup("annotation-multigate");
        req.reference = "U9".into();
        req.x_nm = if swap { 0 } else { 50_000_000 };
        let second = gates::preview(&model, &req).unwrap().symbol.uuid;
        action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
        action(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: req.proposal_id,
            },
        );
        let before = raw(&model, first);
        let old_binding = binding(&model, req.component_instance);
        let old_epochs = [
            model.objects[&first].object_revision,
            model.objects[&second].object_revision,
        ];
        let owner_epoch = model.objects[&req.component_instance].object_revision;
        let identities = model.electrical_identities.clone();
        let partitions = electrical_test_partitions(&model, &[]).unwrap();
        let plan = preview(&model, req.sheet, &[]).unwrap();
        assert_eq!(plan.symbol_count, 2);
        assert_eq!(plan.changes.len(), if swap { 2 } else { 1 });
        let draft = action(
            &mut model,
            &root,
            Action::CreateAnnotation {
                proposal_id: Uuid::new_v4(),
                sheet: req.sheet,
                instance_path: vec![],
            },
        );
        assert_eq!(draft.annotation, Some(plan.clone()));
        assert_eq!(raw(&model, first), before);
        action(
            &mut model,
            &root,
            Action::Reject {
                proposal_id: draft.proposal_id,
            },
        );
        assert_eq!(raw(&model, first), before);
        assert_eq!(binding(&model, req.component_instance), old_binding);
        let draft = action(
            &mut model,
            &root,
            Action::CreateAnnotation {
                proposal_id: Uuid::new_v4(),
                sheet: req.sheet,
                instance_path: vec![],
            },
        );
        assert_eq!(
            model.proposals[&draft.proposal_id]
                .batch
                .operations
                .iter()
                .filter(|op| matches!(op, Operation::SetComponentInstance { .. }))
                .count(),
            1
        );
        // A recovered receipt cannot advance the untouched sibling or alter Gate data.
        if !swap {
            let mut forged = model.proposals[&draft.proposal_id].clone();
            for op in &mut forged.batch.operations {
                if let Operation::SetComponentInstance {
                    component_instance, ..
                } = op
                {
                    for member in component_instance["placed_symbol_refs"]
                        .as_array_mut()
                        .unwrap()
                    {
                        if member["object_id"] == json!(first) {
                            member["object_revision"] = json!(old_epochs[0].0 + 1);
                        }
                    }
                }
            }
            assert!(validate_recorded(&forged).is_err());
        }
        action(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: draft.proposal_id,
            },
        );
        model = ProjectResolver::new(&root).resolve().unwrap();
        let after = raw(&model, first);
        let mut expected = before.clone();
        for change in &plan.changes {
            expected["symbols"][change.symbol.to_string()]["reference"] = json!(change.reference);
        }
        assert_eq!(after, expected);
        assert_eq!(binding(&model, req.component_instance), old_binding);
        assert_eq!(
            model.objects[&req.component_instance].object_revision.0,
            owner_epoch.0 + 1
        );
        assert_eq!(
            model.objects[&first].object_revision.0,
            old_epochs[0].0 + u64::from(swap)
        );
        assert_eq!(
            model.objects[&second].object_revision.0,
            old_epochs[1].0 + 1
        );
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
        assert!(preview(&model, req.sheet, &[]).unwrap().changes.is_empty());
        assert_eq!(
            summary(&model, &model.proposals[&draft.proposal_id], false).unwrap(),
            plan
        );
        for _ in 0..2 {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(raw(&model, first), before);
            assert_eq!(binding(&model, req.component_instance), old_binding);
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(raw(&model, first), after);
            assert_eq!(binding(&model, req.component_instance), old_binding);
            assert_eq!(model.electrical_identities, identities);
            assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn annotation_multigate_stale_member_refuses_complete_sheet_without_writes() {
    let (root, mut model, first, mut req) = setup("annotation-multigate-stale");
    req.reference = "U9".into();
    let second = gates::preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let before = raw(&model, first);
    let journal = model.journal.clone();
    model.objects.get_mut(&second).unwrap().object_revision.0 += 1;
    let request = NativeProposalRequest {
        project_root: root.clone(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action: Action::CreateAnnotation {
            proposal_id: Uuid::new_v4(),
            sheet: req.sheet,
            instance_path: vec![],
        },
    };
    assert!(proposal::execute(&mut model, &request).is_err());
    assert_eq!(model.journal, journal);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&reopened, first), before);
    assert_eq!(reopened.journal, journal);
    std::fs::remove_dir_all(root).unwrap();
}
