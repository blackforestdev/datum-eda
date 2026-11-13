use super::*;

#[test]
fn render_rs274x_copper_layer_assigns_apertures_by_width() {
    let tracks = vec![
        Track::straight(
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            Point { x: 0, y: 0 },
            Point { x: 1_000_000, y: 0 },
            200_000,
            1,
        ),
        Track::straight(
            uuid::Uuid::from_u128(1),
            uuid::Uuid::nil(),
            Point { x: 0, y: 500_000 },
            Point {
                x: 1_000_000,
                y: 500_000,
            },
            300_000,
            1,
        ),
    ];

    let gerber =
        render_rs274x_copper_layer(1, &[], &tracks, &[], &[]).expect("copper should render");
    assert!(gerber.contains("%ADD10C,0.200000*%"));
    assert!(gerber.contains("%ADD11C,0.300000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.ends_with("M02*\n"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_width() {
    let tracks = vec![Track::straight(
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        Point { x: 0, y: 0 },
        Point { x: 1, y: 1 },
        0,
        1,
    )];

    let err =
        render_rs274x_copper_layer(1, &[], &tracks, &[], &[]).expect_err("copper should fail");
    assert!(matches!(err, ExportError::InvalidTrackWidth));
}

#[test]
fn render_rs274x_copper_layer_emits_zone_region() {
    let zones = vec![Zone {
        uuid: uuid::Uuid::nil(),
        net: uuid::Uuid::nil(),
        polygon: Polygon {
            vertices: vec![
                Point { x: 0, y: 0 },
                Point { x: 1_000_000, y: 0 },
                Point {
                    x: 1_000_000,
                    y: 500_000,
                },
            ],
            closed: true,
        },
        layer: 1,
        priority: 1,
        thermal_relief: true,
        thermal_gap: 0,
        thermal_spoke_width: 0,
    }];

    let gerber = render_rs274x_copper_layer(1, &[], &[], &zones, &[]).expect("zone should render");
    assert!(gerber.contains("G36*"));
    assert!(gerber.contains("G37*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
}

#[test]
fn render_rs274x_copper_layer_emits_via_flashes() {
    let vias = vec![Via {
        uuid: uuid::Uuid::nil(),
        net: uuid::Uuid::nil(),
        position: Point {
            x: 250_000,
            y: 750_000,
        },
        drill: 300_000,
        diameter: 600_000,
        from_layer: 1,
        to_layer: 2,
    }];

    let gerber = render_rs274x_copper_layer(1, &[], &[], &[], &vias).expect("via should render");
    assert!(gerber.contains("%ADD10C,0.600000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X250000Y750000D03*"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_via_diameter() {
    let vias = vec![Via {
        uuid: uuid::Uuid::nil(),
        net: uuid::Uuid::nil(),
        position: Point { x: 0, y: 0 },
        drill: 300_000,
        diameter: 0,
        from_layer: 1,
        to_layer: 2,
    }];

    let err =
        render_rs274x_copper_layer(1, &[], &[], &[], &vias).expect_err("via diameter should fail");
    assert!(matches!(err, ExportError::InvalidViaDiameter));
}

#[test]
fn render_rs274x_copper_layer_emits_pad_flashes() {
    let pads = vec![PlacedPad {
        layer_connection: Default::default(),
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point {
            x: 500_000,
            y: 250_000,
        },
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter: 450_000,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }];

    let gerber = render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect("pad should render");
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X500000Y250000D03*"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_pad_diameter() {
    let pads = vec![PlacedPad {
        layer_connection: Default::default(),
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter: 0,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }];

    let err =
        render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect_err("pad diameter should fail");
    assert!(matches!(err, ExportError::InvalidPadDiameter));
}

#[test]
fn render_rs274x_copper_layer_emits_rectangular_pad_flashes() {
    let pads = vec![PlacedPad {
        layer_connection: Default::default(),
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point {
            x: 500_000,
            y: 250_000,
        },
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Rect,
        diameter: 0,
        width: 800_000,
        height: 400_000,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }];

    let gerber = render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect("pad should render");
    assert!(gerber.contains("%ADD10R,0.800000X0.400000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X500000Y250000D03*"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_rectangular_pad_width() {
    let pads = vec![PlacedPad {
        layer_connection: Default::default(),
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Rect,
        diameter: 0,
        width: 0,
        height: 400_000,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }];

    let err = render_rs274x_copper_layer(1, &pads, &[], &[], &[])
        .expect_err("pad rectangle width should fail");
    assert!(matches!(err, ExportError::InvalidPadWidth));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_rectangular_pad_height() {
    let pads = vec![PlacedPad {
        layer_connection: Default::default(),
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        copper_layers: Vec::new(),
        shape: PadShape::Rect,
        diameter: 0,
        width: 400_000,
        height: 0,
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    }];

    let err = render_rs274x_copper_layer(1, &pads, &[], &[], &[])
        .expect_err("pad rectangle height should fail");
    assert!(matches!(err, ExportError::InvalidPadHeight));
}

#[test]
fn rational_center_refusal_carries_source_bound_error_witness() {
    let mut track = Track::straight(
        uuid::Uuid::from_u128(123),
        uuid::Uuid::nil(),
        Point::new(0, 0),
        Point::new(3, 0),
        1,
        1,
    );
    track.midpoint = Some(Point::new(1, 3));
    let err = render_rs274x_copper_layer(1, &[], &[track.clone()], &[], &[]).unwrap_err();
    match err {
        ExportError::DeferredArcManufacturing { source_id, witness } => {
            assert_eq!(source_id, track.uuid);
            assert_eq!(witness.source(), &track);
            assert_ne!(witness.disposition(), ArcOutputDisposition::Exact);
        }
        other => panic!("unexpected {other:?}"),
    }
    // Refusal remains unpassed mandatory positive T04 coverage.
}

fn shape_regression_pad(shape: PadShape, rotation: i32, ratio: u32) -> PlacedPad {
    serde_json::from_value(serde_json::json!({
        "uuid": uuid::Uuid::from_u128(1601),
        "package": uuid::Uuid::nil(), "name": "CA16", "net": null,
        "position": {"x": 500000, "y": 250000}, "layer": 1,
        "shape": shape, "diameter": 1000000, "width": 2000000, "height": 1000000,
        "rotation": rotation, "roundrect_rratio_ppm": ratio
    }))
    .unwrap()
}

#[test]
fn copper_refuses_oval_and_roundrect_in_every_degree_without_rectangle_substitution() {
    for shape in [PadShape::Oval, PadShape::RoundRect] {
        for ratio in [0, 1, 250_000, 500_000] {
            for rotation in (-360..=360).chain([i32::MIN, i32::MAX]) {
                let pad = shape_regression_pad(shape, rotation, ratio);
                // A valid accepted pad precedes the unsupported pad: no partial
                // layer may escape when unsupported geometry is encountered.
                let mut circle = shape_regression_pad(PadShape::Circle, 0, ratio);
                circle.uuid = uuid::Uuid::from_u128(1600);
                let id = pad.uuid;
                let pads = [circle, pad];
                let error = render_rs274x_copper_layer(1, &pads, &[], &[], &[])
                    .expect_err("authored shape must not silently become rectangular copper");
                assert!(
                    matches!(error, ExportError::UnsupportedPadShape { source_id, shape: actual }
                    if source_id == id && actual == shape),
                    "shape={shape:?} rotation={rotation} ratio={ratio}: {error:?}"
                );
            }
        }
    }
}

#[test]
fn copper_retains_circle_and_quarter_turn_rectangle_geometry() {
    for rotation in [0, 90, 180, 270, -90, 450] {
        let pad = shape_regression_pad(PadShape::Rect, rotation, 250_000);
        let original = pad.clone();
        let gerber = render_rs274x_copper_layer(1, std::slice::from_ref(&pad), &[], &[], &[])
            .expect("quarter-turn rectangles remain supported");
        let dimensions = if rotation.rem_euclid(180) == 0 {
            "2.000000X1.000000"
        } else {
            "1.000000X2.000000"
        };
        assert!(gerber.contains(&format!("%ADD10R,{dimensions}*%")));
        assert!(gerber.contains("X500000Y250000D03*"));
        assert_eq!(pad, original);
    }
    for rotation in [0, 37, 90, -90, i32::MIN, i32::MAX] {
        let pad = shape_regression_pad(PadShape::Circle, rotation, 250_000);
        let gerber = render_rs274x_copper_layer(1, &[pad], &[], &[], &[]).unwrap();
        assert!(gerber.contains("%ADD10C,1.000000*%"));
        assert!(!gerber.contains("%ADD10R"));
    }
}

#[test]
fn copper_retains_arbitrary_rectangle_rotation_refusal() {
    for rotation in [1, 37, 89, 91, -37, i32::MIN, i32::MAX] {
        let pad = shape_regression_pad(PadShape::Rect, rotation, 250_000);
        let id = pad.uuid;
        let error = render_rs274x_copper_layer(1, &[pad], &[], &[], &[]).unwrap_err();
        assert!(
            matches!(error, ExportError::UnsupportedPadRotation { source_id, rotation: actual }
            if source_id == id && actual == rotation)
        );
    }
}
