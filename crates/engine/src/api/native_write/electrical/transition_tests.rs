//! Canonical transition persistence with a supplied complete fixture partition.
//! This is not proof of the pending E3 topology producer.
use super::super::{commit_prepared, schematic_connectivity::build_create_schematic_wire};
use super::tests::{create, model_with_wire, net, provenance};
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::SchematicWire;
use crate::substrate::{
    ElectricalOccurrence, NetAnchorReason, NetRelationshipIntent, PreviousNetGroup,
    ProjectResolver, plan_net_identity_transition,
};
use std::collections::BTreeSet;
use uuid::Uuid;

#[test]
fn anchor_loss_merge_and_binding_are_one_atomic_replayable_write() {
    let (root, mut model, anchor) = model_with_wire("e1_atomic_transition");
    let sheet = Uuid::new_v5(&model.project.project_id, b"sheet");
    let mut members = vec![];
    for (from, to) in [(10_000_000, 20_000_000), (30_000_000, 40_000_000)] {
        let wire = SchematicWire {
            uuid: Uuid::new_v4(),
            from: Point::new(from, 0),
            to: Point::new(to, 0),
        };
        let write = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        members.push(ElectricalOccurrence {
            class: "wires".into(),
            source_id: wire.uuid,
            instance_path: vec![],
        });
    }
    let a = Uuid::from_u128(100);
    let b = Uuid::from_u128(200);
    create(&mut model, &root, net(a, anchor.clone()));
    create(&mut model, &root, net(b, members[1].clone()));
    let relation_id = Uuid::new_v4();
    let relation = ElectricalIdentityRecord {
        id: relation_id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: b,
            board_net: None,
            intent: NetRelationshipIntent::Pending,
            evidence: vec![],
        },
    };
    create(&mut model, &root, relation.clone());
    let bridge = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(20_000_000, 0),
        to: Point::new(30_000_000, 0),
    };
    let bridge_member = ElectricalOccurrence {
        class: "wires".into(),
        source_id: bridge.uuid,
        instance_path: vec![],
    };
    let transition = plan_net_identity_transition(
        model.model_revision.clone(),
        &[
            PreviousNetGroup {
                net_id: a,
                anchor: anchor.clone(),
                members: BTreeSet::from([anchor.clone(), members[0].clone()]),
            },
            PreviousNetGroup {
                net_id: b,
                anchor: members[1].clone(),
                members: BTreeSet::from([members[1].clone()]),
            },
        ],
        vec![BTreeSet::from([
            members[0].clone(),
            members[1].clone(),
            bridge_member,
        ])],
        || panic!("no intermediate ID allocation"),
    )
    .unwrap();
    let source = model
        .materialized_source_shard_value_by_relative_path("schematic/sheets/main.json")
        .unwrap();
    let topology = vec![
        Operation::DeleteSchematicWire {
            sheet_id: sheet,
            wire_id: anchor.source_id,
            wire: source["wires"][anchor.source_id.to_string()].clone(),
        },
        Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: bridge.uuid,
            wire: serde_json::to_value(bridge).unwrap(),
        },
    ];
    let before = model.clone();
    let write =
        build_net_identity_transition(&model, provenance(), transition.clone(), topology.clone())
            .unwrap();
    assert!(
        commit_prepared(&mut model, &root, write).is_err(),
        "retired binding must be reconciled"
    );
    assert_eq!(model, before);
    let mut next_relation = relation.clone();
    next_relation.object_revision = ObjectRevision(1);
    if let ElectricalIdentity::NetRelationship { logical_net, .. } = &mut next_relation.identity {
        *logical_net = a;
    }
    let mut operations = topology;
    operations.push(Operation::SetElectricalIdentity {
        previous: Box::new(relation),
        record: next_relation,
    });
    let write =
        build_net_identity_transition(&model, provenance(), transition, operations).unwrap();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        matches!(&model.electrical_identities[&a].identity, ElectricalIdentity::Net { anchor: next, anchor_reason: NetAnchorReason::DeletedAnchorFallback, .. } if next == &members[0])
    );
    assert!(matches!(
        model.electrical_identities[&b].identity,
        ElectricalIdentity::Net { retired: true, .. }
    ));
    assert!(!model.objects.contains_key(&anchor.source_id));
    assert_eq!(model.journal.len(), before.journal.len() + 1);
    assert!(report.transaction.operations.iter().any(|op| matches!(op, Operation::SetElectricalIdentity { record, .. } if record.id == relation_id)));
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities,
        model.electrical_identities
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    assert!(model.objects.contains_key(&anchor.source_id));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities,
        model.electrical_identities
    );
}
