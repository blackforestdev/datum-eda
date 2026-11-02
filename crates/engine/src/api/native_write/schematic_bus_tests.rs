//! Real authored scalar nets remain separate through native bus declaration and history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    electrical::topology_tests::fixture,
    registry::find_native_write_verb,
    schematic_connectivity::{build_create_schematic_label, build_create_schematic_wire},
};
use crate::schematic::{LabelKind, NetLabel, Schematic, SchematicWire};
use crate::substrate::{CommitSource, DesignModel, ElectricalIdentity, ProjectResolver};
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "native bus authoring")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn build(
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
fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, Vec<SchematicWire>) {
    let (root, mut model, _, sheet) = fixture(name);
    let mut wires = Vec::new();
    for index in 0..4 {
        let wire = SchematicWire {
            uuid: Uuid::new_v4(),
            from: Point::new(0, index * 1000),
            to: Point::new(100, index * 1000),
        };
        let label = NetLabel {
            uuid: Uuid::new_v4(),
            name: format!("D{index}"),
            kind: LabelKind::Local,
            position: wire.from,
        };
        let mut write = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
        write.batch.operations.extend(
            build_create_schematic_label(&model, provenance(), sheet, &label)
                .unwrap()
                .batch
                .operations,
        );
        let write = prepare_native_birth(&model, write).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        wires.push(wire);
    }
    (root, model, sheet, wires)
}
fn params(sheet: Uuid) -> serde_json::Value {
    serde_json::json!({"sheet":sheet,"instance_path":[],"name":"DATA","members":["D0","D1","D2","D3"],"points":[{"x":1000,"y":-1000},{"x":1000,"y":5000}]})
}
fn tap(wire: Uuid) -> serde_json::Value {
    serde_json::json!({"wire":wire,"x_nm":1000,"y_nm":100,"wire_end":"to"})
}
fn net_ids(model: &DesignModel) -> BTreeSet<Uuid> {
    model
        .electrical_identities
        .values()
        .filter_map(|r| {
            matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }).then_some(r.id)
        })
        .collect()
}
fn history(model: &mut DesignModel, root: &Path, redo: bool) {
    if redo {
        model
            .commit_journal_redo(root, provenance().into())
            .unwrap();
    } else {
        model
            .commit_journal_undo(root, provenance().into())
            .unwrap();
    }
    *model = ProjectResolver::new(root).resolve().unwrap();
}
#[test]
fn native_bus_four_real_members_entry_history_and_delete_never_short_scalar_nets() {
    let (root, mut model, sheet, wires) = setup("native_bus_members");
    let before = source(&model);
    let nets = net_ids(&model);
    assert_eq!(nets.len(), 4);
    let mut p = params(sheet);
    p["entries"] = serde_json::json!([tap(wires[0].uuid), tap(wires[1].uuid)]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus = write.primary_object_id.unwrap();
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before,
        "prepare/preview has no writes"
    );
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let placed = source(&model);
    assert_eq!(placed.sheets[&sheet].bus_entries.len(), 2);
    let entry = placed.sheets[&sheet]
        .bus_entries
        .values()
        .find(|e| e.wire == Some(wires[0].uuid))
        .unwrap()
        .clone();
    assert_eq!(entry.bus, bus);
    assert_eq!(entry.wire, Some(wires[0].uuid));
    assert_eq!(entry.position, Point::new(1000, 100));
    assert_eq!(entry.size, Point::new(-900, -100));
    assert_eq!(net_ids(&model), nets);
    assert_eq!(crate::connectivity::schematic_net_info(&placed).len(), 4);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let membership = snapshot
        .bus_for_origin(snapshot.revision(), &members::origin("buses", bus, &[]))
        .unwrap();
    assert_eq!(membership.related_subjects, nets);
    let wire_net = snapshot
        .schematic_run(
            snapshot.revision(),
            &members::origin("wires", wires[0].uuid, &[]),
        )
        .unwrap();
    assert_eq!(
        wire_net.owned.iter().filter(|o| o.class == "wires").count(),
        1
    );
    assert!(
        wire_net
            .owned
            .contains(&members::origin("wires", wires[0].uuid, &[]))
    );
    assert!(
        !wire_net
            .owned
            .iter()
            .any(|o| o.class == "buses" || o.class == "bus_entries")
    );
    let bus_identity = membership.subject_id;
    let declaration = model.electrical_identities[&bus_identity].clone();
    drop(snapshot);
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(net_ids(&model), nets);
    history(&mut model, &root, true);
    assert_eq!(source(&model), placed);
    assert_eq!(net_ids(&model), nets);
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus",
        serde_json::json!({"bus":bus,"instance_path":[]}),
    )
    .unwrap();
    // One qualified parent deletion removes the complete owned source subject.
    assert_eq!(source(&model), placed, "preparation must not write");
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    assert_eq!(net_ids(&model), nets);
    assert!(
        matches!(&model.electrical_identities[&bus_identity].identity,
        ElectricalIdentity::Bus { representations, retired: false, .. } if representations.is_empty())
    );
    history(&mut model, &root, false);
    assert_eq!(
        source(&model),
        placed,
        "one undo restores spine and entry UUIDs and geometry"
    );
    assert_eq!(model.electrical_identities[&bus_identity], declaration);
    assert_eq!(net_ids(&model), nets);
    history(&mut model, &root, true);
    assert_eq!(source(&model), before);
    assert_eq!(net_ids(&model), nets);
    history(&mut model, &root, false);
    assert_eq!(source(&model), placed);
    // Individual child deletion remains available independently.
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus_entry",
        serde_json::json!({"bus":bus,"bus_entry":entry.uuid,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let without_entry = source(&model);
    history(&mut model, &root, false);
    assert_eq!(source(&model), placed);
    history(&mut model, &root, true);
    assert_eq!(source(&model), without_entry);
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus",
        serde_json::json!({"bus":bus,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    assert_eq!(net_ids(&model), nets);
    assert!(
        matches!(&model.electrical_identities[&bus_identity].identity,ElectricalIdentity::Bus{representations,retired:false,..} if representations.is_empty()),
        "deleting spine preserves semantic declaration"
    );
    history(&mut model, &root, false);
    assert_eq!(source(&model), without_entry);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_bus_members_edit_and_entry_alias_preserve_source_identity_and_geometry() {
    let (root, mut model, sheet, wires) = setup("native_bus_edit");
    let write = build(&model, &root, "datum.schematic.create_bus", params(sheet)).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let original = source(&model).sheets[&sheet].buses[&id].clone();
    let write = build(
        &model,
        &root,
        "datum.schematic.create_bus",
        serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"members":["D0","D1"]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let edited = source(&model).sheets[&sheet].buses[&id].clone();
    assert_eq!(
        (edited.uuid, edited.name.clone(), edited.segments.clone()),
        (
            original.uuid,
            original.name.clone(),
            original.segments.clone()
        )
    );
    assert_eq!(edited.members, vec!["D0", "D1"]);
    assert_eq!(net_ids(&model).len(), 4);
    history(&mut model, &root, false);
    assert_eq!(source(&model).sheets[&sheet].buses[&id], original);
    history(&mut model, &root, true);
    assert_eq!(source(&model).sheets[&sheet].buses[&id], edited);
    let entry = serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"entries":[tap(wires[0].uuid)]});
    let write = build(
        &model,
        &root,
        "datum.schematic.place_bus_entry",
        entry.clone(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    assert!(
        build(&model, &root, "datum.schematic.place_bus_entry", entry).is_err(),
        "duplicate entry refuses"
    );
    let remove_bound =
        serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"members":["D1"]});
    assert!(
        build(&model, &root, "datum.schematic.create_bus", remove_bound)
            .unwrap_err()
            .to_string()
            .contains("declared scalar")
    );
    let wrong_member = serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"entries":[tap(wires[2].uuid)]});
    assert!(build(&model, &root, "datum.schematic.create_bus", wrong_member).is_err());
    assert_eq!(source(&model), before);
    let mut same_named = params(sheet);
    same_named["members"] = serde_json::json!(["D0", "D1"]);
    let write = build(&model, &root, "datum.schematic.create_bus", same_named).unwrap();
    let other_bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let one = snapshot
        .bus_for_origin(snapshot.revision(), &members::origin("buses", id, &[]))
        .unwrap();
    let two = snapshot
        .bus_for_origin(
            snapshot.revision(),
            &members::origin("buses", other_bus, &[]),
        )
        .unwrap();
    assert_ne!(
        one.subject_id, two.subject_id,
        "equal names/members do not equate declared Buses"
    );
    let entry_id = source(&model).sheets[&sheet]
        .bus_entries
        .values()
        .next()
        .unwrap()
        .uuid;
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.delete_bus_entry",
            serde_json::json!({"bus":other_bus,"bus_entry":entry_id,"instance_path":[]})
        )
        .unwrap_err()
        .to_string()
        .contains("does not belong")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_bus_rejects_missing_members_wrong_source_occurrence_geometry_and_stale_edits() {
    let (root, mut model, sheet, wires) = setup("native_bus_guards");
    let before = source(&model);
    for (field, value) in [
        ("members", serde_json::json!(["MISSING"])),
        ("members", serde_json::json!(["D0", "D0"])),
        ("members", serde_json::json!([])),
        ("name", serde_json::json!("")),
        ("points", serde_json::json!([{"x":0,"y":0},{"x":0,"y":0}])),
        ("sheet", serde_json::json!(Uuid::new_v4())),
        ("instance_path", serde_json::json!([Uuid::new_v4()])),
    ] {
        let mut p = params(sheet);
        p[field] = value;
        assert!(
            build(&model, &root, "datum.schematic.create_bus", p).is_err(),
            "{field}"
        );
    }
    assert_eq!(source(&model), before);
    let write = build(&model, &root, "datum.schematic.create_bus", params(sheet)).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut entry = serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"entries":[tap(wires[0].uuid)]});
    entry["entries"][0]["x_nm"] = serde_json::json!(999);
    assert!(
        build(&model, &root, "datum.schematic.create_bus", entry)
            .unwrap_err()
            .to_string()
            .contains("exactly")
    );
    for (field, value) in [
        ("sheet", serde_json::json!(Uuid::new_v4())),
        ("bus", serde_json::json!(Uuid::new_v4())),
        ("instance_path", serde_json::json!([Uuid::new_v4()])),
        ("instance_path", serde_json::Value::Null),
    ] {
        let mut p = serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"name":"EDIT"});
        p[field] = value;
        assert!(
            build(&model, &root, "datum.schematic.create_bus", p).is_err(),
            "{field}"
        );
    }
    let request = serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"name":"FIRST"});
    let stale = build(&model, &root, "datum.schematic.create_bus", request).unwrap();
    let write = build(
        &model,
        &root,
        "datum.schematic.create_bus",
        serde_json::json!({"sheet":sheet,"bus":id,"instance_path":[],"name":"CURRENT"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let rev = model.model_revision.clone();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(source(&model), before);
    assert_eq!(model.model_revision, rev);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    // Different display names on one actual scalar Net must not declare two members.
    let alias = NetLabel {
        uuid: Uuid::new_v4(),
        name: "ALIAS".into(),
        kind: LabelKind::Local,
        position: wires[0].from,
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &alias).unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut duplicate = params(sheet);
    duplicate["members"] = serde_json::json!(["D0", "ALIAS"]);
    assert!(
        build(&model, &root, "datum.schematic.create_bus", duplicate)
            .unwrap_err()
            .to_string()
            .contains("same scalar Net")
    );
    // Same spelling in distinct Local/Global scopes is ambiguous, not permission to short.
    let scoped = NetLabel {
        uuid: Uuid::new_v4(),
        name: "D0".into(),
        kind: LabelKind::Global,
        position: Point::new(9000, 9000),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &scoped).unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let ids = net_ids(&model);
    assert!(
        build(&model, &root, "datum.schematic.create_bus", params(sheet))
            .unwrap_err()
            .to_string()
            .contains("one existing scalar Net")
    );
    assert_eq!(source(&model), before);
    assert_eq!(net_ids(&model), ids);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_bus_entry_geometry_is_exact_endpoint_bound_and_overflow_checked() {
    let bus = Bus {
        uuid: Uuid::new_v4(),
        name: "B".into(),
        members: vec!["D0".into()],
        segments: vec![Point::new(17, 0), Point::new(17, 100)],
    };
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(1, 2),
        to: Point::new(3, 4),
    };
    assert_eq!(
        entry_geometry(&bus, &wire, Point::new(17, 23), WireEnd::From).unwrap(),
        Point::new(-16, -21)
    );
    assert_eq!(
        entry_geometry(&bus, &wire, Point::new(17, 23), WireEnd::To).unwrap(),
        Point::new(-14, -19)
    );
    assert!(entry_geometry(&bus, &wire, Point::new(18, 23), WireEnd::To).is_err());
    let extreme = Bus {
        segments: vec![Point::new(i64::MIN, 0), Point::new(i64::MIN, 100)],
        ..bus
    };
    assert!(entry_geometry(&extreme, &wire, Point::new(i64::MIN, 20), WireEnd::To).is_err());
}

#[test]
fn native_bus_entry_refuses_real_foreign_sheet_wire() {
    use crate::api::native_write::schematic_sheets::build_create_schematic_sheet;
    let (root, mut model, sheet, _) = setup("native_bus_foreign_wire");
    let write = build(&model, &root, "datum.schematic.create_bus", params(sheet)).unwrap();
    let bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let other_sheet = Uuid::new_v4();
    let write = build_create_schematic_sheet(&model,provenance(),source(&model).uuid,other_sheet,"sheets/other.json",
        serde_json::json!({"schema_version":1,"uuid":other_sheet,"name":"Other","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(100, 0),
    };
    let write = build_create_schematic_wire(&model, provenance(), other_sheet, &wire).unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let revision = model.model_revision.clone();
    let error = build(
        &model,
        &root,
        "datum.schematic.create_bus",
        serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":[],"entries":[tap(wire.uuid)]}),
    )
    .unwrap_err();
    assert!(error.to_string().contains("different source sheet"));
    assert_eq!(source(&model), before);
    assert_eq!(model.model_revision, revision);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}
