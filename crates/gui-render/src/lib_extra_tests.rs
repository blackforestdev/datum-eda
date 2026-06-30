#![allow(clippy::wildcard_imports)]
use super::*;
use std::path::PathBuf;

#[test]
fn roundrect_ratio_changes_corner_radius() {
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 120.0,
    };
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 2_000_000,
        max_y: 1_200_000,
    };
    let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
    let small = datum_gui_protocol::PadPrimitive {
        object_id: "pad:rr-small".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "rr-small".to_string(),
        pad_uuid: "rr-small".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L1".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        },
        bounds: datum_gui_protocol::RectNm {
            min_x: 700_000,
            min_y: 350_000,
            max_x: 1_300_000,
            max_y: 850_000,
        },
        shape_kind: "roundrect".to_string(),
        roundrect_rratio_ppm: 100_000,
        mask_layer_ids: vec![],
        paste_layer_ids: vec![],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        drill_nm: None,
        rotation_degrees: 0.0,
    };
    let mut large = small.clone();
    large.pad_uuid = "rr-large".to_string();
    large.object_id = "pad:rr-large".to_string();
    large.source_object_uuid = "rr-large".to_string();
    large.roundrect_rratio_ppm = 400_000;
    let small_points = projected_pad_outline(&small, &projection, 0.0);
    let large_points = projected_pad_outline(&large, &projection, 0.0);
    assert_ne!(small_points[0], large_points[0]);
}

#[test]
fn rotated_rect_pad_produces_non_axis_aligned_geometry() {
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 120.0,
    };
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 2_000_000,
        max_y: 1_200_000,
    };
    let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
    let pad = datum_gui_protocol::PadPrimitive {
        object_id: "pad:rot".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "rot".to_string(),
        pad_uuid: "rot".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L1".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        },
        bounds: datum_gui_protocol::RectNm {
            min_x: 700_000,
            min_y: 450_000,
            max_x: 1_300_000,
            max_y: 750_000,
        },
        shape_kind: "rect".to_string(),
        roundrect_rratio_ppm: 250_000,
        mask_layer_ids: vec![],
        paste_layer_ids: vec![],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        drill_nm: None,
        rotation_degrees: 45.0,
    };

    let points = projected_pad_outline(&pad, &projection, 0.0);
    assert_eq!(points.len(), 4);
    assert!((points[0].0 - points[1].0).abs() > 0.1);
    assert!((points[0].1 - points[1].1).abs() > 0.1);
}

#[test]
fn derived_mask_pad_expands_by_clearance() {
    let pad = datum_gui_protocol::PadPrimitive {
        object_id: "pad:mask".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "mask".to_string(),
        pad_uuid: "mask".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L0".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        },
        bounds: datum_gui_protocol::RectNm {
            min_x: 900_000,
            min_y: 500_000,
            max_x: 1_100_000,
            max_y: 700_000,
        },
        shape_kind: "rect".to_string(),
        roundrect_rratio_ppm: 250_000,
        mask_layer_ids: vec!["L39".to_string()],
        paste_layer_ids: vec![],
        solder_mask_margin_nm: 25_000,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        drill_nm: None,
        rotation_degrees: 0.0,
    };
    let setup = datum_gui_protocol::ScenePadExpansionSetup {
        pad_to_mask_clearance_nm: 25_000,
        ..Default::default()
    };
    let derived = derived_process_pad(&pad, "L39", PadProcessLayerKind::Mask, &setup);
    assert_eq!(derived.layer_id, "L39");
    assert_eq!(derived.bounds.min_x, 875_000);
    assert_eq!(derived.bounds.max_x, 1_125_000);
    assert_eq!(derived.bounds.min_y, 475_000);
    assert_eq!(derived.bounds.max_y, 725_000);
    assert_eq!(derived.drill_nm, None);
}

#[test]
fn derived_paste_pad_applies_ratio_and_clearance() {
    let pad = datum_gui_protocol::PadPrimitive {
        object_id: "pad:paste".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "paste".to_string(),
        pad_uuid: "paste".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L0".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        },
        bounds: datum_gui_protocol::RectNm {
            min_x: 900_000,
            min_y: 500_000,
            max_x: 1_100_000,
            max_y: 700_000,
        },
        shape_kind: "rect".to_string(),
        roundrect_rratio_ppm: 250_000,
        mask_layer_ids: vec![],
        paste_layer_ids: vec!["L35".to_string()],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: -10_000,
        solder_paste_margin_ratio_ppm: -100_000,
        drill_nm: None,
        rotation_degrees: 0.0,
    };
    let setup = datum_gui_protocol::ScenePadExpansionSetup {
        pad_to_paste_clearance_nm: -10_000,
        pad_to_paste_ratio_ppm: -100_000,
        ..Default::default()
    };
    let derived = derived_process_pad(&pad, "L35", PadProcessLayerKind::Paste, &setup);
    assert_eq!(derived.layer_id, "L35");
    assert_eq!(derived.bounds.min_x, 920_000);
    assert_eq!(derived.bounds.max_x, 1_080_000);
    assert_eq!(derived.bounds.min_y, 520_000);
    assert_eq!(derived.bounds.max_y, 680_000);
}

#[test]
fn derived_process_pad_uses_pad_level_overrides_not_board_globals() {
    let pad = datum_gui_protocol::PadPrimitive {
        object_id: "pad:override".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "override".to_string(),
        pad_uuid: "override".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L0".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        },
        bounds: datum_gui_protocol::RectNm {
            min_x: 900_000,
            min_y: 500_000,
            max_x: 1_100_000,
            max_y: 700_000,
        },
        shape_kind: "rect".to_string(),
        roundrect_rratio_ppm: 250_000,
        mask_layer_ids: vec!["L39".to_string()],
        paste_layer_ids: vec!["L35".to_string()],
        solder_mask_margin_nm: 50_000,
        solder_paste_margin_nm: -50_000,
        solder_paste_margin_ratio_ppm: 0,
        drill_nm: None,
        rotation_degrees: 0.0,
    };
    let setup = datum_gui_protocol::ScenePadExpansionSetup {
        pad_to_mask_clearance_nm: 0,
        pad_to_paste_clearance_nm: 0,
        pad_to_paste_ratio_ppm: 0,
        ..Default::default()
    };
    let mask = derived_process_pad(&pad, "L39", PadProcessLayerKind::Mask, &setup);
    let paste = derived_process_pad(&pad, "L35", PadProcessLayerKind::Paste, &setup);
    assert_eq!(mask.bounds.min_x, 850_000);
    assert_eq!(mask.bounds.max_x, 1_150_000);
    assert_eq!(paste.bounds.min_x, 950_000);
    assert_eq!(paste.bounds.max_x, 1_050_000);
}

#[test]
fn render_stage_orders_layer_type_then_side() {
    let layers = vec![
        datum_gui_protocol::SceneLayer {
            layer_id: "L0".to_string(),
            name: "F.Cu".to_string(),
            kind: "copper".to_string(),
            render_order: 0,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "L38".to_string(),
            name: "B.Mask".to_string(),
            kind: "mask".to_string(),
            render_order: 1,
            visible_by_default: false,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "L39".to_string(),
            name: "F.Mask".to_string(),
            kind: "mask".to_string(),
            render_order: 2,
            visible_by_default: false,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "L34".to_string(),
            name: "B.Paste".to_string(),
            kind: "paste".to_string(),
            render_order: 3,
            visible_by_default: false,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "L35".to_string(),
            name: "F.Paste".to_string(),
            kind: "paste".to_string(),
            render_order: 4,
            visible_by_default: false,
        },
    ];
    assert!(scene_layer_stack_priority("L39", &layers) > scene_layer_stack_priority("L0", &layers));
    assert!(
        scene_layer_stack_priority("L35", &layers) > scene_layer_stack_priority("L39", &layers)
    );
    assert!(
        scene_layer_stack_priority("L39", &layers) > scene_layer_stack_priority("L38", &layers)
    );
    assert!(
        scene_layer_stack_priority("L35", &layers) > scene_layer_stack_priority("L34", &layers)
    );
}

#[test]
fn component_polygon_graphic_adds_fill_and_outline() {
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 160.0,
    };
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 2_400_000,
        max_y: 1_600_000,
    };
    let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
    let graphic = ComponentGraphicPrimitive {
        graphic_id: "g1".to_string(),
        component_uuid: "U1".to_string(),
        layer_id: Some("L1".to_string()),
        primitive_kind: "polygon".to_string(),
        render_role: "component_mechanical".to_string(),
        width_nm: Some(120_000),
        closed: true,
        path: vec![
            PointNm {
                x: 300_000,
                y: 300_000,
            },
            PointNm {
                x: 2_100_000,
                y: 300_000,
            },
            PointNm {
                x: 2_100_000,
                y: 1_300_000,
            },
            PointNm {
                x: 300_000,
                y: 1_300_000,
            },
        ],
        holes: Vec::new(),
    };
    let mut out = Vec::new();

    push_component_graphic_primitive(&mut out, &graphic, &projection, false, false, false);

    assert!(out.len() > 1);
}

#[test]
fn layer_appearance_distinguishes_top_and_bottom_copper() {
    let top = resolve_layer_appearance(Some("F.Cu"));
    let bottom = resolve_layer_appearance(Some("B.Cu"));

    assert_ne!(top.authored_track, bottom.authored_track);
    assert_ne!(top.proposal, bottom.proposal);
    assert_ne!(top.silkscreen, bottom.silkscreen);
}

#[test]
fn detail_tier_changes_with_projected_board_scale() {
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 1200.0,
        height: 800.0,
    };
    let fine_bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 20_000_000,
        max_y: 10_000_000,
    };
    let coarse_bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 300_000_000,
        max_y: 200_000_000,
    };

    let fine_projection = Projection::new(
        viewport,
        &fine_bounds,
        CameraState::fit_to_bounds(&fine_bounds),
    );
    let coarse_projection = Projection::new(
        viewport,
        &coarse_bounds,
        CameraState::fit_to_bounds(&coarse_bounds),
    );

    assert_eq!(detail_tier(&fine_projection), DetailTier::Fine);
    assert_eq!(detail_tier(&coarse_projection), DetailTier::Coarse);
}

#[test]
fn debug_datum_test_q1_q2_component_geometry() {
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
        board_file: Some(PathBuf::from(
            "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
        )),
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
    };
    let state = datum_gui_protocol::load_board_editor_workspace_state(&request)
        .expect("datum-test workspace should load");
    for reference in ["Q1", "Q2"] {
        let component = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == reference)
            .unwrap_or_else(|| panic!("missing component {reference}"));
        let pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid)
            .collect();
        let body = inferred_component_body_bounds(&pads);
        eprintln!(
            "{reference}: object_id={} component_uuid={} pos=({}, {}) body={body:?}",
            component.object_id,
            component.component_uuid,
            component.position.x,
            component.position.y,
        );
        for pad in pads {
            eprintln!(
                "  pad {} center=({}, {}) bounds=({}, {}, {}, {})",
                pad.object_id,
                pad.center.x,
                pad.center.y,
                pad.bounds.min_x,
                pad.bounds.min_y,
                pad.bounds.max_x,
                pad.bounds.max_y
            );
        }
    }
}

#[test]
fn inferred_component_body_geometry_handles_quarter_turn_parts() {
    let pads = vec![
        datum_gui_protocol::PadPrimitive {
            object_id: "pad:a".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "a".to_string(),
            pad_uuid: "a".to_string(),
            component_uuid: "QX".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L0".to_string()],
            center: PointNm { x: 0, y: 900_000 },
            bounds: datum_gui_protocol::RectNm {
                min_x: -250_000,
                min_y: 600_000,
                max_x: 250_000,
                max_y: 1_200_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 0,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 90.0,
        },
        datum_gui_protocol::PadPrimitive {
            object_id: "pad:b".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "b".to_string(),
            pad_uuid: "b".to_string(),
            component_uuid: "QX".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L0".to_string()],
            center: PointNm { x: 0, y: -900_000 },
            bounds: datum_gui_protocol::RectNm {
                min_x: -250_000,
                min_y: -1_200_000,
                max_x: 250_000,
                max_y: -600_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 0,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 90.0,
        },
        datum_gui_protocol::PadPrimitive {
            object_id: "pad:c".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "c".to_string(),
            pad_uuid: "c".to_string(),
            component_uuid: "QX".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L0".to_string()],
            center: PointNm { x: 800_000, y: 0 },
            bounds: datum_gui_protocol::RectNm {
                min_x: 550_000,
                min_y: -300_000,
                max_x: 1_050_000,
                max_y: 300_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 0,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 90.0,
        },
    ];
    let pad_refs: Vec<_> = pads.iter().collect();

    let (_, width, height, rotation_degrees) =
        inferred_component_body_geometry(&pad_refs, 90.0).expect("body geometry");

    assert_eq!(rotation_degrees.round() as i32, 90);
    assert!(
        height > width,
        "quarter-turn body should stay tall, got {width}x{height}"
    );
}
