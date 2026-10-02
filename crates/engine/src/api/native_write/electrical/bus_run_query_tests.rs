//! Exact native oracles for PM055, independent of graph traversal order.
use super::*;
use crate::schematic::{Bus, BusEntry};
use crate::substrate::ModelRevision;

fn spine(x: i64, y: i64, end_x: i64, end_y: i64) -> Bus {
    Bus {
        uuid: Uuid::new_v4(),
        name: "DATA".into(),
        members: vec!["DATA0".into()],
        segments: vec![Point::new(x, y), Point::new(end_x, end_y)],
    }
}
fn put(model: &mut DesignModel, root: &std::path::Path, sheet: Uuid, buses: &[Bus]) {
    let prepared = BatchComposer::compose(model, provenance())
        .push_ops(
            buses
                .iter()
                .map(|b| Operation::CreateSchematicBus {
                    sheet_id: sheet,
                    bus_id: b.uuid,
                    bus: serde_json::to_value(b).unwrap(),
                })
                .collect::<Vec<_>>(),
        )
        .finish()
        .unwrap();
    commit_prepared(model, root, prepared).unwrap();
}
fn declare(
    model: &mut DesignModel,
    root: &std::path::Path,
    reps: BTreeSet<ElectricalOccurrence>,
    scalar_nets: BTreeSet<Uuid>,
) -> Uuid {
    let id = Uuid::new_v4();
    create(
        model,
        root,
        ElectricalIdentityRecord {
            id,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Bus {
                name: "DATA".into(),
                representations: reps,
                scalar_nets,
                retired: false,
            },
        },
    );
    id
}

#[test]
fn bus_run_foreign_contact_never_equates_equal_names_or_scalar_members() {
    let (root, mut model, _, sheet) = fixture("bus_run_foreign");
    let wire = super::super::topology_tests::wire(&mut model, &root, sheet, 0, 10);
    let scalar = Uuid::new_v4();
    create(
        &mut model,
        &root,
        net(scalar, member("wires", wire.uuid, &[])),
    );
    let a = spine(0, 0, 10, 0);
    let b = spine(10, 0, 20, 0);
    let remote = spine(50, 0, 60, 0);
    let foreign = spine(10, 0, 10, 20);
    put(
        &mut model,
        &root,
        sheet,
        &[a.clone(), b.clone(), remote.clone(), foreign.clone()],
    );
    let entry = BusEntry {
        uuid: Uuid::new_v4(),
        bus: b.uuid,
        wire: Some(wire.uuid),
        position: Point::new(15, 0),
        size: Point::new(1, 1),
    };
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "DATA[0..1]".into(),
        kind: LabelKind::Local,
        position: Point::new(5, 0),
    };
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(vec![
            Operation::CreateSchematicBusEntry {
                sheet_id: sheet,
                bus_entry_id: entry.uuid,
                bus_entry: serde_json::to_value(&entry).unwrap(),
            },
            Operation::CreateSchematicLabel {
                sheet_id: sheet,
                label_id: label.uuid,
                label: serde_json::to_value(&label).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let origin = member("buses", a.uuid, &[]);
    let expected = BTreeSet::from([
        origin.clone(),
        member("buses", b.uuid, &[]),
        member("bus_entries", entry.uuid, &[]),
        member("labels", label.uuid, &[]),
    ]);
    let mut complete = expected.clone();
    complete.insert(member("buses", remote.uuid, &[]));
    let declaration = declare(
        &mut model,
        &root,
        complete
            .iter()
            .filter(|o| o.class != "bus_entries")
            .cloned()
            .collect(),
        BTreeSet::from([scalar]),
    );
    let other = declare(
        &mut model,
        &root,
        BTreeSet::from([member("buses", foreign.uuid, &[])]),
        BTreeSet::from([scalar]),
    );
    let files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let run = snapshot.bus_run(snapshot.revision(), &origin).unwrap();
    assert_eq!(run.membership.subject_id, declaration);
    assert_eq!(run.membership.owned, expected);
    assert_eq!(
        snapshot
            .bus(snapshot.revision(), declaration)
            .unwrap()
            .owned,
        complete
    );
    assert_eq!(run.membership.related_subjects, BTreeSet::from([scalar]));
    assert!(
        !run.membership
            .owned
            .contains(&member("wires", wire.uuid, &[]))
    );
    assert_eq!(
        run.contacts,
        BTreeSet::from([
            crate::substrate::CrossBusContact {
                origin_bus: declaration,
                foreign_bus: other,
                origin_member: origin.clone(),
                foreign_member: member("buses", foreign.uuid, &[])
            },
            crate::substrate::CrossBusContact {
                origin_bus: declaration,
                foreign_bus: other,
                origin_member: member("buses", b.uuid, &[]),
                foreign_member: member("buses", foreign.uuid, &[])
            },
        ])
    );
    for (path, bytes) in files {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    assert!(matches!(
        snapshot.bus_run(&ModelRevision("wrong".into()), &origin),
        Err(ElectricalQueryFailure::StaleRevision)
    ));
    assert!(matches!(
        snapshot.bus_run(snapshot.revision(), &member("bus_entries", entry.uuid, &[])),
        Err(ElectricalQueryFailure::InvalidOccurrence { .. })
    ));
    let mut moved = b.clone();
    moved.segments = vec![Point::new(30, 0), Point::new(40, 0)];
    let prepared = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicBus {
            sheet_id: sheet,
            bus_id: b.uuid,
            bus: serde_json::to_value(&moved).unwrap(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let current = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        current
            .bus_run(current.revision(), &origin)
            .unwrap()
            .membership
            .owned,
        BTreeSet::from([origin.clone(), member("labels", label.uuid, &[])])
    );
    assert_eq!(snapshot.bus_run(snapshot.revision(), &origin).unwrap(), run);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let replay = ElectricalSelectionSnapshot::capture(&reopened).unwrap();
    assert_eq!(
        replay
            .bus_run(replay.revision(), &origin)
            .unwrap()
            .membership
            .owned,
        expected
    );
    assert_eq!(
        replay.bus_run(replay.revision(), &origin).unwrap().contacts,
        run.contacts
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn bus_run_complete_257_spines_and_full_integer_contacts() {
    let (root, mut model, _, sheet) = fixture("bus_run_uncapped");
    let buses: Vec<_> = (0..257)
        .map(|i| spine(i * 10, 0, (i + 1) * 10, 0))
        .collect();
    put(&mut model, &root, sheet, &buses);
    let expected = buses.iter().map(|b| member("buses", b.uuid, &[])).collect();
    let id = declare(&mut model, &root, expected, BTreeSet::new());
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let run = snapshot
        .bus_run(snapshot.revision(), &member("buses", buses[0].uuid, &[]))
        .unwrap();
    assert_eq!(
        run.membership.owned,
        buses.iter().map(|b| member("buses", b.uuid, &[])).collect()
    );
    assert_eq!(run.membership.subject_id, id);
    assert!(run.contacts.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn bus_run_requires_unique_declaration_and_available_spine() {
    let (root, mut model, _, sheet) = fixture("bus_run_unavailable");
    let a = spine(0, 0, 10, 0);
    put(&mut model, &root, sheet, std::slice::from_ref(&a));
    let origin = member("buses", a.uuid, &[]);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(matches!(
        snapshot.bus_run(snapshot.revision(), &origin),
        Err(ElectricalQueryFailure::UnavailableAssignment { .. })
    ));
    declare(
        &mut model,
        &root,
        BTreeSet::from([origin.clone()]),
        BTreeSet::new(),
    );
    let mut no_geometry = a.clone();
    no_geometry.segments.clear();
    let prepared = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicBus {
            sheet_id: sheet,
            bus_id: a.uuid,
            bus: serde_json::to_value(no_geometry).unwrap(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(matches!(
        snapshot.bus_run(snapshot.revision(), &origin),
        Err(ElectricalQueryFailure::UnavailableGeometry { .. })
    ));
    assert!(matches!(
        snapshot.bus_run(
            snapshot.revision(),
            &member("buses", a.uuid, &[Uuid::new_v4()])
        ),
        Err(ElectricalQueryFailure::InvalidOccurrence { .. })
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn bus_run_repeated_sheet_paths_and_interfaces_do_not_join_physical_components() {
    let (root, mut model, schematic, sheet) = fixture("bus_run_occurrences");
    let drawing = spine(0, 0, 10, 0);
    put(&mut model, &root, sheet, std::slice::from_ref(&drawing));
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: sheet,
        name: "same".into(),
    };
    let prepared = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/bus.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut paths = vec![];
    for _ in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            name: "same".into(),
            position: Point::zero(),
            ports: vec![],
        };
        let prepared = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        paths.push(instance.uuid);
    }
    let a = member("buses", drawing.uuid, &[paths[0]]);
    let b = member("buses", drawing.uuid, &[paths[1]]);
    let left = declare(
        &mut model,
        &root,
        BTreeSet::from([a.clone()]),
        BTreeSet::new(),
    );
    let right = declare(
        &mut model,
        &root,
        BTreeSet::from([b.clone()]),
        BTreeSet::new(),
    );
    create(
        &mut model,
        &root,
        ElectricalIdentityRecord {
            id: Uuid::new_v4(),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::BusInterface {
                left_bus: left,
                right_bus: right,
                left_occurrence: a.clone(),
                right_occurrence: b.clone(),
                equivalence: crate::substrate::BusInterfaceEquivalence::Related,
                scalar_mapping: BTreeSet::new(),
            },
        },
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    for (origin, id, other, related) in [(&a, left, right, &b), (&b, right, left, &a)] {
        let run = snapshot.bus_run(snapshot.revision(), origin).unwrap();
        assert_eq!(run.membership.subject_id, id);
        assert_eq!(run.membership.owned, BTreeSet::from([origin.clone()]));
        assert_eq!(run.membership.related_subjects, BTreeSet::from([other]));
        assert_eq!(run.membership.related, BTreeSet::from([related.clone()]));
        assert!(run.contacts.is_empty());
    }
    assert!(matches!(
        snapshot.bus_run(snapshot.revision(), &member("buses", drawing.uuid, &[])),
        Err(ElectricalQueryFailure::InvalidOccurrence { .. })
    ));
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let captured = ElectricalSelectionSnapshot::capture(&reopened).unwrap();
    assert_eq!(
        captured.bus_run(captured.revision(), &a).unwrap(),
        snapshot.bus_run(snapshot.revision(), &a).unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}
