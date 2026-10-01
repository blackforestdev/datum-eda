//! Final-state identity composition and exact complete membership, never caps.
use super::*;
use crate::substrate::BusInterfaceEquivalence;

#[test]
fn native_merge_rebinds_bus_and_refuses_forged_interface_atomically() {
    let (root, mut model, _, sheet) = fixture("e1_auto_bus_binding");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let b = wire(&mut model, &root, sheet, 20, 30);
    adopt(&mut model, &root);
    let ids = BTreeSet::from([
        id_at(&model, &member("wires", a.uuid, &[])),
        id_at(&model, &member("wires", b.uuid, &[])),
    ]);
    let mut projections = vec![];
    for _ in 0..2 {
        let drawing = crate::schematic::Bus {
            uuid: Uuid::new_v4(),
            name: "drawing".into(),
            members: vec![],
            segments: vec![Point::new(0, 100), Point::new(10, 100)],
        };
        let write = build_create_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        projections.push(member("buses", drawing.uuid, &[]));
    }
    let bus = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "declaration".into(),
            scalar_nets: ids.clone(),
            representations: projections.iter().cloned().collect(),
            retired: false,
        },
    };
    super::super::tests::create(&mut model, &root, bus.clone());
    let interface = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::BusInterface {
            left_bus: bus.id,
            right_bus: bus.id,
            left_occurrence: projections[0].clone(),
            right_occurrence: projections[1].clone(),
            equivalence: BusInterfaceEquivalence::Identity,
            scalar_mapping: ids.iter().map(|id| (*id, *id)).collect(),
        },
    };
    super::super::tests::create(&mut model, &root, interface.clone());
    let bridge = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(10, 0),
        to: Point::new(20, 0),
    };
    let op = Operation::CreateSchematicWire {
        sheet_id: sheet,
        wire_id: bridge.uuid,
        wire: serde_json::to_value(&bridge).unwrap(),
    };
    let mut invalid = interface.clone();
    invalid.object_revision = ObjectRevision(1);
    if let ElectricalIdentity::BusInterface { scalar_mapping, .. } = &mut invalid.identity {
        scalar_mapping.clear();
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_op(op.clone())
        .push_op(Operation::SetElectricalIdentity {
            previous: Box::new(interface.clone()),
            record: invalid,
        })
        .finish()
        .unwrap();
    let before = model.clone();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(model, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let write = BatchComposer::compose(&model, provenance())
        .push_op(op)
        .finish()
        .unwrap();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    let winner = *ids.first().unwrap();
    assert!(
        matches!(&model.electrical_identities[&bus.id].identity, ElectricalIdentity::Bus { scalar_nets, .. } if scalar_nets == &BTreeSet::from([winner]))
    );
    assert!(
        matches!(&model.electrical_identities[&interface.id].identity, ElectricalIdentity::BusInterface { scalar_mapping, .. } if scalar_mapping == &[(winner,winner)].into_iter().collect())
    );
    for id in [bus.id, interface.id] {
        assert!(report.transaction.operations.iter().any(
            |op| matches!(op, Operation::SetElectricalIdentity {record,..} if record.id == id)
        ));
    }
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    persisted(&root, &model);
}

#[test]
fn complete_deletion_retires_identity_and_next_authored_source_allocates_fresh() {
    let (root, mut model, _, sheet) = fixture("e1_all_members_deleted");
    let a = wire(&mut model, &root, sheet, 0, 10);
    adopt(&mut model, &root);
    let old = id_at(&model, &member("wires", a.uuid, &[]));
    let relation = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: old,
            board_net: None,
            intent: NetRelationshipIntent::SchematicOnly,
            evidence: vec![],
        },
    };
    super::super::tests::create(&mut model, &root, relation.clone());
    let before = model.clone();
    let write = build_delete_schematic_wire(&model, provenance(), sheet, &a).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(electrical_test_partitions(&model, &[]).unwrap().is_empty());
    assert!(matches!(
        model.electrical_identities[&old].identity,
        ElectricalIdentity::Net { retired: true, .. }
    ));
    assert!(
        matches!(model.electrical_identities[&relation.id].identity, ElectricalIdentity::NetRelationship { logical_net, intent: NetRelationshipIntent::Pending, .. } if logical_net == old)
    );
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let b = wire(&mut model, &root, sheet, 0, 10);
    assert_ne!(id_at(&model, &member("wires", b.uuid, &[])), old);
    persisted(&root, &model);
}

#[test]
fn complete_partition_exceeds_context_cap_and_reads_do_not_author_identity() {
    let (root, mut model, _, sheet) = fixture("e1_complete_257");
    let wires: Vec<_> = (0..257)
        .map(|i| SchematicWire {
            uuid: Uuid::new_v4(),
            from: Point::new(i * 10, 0),
            to: Point::new((i + 1) * 10, 0),
        })
        .collect();
    let mut builder = BatchComposer::compose(&model, provenance());
    for w in &wires {
        builder = builder.push_op(Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: w.uuid,
            wire: serde_json::to_value(w).unwrap(),
        });
    }
    let write = builder.finish().unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let expected = BTreeSet::from_iter(wires.iter().map(|w| member("wires", w.uuid, &[])));
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        vec![expected.clone()]
    );
    adopt(&mut model, &root);
    let before = model.clone();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    for _ in 0..3 {
        let reopened = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            electrical_test_partitions(&reopened, &[]).unwrap(),
            vec![expected.clone()]
        );
        assert_eq!(reopened.electrical_identities, model.electrical_identities);
    }
    assert_eq!(model, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    // Recognition must not materialize billions of strings from display syntax.
    let range = NetLabel {
        uuid: Uuid::new_v4(),
        name: "D[-2147483648..2147483647]".into(),
        kind: LabelKind::Local,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &range).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        vec![expected]
    );
    assert_eq!(model.electrical_identities, before.electrical_identities);
}

#[test]
fn simultaneous_split_merge_uses_final_nominees_and_equivalent_operation_order() {
    let (root, mut model, _, sheet) = fixture("e1_atomic_split_merge");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let bridge = wire(&mut model, &root, sheet, 10, 20);
    let b = wire(&mut model, &root, sheet, 20, 30);
    let c = wire(&mut model, &root, sheet, 40, 50);
    let first = Uuid::from_u128(1000);
    let second = Uuid::from_u128(2000);
    super::super::tests::create(
        &mut model,
        &root,
        super::super::tests::net(first, member("wires", b.uuid, &[])),
    );
    super::super::tests::create(
        &mut model,
        &root,
        super::super::tests::net(second, member("wires", c.uuid, &[])),
    );
    let join = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(30, 0),
        to: Point::new(40, 0),
    };
    let deletion = Operation::DeleteSchematicWire {
        sheet_id: sheet,
        wire_id: bridge.uuid,
        wire: serde_json::to_value(&bridge).unwrap(),
    };
    let creation = Operation::CreateSchematicWire {
        sheet_id: sheet,
        wire_id: join.uuid,
        wire: serde_json::to_value(&join).unwrap(),
    };
    let write = BatchComposer::compose(&model, provenance())
        .push_op(deletion.clone())
        .push_op(creation.clone())
        .finish()
        .unwrap();
    let mut reverse = BatchComposer::compose(&model, provenance())
        .push_op(creation)
        .push_op(deletion)
        .finish()
        .unwrap();
    reverse.batch.batch_id = write.batch.batch_id;
    let new_id = Uuid::new_v5(&write.batch.batch_id, b"datum:net-allocation:0");
    let mut one = model.clone();
    let mut two = model.clone();
    one.commit(write.batch.clone()).unwrap();
    two.commit(reverse.batch).unwrap();
    assert_eq!(one.electrical_identities, two.electrical_identities);
    let before = model.clone();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(model.electrical_identities, one.electrical_identities);
    let mut expected = vec![
        BTreeSet::from([member("wires", a.uuid, &[])]),
        BTreeSet::from([
            member("wires", b.uuid, &[]),
            member("wires", c.uuid, &[]),
            member("wires", join.uuid, &[]),
        ]),
    ];
    expected.sort();
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), expected);
    assert_eq!(id_at(&model, &member("wires", a.uuid, &[])), new_id);
    assert_eq!(id_at(&model, &member("wires", b.uuid, &[])), first);
    assert!(matches!(
        model.electrical_identities[&second].identity,
        ElectricalIdentity::Net { retired: true, .. }
    ));
    assert_eq!(
        report
            .transaction
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::CreateElectricalIdentity { .. }))
            .count(),
        1
    );
    let after = model.electrical_identities.clone();
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, after);
    persisted(&root, &model);
}

#[test]
fn proposal_preview_and_accepted_apply_share_recorded_transition_and_postimage_revision() {
    use crate::substrate::{
        ProposalCreateRequest, ProposalSource, ProposalStatus, apply_accepted_proposal,
        create_draft_proposal_from_batch, preview_proposal_diff_journaled, review_proposal_status,
    };
    let (root, mut model, _, sheet) = fixture("e1_proposal_topology");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let bridge = wire(&mut model, &root, sheet, 10, 20);
    let b = wire(&mut model, &root, sheet, 20, 30);
    adopt(&mut model, &root);
    let before = model.electrical_identities.clone();
    let write = build_delete_schematic_wire(&model, provenance(), sheet, &bridge).unwrap();
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: write.batch,
            rationale: "native split proof".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    assert_eq!(model.electrical_identities, before);
    assert!(
        proposal
            .batch
            .operations
            .iter()
            .any(|op| matches!(op, Operation::CreateElectricalIdentity { .. }))
    );
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let preview = preview_proposal_diff_journaled(&model, &root, proposal.proposal_id).unwrap();
    assert_eq!(model.electrical_identities, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    review_proposal_status(
        &mut model,
        &root,
        proposal.proposal_id,
        ProposalStatus::Accepted,
    )
    .unwrap();
    let report = apply_accepted_proposal(&mut model, &root, proposal.proposal_id).unwrap();
    assert_eq!(
        report.transaction.after_model_revision,
        preview.preview_after_model_revision
    );
    assert_ne!(
        id_at(&model, &member("wires", a.uuid, &[])),
        id_at(&model, &member("wires", b.uuid, &[]))
    );
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    persisted(&root, &model);
}

#[test]
fn crossed_native_final_groups_preserve_both_anchor_identities_without_allocation() {
    let (root, mut model, _, sheet) = fixture("e1_crossed_native");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let b = wire(&mut model, &root, sheet, 10, 20);
    let c = wire(&mut model, &root, sheet, 100, 110);
    let d = wire(&mut model, &root, sheet, 110, 120);
    let ids = [Uuid::from_u128(1000), Uuid::from_u128(2000)];
    super::super::tests::create(
        &mut model,
        &root,
        super::super::tests::net(ids[0], member("wires", a.uuid, &[])),
    );
    super::super::tests::create(
        &mut model,
        &root,
        super::super::tests::net(ids[1], member("wires", c.uuid, &[])),
    );
    let before = model.electrical_identities.clone();
    let mut builder = BatchComposer::compose(&model, provenance());
    for (w, start, end) in [(&b, 110, 120), (&d, 10, 20)] {
        let moved = SchematicWire {
            from: Point::new(start, 0),
            to: Point::new(end, 0),
            ..w.clone()
        };
        builder = builder
            .push_op(Operation::DeleteSchematicWire {
                sheet_id: sheet,
                wire_id: w.uuid,
                wire: serde_json::to_value(w).unwrap(),
            })
            .push_op(Operation::CreateSchematicWire {
                sheet_id: sheet,
                wire_id: w.uuid,
                wire: serde_json::to_value(moved).unwrap(),
            });
    }
    let write = builder.finish().unwrap();
    let report = commit_prepared(&mut model, &root, write).unwrap();
    let mut expected = vec![
        BTreeSet::from([member("wires", a.uuid, &[]), member("wires", d.uuid, &[])]),
        BTreeSet::from([member("wires", c.uuid, &[]), member("wires", b.uuid, &[])]),
    ];
    expected.sort();
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), expected);
    assert_eq!(id_at(&model, &member("wires", a.uuid, &[])), ids[0]);
    assert_eq!(id_at(&model, &member("wires", c.uuid, &[])), ids[1]);
    assert!(
        !report
            .transaction
            .operations
            .iter()
            .any(|op| matches!(op, Operation::CreateElectricalIdentity { .. }))
    );
    let after = model.electrical_identities.clone();
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, after);
    persisted(&root, &model);
}
