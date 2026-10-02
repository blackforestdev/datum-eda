//! E1-IR-02: terminal provenance cannot substitute for logical occurrence membership.
use super::super::{commit_prepared, schematic_connectivity::*, schematic_sheets::*};
use super::binding_fixture::{BindingFixture, fixture};
use super::tests::provenance;
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::{SchematicWire, SheetDefinition, SheetInstance};
use crate::substrate::{
    ElectricalOccurrence, NetAnchorReason, NetCorrespondenceStatus, NetRelationshipIntent,
    ProjectResolver, electrical_test_partitions, net_correspondence_status,
};
use std::collections::BTreeSet;
use uuid::Uuid;

fn certified(f: &mut BindingFixture) {
    let write = build_adopt_pin_correspondence(
        &f.model,
        provenance(),
        f.relationship.clone(),
        &[PlacedPinAdoption {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            placed_pin_id: f.symbol.pins[0].uuid,
            library_pin_id: f.library_pin,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.relationship = f.model.electrical_identities[&f.relationship.id].clone();
    assert_eq!(
        net_correspondence_status(&f.model, f.logical),
        NetCorrespondenceStatus::Complete
    );
}
fn refused(f: &mut BindingFixture, write: PreparedWrite) {
    let before = f.model.clone();
    let bytes = super::binding_fixture::authored_bytes(&f.root, &f.model);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let source = f
        .model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicSheet)
        .unwrap();
    assert!(commit_prepared(&mut f.model, &f.root, write).is_err());
    assert_eq!(f.model, before);
    assert_eq!(
        super::binding_fixture::authored_bytes(&f.root, &f.model),
        bytes
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
        journal
    );
    assert_eq!(
        f.model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicSheet)
            .unwrap(),
        source
    );
    persisted(f);
}
fn persisted(f: &BindingFixture) {
    let reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        reopened.electrical_identities,
        f.model.electrical_identities
    );
    assert_eq!(
        electrical_test_partitions(&reopened, &[]).unwrap(),
        electrical_test_partitions(&f.model, &[]).unwrap()
    );
}
fn member(id: Uuid, path: &[Uuid]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: "pins".into(),
        source_id: id,
        instance_path: path.to_vec(),
    }
}
fn id_at(model: &DesignModel, member: &ElectricalOccurrence) -> Uuid {
    let groups = electrical_test_partitions(model, &[]).unwrap();
    let group = groups.iter().find(|g| g.contains(member)).unwrap();
    model
        .electrical_identities
        .values()
        .find_map(|r| match &r.identity {
            ElectricalIdentity::Net {
                anchor,
                retired: false,
                ..
            } if group.contains(anchor) => Some(r.id),
            _ => None,
        })
        .unwrap()
}

#[test]
fn wrong_logical_net_refuses_create_and_update_and_never_reports_complete() {
    let mut f = fixture("e1_ir02_wrong_logical");
    certified(&mut f);
    let remote = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(100, 0),
        to: Point::new(110, 0),
    };
    let write = build_create_schematic_wire(&f.model, provenance(), f.sheet, &remote).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let unrelated = f
        .model
        .electrical_identities
        .values()
        .find_map(|r| match &r.identity {
            ElectricalIdentity::Net {
                anchor,
                retired: false,
                ..
            } if anchor.source_id == remote.uuid => Some(r.id),
            _ => None,
        })
        .unwrap();
    let mut wrong = f.model.electrical_identities[&f.relationship.id].clone();
    if let ElectricalIdentity::NetRelationship { logical_net, .. } = &mut wrong.identity {
        *logical_net = unrelated;
    }
    let write = build_set_electrical_identity(&f.model, provenance(), wrong.clone()).unwrap();
    refused(&mut f, write);
    let mut new = wrong.clone();
    new.id = Uuid::new_v4();
    let write = build_create_electrical_identity(&f.model, provenance(), new).unwrap();
    refused(&mut f, write);
    // Defensive read oracle: even an invalid in-memory record cannot certify.
    let mut invalid = f.model.clone();
    invalid
        .electrical_identities
        .insert(wrong.id, wrong.clone());
    assert_eq!(
        net_correspondence_status(&invalid, unrelated),
        NetCorrespondenceStatus::Mismatch
    );
    assert_eq!(
        net_correspondence_status(&f.model, f.logical),
        NetCorrespondenceStatus::Complete
    );
    // Contradictory source evidence remains a lawful explicitly incomplete state.
    if let ElectricalIdentity::NetRelationship { intent, .. } = &mut wrong.identity {
        *intent = NetRelationshipIntent::Pending;
    }
    let before = f.model.electrical_identities.clone();
    let write = build_set_electrical_identity(&f.model, provenance(), wrong).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    assert_eq!(
        net_correspondence_status(&f.model, unrelated),
        NetCorrespondenceStatus::Pending
    );
    persisted(&f);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(f.model.electrical_identities, before);
    persisted(&f);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&f.root).resolve().unwrap(), unrelated),
        NetCorrespondenceStatus::Pending
    );
}

#[test]
fn same_reusable_pin_in_another_sheet_occurrence_cannot_certify_the_logical_net() {
    let mut f = fixture("e1_ir02_occurrence");
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: f.sheet,
        name: "REUSED".into(),
    };
    let write = build_create_schematic_definition(
        &f.model,
        provenance(),
        f.schematic,
        definition.uuid,
        "definitions/d.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut paths = vec![];
    for _ in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            name: "same display".into(),
            position: Point::new(0, 0),
            ports: vec![],
        };
        let write = build_create_schematic_sheet_instance(
            &f.model,
            provenance(),
            f.schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        paths.push(instance.uuid);
    }
    let pin = f.symbol.pins[0].uuid;
    let mut expected = vec![
        BTreeSet::from([member(pin, &[paths[0]])]),
        BTreeSet::from([member(pin, &[paths[1]])]),
    ];
    expected.sort();
    assert_eq!(electrical_test_partitions(&f.model, &[]).unwrap(), expected);
    let ids = [
        id_at(&f.model, &member(pin, &[paths[0]])),
        id_at(&f.model, &member(pin, &[paths[1]])),
    ];
    assert_ne!(ids[0], ids[1]);
    f.logical = ids[0];
    if let ElectricalIdentity::NetRelationship {
        logical_net,
        evidence,
        ..
    } = &mut f.relationship.identity
    {
        *logical_net = ids[0];
        evidence[0].schematic_terminal.instance_path = vec![paths[0]];
    }
    certified(&mut f);
    persisted(&f);
    let before = f.model.electrical_identities.clone();
    for change_terminal in [false, true] {
        let mut wrong = f.relationship.clone();
        if let ElectricalIdentity::NetRelationship {
            logical_net,
            evidence,
            ..
        } = &mut wrong.identity
        {
            if change_terminal {
                evidence[0].schematic_terminal.instance_path = vec![paths[1]];
            } else {
                *logical_net = ids[1];
            }
        }
        let mut invalid = f.model.clone();
        invalid
            .electrical_identities
            .insert(wrong.id, wrong.clone());
        assert_eq!(
            net_correspondence_status(&invalid, if change_terminal { ids[0] } else { ids[1] }),
            NetCorrespondenceStatus::Mismatch
        );
        let write = build_set_electrical_identity(&f.model, provenance(), wrong).unwrap();
        refused(&mut f, write);
    }
    let write =
        build_set_electrical_identity(&f.model, provenance(), f.relationship.clone()).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    persisted(&f);
    assert_eq!(
        net_correspondence_status(&f.model, ids[0]),
        NetCorrespondenceStatus::Complete
    );
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(f.model.electrical_identities, before);
    persisted(&f);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&f.root).resolve().unwrap(), ids[0]),
        NetCorrespondenceStatus::Complete
    );
}

#[test]
fn final_split_membership_controls_explicit_binding_and_refuses_the_entire_topology_batch() {
    let mut f = fixture("e1_ir02_final_split");
    certified(&mut f);
    // Explicit fresh allocation in the same canonical topology batch makes the
    // remote anchor the known lowest nominee; no existing identity is replaced.
    let remote = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(100, 0),
        to: Point::new(110, 0),
    };
    let remote_id = Uuid::from_u128(1);
    let record = ElectricalIdentityRecord {
        id: remote_id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Net {
            anchor: ElectricalOccurrence {
                class: "wires".into(),
                source_id: remote.uuid,
                instance_path: vec![],
            },
            anchor_reason: NetAnchorReason::UnclaimedFinalGroup,
            retired: false,
            predecessors: BTreeSet::new(),
        },
    };
    let write = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::CreateSchematicWire {
            sheet_id: f.sheet,
            wire_id: remote.uuid,
            wire: serde_json::to_value(&remote).unwrap(),
        })
        .push_op(Operation::CreateElectricalIdentity { record })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let join = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(100, 0),
    };
    let write = build_create_schematic_wire(&f.model, provenance(), f.sheet, &join).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut binding = f.model.electrical_identities[&f.relationship.id].clone();
    if let ElectricalIdentity::NetRelationship {
        logical_net,
        intent,
        ..
    } = &mut binding.identity
    {
        assert_eq!(*logical_net, remote_id);
        *intent = NetRelationshipIntent::Implemented;
    }
    let write = build_set_electrical_identity(&f.model, provenance(), binding).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    assert_eq!(
        net_correspondence_status(&f.model, remote_id),
        NetCorrespondenceStatus::Complete
    );
    let deletion = build_delete_schematic_wire(&f.model, provenance(), f.sheet, &join).unwrap();
    let binding = build_set_electrical_identity(
        &f.model,
        provenance(),
        f.model.electrical_identities[&f.relationship.id].clone(),
    )
    .unwrap();
    let write = BatchComposer::compose(&f.model, provenance())
        .push_ops(deletion.batch.operations)
        .push_ops(binding.batch.operations)
        .finish()
        .unwrap();
    // Pre-batch evidence is valid; final pin partition is distinct from retained
    // remote anchor. Validation must not certify against the old graph.
    refused(&mut f, write);
    let write = build_delete_schematic_wire(&f.model, provenance(), f.sheet, &join).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    assert_eq!(
        net_correspondence_status(&f.model, remote_id),
        NetCorrespondenceStatus::Pending
    );
    persisted(&f);
    let correct = id_at(&f.model, &member(f.symbol.pins[0].uuid, &[]));
    assert_ne!(correct, remote_id);
    let mut binding = f.model.electrical_identities[&f.relationship.id].clone();
    if let ElectricalIdentity::NetRelationship {
        logical_net,
        intent,
        ..
    } = &mut binding.identity
    {
        *logical_net = correct;
        *intent = NetRelationshipIntent::Implemented;
    }
    let before = f.model.electrical_identities.clone();
    let write = build_set_electrical_identity(&f.model, provenance(), binding).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    assert_eq!(
        net_correspondence_status(&f.model, correct),
        NetCorrespondenceStatus::Complete
    );
    persisted(&f);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(f.model.electrical_identities, before);
    persisted(&f);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&f.root).resolve().unwrap(), correct),
        NetCorrespondenceStatus::Complete
    );
}
