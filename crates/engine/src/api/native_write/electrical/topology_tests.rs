//! Actual native source edits, independent occurrence-set oracles and persistence.
use super::super::{commit_prepared, genesis, schematic_connectivity::*, schematic_sheets::*};
use super::tests::provenance;
use super::*;
use crate::{
    ir::geometry::Point,
    schematic::{LabelKind, NetLabel, SchematicWire, SheetDefinition, SheetInstance},
    substrate::{
        ElectricalOccurrence, NetAnchorReason, NetRelationshipIntent, ProjectResolver,
        electrical_test_partitions,
    },
};
use std::collections::BTreeSet;
use uuid::Uuid;

fn fixture(name: &str) -> (std::path::PathBuf, DesignModel, Uuid, Uuid) {
    let root = super::super::test_support::temp_project_root(name);
    let ids = genesis::bootstrap_native_project(
        &root,
        genesis::GenesisSpec {
            project_name: name.into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = Uuid::new_v4();
    add_sheet(
        &mut model,
        &root,
        ids.schematic_uuid,
        sheet,
        "sheets/main.json",
    );
    (root, model, ids.schematic_uuid, sheet)
}
fn add_sheet(
    model: &mut DesignModel,
    root: &std::path::Path,
    schematic: Uuid,
    id: Uuid,
    path: &str,
) {
    let source = serde_json::json!({"schema_version":1,"uuid":id,"name":"Main","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}});
    let write =
        build_create_schematic_sheet(model, provenance(), schematic, id, path, source).unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn wire(
    model: &mut DesignModel,
    root: &std::path::Path,
    sheet: Uuid,
    from: i64,
    to: i64,
) -> SchematicWire {
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(from, 0),
        to: Point::new(to, 0),
    };
    let write = build_create_schematic_wire(model, provenance(), sheet, &wire).unwrap();
    commit_prepared(model, root, write).unwrap();
    wire
}
fn member(class: &str, id: Uuid, path: &[Uuid]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id,
        instance_path: path.to_vec(),
    }
}
fn id_at(model: &DesignModel, reference: &ElectricalOccurrence) -> Uuid {
    let groups = electrical_test_partitions(model, &[]).unwrap();
    let group = groups.iter().find(|g| g.contains(reference)).unwrap();
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
fn adopt(model: &mut DesignModel, root: &std::path::Path) {
    let write = build_adopt_schematic_net_identities(model, provenance()).unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn persisted(root: &std::path::Path, model: &DesignModel) {
    assert_eq!(
        ProjectResolver::new(root)
            .resolve()
            .unwrap()
            .electrical_identities,
        model.electrical_identities
    );
}

#[test]
fn native_split_merge_anchor_loss_and_pending_bindings_are_automatic() {
    let (root, mut model, _, sheet) = fixture("e1_native_topology");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let bridge = wire(&mut model, &root, sheet, 10, 20);
    let b = wire(&mut model, &root, sheet, 20, 30);
    let original = BTreeSet::from([
        member("wires", a.uuid, &[]),
        member("wires", bridge.uuid, &[]),
        member("wires", b.uuid, &[]),
    ]);
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        vec![original]
    );
    assert!(model.electrical_identities.is_empty());
    adopt(&mut model, &root);
    let old = id_at(&model, &member("wires", a.uuid, &[]));
    let old_record = model.electrical_identities[&old].clone();
    let relation = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: old,
            board_net: None,
            intent: NetRelationshipIntent::Pending,
            evidence: vec![],
        },
    };
    super::tests::create(&mut model, &root, relation.clone());
    let before = model.clone();
    let write = build_delete_schematic_wire(&model, provenance(), sheet, &bridge).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut expected = vec![
        BTreeSet::from([member("wires", a.uuid, &[])]),
        BTreeSet::from([member("wires", b.uuid, &[])]),
    ];
    expected.sort();
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), expected);
    let left = id_at(&model, &member("wires", a.uuid, &[]));
    let right = id_at(&model, &member("wires", b.uuid, &[]));
    assert_ne!(left, right);
    let old_anchor = match &old_record.identity {
        ElectricalIdentity::Net { anchor, .. } => anchor,
        _ => unreachable!(),
    };
    if old_anchor.source_id == a.uuid {
        assert_eq!(old, left)
    } else if old_anchor.source_id == b.uuid {
        assert_eq!(old, right)
    } else {
        let min = member("wires", a.uuid.min(b.uuid), &[]);
        assert_eq!(old, id_at(&model, &min));
        assert!(matches!(
            model.electrical_identities[&old].identity,
            ElectricalIdentity::Net {
                anchor_reason: NetAnchorReason::DeletedAnchorFallback,
                ..
            }
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
    let write = build_create_schematic_wire(&model, provenance(), sheet, &bridge).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let winner = left.min(right);
    assert_eq!(id_at(&model, &member("wires", a.uuid, &[])), winner);
    assert!(matches!(
        model.electrical_identities[&left.max(right)].identity,
        ElectricalIdentity::Net { retired: true, .. }
    ));
    assert_eq!(
        model
            .electrical_identities
            .values()
            .filter(|r| matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }))
            .count(),
        1
    );
    persisted(&root, &model);
}

#[test]
fn repeated_sheet_local_names_isolate_and_globals_join_without_path_collapse() {
    let (root, mut model, schematic, sheet) = fixture("e1_occurrence_topology");
    let w = wire(&mut model, &root, sheet, 0, 10);
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "LOCAL".into(),
        kind: LabelKind::Local,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: sheet,
        name: "D".into(),
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/d.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut paths = vec![];
    for name in ["I", "K"] {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            name: name.into(),
            position: Point::new(0, 0),
            ports: vec![],
        };
        paths.push(instance.uuid);
        let write = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let mut expected: Vec<_> = paths
        .iter()
        .map(|i| {
            BTreeSet::from([
                member("wires", w.uuid, &[*i]),
                member("labels", label.uuid, &[*i]),
            ])
        })
        .collect();
    expected.sort();
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), expected);
    adopt(&mut model, &root);
    let i = id_at(&model, &member("wires", w.uuid, &[paths[0]]));
    let k = id_at(&model, &member("wires", w.uuid, &[paths[1]]));
    assert_ne!(i, k);
    let before = model.clone();
    let mut global = label.clone();
    global.kind = LabelKind::Global;
    let write = build_set_schematic_label(&model, provenance(), sheet, &global).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        vec![expected.into_iter().flatten().collect()]
    );
    assert_eq!(
        id_at(&model, &member("wires", w.uuid, &[paths[0]])),
        i.min(k)
    );
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before.electrical_identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    persisted(&root, &model);
    global.name = "renamed display".into();
    let before = model.electrical_identities.clone();
    let write = build_set_schematic_label(&model, provenance(), sheet, &global).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(model.electrical_identities, before);
}

#[test]
fn forged_transition_and_stale_source_batch_refuse_without_authored_mutation() {
    let (root, mut model, _, sheet) = fixture("e1_topology_refusal");
    let a = wire(&mut model, &root, sheet, 0, 10);
    adopt(&mut model, &root);
    let stale = build_delete_schematic_wire(&model, provenance(), sheet, &a).unwrap();
    let _b = wire(&mut model, &root, sheet, 10, 20);
    let before = model.clone();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(model, before);
    let old = id_at(&model, &member("wires", a.uuid, &[]));
    let mut forged = model.electrical_identities[&old].clone();
    if let ElectricalIdentity::Net { retired, .. } = &mut forged.identity {
        *retired = true
    }
    forged.object_revision = ObjectRevision(forged.object_revision.0 + 1);
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::DeleteSchematicWire {
            sheet_id: sheet,
            wire_id: a.uuid,
            wire: serde_json::to_value(a).unwrap(),
        })
        .push_op(Operation::SetElectricalIdentity {
            previous: Box::new(model.electrical_identities[&old].clone()),
            record: forged,
        })
        .finish()
        .unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(model, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    persisted(&root, &model);
}

#[path = "topology_atomic_tests.rs"]
mod atomic;
#[path = "topology_hierarchy_tests.rs"]
mod hierarchy;

#[path = "topology_identity_write_tests.rs"]
mod identity_writes;
