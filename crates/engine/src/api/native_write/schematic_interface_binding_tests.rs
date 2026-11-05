//! Canonical GUI Proposal lifecycle, connectivity isolation and cold history.
use super::super::{
    commit_prepared,
    electrical::topology_tests,
    schematic_child_sheet::{self, ChildSheetRequest},
    schematic_connectivity::*,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use super::*;
use crate::{
    ir::geometry::Point,
    schematic::{HierarchicalPort, LabelKind, NetLabel, PortDirection},
    substrate::{ProjectResolver, electrical_test_partitions},
};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "interface test")
}
fn act(
    model: &mut DesignModel,
    root: &Path,
    action: Action,
) -> Result<proposal::NativeProposalResponse, EngineError> {
    proposal::execute(
        model,
        &NativeProposalRequest {
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        },
    )
}
#[test]
fn explicit_interface_proposal_preserves_endpoints_isolates_nets_and_replays_cold() {
    let (root, mut model, _, parent) = topology_tests::fixture("interface_proposal");
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Child".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let parent_wire = wire(&mut model, &root, parent, 0, 10);
    let child_wire = wire(&mut model, &root, child.sheet, 0, 10);
    let unrelated = wire(&mut model, &root, parent, 20, 30);
    let port = HierarchicalPort {
        uuid: Uuid::new_v4(),
        name: "SIGNAL".into(),
        direction: PortDirection::Passive,
        position: Point::new(0, 0),
    };
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: port.name.clone(),
        kind: LabelKind::Hierarchical,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_label(&model, provenance(), child.sheet, &label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let write = build_create_schematic_port(&model, provenance(), parent, &port).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let adopted =
        super::super::electrical::build_adopt_schematic_net_identities(&model, provenance())
            .unwrap();
    commit_prepared(&mut model, &root, adopted).unwrap();
    assert!(!model.electrical_identities.is_empty());
    let before = source(&model).unwrap();
    let partitions_before = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(partitions_before.len(), 3);
    let mut request = InterfaceBindingRequest {
        proposal_id: Uuid::new_v4(),
        instance: child.instance,
        instance_path: vec![child.instance],
        parent_port: port.uuid,
        bind: true,
    };
    for bad in [
        InterfaceBindingRequest {
            instance_path: vec![],
            ..request.clone()
        },
        InterfaceBindingRequest {
            parent_port: Uuid::new_v4(),
            ..request.clone()
        },
        InterfaceBindingRequest {
            bind: false,
            ..request.clone()
        },
    ] {
        let unchanged = model.clone();
        assert!(act(&mut model, &root, Action::CreateInterfaceBinding(bad)).is_err());
        assert_eq!(model, unchanged);
    }
    let expected = preview(&model, &request).unwrap();
    assert_eq!(expected.child_label, label.uuid);
    let draft = act(
        &mut model,
        &root,
        Action::CreateInterfaceBinding(request.clone()),
    )
    .unwrap();
    assert_eq!(draft.interface_binding, Some(expected));
    assert_eq!(source(&model).unwrap(), before);
    act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: request.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(source(&model).unwrap(), before);
    request.proposal_id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateInterfaceBinding(request.clone()),
    )
    .unwrap();
    // A forged reviewed payload must never reach Accepted or alter history.
    let original = model.proposals[&request.proposal_id].clone();
    let mut tampered = original.clone();
    for operation in &mut tampered.batch.operations {
        if let crate::substrate::Operation::SetSchematicSheetInstance { instance, .. } = operation {
            instance["name"] = json!("unreviewed rename");
        }
    }
    model.proposals.insert(request.proposal_id, tampered);
    let unchanged = model.clone();
    assert!(
        act(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: request.proposal_id
            }
        )
        .is_err()
    );
    assert_eq!(model, unchanged);
    let mut forged_identity = original.clone();
    let count = forged_identity.batch.operations.len();
    forged_identity.batch.operations.retain(|operation| {
        !matches!(
            operation,
            crate::substrate::Operation::CreateElectricalIdentity { .. }
                | crate::substrate::Operation::SetElectricalIdentity { .. }
        )
    });
    assert!(forged_identity.batch.operations.len() < count);
    model.proposals.insert(request.proposal_id, forged_identity);
    let unchanged = model.clone();
    assert!(
        act(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: request.proposal_id
            }
        )
        .is_err()
    );
    assert_eq!(model, unchanged);
    model.proposals.insert(request.proposal_id, original);
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    )
    .unwrap();
    let bound = source(&model).unwrap();
    let groups = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(groups.len(), 2);
    let connected = groups
        .iter()
        .find(|group| group.iter().any(|m| m.source_id == parent_wire.uuid))
        .unwrap();
    assert!(
        connected
            .iter()
            .any(|m| m.source_id == child_wire.uuid && m.instance_path == vec![child.instance])
    );
    assert!(!connected.iter().any(|m| m.source_id == unrelated.uuid));
    assert_eq!(bound.sheets, before.sheets);
    assert!(preview(&model, &request).is_err());
    request.proposal_id = Uuid::new_v4();
    request.bind = false;
    act(
        &mut model,
        &root,
        Action::CreateInterfaceBinding(request.clone()),
    )
    .unwrap();
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(source(&model).unwrap(), before);
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        partitions_before
    );
    let unbound_ids = model.electrical_identities.clone();
    for _ in 0..2 {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model).unwrap(), bound);
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), groups);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model).unwrap(), before);
        assert_eq!(model.electrical_identities, unbound_ids);
    }
    request.proposal_id = Uuid::new_v4();
    request.bind = true;
    act(
        &mut model,
        &root,
        Action::CreateInterfaceBinding(request.clone()),
    )
    .unwrap();
    wire(&mut model, &root, parent, 40, 50);
    let unchanged = model.clone();
    assert!(
        act(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: request.proposal_id
            }
        )
        .is_err()
    );
    assert_eq!(model, unchanged);
    // Unbound authoring is valid, but an ambiguous selected match is not.
    let duplicate = NetLabel {
        uuid: Uuid::new_v4(),
        position: Point::new(70, 0),
        ..label.clone()
    };
    let write =
        build_create_schematic_label(&model, provenance(), child.sheet, &duplicate).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    request.proposal_id = Uuid::new_v4();
    let unchanged = model.clone();
    assert!(
        act(
            &mut model,
            &root,
            Action::CreateInterfaceBinding(request.clone())
        )
        .is_err()
    );
    assert_eq!(model, unchanged);
    let write =
        build_delete_schematic_label(&model, provenance(), child.sheet, &duplicate).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut repeated = source(&model).unwrap().sheet_instances[&child.instance].clone();
    repeated.uuid = Uuid::new_v4();
    let write = schematic_sheets::build_create_schematic_sheet_instance(
        &model,
        provenance(),
        before.uuid,
        repeated.uuid,
        serde_json::to_value(repeated).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let unchanged = model.clone();
    assert!(act(&mut model, &root, Action::CreateInterfaceBinding(request)).is_err());
    assert_eq!(model, unchanged);
    std::fs::remove_dir_all(root).unwrap();
}

fn wire(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    from: i64,
    to: i64,
) -> crate::schematic::SchematicWire {
    let wire = crate::schematic::SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(from, 0),
        to: Point::new(to, 0),
    };
    let write = build_create_schematic_wire(model, provenance(), sheet, &wire).unwrap();
    commit_prepared(model, root, write).unwrap();
    wire
}
