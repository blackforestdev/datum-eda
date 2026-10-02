//! Native selection query oracles authored independently of the production index.
use super::super::{commit_prepared, schematic_sheets::*};
use super::tests::{create, net, provenance};
use super::topology_tests::fixture;
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::{LabelKind, NetLabel, SchematicWire, SheetDefinition, SheetInstance};
use crate::substrate::{
    ElectricalOccurrence, ElectricalQueryFailure, ElectricalSelectionSnapshot,
    NetRelationshipIntent, ProjectResolver,
};
use std::collections::BTreeSet;
use uuid::Uuid;

fn member(class: &str, id: Uuid, path: &[Uuid]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id,
        instance_path: path.into(),
    }
}
fn schematic_only(model: &mut DesignModel, root: &std::path::Path, logical: Uuid) {
    create(
        model,
        root,
        ElectricalIdentityRecord {
            id: Uuid::new_v4(),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: logical,
                board_net: None,
                intent: NetRelationshipIntent::SchematicOnly,
                evidence: vec![],
            },
        },
    );
}

#[test]
fn complete_257_branch_net_is_not_a_context_envelope_and_runs_are_physical() {
    let (root, mut model, _, sheet) = fixture("selection_uncapped");
    let mut expected = BTreeSet::new();
    let mut operations = vec![];
    let mut origin = None;
    for i in 0..257 {
        let wire = SchematicWire {
            uuid: Uuid::new_v4(),
            from: Point::new(0, i * 100),
            to: Point::new(10, i * 100),
        };
        let label = NetLabel {
            uuid: Uuid::new_v4(),
            name: "GLOBAL".into(),
            kind: LabelKind::Global,
            position: wire.from,
        };
        let wm = member("wires", wire.uuid, &[]);
        let lm = member("labels", label.uuid, &[]);
        if i == 0 {
            origin = Some((wm.clone(), lm.clone()));
        }
        expected.extend([wm, lm]);
        operations.push(Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire).unwrap(),
        });
        operations.push(Operation::CreateSchematicLabel {
            sheet_id: sheet,
            label_id: label.uuid,
            label: serde_json::to_value(label).unwrap(),
        });
    }
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(operations)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let (wire, label) = origin.unwrap();
    let logical = Uuid::new_v4();
    create(&mut model, &root, net(logical, wire.clone()));
    schematic_only(&mut model, &root, logical);
    let before = model.clone();
    let files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        snapshot
            .global_net(&model.model_revision, logical)
            .unwrap()
            .owned,
        expected
    );
    assert_eq!(
        snapshot
            .schematic_run(&model.model_revision, &wire)
            .unwrap()
            .owned,
        BTreeSet::from([wire.clone(), label])
    );
    assert_eq!(model, before);
    for (path, bytes) in files {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&reopened)
            .unwrap()
            .global_net(&reopened.model_revision, logical)
            .unwrap()
            .owned,
        expected
    );
    assert_eq!(
        snapshot.global_net(&crate::substrate::ModelRevision("wrong".into()), logical),
        Err(ElectricalQueryFailure::StaleRevision)
    );
    assert_eq!(
        snapshot.global_net(&model.model_revision, Uuid::nil()),
        Err(ElectricalQueryFailure::AbsentIdentity { id: Uuid::nil() })
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn repeated_sheet_source_has_distinct_occurrences_and_queries_preserve_ids() {
    let (root, mut model, schematic, sheet) = fixture("selection_occurrence");
    let wire = super::topology_tests::wire(&mut model, &root, sheet, 0, 10);
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
        "definitions/reusable.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut expected = vec![];
    for _ in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            name: "same".into(),
            position: Point::new(0, 0),
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
        expected.push(member("wires", wire.uuid, &[instance.uuid]));
    }
    let prepared = build_adopt_schematic_net_identities(&model, provenance()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let ids: Vec<_> = expected
        .iter()
        .map(|origin| super::topology_tests::id_at(&model, origin))
        .collect();
    assert_ne!(ids[0], ids[1]);
    for id in &ids {
        schematic_only(&mut model, &root, *id);
    }
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    for (origin, id) in expected.iter().zip(ids) {
        assert_eq!(
            snapshot.global_net(snapshot.revision(), id).unwrap().owned,
            BTreeSet::from([origin.clone()])
        );
        assert_eq!(
            snapshot
                .schematic_run(snapshot.revision(), origin)
                .unwrap()
                .subject_id,
            id
        );
    }
    assert!(matches!(
        snapshot.schematic_run(snapshot.revision(), &member("wires", wire.uuid, &[])),
        Err(ElectricalQueryFailure::UnavailableAssignment { .. })
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn capture_refuses_changed_source_and_keeps_previously_captured_result_immutable() {
    let (root, mut model, _, sheet) = fixture("selection_capture");
    let wire = super::topology_tests::wire(&mut model, &root, sheet, 0, 10);
    let origin = member("wires", wire.uuid, &[]);
    let logical = Uuid::new_v4();
    create(&mut model, &root, net(logical, origin.clone()));
    schematic_only(&mut model, &root, logical);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == crate::substrate::SourceShardKind::SchematicSheet)
        .unwrap()
        .path
        .clone();
    let bytes = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let compact = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &compact).unwrap();
    let compact_model = ProjectResolver::new(&root).resolve().unwrap();
    let compact_snapshot = ElectricalSelectionSnapshot::capture(&compact_model).unwrap();
    assert_eq!(
        compact_snapshot
            .global_net(compact_snapshot.revision(), logical)
            .unwrap()
            .owned,
        BTreeSet::from([origin.clone()])
    );
    assert_eq!(std::fs::read(&path).unwrap(), compact);
    value["name"] = serde_json::json!("unexpected external change");
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(matches!(
        ElectricalSelectionSnapshot::capture(&model),
        Err(ElectricalQueryFailure::UnavailableBasis { .. })
    ));
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), logical)
            .unwrap()
            .owned,
        BTreeSet::from([origin])
    );
    std::fs::write(path, bytes).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn binding_failure_is_subject_local_and_cross_domain_membership_never_prunes_board() {
    let super::binding_fixture::BindingFixture {
        root,
        mut model,
        sheet,
        symbol,
        logical,
        relationship,
        library_pin,
        ..
    } = super::binding_fixture::fixture("selection_binding");
    create(&mut model, &root, relationship.clone());
    assert!(matches!(
        ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .global_net(&model.model_revision, logical),
        Err(ElectricalQueryFailure::UnavailableBinding {
            status: crate::substrate::NetCorrespondenceStatus::Unverified,
            ..
        })
    ));
    let prepared = build_adopt_pin_correspondence(
        &model,
        provenance(),
        relationship.clone(),
        &[PlacedPinAdoption {
            sheet_id: sheet,
            symbol_id: symbol.uuid,
            placed_pin_id: symbol.pins[0].uuid,
            library_pin_id: library_pin,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let relation = model.electrical_identities[&relationship.id].clone();
    let ElectricalIdentity::NetRelationship { evidence, .. } = &relation.identity else {
        panic!()
    };
    let expected = BTreeSet::from([
        member("pins", symbol.pins[0].uuid, &[]),
        member("pads", evidence[0].board_pad.object_id, &[]),
    ]);
    let captured = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        captured
            .global_net(captured.revision(), logical)
            .unwrap()
            .owned,
        expected
    );
    let wire = super::topology_tests::wire(&mut model, &root, sheet, 100, 110);
    let unrelated = super::topology_tests::id_at(&model, &member("wires", wire.uuid, &[]));
    schematic_only(&mut model, &root, unrelated);
    let mut pending = model.electrical_identities[&relationship.id].clone();
    let ElectricalIdentity::NetRelationship { intent, .. } = &mut pending.identity else {
        panic!()
    };
    *intent = NetRelationshipIntent::Pending;
    let prepared = build_set_electrical_identity(&model, provenance(), pending).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(matches!(
        snapshot.global_net(snapshot.revision(), logical),
        Err(ElectricalQueryFailure::UnavailableBinding {
            status: crate::substrate::NetCorrespondenceStatus::Pending,
            ..
        })
    ));
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), unrelated)
            .unwrap()
            .owned,
        BTreeSet::from([member("wires", wire.uuid, &[])])
    );
    assert_eq!(
        captured
            .global_net(captured.revision(), logical)
            .unwrap()
            .owned,
        expected
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&reopened).unwrap();
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), logical)
            .unwrap()
            .owned,
        expected
    );
    std::fs::remove_dir_all(root).unwrap();
}
