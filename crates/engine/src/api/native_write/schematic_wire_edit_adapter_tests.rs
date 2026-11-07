//! Canonical handle edits: original T geometry, raw ownership and cold history.
use super::{
    commit_prepared,
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    electrical::topology_tests,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_edit::prepare_native_birth,
    schematic_symbol_payload::source_sheet_payload,
    schematic_wire_edit::{WireEditBasis, WireHandle, WireRouting},
};
use crate::{
    ir::geometry::Point,
    substrate::{
        CommitSource, DesignModel, ElectricalSelectionSnapshot, Operation, ProjectResolver,
    },
};
use serde_json::{Value, json};
use std::path::Path;
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "canonical wire handle edit")
}
fn build(
    model: &DesignModel,
    root: &Path,
    object: Uuid,
    handle: Value,
    destination: Point,
    seed: Uuid,
) -> Result<PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"object":object,"instance_path":[],"x_nm":destination.x,"y_nm":destination.y,
            "wire_edit":{"handle":handle,"routing":"free","edit_seed":seed}}),
    )
}
fn commit(model: &mut DesignModel, root: &Path, ops: Vec<Operation>) {
    let prepared = BatchComposer::compose(model, provenance())
        .push_ops(ops)
        .finish()
        .unwrap();
    let prepared = prepare_native_birth(model, prepared).unwrap();
    commit_prepared(model, root, prepared).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
}
fn wire(sheet: Uuid, id: Uuid, from: Point, to: Point, tag: &str) -> Operation {
    Operation::CreateSchematicWire {
        sheet_id: sheet,
        wire_id: id,
        wire: json!({"uuid":id,"from":{"x":from.x,"y":from.y,"future_point":"from"},
            "to":{"x":to.x,"y":to.y,"future_point":"to"},"vendor":{"origin":tag,"literal":"${KEEP}"}}),
    }
}
fn sheet(model: &DesignModel, id: Uuid) -> crate::schematic::Sheet {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&id]
        .clone()
}
#[test]
fn wire_handle_unsplit_and_segmented_t_keep_unowned_geometry_and_cold_raw_history() {
    for segmented in [false, true] {
        let (root, mut model, _, sheet_id) = topology_tests::fixture("wire_handle_owner_t");
        let through = Uuid::new_v4();
        let lower = Uuid::new_v4();
        let branch = Uuid::new_v4();
        let junction = Uuid::new_v4();
        let upper = Point::new(-2_400_000, -3_290_000);
        let node = Point::new(-2_400_000, 1_200_000);
        let bottom = Point::new(-2_400_000, 2_690_000);
        let base = Point::new(1_720_000, 700_000);
        let mut ops = vec![
            wire(
                sheet_id,
                through,
                upper,
                if segmented { node } else { bottom },
                "through",
            ),
            wire(sheet_id, branch, base, node, "branch"),
            Operation::PlaceSchematicMarker {
                sheet_id,
                marker_id: junction,
                marker_kind: crate::substrate::SchematicMarkerKind::Junction,
                marker: json!({"uuid":junction,"position":{"x":node.x,"y":node.y,"future_point":"junction"},"vendor":"keep"}),
            },
        ];
        if segmented {
            ops.push(wire(sheet_id, lower, node, bottom, "lower"));
        }
        commit(&mut model, &root, ops);
        let before = source_sheet_payload(&model, sheet_id, branch).unwrap();
        let count = model.journal.len();
        if segmented {
            assert!(
                build(
                    &model,
                    &root,
                    branch,
                    json!({"kind":"endpoint","end":"from"}),
                    upper,
                    Uuid::new_v4()
                )
                .is_err()
            );
            assert_eq!(model.journal.len(), count);
            assert_eq!(
                source_sheet_payload(&model, sheet_id, branch).unwrap(),
                before
            );
        }
        let handle = json!({"kind":"endpoint","end":"to"});
        // The exact off-grid target is authored, not rounded to a display grid.
        let prepared = build(&model, &root, branch, handle, upper, Uuid::new_v4()).unwrap();
        assert_eq!(model.journal.len(), count);
        assert_eq!(
            source_sheet_payload(&model, sheet_id, branch).unwrap(),
            before
        );
        commit_prepared(&mut model, &root, prepared).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        let after = source_sheet_payload(&model, sheet_id, branch).unwrap();
        let mut expected = before.clone();
        expected["wires"][branch.to_string()]["to"]["x"] = json!(upper.x);
        expected["wires"][branch.to_string()]["to"]["y"] = json!(upper.y);
        assert_eq!(
            after, expected,
            "only the explicitly edited endpoint changes"
        );
        assert_eq!(sheet(&model, sheet_id).wires[&branch].from, base);
        for redo in [false, true] {
            if redo {
                model
                    .commit_journal_redo(&root, provenance().into())
                    .unwrap();
            } else {
                model
                    .commit_journal_undo(&root, provenance().into())
                    .unwrap();
            }
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(
                source_sheet_payload(&model, sheet_id, branch).unwrap(),
                if redo { after.clone() } else { before.clone() }
            );
        }
        // Return to the same original graph, then exercise logical Vertex and
        // upper-span editing for BOTH unsplit and physically segmented sources.
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        for handle in [
            WireHandle::Vertex { point: node },
            WireHandle::Segment {
                from: upper,
                to: node,
            },
        ] {
            let captured = sheet(&model, sheet_id);
            let seed = Uuid::new_v4();
            let at = match handle {
                WireHandle::Vertex { .. } => Point::new(-2_400_000, 700_000),
                _ => Point::new(-2_800_000, -1_045_000),
            };
            if !segmented && matches!(handle, WireHandle::Segment { .. }) {
                let count = model.journal.len();
                assert!(
                    build(
                        &model,
                        &root,
                        through,
                        serde_json::to_value(handle).unwrap(),
                        at,
                        seed
                    )
                    .is_err(),
                    "an unsplit gated T cannot be hardwired by adding bridge endpoints"
                );
                assert_eq!(
                    source_sheet_payload(&model, sheet_id, branch).unwrap(),
                    before
                );
                assert_eq!(model.journal.len(), count);
                continue;
            }
            let plan = WireEditBasis::capture(&captured, through, seed)
                .unwrap()
                .plan_move(&handle, at, WireRouting::Free)
                .unwrap();
            let prepared = build(
                &model,
                &root,
                through,
                serde_json::to_value(&handle).unwrap(),
                at,
                seed,
            )
            .unwrap();
            commit_prepared(&mut model, &root, prepared).unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            let moved = sheet(&model, sheet_id);
            assert_eq!(
                moved,
                plan.apply_to_sheet(&captured),
                "canonical plan matches preview geometry"
            );
            assert!(
                moved
                    .wires
                    .values()
                    .any(|w| w.from == upper || w.to == upper)
            );
            assert!(
                moved
                    .wires
                    .values()
                    .any(|w| w.from == bottom || w.to == bottom)
            );
            assert_eq!(moved.wires[&branch].from, base, "Q1 base stays fixed");
            if matches!(handle, WireHandle::Vertex { .. }) {
                assert_eq!(moved.junctions[&junction].position, at);
                assert_eq!(moved.wires[&branch].to, at);
            }
            let raw_after = source_sheet_payload(&model, sheet_id, branch).unwrap();
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(
                source_sheet_payload(&model, sheet_id, branch).unwrap(),
                before
            );
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(
                source_sheet_payload(&model, sheet_id, branch).unwrap(),
                raw_after
            );
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn wire_segment_new_edges_clone_raw_origin_and_preserve_identity_through_history() {
    let (root, mut model, _, sheet_id) = topology_tests::fixture("wire_segment_raw_plan");
    let id = Uuid::new_v4();
    let a = Point::new(0, 0);
    let b = Point::new(10_000, 0);
    commit(&mut model, &root, vec![wire(sheet_id, id, a, b, "single")]);
    let before = source_sheet_payload(&model, sheet_id, id).unwrap();
    let seed = Uuid::new_v4();
    let destination = Point::new(5_000, 2_000);
    let handle = WireHandle::Segment { from: a, to: b };
    let captured = sheet(&model, sheet_id);
    let plan = WireEditBasis::capture(&captured, id, seed)
        .unwrap()
        .plan_move(&handle, destination, WireRouting::Free)
        .unwrap();
    assert!(
        !plan.new_wire_origins.is_empty(),
        "segment shift creates connecting bends"
    );
    let prepared = build(
        &model,
        &root,
        id,
        serde_json::to_value(handle).unwrap(),
        destination,
        seed,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(sheet(&model, sheet_id), plan.apply_to_sheet(&captured));
    let after = source_sheet_payload(&model, sheet_id, id).unwrap();
    for (new_id, moved) in &plan.wires {
        let payload = &after["wires"][new_id.to_string()];
        let mut expected = before["wires"][id.to_string()].clone();
        expected["uuid"] = json!(new_id);
        expected["from"]["x"] = json!(moved.from.x);
        expected["from"]["y"] = json!(moved.from.y);
        expected["to"]["x"] = json!(moved.to.x);
        expected["to"]["y"] = json!(moved.to.y);
        assert_eq!(
            *payload, expected,
            "origin extensions survive each new piece"
        );
    }
    for redo in [false, true] {
        if redo {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&model, sheet_id, id).unwrap(),
            if redo { after.clone() } else { before.clone() }
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn wire_handle_guards_refuse_missing_handle_nil_seed_lock_and_new_id_collision_without_writes() {
    let (root, mut model, _, sheet_id) = topology_tests::fixture("wire_handle_admission");
    let id = Uuid::new_v4();
    let a = Point::new(0, 0);
    let b = Point::new(10_000, 0);
    commit(&mut model, &root, vec![wire(sheet_id, id, a, b, "main")]);
    let seed = Uuid::new_v4();
    let destination = Point::new(5_000, 2_000);
    let handle = WireHandle::Segment { from: a, to: b };
    let plan = WireEditBasis::capture(&sheet(&model, sheet_id), id, seed)
        .unwrap()
        .plan_move(&handle, destination, WireRouting::Free)
        .unwrap();
    let collision = *plan.new_wire_origins.keys().next().unwrap();
    commit(
        &mut model,
        &root,
        vec![wire(
            sheet_id,
            collision,
            Point::new(100_000, 0),
            Point::new(110_000, 0),
            "foreign",
        )],
    );
    let before = source_sheet_payload(&model, sheet_id, id).unwrap();
    let journal = model.journal.len();
    assert!(
        build(
            &model,
            &root,
            id,
            serde_json::to_value(&handle).unwrap(),
            destination,
            seed
        )
        .is_err()
    );
    assert!(
        build(
            &model,
            &root,
            id,
            json!({"kind":"endpoint","end":"to"}),
            Point::new(20_000, 0),
            Uuid::nil()
        )
        .is_err()
    );
    let no_handle = (find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"object":id,"instance_path":[],"x_nm":10,"y_nm":20}),
    );
    assert!(no_handle.is_err());
    assert_eq!(source_sheet_payload(&model, sheet_id, id).unwrap(), before);
    assert_eq!(model.journal.len(), journal);
    let mut locked = before["wires"][id.to_string()].clone();
    locked["locked"] = json!(true);
    commit(
        &mut model,
        &root,
        vec![
            Operation::DeleteSchematicWire {
                sheet_id,
                wire_id: id,
                wire: before["wires"][id.to_string()].clone(),
            },
            Operation::CreateSchematicWire {
                sheet_id,
                wire_id: id,
                wire: locked,
            },
        ],
    );
    let before = source_sheet_payload(&model, sheet_id, id).unwrap();
    let journal = model.journal.len();
    assert!(
        build(
            &model,
            &root,
            id,
            json!({"kind":"endpoint","end":"to"}),
            Point::new(20_000, 0),
            Uuid::new_v4()
        )
        .is_err()
    );
    assert_eq!(source_sheet_payload(&model, sheet_id, id).unwrap(), before);
    assert_eq!(model.journal.len(), journal);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_endpoint_to_foreign_interior_places_explicit_t_and_nominates_one_final_merge() {
    let (root, mut model, _, sheet_id) = topology_tests::fixture("wire_endpoint_foreign_t");
    let moving = Uuid::new_v4();
    let foreign = Uuid::new_v4();
    let from = Point::new(0, 0);
    let destination = Point::new(20_000, 7_310);
    commit(
        &mut model,
        &root,
        vec![
            wire(sheet_id, moving, from, Point::new(10_000, 0), "moving"),
            wire(
                sheet_id,
                foreign,
                Point::new(20_000, -10_000),
                Point::new(20_000, 10_000),
                "fixed",
            ),
        ],
    );
    let before = source_sheet_payload(&model, sheet_id, moving).unwrap();
    let before_identities = model.electrical_identities.clone();
    let before_ids = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .net_ids();
    let seed = Uuid::new_v4();
    let count = model.journal.len();
    let request = json!({"kind":"endpoint","end":"to"});
    let prepared = build(&model, &root, moving, request.clone(), destination, seed).unwrap();
    let prepared_again = build(&model, &root, moving, request, destination, seed).unwrap();
    let marker_ids = |p: &PreparedWrite| {
        p.batch
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::PlaceSchematicMarker { marker_id, .. } => Some(*marker_id),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let markers = marker_ids(&prepared);
    assert_eq!(markers.len(), 1);
    assert_eq!(
        marker_ids(&prepared_again),
        markers,
        "same gesture captures same physical T marker"
    );
    assert_eq!(
        source_sheet_payload(&model, sheet_id, moving).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count, "preparation is read-only");
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.journal.len(),
        count + 1,
        "one before-to-final design batch"
    );
    let after_ids = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .net_ids();
    assert_eq!(
        after_ids.len() + 1,
        before_ids.len(),
        "intended T reconnection merges two scalar groups"
    );
    let after = source_sheet_payload(&model, sheet_id, moving).unwrap();
    let mut expected = before.clone();
    expected["wires"][moving.to_string()]["to"]["x"] = json!(destination.x);
    expected["wires"][moving.to_string()]["to"]["y"] = json!(destination.y);
    expected["junctions"][markers[0].to_string()] =
        json!({"uuid":markers[0],"position":{"x":destination.x,"y":destination.y}});
    assert_eq!(
        after, expected,
        "foreign edge and opposite endpoint stay fixed; new marker is explicit"
    );
    let after_identities = model.electrical_identities.clone();
    for redo in [false, true] {
        if redo {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&model, sheet_id, moving).unwrap(),
            if redo { after.clone() } else { before.clone() }
        );
        assert_eq!(
            model.electrical_identities,
            if redo {
                after_identities.clone()
            } else {
                before_identities.clone()
            }
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_junction_vertex_payload_is_qualified_and_preserves_marker_extensions_and_history() {
    let (root, mut model, _, sheet_id) = topology_tests::fixture("junction_explicit_vertex");
    let through = Uuid::new_v4();
    let branch = Uuid::new_v4();
    let junction = Uuid::new_v4();
    let node = Point::new(0, 0);
    let at = Point::new(0, 2_730);
    commit(
        &mut model,
        &root,
        vec![
            wire(
                sheet_id,
                through,
                Point::new(0, -10_000),
                Point::new(0, 10_000),
                "through",
            ),
            wire(sheet_id, branch, Point::new(10_000, 0), node, "branch"),
            Operation::PlaceSchematicMarker {
                sheet_id,
                marker_id: junction,
                marker_kind: crate::substrate::SchematicMarkerKind::Junction,
                marker: json!({"uuid":junction,"position":{"x":0,"y":0,"future_point":"keep"},"vendor":{"literal":"${J}"}}),
            },
        ],
    );
    let before = source_sheet_payload(&model, sheet_id, junction).unwrap();
    let count = model.journal.len();
    for invalid_handle in [
        json!({"kind":"vertex","point":{"x":1,"y":0}}),
        json!({"kind":"endpoint","end":"from"}),
    ] {
        assert!(build(&model, &root, junction, invalid_handle, at, Uuid::new_v4()).is_err());
    }
    let edit = json!({"kind":"vertex","point":{"x":0,"y":0}});
    assert!(
        build(
            &model,
            &root,
            junction,
            edit.clone(),
            Point::new(-3_110, 2_730),
            Uuid::new_v4()
        )
        .is_err(),
        "off-through reshaping must not hardwire the gated junction"
    );
    assert_eq!(
        source_sheet_payload(&model, sheet_id, junction).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    let prepared = build(&model, &root, junction, edit.clone(), at, Uuid::new_v4()).unwrap();
    assert_eq!(prepared.primary_object_id, Some(junction));
    assert_eq!(
        source_sheet_payload(&model, sheet_id, junction).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source_sheet_payload(&model, sheet_id, junction).unwrap();
    let mut marker = before["junctions"][junction.to_string()].clone();
    marker["position"]["x"] = json!(at.x);
    marker["position"]["y"] = json!(at.y);
    assert_eq!(after["junctions"][junction.to_string()], marker);
    assert_eq!(
        sheet(&model, sheet_id).wires[&branch].from,
        Point::new(10_000, 0)
    );
    assert_eq!(sheet(&model, sheet_id).wires[&branch].to, at);
    for redo in [false, true] {
        if redo {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&model, sheet_id, junction).unwrap(),
            if redo { after.clone() } else { before.clone() }
        );
    }
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let original = before["junctions"][junction.to_string()].clone();
    let mut locked = original.clone();
    locked["locked"] = json!(true);
    commit(
        &mut model,
        &root,
        vec![
            Operation::DeleteSchematicJunction {
                sheet_id,
                junction_id: junction,
                junction: original,
            },
            Operation::PlaceSchematicMarker {
                sheet_id,
                marker_id: junction,
                marker_kind: crate::substrate::SchematicMarkerKind::Junction,
                marker: locked,
            },
        ],
    );
    let locked_source = source_sheet_payload(&model, sheet_id, junction).unwrap();
    assert!(build(&model, &root, junction, edit, at, Uuid::new_v4()).is_err());
    assert_eq!(
        source_sheet_payload(&model, sheet_id, junction).unwrap(),
        locked_source
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_wire_handle_does_not_reshape_a_surviving_bus_entry_owner() {
    let (root, mut model, _, sheet_id) = topology_tests::fixture("wire_handle_bus_owner");
    let id = Uuid::new_v4();
    let label = Uuid::new_v4();
    commit(
        &mut model,
        &root,
        vec![
            wire(sheet_id, id, Point::new(0, 0), Point::new(100, 0), "member"),
            Operation::CreateSchematicLabel {
                sheet_id,
                label_id: label,
                label: json!({"uuid":label,"name":"D0","kind":"Local","position":{"x":0,"y":0}}),
            },
        ],
    );
    let prepared = (find_native_write_verb("datum.schematic.create_bus")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"sheet":sheet_id,"instance_path":[],"name":"DATA","members":["D0"],
            "points":[{"x":1000,"y":-1000},{"x":1000,"y":1000}],
            "entries":[{"wire":id,"x_nm":1000,"y_nm":100,"wire_end":"to"}]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before = source_sheet_payload(&model, sheet_id, id).unwrap();
    let count = model.journal.len();
    let failure = build(
        &model,
        &root,
        id,
        json!({"kind":"endpoint","end":"to"}),
        Point::new(300, 0),
        Uuid::new_v4(),
    )
    .unwrap_err();
    assert!(failure.to_string().contains("bus entry"));
    assert_eq!(source_sheet_payload(&model, sheet_id, id).unwrap(), before);
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}
