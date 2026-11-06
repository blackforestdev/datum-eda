//! Regression for the owner's R1/C1 junction and Q1 base connection.
use crate::api::native_write::{
    commit_prepared,
    context::{PreparedWrite, WriteProvenance},
    electrical::topology_tests,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_connectivity::{build_create_schematic_junction, build_create_schematic_wire},
    schematic_edit::prepare_native_birth,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, Schematic, SchematicWire},
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver},
};
use std::path::Path;
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "move connected junction")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn move_to(
    model: &DesignModel,
    root: &Path,
    junction: Uuid,
    at: Point,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"object":junction,"instance_path":[],"x_nm":at.x,"y_nm":at.y}),
    )
}

#[test]
fn connected_junction_slide_moves_branch_endpoint_and_preserves_through_wire_and_history() {
    let (root, mut model, _, sheet) = topology_tests::fixture("owner_junction_slide");
    // Actual coordinates from owner QA. The branch's fixed end is Q1 base;
    // the junction lies in the interior of the continuous R1/C1 wire.
    let through = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(-2_400_000, -3_290_000),
        to: Point::new(-2_400_000, 2_690_000),
    };
    let junction = Junction {
        uuid: Uuid::new_v4(),
        position: Point::new(-2_400_000, 1_200_000),
    };
    let branch = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(1_720_000, 700_000),
        to: junction.position,
    };
    for wire in [&through, &branch] {
        let prepared = build_create_schematic_wire(&model, provenance(), sheet, wire).unwrap();
        let prepared = prepare_native_birth(&model, prepared).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let prepared = build_create_schematic_junction(&model, provenance(), sheet, &junction).unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let before = source(&model);
    let at = Point::new(-2_400_000, 700_000);
    let count = model.journal.len();
    let prepared = move_to(&model, &root, junction.uuid, at).unwrap();
    assert_eq!(model.journal.len(), count, "preparation never commits");
    assert_eq!(source(&model), before);
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let moved = source(&model);
    assert_eq!(moved.sheets[&sheet].junctions[&junction.uuid].position, at);
    assert_eq!(moved.sheets[&sheet].wires[&through.uuid], through);
    assert_eq!(moved.sheets[&sheet].wires[&branch.uuid].from, branch.from);
    assert_eq!(
        moved.sheets[&sheet].wires[&branch.uuid].to, at,
        "moving a connected junction must carry its incident endpoint"
    );
    let mut expected = before.clone();
    expected
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .junctions
        .get_mut(&junction.uuid)
        .unwrap()
        .position = at;
    expected
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .wires
        .get_mut(&branch.uuid)
        .unwrap()
        .to = at;
    assert_eq!(moved, expected, "all unowned geometry stays fixed");
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), expected);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires frozen owner baseline and moved GUI copy; read-only"]
fn owner_junction_slide_preserves_authored_identity_and_complete_membership() {
    use crate::connectivity::{partitions, physical_partitions};
    use std::collections::BTreeSet;
    let resolve = |key| {
        ProjectResolver::new(std::path::PathBuf::from(
            std::env::var_os(key).expect("owner comparison source"),
        ))
        .resolve()
        .unwrap()
    };
    let before = resolve("DATUM_JUNCTION_REPRO_ROOT");
    let after = resolve("DATUM_JUNCTION_MOVED_ROOT");
    assert_eq!(before.project.project_id, after.project.project_id);
    assert!(!before.electrical_identities.is_empty());
    assert_eq!(before.electrical_identities, after.electrical_identities);
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&before)
            .unwrap()
            .net_ids(),
        ElectricalSelectionSnapshot::capture(&after)
            .unwrap()
            .net_ids()
    );
    let old = source(&before);
    let moved = source(&after);
    assert_ne!(
        old, moved,
        "comparison must include an actual committed move"
    );
    assert_eq!(
        partitions(&old)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>(),
        partitions(&moved)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        physical_partitions(&old)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>(),
        physical_partitions(&moved)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
}
