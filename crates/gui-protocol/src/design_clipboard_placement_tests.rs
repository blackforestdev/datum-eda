//! Clipboard gesture tests use shared projection and typed requests, without writes.
use super::*;
use eda_engine::{
    api::native_write::design_clipboard::{ClipboardClass, ClipboardItem},
    schematic::{Schematic, Sheet},
};
use std::collections::HashMap;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
fn sheet(n: u128) -> Sheet {
    serde_json::from_value(serde_json::json!({"uuid":id(n),"name":"Paste target","frame":null,"symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap()
}
fn bundle() -> DesignClipboardBundle {
    DesignClipboardBundle {
        format: "datum.design-clipboard".into(),
        version: 1,
        source_project: id(90),
        source_revision: "copied-source".into(),
        source_sheet: id(91),
        instance_path: vec![],
        reference: Point::zero(),
        items: vec![
            ClipboardItem {
                class: ClipboardClass::Wire,
                id: id(200),
                raw: serde_json::json!({"uuid":id(200),"from":{"x":0,"y":0,"contact":{"keep":"from"}},"to":{"x":100,"y":0},"vendor":{"nested":[1,2,3]}}),
            },
            ClipboardItem {
                class: ClipboardClass::Junction,
                id: id(201),
                raw: serde_json::json!({"uuid":id(201),"position":{"x":0,"y":0},"vendor":"marker-keep"}),
            },
        ],
    }
}
fn context(child: bool) -> (SchematicAuthoring, BoardReviewSceneV1) {
    let mut root = sheet(10);
    let mut nested = sheet(11);
    for (target, wire_id) in [(&mut root, id(100)), (&mut nested, id(101))] {
        target.wires.insert(
            wire_id,
            eda_engine::schematic::SchematicWire {
                uuid: wire_id,
                from: Point::new(1000, -1000),
                to: Point::new(1000, 1000),
            },
        );
    }
    let target = if child { 11 } else { 10 };
    let path = if child { vec![id(20), id(21)] } else { vec![] };
    let mut source = Schematic {
        uuid: id(2),
        sheets: HashMap::from([(id(10), root), (id(11), nested)]),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: vec![],
    };
    if child {
        source.sheets.insert(id(12), sheet(12));
        source.sheet_definitions.insert(
            id(30),
            eda_engine::schematic::SheetDefinition {
                uuid: id(30),
                root_sheet: id(12),
                name: "Middle".into(),
            },
        );
        source.sheet_definitions.insert(
            id(31),
            eda_engine::schematic::SheetDefinition {
                uuid: id(31),
                root_sheet: id(11),
                name: "Target".into(),
            },
        );
        source.sheet_instances.insert(
            id(20),
            eda_engine::schematic::SheetInstance {
                uuid: id(20),
                definition: id(30),
                parent_sheet: Some(id(10)),
                position: Point::zero(),
                name: "Middle".into(),
                ports: vec![],
            },
        );
        source.sheet_instances.insert(
            id(21),
            eda_engine::schematic::SheetInstance {
                uuid: id(21),
                definition: id(31),
                parent_sheet: Some(id(12)),
                position: Point::zero(),
                name: "Target".into(),
                ports: vec![],
            },
        );
    }
    let document = crate::schematic_documents::SchematicDocumentKey {
        project: id(1),
        schematic: id(2),
        source_sheet: id(target),
        instance_path: path,
    };
    let mut base = crate::schematic_scene_import::project_schematic_sheet_source(
        &source,
        &source.sheets[&id(target)],
        std::path::Path::new("schematic/schematic.json"),
    )
    .unwrap();
    base.project_uuid = id(1).to_string();
    base.source_revision = "captured-revision".into();
    base.schematic_document = Some(document.clone());
    let state = SchematicAuthoring {
        source: Some(source),
        sheet: Some(id(10)),
        circuit_basis: Some(crate::AuthoringViewBasis {
            project: id(1),
            revision: "captured-revision".into(),
            pane: crate::PaneId(3),
            content: crate::PaneContent::Schematic,
            source: id(target),
            document: Some(document),
        }),
        ..Default::default()
    };
    (state, base)
}

#[test]
fn preview_and_request_share_core_geometry_seed_t_markers_and_raw_bundle() {
    let (mut state, base) = context(false);
    let source = state.source.clone();
    let input = bundle();
    let placement = DesignClipboardPlacement::capture(input.clone(), id(300)).unwrap();
    state.design_placement = Some(placement.clone());
    state.position = Some(p(900, 0));
    let preview = placement.preview_sheet(&state, p(900, 0)).unwrap();
    assert_eq!(preview.wires.len(), 2);
    assert_eq!(
        preview.junctions.len(),
        2,
        "copied marker plus deterministic endpoint T"
    );
    let request = placement.request(&state, p(900, 0)).unwrap();
    let (method, value) = request.verb_params();
    assert_eq!(method, "datum.schematic.move");
    assert_eq!(value["sheet"], serde_json::json!(id(10)));
    let instance_path: Vec<Uuid> = serde_json::from_value(value["instance_path"].clone()).unwrap();
    assert!(instance_path.is_empty());
    let edit: DesignEditIntent = serde_json::from_value(value["design_edit"].clone()).unwrap();
    let DesignEditIntent::PlaceBundle {
        bundle,
        placement: pose,
        edit_seed,
        contact_policy,
    } = edit
    else {
        panic!("place bundle")
    };
    assert_eq!(bundle, input);
    assert_eq!(
        bundle.items[0].raw["vendor"]["nested"],
        serde_json::json!([1, 2, 3])
    );
    assert_eq!(edit_seed, id(300));
    let core = preview_placement(
        &source.as_ref().unwrap().sheets[&id(10)],
        &bundle,
        &pose,
        &edit_seed,
        contact_policy,
    )
    .unwrap();
    assert_eq!(preview, core);
    let scene = state
        .preview_scene(&base, std::path::Path::new("."))
        .unwrap();
    for placed in input.placed(pose, edit_seed).unwrap() {
        assert!(
            scene
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == placed.id.to_string())
        );
    }
    assert_eq!(state.source, source);
    assert!(state.pending_request.is_none());
}

#[test]
fn clipboard_uses_common_rxy_pose_and_cancel_restores_source_without_chosen_symbol() {
    let (mut state, base) = context(false);
    let source = state.source.clone();
    state.base = Some(base.clone());
    state.design_placement = Some(DesignClipboardPlacement::capture(bundle(), id(300)).unwrap());
    assert!(state.chosen.is_none());
    assert!(state.active());
    for transform in [
        PlacementTransform::Rotate,
        PlacementTransform::FlipX,
        PlacementTransform::FlipY,
    ] {
        assert!(state.transform_placement(transform));
    }
    assert_eq!((state.rotation_deg, state.mirrored), (270, false));
    let placement = state.design_placement.as_ref().unwrap();
    let preview = placement.preview_sheet(&state, p(900, 0)).unwrap();
    assert!(
        preview
            .wires
            .values()
            .any(|wire| wire.from == Point::new(900, 0) && wire.to == Point::new(900, -100))
    );
    let SchematicEditRequest::DesignEdit {
        edit: DesignEditIntent::PlaceBundle {
            placement: pose, ..
        },
        ..
    } = placement.request(&state, p(900, 0)).unwrap()
    else {
        panic!("placement request")
    };
    assert_eq!((pose.rotation_deg, pose.mirrored), (270, false));
    assert!(state.pending_request.is_none());
    assert_eq!(state.cancel(), Some(base));
    assert!(state.design_placement.is_none());
    assert!(!state.active());
    assert_eq!(state.source, source);
    assert_eq!((state.rotation_deg, state.mirrored), (0, false));
    assert!(state.pending_request.is_none());
}

#[test]
fn qualified_child_placement_never_falls_back_to_primary_and_refuses_foreign_scene() {
    let (mut state, base) = context(true);
    let source = state.source.clone();
    let placement = DesignClipboardPlacement::capture(bundle(), id(300)).unwrap();
    state.design_placement = Some(placement.clone());
    state.position = Some(p(900, 0));
    let SchematicEditRequest::DesignEdit {
        sheet,
        instance_path,
        ..
    } = placement.request(&state, p(900, 0)).unwrap()
    else {
        panic!("design edit")
    };
    assert_eq!(sheet, id(11));
    assert_eq!(instance_path, vec![id(20), id(21)]);
    assert_eq!(state.sheet, Some(id(10)));
    let scene = state
        .preview_scene(&base, std::path::Path::new("."))
        .unwrap();
    assert_eq!(scene.schematic_document, base.schematic_document);
    assert_eq!(state.source, source);
    assert_eq!(
        state.source.as_ref().unwrap().sheets[&id(10)],
        source.as_ref().unwrap().sheets[&id(10)]
    );
    let mut foreign = base.clone();
    foreign.schematic_document.as_mut().unwrap().source_sheet = id(10);
    assert!(
        placement
            .scene_at(&state, &foreign, std::path::Path::new("."), p(900, 0))
            .is_err()
    );
    state.circuit_basis = None;
    assert!(
        placement.request(&state, p(900, 0)).is_err(),
        "qualified paste has no primary fallback"
    );
}

#[test]
fn capture_and_repeated_preview_keep_one_intent_and_reject_invalid_or_pending_input() {
    let (mut state, base) = context(false);
    let placement = DesignClipboardPlacement::capture(bundle(), id(300)).unwrap();
    state.design_placement = Some(placement.clone());
    state.position = Some(p(900, 0));
    assert_eq!(
        placement.request(&state, p(900, 0)).unwrap(),
        placement.request(&state, p(900, 0)).unwrap()
    );
    assert_eq!(
        placement.preview_sheet(&state, p(900, 0)).unwrap(),
        placement.preview_sheet(&state, p(900, 0)).unwrap()
    );
    let mut stale = base.clone();
    stale.source_revision = "old-revision".into();
    assert!(
        placement
            .scene_at(&state, &stale, std::path::Path::new("."), p(900, 0))
            .is_err()
    );
    state.pending_request = Some(placement.request(&state, p(900, 0)).unwrap());
    assert!(placement.request(&state, p(900, 0)).is_err());
    let mut invalid = bundle();
    invalid.format = "unrecognized clipboard".into();
    assert!(DesignClipboardPlacement::capture(invalid, id(300)).is_err());
}

#[test]
fn missing_clipboard_pose_keeps_base_and_recovers_without_origin_ghost() {
    for child in [false, true] {
        let (mut state, base) = context(child);
        let source = state.source.clone();
        state.design_placement =
            Some(DesignClipboardPlacement::capture(bundle(), id(300)).unwrap());
        assert!(state.active());
        assert_eq!(
            state
                .preview_scene(&base, std::path::Path::new("."))
                .unwrap(),
            base
        );
        state.position = Some(p(900, 0));
        let valid = state
            .preview_scene(&base, std::path::Path::new("."))
            .unwrap();
        assert_ne!(valid, base);
        state.circuit_preview = Some(valid);
        state.position = None;
        assert_eq!(
            state
                .preview_scene(&base, std::path::Path::new("."))
                .unwrap(),
            base
        );
        assert!(state.design_placement.is_some());
        assert_eq!(state.source, source);
        assert!(state.pending_request.is_none());
    }
}
