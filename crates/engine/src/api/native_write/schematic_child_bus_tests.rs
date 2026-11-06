//! Unique-child Bus/member edits use the same canonical scalar identities and history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    electrical::topology_tests,
    registry::find_native_write_verb,
    schematic_child_sheet::{self, ChildSheetRequest},
};
use crate::substrate::{CommitSource, ElectricalIdentity, ProjectResolver};
use serde_json::{Value, json};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "unique child bus")
}
fn prepare(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: Value,
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
fn net_ids(model: &DesignModel) -> BTreeSet<Uuid> {
    model
        .electrical_identities
        .values()
        .filter_map(|record| {
            matches!(
                record.identity,
                ElectricalIdentity::Net { retired: false, .. }
            )
            .then_some(record.id)
        })
        .collect()
}
#[test]
fn unique_child_bus_members_entry_and_cold_history_preserve_scalar_nets() {
    let (root, mut model, _, parent) = topology_tests::fixture("unique_child_bus");
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Bus child".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let path = vec![child.instance];
    let mut wires = Vec::new();
    for index in 0..2 {
        let write = prepare(
            &model,
            &root,
            "datum.schematic.draw_wire",
            json!({"sheet":child.sheet,
            "instance_path":path,"points":[{"x":0,"y":index*1000},{"x":100,"y":index*1000}]}),
        )
        .unwrap();
        wires.push(
            write
                .batch
                .operations
                .iter()
                .find_map(|operation| {
                    if let Operation::CreateSchematicWire { wire_id, .. } = operation {
                        Some(*wire_id)
                    } else {
                        None
                    }
                })
                .unwrap(),
        );
        commit_prepared(&mut model, &root, write).unwrap();
        let write = prepare(&model, &root, "datum.schematic.place_label", json!({"sheet":child.sheet,
            "instance_path":path,"kind":"local","name":format!("D{index}"),"x_nm":0,"y_nm":index*1000})).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let before = schematic(&model).unwrap();
    let nets = net_ids(&model);
    assert_eq!(nets.len(), 2);
    let params = |instance_path: Vec<Uuid>| {
        json!({"sheet":child.sheet,"instance_path":instance_path,
        "name":"DATA","members":["D0","D1"],"points":[{"x":1000,"y":-1000},{"x":1000,"y":5000}],
        "entries":[{"wire":wires[0],"x_nm":1000,"y_nm":100,"wire_end":"to"}]})
    };
    for wrong in [vec![], vec![Uuid::new_v4()]] {
        assert!(prepare(&model, &root, "datum.schematic.create_bus", params(wrong)).is_err());
    }
    let write = prepare(
        &model,
        &root,
        "datum.schematic.create_bus",
        params(path.clone()),
    )
    .unwrap();
    let bus = write.primary_object_id.unwrap();
    assert_eq!(
        schematic(&model).unwrap(),
        before,
        "Bus preview writes no source"
    );
    commit_prepared(&mut model, &root, write).unwrap();
    let after = schematic(&model).unwrap();
    assert_eq!(after.sheets[&parent], before.sheets[&parent]);
    assert_eq!(
        after.sheets[&child.sheet].buses[&bus].members,
        vec!["D0", "D1"]
    );
    assert_eq!(after.sheets[&child.sheet].bus_entries.len(), 1);
    assert_eq!(
        net_ids(&model),
        nets,
        "Bus grouping must never short its distinct scalar nets"
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
    assert_eq!(net_ids(&model), nets);
    let write = prepare(
        &model,
        &root,
        "datum.schematic.delete_bus",
        json!({"bus":bus,"instance_path":path}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let deleted = schematic(&model).unwrap();
    assert!(deleted.sheets[&child.sheet].buses.is_empty());
    assert!(deleted.sheets[&child.sheet].bus_entries.is_empty());
    assert_eq!(
        deleted.sheets[&child.sheet].wires,
        after.sheets[&child.sheet].wires
    );
    assert_eq!(net_ids(&model), nets);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
}
