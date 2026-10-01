use super::super::commit_prepared;
use super::super::schematic_connectivity::{
    build_create_schematic_bus, build_delete_schematic_bus,
};
use super::super::schematic_sheets::{
    build_create_schematic_definition, build_create_schematic_sheet_instance,
};
use super::tests::{create, model_with_wire, net, provenance};
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::{SheetDefinition, SheetInstance};
use crate::substrate::{BusInterfaceEquivalence, ElectricalOccurrence, ProjectResolver};
use std::collections::BTreeSet;
use uuid::Uuid;

#[test]
fn occurrence_qualified_bus_identity_and_interface_reopen() {
    let (root, mut model, anchor) = model_with_wire("electrical_hierarchy");
    let sheet = Uuid::new_v5(&model.project.project_id, b"sheet");
    let schematic = Uuid::new_v5(&model.project.project_id, b"schematic");
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: sheet,
        name: "Reusable".into(),
    };
    let prepared = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/reusable.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut paths = vec![];
    for name in ["A", "B"] {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: Point::new(0, 0),
            name: name.into(),
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
    let drawing = crate::schematic::Bus {
        uuid: Uuid::new_v4(),
        name: "DATA".into(),
        members: vec!["N".into()],
        segments: vec![Point::new(0, 0), Point::new(10, 10)],
    };
    let prepared = build_create_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let a = ElectricalOccurrence {
        class: "buses".into(),
        source_id: drawing.uuid,
        instance_path: vec![paths[0]],
    };
    let b = ElectricalOccurrence {
        instance_path: vec![paths[1]],
        ..a.clone()
    };
    let id = Uuid::new_v4();
    create(
        &mut model,
        &root,
        ElectricalIdentityRecord {
            id,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Bus {
                name: "DECLARED".into(),
                scalar_nets: BTreeSet::from([n]),
                representations: BTreeSet::from([a.clone(), b.clone()]),
                retired: false,
            },
        },
    );
    let interface = Uuid::new_v4();
    create(
        &mut model,
        &root,
        ElectricalIdentityRecord {
            id: interface,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::BusInterface {
                left_bus: id,
                right_bus: id,
                left_occurrence: a.clone(),
                right_occurrence: b,
                equivalence: BusInterfaceEquivalence::Identity,
                scalar_mapping: BTreeSet::from([(n, n)]),
            },
        },
    );
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities,
        model.electrical_identities
    );
    let duplicate = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "OTHER".into(),
            scalar_nets: BTreeSet::from([n]),
            representations: BTreeSet::from([a]),
            retired: false,
        },
    };
    let before = model.clone();
    let prepared = build_create_electrical_identity(&model, provenance(), duplicate).unwrap();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    assert_eq!(model, before);
    let invalid = net(
        Uuid::new_v4(),
        ElectricalOccurrence {
            instance_path: vec![paths[0], paths[0]],
            ..anchor_from(&model, n)
        },
    );
    let prepared = build_create_electrical_identity(&model, provenance(), invalid).unwrap();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    // Deleting a reused source removes both projections but must not silently
    // remove its explicit semantic interface. Refuse until the caller does so.
    let before = model.clone();
    let deletion = build_delete_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
    assert!(commit_prepared(&mut model, &root, deletion).is_err());
    assert_eq!(model, before);
    let deletion = build_delete_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
    let interface_deletion =
        build_delete_electrical_identity(&model, provenance(), interface).unwrap();
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(deletion.batch.operations)
        .push_ops(interface_deletion.batch.operations)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        matches!(&model.electrical_identities[&id].identity, ElectricalIdentity::Bus { representations, retired: false, .. } if representations.is_empty())
    );
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
fn anchor_from(model: &DesignModel, id: Uuid) -> ElectricalOccurrence {
    let ElectricalIdentity::Net { anchor, .. } = &model.electrical_identities[&id].identity else {
        panic!()
    };
    anchor.clone()
}
