//! Explicit parent interfaces and occurrence copies, using canonical authoring.
use super::*;
use crate::schematic::{HierarchicalPort, PortDirection};

#[test]
fn nested_instance_interfaces_join_only_their_parent_occurrence_and_refuse_missing_binding() {
    let (root, mut model, schematic, parent) = fixture("e1_nested_interfaces");
    let child = Uuid::new_v4();
    add_sheet(&mut model, &root, schematic, child, "sheets/child.json");
    let parent_wire = wire(&mut model, &root, parent, 0, 10);
    let child_wire = wire(&mut model, &root, child, 0, 10);
    let port = HierarchicalPort {
        uuid: Uuid::new_v4(),
        name: "IFACE".into(),
        direction: PortDirection::Passive,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_port(&model, provenance(), parent, &port).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: port.name.clone(),
        kind: LabelKind::Hierarchical,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_label(&model, provenance(), child, &label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut defs = vec![];
    for (sheet, path) in [
        (parent, "definitions/parent.json"),
        (child, "definitions/child.json"),
    ] {
        let definition = SheetDefinition {
            uuid: Uuid::new_v4(),
            root_sheet: sheet,
            name: "same display".into(),
        };
        let write = build_create_schematic_definition(
            &model,
            provenance(),
            schematic,
            definition.uuid,
            path,
            serde_json::to_value(&definition).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        defs.push(definition.uuid);
    }
    let nested = SheetInstance {
        uuid: Uuid::new_v4(),
        definition: defs[1],
        parent_sheet: Some(parent),
        name: "nested".into(),
        position: Point::new(0, 0),
        ports: vec![port.uuid],
    };
    let write = build_create_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        nested.uuid,
        serde_json::to_value(&nested).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut parents = vec![];
    for _ in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: defs[0],
            parent_sheet: None,
            name: "same display".into(),
            position: Point::new(0, 0),
            ports: vec![],
        };
        let write = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        parents.push(instance.uuid);
    }
    let mut expected: Vec<_> = parents
        .iter()
        .map(|i| {
            BTreeSet::from([
                member("wires", parent_wire.uuid, &[*i]),
                member("ports", port.uuid, &[*i]),
                member("wires", child_wire.uuid, &[*i, nested.uuid]),
                member("labels", label.uuid, &[*i, nested.uuid]),
            ])
        })
        .collect();
    expected.sort();
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), expected);
    adopt(&mut model, &root);
    let ids: Vec<_> = parents
        .iter()
        .map(|i| {
            id_at(
                &model,
                &member("wires", child_wire.uuid, &[*i, nested.uuid]),
            )
        })
        .collect();
    assert_ne!(ids[0], ids[1]);
    let before = model.clone();
    let mut missing = nested.clone();
    missing.ports.clear();
    let write = build_set_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        nested.uuid,
        serde_json::to_value(&nested).unwrap(),
        serde_json::to_value(&missing).unwrap(),
    )
    .unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(model, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let copy = SheetInstance {
        uuid: Uuid::new_v4(),
        definition: defs[0],
        parent_sheet: None,
        name: "same display".into(),
        position: Point::new(0, 0),
        ports: vec![],
    };
    let write = build_create_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        copy.uuid,
        serde_json::to_value(&copy).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let new_id = id_at(
        &model,
        &member("wires", child_wire.uuid, &[copy.uuid, nested.uuid]),
    );
    assert!(!ids.contains(&new_id));
    for (i, id) in parents.iter().zip(ids) {
        assert_eq!(
            id_at(
                &model,
                &member("wires", child_wire.uuid, &[*i, nested.uuid])
            ),
            id
        );
    }
    let copied = model.electrical_identities.clone();
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, copied);
    persisted(&root, &model);
}
