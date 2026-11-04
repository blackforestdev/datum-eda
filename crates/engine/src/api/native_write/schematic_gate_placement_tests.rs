//! Real native library/Proposal lifecycle, exact siblings and canonical history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    context::{BatchComposer, WriteProvenance},
    library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_delete_proposal::{
        self as proposal, DeleteClass, DeleteTarget, NativeProposalAction as Action,
        NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, ProjectResolver};
use std::path::{Path, PathBuf};
#[path = "schematic_gate_test_fixture.rs"]
mod fixture;
pub(crate) use fixture::{action, binding, provenance, raw, setup};
use fixture::{native, roundtrip};
#[test]
fn native_gate_placement_preview_cancel_accept_and_multi_member_edits_history() {
    let (root, mut model, first, mut req) = setup("gate-lifecycle");
    let before = raw(&model, first);
    let old_binding = binding(&model, req.component_instance);
    let planned = preview(&model, &req).unwrap();
    assert_eq!(planned, preview(&model, &req).unwrap());
    let material = materialize(
        &model,
        &options(&model, first, &[]).unwrap(),
        req.gate,
        req.proposal_id,
    )
    .unwrap();
    assert_eq!(
        material
            .placed(
                &req.reference,
                &req.value,
                Point::new(req.x_nm, req.y_nm),
                req.rotation_deg,
                req.mirrored
            )
            .unwrap(),
        planned.symbol
    );
    assert_eq!(raw(&model, first), before);
    let d = action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    assert_eq!(d.gate_placement, Some(planned.summary));
    action(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: req.proposal_id,
        },
    );
    assert_eq!(raw(&model, first), before);
    assert_eq!(binding(&model, req.component_instance), old_binding);
    req.proposal_id = Uuid::new_v4();
    let expected = preview(&model, &req).unwrap();
    let second = expected.symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let after = raw(&model, first);
    assert_eq!(
        after["symbols"][first.to_string()],
        before["symbols"][first.to_string()]
    );
    assert_eq!(
        serde_json::from_value::<PlacedSymbol>(after["symbols"][second.to_string()].clone())
            .unwrap(),
        expected.symbol
    );
    let first_pins: BTreeSet<_> =
        serde_json::from_value::<PlacedSymbol>(before["symbols"][first.to_string()].clone())
            .unwrap()
            .pins
            .iter()
            .map(|p| p.uuid)
            .collect();
    assert!(
        expected
            .symbol
            .pins
            .iter()
            .all(|p| !first_pins.contains(&p.uuid))
    );
    roundtrip(
        &mut model,
        &root,
        req.component_instance,
        &before,
        &old_binding,
        &after,
    );
    let source = source(&model).unwrap();
    let from = source.sheets[&req.sheet].symbols[&first].pins[0].position;
    let to = source.sheets[&req.sheet].symbols[&second].pins[1].position;
    let wire = Uuid::new_v4();
    let write=BatchComposer::compose(&model,provenance()).push_op(Operation::CreateSchematicWire{sheet_id:req.sheet,wire_id:wire,wire:json!({"uuid":wire,"from":{"x":from.x,"y":from.y,"extra":"from"},"to":{"x":to.x,"y":to.y,"extra":"to"},"extension":"attached"})}).finish().unwrap();
    let write = super::super::schematic_edit::prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let partitions = crate::substrate::electrical_test_partitions(&model, &[]).unwrap();
    let identities = model.electrical_identities.clone();
    let field = source.sheets[&req.sheet].symbols[&first].fields[0].uuid;
    for (verb, params) in [
        (
            "datum.schematic.move_symbol",
            json!({"symbol":second,"instance_path":[],"x_nm":60_000_000,"y_nm":30_000_000,"rotation_deg":90,"mirrored":true}),
        ),
        (
            "datum.schematic.set_symbol_reference",
            json!({"symbol":first,"instance_path":[],"reference":"U7","value":"explicit renamed gate"}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":first,"instance_path":[],"field":"custom","field_id":field,"visible":true,"position":{"x":7_000_000,"y":8_000_000}}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":second,"instance_path":[],"field":"custom","key":"gate note","value":"retain siblings"}),
        ),
        (
            "datum.schematic.move_symbol",
            json!({"symbols":[first,second],"instance_path":[],"reference":{"x":0,"y":0},"x_nm":5_000_000,"y_nm":5_000_000,"rotation_deg":90,"mirrored":true}),
        ),
    ] {
        let sheet = raw(&model, first);
        let ci = binding(&model, req.component_instance);
        native(&mut model, &root, verb, params);
        let after = raw(&model, first);
        assert_eq!(
            model
                .journal
                .last()
                .unwrap()
                .operations
                .iter()
                .filter(|o| matches!(o, Operation::SetComponentInstance { .. }))
                .count(),
            1
        );
        assert_eq!(
            after["wires"][wire.to_string()]["extension"],
            json!("attached")
        );
        assert_eq!(
            crate::substrate::electrical_test_partitions(&model, &[]).unwrap(),
            partitions
        );
        assert_eq!(model.electrical_identities, identities);
        roundtrip(
            &mut model,
            &root,
            req.component_instance,
            &sheet,
            &ci,
            &after,
        );
    }
    assert!(super::super::schematic_assignment::options(&model, first, &[]).is_err());
    assert!(super::super::schematic_annotation::preview(&model, req.sheet, &[]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_gate_placement_subset_and_last_delete_keep_sibling_owner_and_history() {
    let (root, mut model, first, req) = setup("gate-delete");
    let second = preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let both = raw(&model, first);
    let ci = binding(&model, req.component_instance);
    let delete = |symbol| Action::CreateDelete {
        proposal_id: Uuid::new_v4(),
        sheet: req.sheet,
        instance_path: vec![],
        targets: vec![DeleteTarget {
            class: DeleteClass::Symbol,
            id: symbol,
        }],
    };
    let d = action(&mut model, &root, delete(first));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: d.proposal_id,
        },
    );
    assert!(!model.objects.contains_key(&first));
    assert_eq!(
        model.component_instances[&req.component_instance].placed_symbol_refs,
        vec![second]
    );
    let one = raw(&model, second);
    assert_eq!(
        one["symbols"][second.to_string()],
        both["symbols"][second.to_string()]
    );
    roundtrip(&mut model, &root, req.component_instance, &both, &ci, &one);
    let before_binding = binding(&model, req.component_instance);
    let d = action(&mut model, &root, delete(second));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: d.proposal_id,
        },
    );
    assert!(!model.objects.contains_key(&req.component_instance));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(binding(&model, req.component_instance), before_binding);
    assert_eq!(raw(&model, second), one);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        !model
            .component_instances
            .contains_key(&req.component_instance)
    );
    std::fs::remove_dir_all(root).unwrap();
}
