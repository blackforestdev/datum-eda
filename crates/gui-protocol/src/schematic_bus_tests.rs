//! Shared Bus authoring keeps scalar-wire geometry and immutable source files intact.
use super::*;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent,
    PaneId,
};
use eda_engine::api::native_write::schematic_bus::WireEnd;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{Bus, BusEntry, SchematicWire};

fn basis(sheet: Uuid) -> AuthoringViewBasis {
    AuthoringViewBasis {
        project: Uuid::new_v4(),
        revision: "test-source".into(),
        pane: PaneId(1),
        content: PaneContent::Schematic,
        source: sheet,
        document: None,
    }
}
#[test]
fn bus_polyline_preview_and_member_edit_preserve_scalar_source() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let source = state.source.clone();
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    state.bus = true;
    state.wire = true;
    state.reference = "DATA".into();
    state.value = "D0, D1".into();
    state.vertices = vec![
        PointNm {
            x: 80_000_000,
            y: 80_000_000,
        },
        PointNm {
            x: 80_000_000,
            y: 90_000_000,
        },
    ];
    state.position = Some(PointNm {
        x: 90_000_000,
        y: 90_000_000,
    });
    state.base = Some(base.clone());
    let preview = state.preview_scene(&base, root).unwrap();
    let ghost = preview
        .board_graphics
        .iter()
        .find(|g| g.source_object_uuid == Uuid::nil().to_string())
        .unwrap();
    assert!(ghost.object_id.starts_with("schematic-bus:"));
    assert_eq!(ghost.path.len(), 3);
    assert_eq!(state.source, source);
    let (verb, params) = state.polyline_request().unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.create_bus");
    assert_eq!(params["members"], serde_json::json!(["D0", "D1"]));
    assert_eq!(
        params["points"].as_array().unwrap().len(),
        2,
        "hover is not a committed vertex"
    );
    assert!(params["bus"].is_null());
    state.bus = false;
    assert_eq!(
        state.polyline_request().unwrap().verb_params().0,
        "datum.schematic.draw_wire"
    );
    assert_eq!(state.cancel().unwrap(), base);
    assert!(!state.active() && !state.bus && state.bus_entry.is_none());
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    let sheet = Uuid::new_v4();
    let bus = Bus {
        uuid: Uuid::new_v4(),
        name: "DATA".into(),
        members: vec!["D0".into()],
        segments: vec![Point::new(0, 0), Point::new(0, 10_000_000)],
    };
    let draft = BusEditDraft {
        target: AuthoredSelectionIdentity {
            class: Class::BusSection,
            id: bus.uuid,
            instance_path: vec![Uuid::new_v4()],
        },
        basis: basis(sheet),
        tip: None,
        original: bus.clone(),
        entries: vec![],
    };
    assert!(draft.request("DATA", "D0").unwrap().is_none());
    for input in ["", "D0,", ",D1", "D0,D0", "D0\nD1"] {
        assert!(draft.request("DATA", input).is_err());
    }
    let (_, params) = draft
        .request("DATA", "D0, D1")
        .unwrap()
        .unwrap()
        .verb_params();
    assert!(params.get("points").is_none());
    assert!(params.get("entries").is_none());
    assert_eq!(params["bus"], bus.uuid.to_string());
    assert_eq!(
        params["instance_path"],
        serde_json::json!(draft.target.instance_path)
    );
    assert_eq!(draft.original, bus);
}

#[test]
fn bus_entry_preview_and_delete_keep_parent_wire_and_occurrence_qualified() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let sheet = state.sheet.unwrap();
    let bus = Bus {
        uuid: Uuid::new_v4(),
        name: "DATA".into(),
        members: vec!["D0".into()],
        segments: vec![Point::new(0, 0), Point::new(0, 10_000_000)],
    };
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(2_000_000, 4_000_000),
        to: Point::new(6_000_000, 4_000_000),
    };
    let source = state
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet)
        .unwrap();
    // Consumer copies supplement ordinary GUI proof; these edits never persist a design.
    source.buses.insert(bus.uuid, bus.clone());
    source.wires.insert(wire.uuid, wire.clone());
    let mut draft = BusEntryDraft {
        wire: AuthoredSelectionIdentity {
            class: Class::Wire,
            id: wire.uuid,
            instance_path: vec![Uuid::new_v4()],
        },
        basis: basis(sheet),
        tip: None,
        original: wire.clone(),
        buses: vec![bus.clone()],
        choice: 0,
        wire_end: WireEnd::From,
    };
    let at = PointNm { x: 0, y: 3_000_000 };
    let entry = draft.preview(at, source).unwrap();
    assert_eq!(entry.size, Point::new(2_000_000, 1_000_000));
    draft.cycle_end();
    assert_eq!(
        draft.preview(at, source).unwrap().size,
        Point::new(6_000_000, 1_000_000)
    );
    assert!(
        draft
            .preview(PointNm { x: 1, y: 3_000_000 }, source)
            .is_err()
    );
    let (verb, params) = draft.request(at, source).unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.create_bus");
    assert_eq!(
        params["entries"],
        serde_json::json!([{"wire":wire.uuid,"x_nm":at.x,"y_nm":at.y,"wire_end":"to"}])
    );
    for key in ["members", "name", "points"] {
        assert!(params.get(key).is_none());
    }
    source.wires.get_mut(&wire.uuid).unwrap().to.x += 1;
    assert!(draft.request(at, source).is_err());
    source.wires.insert(wire.uuid, wire);
    let entry = BusEntry {
        uuid: Uuid::new_v4(),
        ..entry
    };
    source.bus_entries.insert(entry.uuid, entry.clone());
    let edit = BusEditDraft {
        target: AuthoredSelectionIdentity {
            class: Class::BusSection,
            id: bus.uuid,
            instance_path: draft.wire.instance_path.clone(),
        },
        basis: basis(sheet),
        tip: None,
        original: bus,
        entries: vec![entry.clone()],
    };
    state.reference = "DATA".into();
    state.value = "D0".into();
    let (verb, params) = edit.delete_entry(&state).unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.delete_bus_entry");
    assert_eq!(
        params,
        serde_json::json!({"bus":entry.bus,"bus_entry":entry.uuid,"instance_path":edit.target.instance_path})
    );
    state.value = "D0,D1".into();
    assert!(edit.delete_entry(&state).is_err());
    state.value = "D0".into();
    state
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .bus_entries
        .get_mut(&entry.uuid)
        .unwrap()
        .bus = Uuid::new_v4();
    assert!(edit.delete_entry(&state).is_err());
}
