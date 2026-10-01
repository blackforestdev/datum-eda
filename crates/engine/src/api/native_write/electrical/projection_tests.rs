use super::super::{
    commit_prepared,
    schematic_connectivity::{build_create_schematic_bus, build_delete_schematic_bus},
};
use super::tests::{create, model_with_wire, net, provenance};
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::Bus;
use crate::substrate::{ElectricalOccurrence, ProjectResolver};
use std::collections::BTreeSet;
use uuid::Uuid;

#[test]
fn deleting_last_bus_drawing_keeps_declaration_and_replays_exact_projection() {
    let (root, mut model, anchor) = model_with_wire("e1_last_bus_projection");
    let sheet = Uuid::new_v5(&model.project.project_id, b"sheet");
    let drawing = Bus {
        uuid: Uuid::new_v4(),
        name: "DRAWING".into(),
        members: vec!["N".into()],
        segments: vec![Point::new(0, 0), Point::new(100, 0)],
    };
    let write = build_create_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let n = Uuid::new_v4();
    create(&mut model, &root, net(n, anchor));
    let id = Uuid::new_v4();
    let declaration = ElectricalIdentityRecord {
        id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "SEMANTIC".into(),
            scalar_nets: BTreeSet::from([n]),
            representations: BTreeSet::from([ElectricalOccurrence {
                source_id: drawing.uuid,
                class: "buses".into(),
                instance_path: vec![],
            }]),
            retired: false,
        },
    };
    create(&mut model, &root, declaration.clone());
    let write = build_delete_schematic_bus(&model, provenance(), sheet, &drawing).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        matches!(&model.electrical_identities[&id].identity, ElectricalIdentity::Bus { representations, scalar_nets, retired: false, name } if representations.is_empty() && scalar_nets == &BTreeSet::from([n]) && name == "SEMANTIC")
    );
    assert!(!model.objects.contains_key(&drawing.uuid));
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
    assert_eq!(model.electrical_identities[&id], declaration);
    assert!(model.objects.contains_key(&drawing.uuid));
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
