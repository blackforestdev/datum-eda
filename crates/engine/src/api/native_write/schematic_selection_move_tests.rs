//! One parametrized `datum.schematic.move` over selected wires and junctions.
use super::*;
use crate::{
    api::native_write::{
        commit_prepared,
        electrical::topology_tests,
        registry::find_native_write_verb,
        schematic_connectivity::{build_create_schematic_junction, build_create_schematic_wire},
        schematic_edit::prepare_native_birth,
    },
    ir::geometry::Point,
    schematic::{Junction, Schematic, SchematicWire},
    substrate::{CommitSource, DesignModel, ProjectResolver},
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "move schematic selection")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn write(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
}
#[test]
fn selected_wire_and_junction_moves_preserve_identity_and_history() {
    let (root, mut model, _, sheet) = topology_tests::fixture("move_wire_junction");
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(1000, 0),
    };
    let w = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
    let w = prepare_native_birth(&model, w).unwrap();
    commit_prepared(&mut model, &root, w).unwrap();
    let junction = Junction {
        uuid: Uuid::new_v4(),
        position: Point::new(0, 0),
    };
    let occupied = Junction {
        uuid: Uuid::new_v4(),
        position: Point::new(5000, 5000),
    };
    for j in [&junction, &occupied] {
        let prepared = build_create_schematic_junction(&model, provenance(), sheet, j).unwrap();
        let prepared = prepare_native_birth(&model, prepared).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let before = source(&model);
    // Refusals leave the model untouched: unchanged destinations, invented
    // occurrence paths, and a junction landing on an occupied point.
    for (verb, params) in [
        (
            "datum.schematic.move",
            serde_json::json!({"object":wire.uuid,"instance_path":[],"x_nm":0,"y_nm":0}),
        ),
        (
            "datum.schematic.move",
            serde_json::json!({"object":wire.uuid,"instance_path":[Uuid::new_v4()],"x_nm":100,"y_nm":0}),
        ),
        (
            "datum.schematic.move",
            serde_json::json!({"object":junction.uuid,"instance_path":[],"x_nm":0,"y_nm":0}),
        ),
        (
            "datum.schematic.move",
            serde_json::json!({"object":junction.uuid,"instance_path":[],"x_nm":5000,"y_nm":5000}),
        ),
        (
            "datum.schematic.move",
            serde_json::json!({"object":junction.uuid,"instance_path":[Uuid::new_v4()],"x_nm":100,"y_nm":100}),
        ),
    ] {
        assert!(write(&model, &root, verb, params).is_err());
    }
    assert_eq!(source(&model), before);
    // Rigid wire translate preserves the authored identity and segment shape.
    let prepared = write(
        &model,
        &root,
        "datum.schematic.move",
        serde_json::json!({"object":wire.uuid,"instance_path":[],"x_nm":2500,"y_nm":-1500}),
    )
    .unwrap();
    assert_eq!(prepared.primary_object_id, Some(wire.uuid));
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let wire_moved = source(&model);
    assert_eq!(
        wire_moved.sheets[&sheet].wires[&wire.uuid].from,
        Point::new(2500, -1500)
    );
    assert_eq!(
        wire_moved.sheets[&sheet].wires[&wire.uuid].to,
        Point::new(3500, -1500)
    );
    // Coincident geometry stays authored where it was: the junction at the
    // original endpoint does not follow the moved wire.
    assert_eq!(
        wire_moved.sheets[&sheet].junctions[&junction.uuid].position,
        Point::new(0, 0)
    );
    // Absolute junction relocation preserves identity under placement admission.
    let prepared = write(
        &model,
        &root,
        "datum.schematic.move",
        serde_json::json!({"object":junction.uuid,"instance_path":[],"x_nm":2500,"y_nm":-1500}),
    )
    .unwrap();
    assert_eq!(prepared.primary_object_id, Some(junction.uuid));
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source(&model);
    assert_eq!(
        after.sheets[&sheet].junctions[&junction.uuid].position,
        Point::new(2500, -1500)
    );
    // Each move is one history entry; two undos restore the exact source.
    for _ in 0..2 {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    assert_eq!(source(&model), before);
    for _ in 0..2 {
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    assert_eq!(source(&model), after);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn moves_refuse_locked_members_and_preserve_raw_extensions() {
    let (root, mut model, _, sheet) = topology_tests::fixture("move_raw_extensions");
    let wire_id = Uuid::new_v4();
    let locked_wire = Uuid::new_v4();
    let locked_junction = Uuid::new_v4();
    // Author raw members directly: the raw payload is the authority and the
    // typed structs deliberately know nothing about these extension fields.
    let ops = vec![
        Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id,
            wire: serde_json::json!({"uuid":wire_id,
                "from":{"x":0,"y":0},"to":{"x":1000,"y":0},
                "note":{"nested":"survives","list":[1,2,3]},"vendor_ext":"keep"}),
        },
        Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: locked_wire,
            wire: serde_json::json!({"uuid":locked_wire,
                "from":{"x":0,"y":5000},"to":{"x":1000,"y":5000},"locked":true}),
        },
        Operation::PlaceSchematicMarker {
            sheet_id: sheet,
            marker_id: locked_junction,
            marker_kind: crate::substrate::SchematicMarkerKind::Junction,
            marker: serde_json::json!({"uuid":locked_junction,
                "position":{"x":7000,"y":7000},"locked":true}),
        },
    ];
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .finish()
        .unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before = source(&model);
    // A raw "locked": true extension refuses the move for both kinds and
    // leaves the model untouched.
    for object in [locked_wire, locked_junction] {
        assert!(
            write(
                &model,
                &root,
                "datum.schematic.move",
                serde_json::json!({"object":object,"instance_path":[],"x_nm":100,"y_nm":100}),
            )
            .is_err()
        );
    }
    assert_eq!(source(&model), before);
    // Raw nested metadata rides through the move byte-identical; only the
    // endpoints change.
    let prepared = write(
        &model,
        &root,
        "datum.schematic.move",
        serde_json::json!({"object":wire_id,"instance_path":[],"x_nm":2000,"y_nm":3000}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let raw = source_sheet_payload(&model, sheet, wire_id).unwrap();
    let member = &raw["wires"][wire_id.to_string()];
    assert_eq!(
        member["note"],
        serde_json::json!({"nested":"survives","list":[1,2,3]})
    );
    assert_eq!(member["vendor_ext"], "keep");
    assert_eq!(member["from"], serde_json::json!({"x":2000,"y":3000}));
    assert_eq!(member["to"], serde_json::json!({"x":3000,"y":3000}));
    // One history entry; undo restores the exact original raw member.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let raw = source_sheet_payload(&model, sheet, wire_id).unwrap();
    let member = &raw["wires"][wire_id.to_string()];
    assert_eq!(member["from"], serde_json::json!({"x":0,"y":0}));
    assert_eq!(member["note"]["nested"], "survives");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn port_move_relocates_position_only_with_set_semantics() {
    let (root, mut model, _, sheet) = topology_tests::fixture("move_port_set");
    // Author one port through the ordinary verb, then clone its exact raw
    // shape into a vendor-extended sibling and a locked sibling.
    let prepared = write(
        &model,
        &root,
        "datum.schematic.place_port",
        serde_json::json!({"sheet":sheet,"name":"CLK","direction":"input",
            "x_nm":10_000,"y_nm":20_000}),
    )
    .unwrap();
    let port_id = prepared.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let raw = source_sheet_payload(&model, sheet, port_id).unwrap();
    let template = raw["ports"][port_id.to_string()].clone();
    let extended = Uuid::new_v4();
    let locked = Uuid::new_v4();
    let mut vendor = template.clone();
    vendor["uuid"] = serde_json::json!(extended);
    vendor["name"] = serde_json::json!("EXT");
    vendor["vendor_ext"] = serde_json::json!({"keep":true});
    let mut sealed = template.clone();
    sealed["uuid"] = serde_json::json!(locked);
    sealed["name"] = serde_json::json!("SEALED");
    sealed["position"] = serde_json::json!({"x":99_000,"y":99_000});
    sealed["locked"] = serde_json::json!(true);
    let prepared = BatchComposer::compose(&model, provenance())
        .push_ops(vec![
            Operation::CreateSchematicPort {
                sheet_id: sheet,
                port_id: extended,
                port: vendor,
            },
            Operation::CreateSchematicPort {
                sheet_id: sheet,
                port_id: locked,
                port: sealed,
            },
        ])
        .finish()
        .unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before = source(&model);
    // Unchanged destination and locked members refuse untouched.
    assert!(
        write(
            &model,
            &root,
            "datum.schematic.move",
            serde_json::json!({"object":port_id,"instance_path":[],"x_nm":10_000,"y_nm":20_000}),
        )
        .is_err()
    );
    assert!(
        write(
            &model,
            &root,
            "datum.schematic.move",
            serde_json::json!({"object":locked,"instance_path":[],"x_nm":1,"y_nm":1}),
        )
        .is_err()
    );
    assert_eq!(source(&model), before);
    // The Set-based move changes position only; name, direction and vendor
    // extensions ride through byte-identical.
    let prepared = write(
        &model,
        &root,
        "datum.schematic.move",
        serde_json::json!({"object":extended,"instance_path":[],"x_nm":40_000,"y_nm":50_000}),
    )
    .unwrap();
    assert_eq!(prepared.primary_object_id, Some(extended));
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let raw = source_sheet_payload(&model, sheet, extended).unwrap();
    let member = &raw["ports"][extended.to_string()];
    assert_eq!(
        member["position"],
        serde_json::json!({"x":40_000,"y":50_000})
    );
    assert_eq!(member["name"], "EXT");
    assert_eq!(member["direction"], template["direction"]);
    assert_eq!(member["vendor_ext"], serde_json::json!({"keep":true}));
    let after = source(&model);
    assert_eq!(
        after.sheets[&sheet].ports[&port_id], before.sheets[&sheet].ports[&port_id],
        "unselected port stays fixed"
    );
    // One history entry restores the exact original raw member.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    std::fs::remove_dir_all(root).unwrap();
}
