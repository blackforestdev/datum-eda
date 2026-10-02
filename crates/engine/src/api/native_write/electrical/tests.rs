use super::super::{
    commit_prepared, schematic_connectivity::build_create_schematic_wire,
    test_support::resolved_model_with_board_package,
};
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::SchematicWire;
use crate::substrate::{
    CommitSource, ElectricalOccurrence, NetCorrespondenceStatus, NetRelationshipIntent,
    PreviousNetGroup, ProjectResolver, net_correspondence_status, plan_net_identity_transition,
};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "E1 identity proof")
}
fn occurrence(id: u128) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: "wires".into(),
        source_id: Uuid::from_u128(id),
        instance_path: vec![],
    }
}
fn set(ids: &[u128]) -> BTreeSet<ElectricalOccurrence> {
    ids.iter().map(|id| occurrence(*id)).collect()
}
fn previous(id: u128, anchor: u128, members: &[u128]) -> PreviousNetGroup {
    PreviousNetGroup {
        net_id: Uuid::from_u128(id),
        anchor: occurrence(anchor),
        members: set(members),
    }
}
pub(super) fn model_with_wire(
    name: &str,
) -> (std::path::PathBuf, DesignModel, ElectricalOccurrence) {
    let (root, mut model, _, _) = resolved_model_with_board_package(name);
    let sheet = model
        .objects
        .values()
        .find(|o| {
            o.kind == "$"
                && o.domain == "schematic"
                && model.source_shards.iter().any(|s| {
                    s.shard_id == o.source_shard_id
                        && s.kind == crate::substrate::SourceShardKind::SchematicSheet
                })
        })
        .map(|o| o.object_id)
        .unwrap_or_else(|| Uuid::new_v5(&model.project.project_id, b"sheet"));
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(10_000_000, 0),
    };
    let prepared = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    (
        root,
        model,
        ElectricalOccurrence {
            class: "wires".into(),
            source_id: wire.uuid,
            instance_path: vec![],
        },
    )
}
pub(super) fn net(id: Uuid, anchor: ElectricalOccurrence) -> ElectricalIdentityRecord {
    ElectricalIdentityRecord {
        id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Net {
            anchor,
            anchor_reason: crate::substrate::NetAnchorReason::Authored,
            retired: false,
            predecessors: BTreeSet::new(),
        },
    }
}
pub(super) fn create(
    model: &mut DesignModel,
    root: &std::path::Path,
    record: ElectricalIdentityRecord,
) {
    let prepared = build_create_electrical_identity(model, provenance(), record).unwrap();
    commit_prepared(model, root, prepared).unwrap();
}

#[test]
fn crossed_final_partition_preserves_both_anchors_without_allocating() {
    let plan = plan_net_identity_transition(
        crate::substrate::ModelRevision("r".into()),
        &[previous(100, 1, &[1, 2]), previous(200, 3, &[3, 4])],
        vec![set(&[1, 4]), set(&[2, 3])],
        || panic!("no temporary identity allocation"),
    )
    .unwrap();
    assert!(plan.retired.is_empty());
    assert_eq!(plan.final_groups[0].net_id, Uuid::from_u128(100));
    assert_eq!(plan.final_groups[1].net_id, Uuid::from_u128(200));
}
#[test]
fn anchor_loss_uses_minimum_surviving_member() {
    let plan = plan_net_identity_transition(
        crate::substrate::ModelRevision("r".into()),
        &[previous(100, 1, &[1, 2, 3])],
        vec![set(&[3]), set(&[2])],
        || Uuid::from_u128(300),
    )
    .unwrap();
    assert_eq!(plan.final_groups[0].net_id, Uuid::from_u128(100));
    assert_eq!(plan.final_groups[0].anchor, occurrence(2));
    assert_eq!(
        plan.final_groups[0].anchor_reason,
        crate::substrate::NetAnchorReason::DeletedAnchorFallback
    );
    assert_eq!(plan.final_groups[1].net_id, Uuid::from_u128(300));
}
#[test]
fn merge_keeps_lowest_nominee_and_records_retirement() {
    let plan = plan_net_identity_transition(
        crate::substrate::ModelRevision("r".into()),
        &[previous(200, 3, &[3, 4]), previous(100, 1, &[1, 2])],
        vec![set(&[1, 2, 3, 4])],
        || panic!(),
    )
    .unwrap();
    assert_eq!(plan.final_groups[0].net_id, Uuid::from_u128(100));
    assert_eq!(plan.retired, BTreeSet::from([Uuid::from_u128(200)]));
    assert_eq!(plan.final_groups[0].predecessors.len(), 2);
}

#[test]
fn mixed_split_merge_does_not_recycle_the_losing_nominee() {
    let mut allocations = 0;
    let plan = plan_net_identity_transition(
        crate::substrate::ModelRevision("r".into()),
        &[previous(100, 1, &[1, 2]), previous(200, 3, &[3, 4])],
        vec![set(&[4]), set(&[1, 2, 3])],
        || {
            allocations += 1;
            Uuid::from_u128(300)
        },
    )
    .unwrap();
    assert_eq!(allocations, 1);
    assert_eq!(plan.final_groups[0].net_id, Uuid::from_u128(100));
    assert_eq!(plan.final_groups[1].net_id, Uuid::from_u128(300));
    assert_eq!(plan.retired, BTreeSet::from([Uuid::from_u128(200)]));
}
#[test]
fn deletion_and_coordinate_replacement_does_not_reuse_identity() {
    let plan = plan_net_identity_transition(
        crate::substrate::ModelRevision("r".into()),
        &[previous(100, 1, &[1])],
        vec![set(&[2])],
        || Uuid::from_u128(300),
    )
    .unwrap();
    assert_eq!(plan.retired, BTreeSet::from([Uuid::from_u128(100)]));
    assert_eq!(plan.final_groups[0].net_id, Uuid::from_u128(300));
}
#[test]
fn duplicate_members_and_recycled_allocations_refuse() {
    let r = crate::substrate::ModelRevision("r".into());
    assert!(
        plan_net_identity_transition(r.clone(), &[], vec![set(&[1]), set(&[1])], Uuid::new_v4)
            .is_err()
    );
    assert!(
        plan_net_identity_transition(r, &[previous(100, 1, &[1])], vec![set(&[2])], || {
            Uuid::from_u128(100)
        })
        .is_err()
    );
}
#[test]
fn canonical_identity_create_set_undo_replay_reopen() {
    let (root, mut model, anchor) = model_with_wire("electrical_persistence");
    let id = Uuid::new_v4();
    create(&mut model, &root, net(id, anchor));
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.electrical_identities, model.electrical_identities);
    let changed = model.electrical_identities[&id].clone();
    // A revision-only write is lawful; fabricated transition provenance is not.
    let prepared = build_set_electrical_identity(&model, provenance(), changed.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(
        model.electrical_identities[&id].object_revision,
        ObjectRevision(1)
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        model.electrical_identities[&id].object_revision,
        ObjectRevision(0)
    );
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
    assert!(!model.journal.last().unwrap().inverse_operations.is_empty());
}
#[test]
fn malformed_anchor_refuses_without_source_or_journal_mutation() {
    let (root, mut model, _) = model_with_wire("electrical_invalid");
    let before = model.clone();
    let id = Uuid::new_v4();
    let prepared =
        build_create_electrical_identity(&model, provenance(), net(id, occurrence(123))).unwrap();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    assert_eq!(model, before);
    assert!(
        !root
            .join(format!(".datum/electrical_identities/{id}.json"))
            .exists()
    );
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().journal,
        before.journal
    );
}
#[test]
fn pending_and_mismatch_are_lawful_authored_states() {
    let (root, mut model, anchor) = model_with_wire("electrical_pending");
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let id = Uuid::new_v4();
    create(
        &mut model,
        &root,
        ElectricalIdentityRecord {
            id,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: n,
                board_net: None,
                intent: NetRelationshipIntent::Pending,
                evidence: vec![],
            },
        },
    );
    assert_eq!(
        net_correspondence_status(&model, n),
        NetCorrespondenceStatus::Pending
    );
    let mut record = model.electrical_identities[&id].clone();
    if let ElectricalIdentity::NetRelationship { intent, .. } = &mut record.identity {
        *intent = NetRelationshipIntent::Mismatch;
    }
    let prepared = build_set_electrical_identity(&model, provenance(), record).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&root).resolve().unwrap(), n),
        NetCorrespondenceStatus::Mismatch
    );
}
#[test]
fn declaration_with_zero_drawings_survives_rename_and_member_order() {
    let (root, mut model, anchor) = model_with_wire("electrical_bus");
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let id = Uuid::new_v4();
    create(
        &mut model,
        &root,
        ElectricalIdentityRecord {
            id,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Bus {
                name: "DATA".into(),
                scalar_nets: BTreeSet::from([n]),
                representations: BTreeSet::new(),
                retired: false,
            },
        },
    );
    let mut changed = model.electrical_identities[&id].clone();
    if let ElectricalIdentity::Bus { name, .. } = &mut changed.identity {
        *name = "RENAMED".into();
    }
    let prepared = build_set_electrical_identity(&model, provenance(), changed).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities
            .contains_key(&id)
    );
    let prepared = build_delete_electrical_identity(&model, provenance(), n).unwrap();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    assert!(model.electrical_identities.contains_key(&n));
}
#[test]
fn repeated_reads_allocate_nothing_and_change_no_authored_bytes() {
    let (root, mut model, anchor) = model_with_wire("electrical_readonly");
    let id = Uuid::new_v4();
    create(&mut model, &root, net(id, anchor));
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let path = root.join(format!(".datum/electrical_identities/{id}.json"));
    let bytes = std::fs::read(&path).unwrap();
    for _ in 0..3 {
        let m = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(m.electrical_identities, model.electrical_identities);
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
}
#[test]
fn stale_prepared_identity_update_refuses() {
    let (root, mut model, anchor) = model_with_wire("electrical_stale");
    let id = Uuid::new_v4();
    create(&mut model, &root, net(id, anchor));
    let record = model.electrical_identities[&id].clone();
    let one = build_set_electrical_identity(&model, provenance(), record.clone()).unwrap();
    let stale = build_set_electrical_identity(&model, provenance(), record).unwrap();
    commit_prepared(&mut model, &root, one).unwrap();
    let before = model.clone();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(model, before);
}

#[test]
fn multiple_net_identity_owners_of_one_anchor_refuse_atomically() {
    let (root, mut model, anchor) = model_with_wire("electrical_duplicate_anchor");
    let id = Uuid::new_v4();
    create(&mut model, &root, net(id, anchor.clone()));
    let before = model.clone();
    let write = build_create_electrical_identity(&model, provenance(), net(Uuid::new_v4(), anchor))
        .unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(model, before);
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities,
        before.electrical_identities
    );
}

fn bus(id: Uuid, members: BTreeSet<Uuid>) -> ElectricalIdentityRecord {
    ElectricalIdentityRecord {
        id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "DATA".into(),
            scalar_nets: members,
            representations: BTreeSet::new(),
            retired: false,
        },
    }
}
#[test]
fn explicit_bus_split_merge_records_survivor_and_replays() {
    let (root, mut model, anchor) = model_with_wire("electrical_bus_split_merge");
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    create(&mut model, &root, bus(a, BTreeSet::from([n])));
    let split = build_bus_transition(
        &model,
        provenance(),
        BTreeSet::from([a]),
        a,
        vec![bus(a, BTreeSet::from([n])), bus(b, BTreeSet::from([n]))],
        BusDistribution::default(),
        vec![],
    )
    .unwrap();
    commit_prepared(&mut model, &root, split).unwrap();
    assert!(matches!(
        model.electrical_identities[&b].identity,
        ElectricalIdentity::Bus { retired: false, .. }
    ));
    let merge = build_bus_transition(
        &model,
        provenance(),
        BTreeSet::from([a, b]),
        a,
        vec![bus(a, BTreeSet::from([n]))],
        BusDistribution::default(),
        vec![],
    )
    .unwrap();
    commit_prepared(&mut model, &root, merge).unwrap();
    assert!(matches!(
        model.electrical_identities[&b].identity,
        ElectricalIdentity::Bus { retired: true, .. }
    ));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(matches!(
        model.electrical_identities[&b].identity,
        ElectricalIdentity::Bus { retired: false, .. }
    ));
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
#[test]
fn bus_split_refuses_missing_distribution_or_reused_identity() {
    let (root, mut model, anchor) = model_with_wire("electrical_bus_refusal");
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let a = Uuid::new_v4();
    create(&mut model, &root, bus(a, BTreeSet::from([n])));
    assert!(
        build_bus_transition(
            &model,
            provenance(),
            BTreeSet::from([a]),
            a,
            vec![bus(a, BTreeSet::new())],
            BusDistribution::default(),
            vec![]
        )
        .is_err()
    );
    assert!(
        build_bus_transition(
            &model,
            provenance(),
            BTreeSet::from([a]),
            a,
            vec![bus(a, BTreeSet::from([n])), bus(n, BTreeSet::new())],
            BusDistribution::default(),
            vec![]
        )
        .is_err()
    );
    let explicit = BusDistribution {
        removed_scalars: BTreeSet::from([n]),
        ..Default::default()
    };
    let prepared = build_bus_transition(
        &model,
        provenance(),
        BTreeSet::from([a]),
        a,
        vec![bus(a, BTreeSet::new())],
        explicit,
        vec![],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
}

#[test]
fn retirement_preserves_provenance_and_only_history_can_restore() {
    let (root, mut model, anchor) = model_with_wire("electrical_retirement");
    let id = Uuid::new_v4();
    create(&mut model, &root, net(id, anchor.clone()));
    let active = model.electrical_identities[&id].clone();
    let source = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicSheet)
        .unwrap();
    let sheet: Uuid = serde_json::from_value(source["uuid"].clone()).unwrap();
    let wire: SchematicWire =
        serde_json::from_value(source["wires"][anchor.source_id.to_string()].clone()).unwrap();
    let prepared = super::super::schematic_connectivity::build_delete_schematic_wire(
        &model,
        provenance(),
        sheet,
        &wire,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(matches!(
        model.electrical_identities[&id].identity,
        ElectricalIdentity::Net { retired: true, .. }
    ));
    let revive = build_set_electrical_identity(&model, provenance(), active).unwrap();
    assert!(commit_prepared(&mut model, &root, revive).is_err());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(matches!(
        model.electrical_identities[&id].identity,
        ElectricalIdentity::Net { retired: false, .. }
    ));
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
