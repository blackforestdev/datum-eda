use super::*;
use crate::board::{BoardText, PadShape, Track, Zone};
use crate::ir::geometry::Point;

#[test]
fn render_rs274x_outline_closed_polygon() {
    let polygon = Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: 1_000_000, y: 0 },
            Point {
                x: 1_000_000,
                y: 500_000,
            },
        ],
        closed: true,
    };

    let gerber = render_rs274x_outline_default(&polygon).expect("outline should render");
    assert!(gerber.contains("%FSLAX46Y46*%"));
    assert!(gerber.contains("%MOMM*%"));
    assert!(gerber.contains("%ADD10C,0.100000*%"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
    assert!(gerber.contains("X0Y0D01*"));
    assert!(gerber.ends_with("M02*\n"));
}

#[test]
fn render_rs274x_outline_open_polygon_does_not_close() {
    let polygon = Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point {
                x: 500_000,
                y: 500_000,
            },
        ],
        closed: false,
    };

    let gerber = render_rs274x_outline_default(&polygon).expect("outline should render");
    let occurrences = gerber.matches("X0Y0").count();
    assert_eq!(occurrences, 1);
}

#[test]
fn render_rs274x_outline_requires_two_vertices() {
    let polygon = Polygon {
        vertices: vec![Point { x: 0, y: 0 }],
        closed: true,
    };

    let err = render_rs274x_outline_default(&polygon).expect_err("outline should fail");
    assert!(matches!(err, ExportError::OutlineTooShort));
}

#[test]
fn render_rs274x_copper_layer_assigns_apertures_by_width() {
    let tracks = vec![
        Track {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            from: Point { x: 0, y: 0 },
            to: Point { x: 1_000_000, y: 0 },
            width: 200_000,
            layer: 1,
        },
        Track {
            uuid: uuid::Uuid::from_u128(1),
            net: uuid::Uuid::nil(),
            from: Point { x: 0, y: 500_000 },
            to: Point {
                x: 1_000_000,
                y: 500_000,
            },
            width: 300_000,
            layer: 1,
        },
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
    let tracks = vec![Track {
        uuid: uuid::Uuid::nil(),
        net: uuid::Uuid::nil(),
        from: Point { x: 0, y: 0 },
        to: Point { x: 1, y: 1 },
        width: 0,
        layer: 1,
    }];

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
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point {
            x: 500_000,
            y: 250_000,
        },
        layer: 1,
        shape: PadShape::Circle,
        diameter: 450_000,
        width: 0,
        height: 0,
    }];

    let gerber = render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect("pad should render");
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X500000Y250000D03*"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_pad_diameter() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        shape: PadShape::Circle,
        diameter: 0,
        width: 0,
        height: 0,
    }];

    let err =
        render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect_err("pad diameter should fail");
    assert!(matches!(err, ExportError::InvalidPadDiameter));
}

#[test]
fn render_rs274x_copper_layer_emits_rectangular_pad_flashes() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point {
            x: 500_000,
            y: 250_000,
        },
        layer: 1,
        shape: PadShape::Rect,
        diameter: 0,
        width: 800_000,
        height: 400_000,
    }];

    let gerber = render_rs274x_copper_layer(1, &pads, &[], &[], &[]).expect("pad should render");
    assert!(gerber.contains("%ADD10R,0.800000X0.400000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X500000Y250000D03*"));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_rectangular_pad_width() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        shape: PadShape::Rect,
        diameter: 0,
        width: 0,
        height: 400_000,
    }];

    let err = render_rs274x_copper_layer(1, &pads, &[], &[], &[])
        .expect_err("pad rectangle width should fail");
    assert!(matches!(err, ExportError::InvalidPadWidth));
}

#[test]
fn render_rs274x_copper_layer_rejects_non_positive_rectangular_pad_height() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        shape: PadShape::Rect,
        diameter: 0,
        width: 400_000,
        height: 0,
    }];

    let err = render_rs274x_copper_layer(1, &pads, &[], &[], &[])
        .expect_err("pad rectangle height should fail");
    assert!(matches!(err, ExportError::InvalidPadHeight));
}

#[test]
fn render_rs274x_soldermask_layer_emits_pad_flashes() {
    let pads = vec![
        PlacedPad {
            uuid: uuid::Uuid::nil(),
            package: uuid::Uuid::from_u128(42),
            name: "1".to_string(),
            net: None,
            position: Point {
                x: 500_000,
                y: 250_000,
            },
            layer: 1,
            shape: PadShape::Circle,
            diameter: 450_000,
            width: 0,
            height: 0,
        },
        PlacedPad {
            uuid: uuid::Uuid::from_u128(1),
            package: uuid::Uuid::from_u128(42),
            name: "2".to_string(),
            net: None,
            position: Point {
                x: 900_000,
                y: 250_000,
            },
            layer: 1,
            shape: PadShape::Rect,
            diameter: 0,
            width: 800_000,
            height: 400_000,
        },
    ];

    let gerber = render_rs274x_soldermask_layer(2, &pads).expect("mask should render");
    assert!(gerber.contains("G04 datum-eda native soldermask layer 2*"));
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("%ADD11R,0.800000X0.400000*%"));
    assert!(gerber.contains("X500000Y250000D03*"));
    assert!(gerber.contains("X900000Y250000D03*"));
}

#[test]
fn render_rs274x_soldermask_layer_rejects_non_positive_rectangular_pad_width() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        shape: PadShape::Rect,
        diameter: 0,
        width: 0,
        height: 400_000,
    }];

    let err =
        render_rs274x_soldermask_layer(2, &pads).expect_err("mask rectangle width should fail");
    assert!(matches!(err, ExportError::InvalidPadWidth));
}

#[test]
fn render_rs274x_paste_layer_emits_pad_flashes() {
    let pads = vec![
        PlacedPad {
            uuid: uuid::Uuid::nil(),
            package: uuid::Uuid::from_u128(42),
            name: "1".to_string(),
            net: None,
            position: Point {
                x: 500_000,
                y: 250_000,
            },
            layer: 1,
            shape: PadShape::Circle,
            diameter: 450_000,
            width: 0,
            height: 0,
        },
        PlacedPad {
            uuid: uuid::Uuid::from_u128(1),
            package: uuid::Uuid::from_u128(42),
            name: "2".to_string(),
            net: None,
            position: Point {
                x: 900_000,
                y: 250_000,
            },
            layer: 1,
            shape: PadShape::Rect,
            diameter: 0,
            width: 800_000,
            height: 400_000,
        },
    ];

    let gerber = render_rs274x_paste_layer(3, &pads).expect("paste should render");
    assert!(gerber.contains("G04 datum-eda native paste layer 3*"));
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("%ADD11R,0.800000X0.400000*%"));
    assert!(gerber.contains("X500000Y250000D03*"));
    assert!(gerber.contains("X900000Y250000D03*"));
}

#[test]
fn render_rs274x_paste_layer_rejects_non_positive_rectangular_pad_width() {
    let pads = vec![PlacedPad {
        uuid: uuid::Uuid::nil(),
        package: uuid::Uuid::from_u128(42),
        name: "1".to_string(),
        net: None,
        position: Point { x: 0, y: 0 },
        layer: 1,
        shape: PadShape::Rect,
        diameter: 0,
        width: 0,
        height: 400_000,
    }];

    let err = render_rs274x_paste_layer(3, &pads).expect_err("paste rectangle width should fail");
    assert!(matches!(err, ExportError::InvalidPadWidth));
}

#[test]
fn render_rs274x_mechanical_layer_emits_closed_keepout_regions() {
    let polygons = vec![Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: 1_000_000, y: 0 },
            Point {
                x: 1_000_000,
                y: 500_000,
            },
        ],
        closed: true,
    }];

    let gerber =
        render_rs274x_mechanical_layer(41, &polygons, &[]).expect("mechanical should render");
    assert!(gerber.contains("G04 datum-eda native mechanical layer 41*"));
    assert!(gerber.contains("%ADD10C,0.100000*%"));
    assert!(gerber.contains("G36*"));
    assert!(gerber.contains("G37*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
}

#[test]
fn render_rs274x_mechanical_layer_emits_component_strokes() {
    let strokes = vec![MechanicalStroke {
        from: Point {
            x: 100_000,
            y: 200_000,
        },
        to: Point {
            x: 900_000,
            y: 200_000,
        },
        width_nm: 150_000,
    }];

    let gerber =
        render_rs274x_mechanical_layer(41, &[], &strokes).expect("mechanical should render");
    assert!(gerber.contains("%ADD10C,0.100000*%"));
    assert!(gerber.contains("%ADD11C,0.150000*%"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X100000Y200000D02*"));
    assert!(gerber.contains("X900000Y200000D01*"));
}

#[test]
fn render_rs274x_mechanical_layer_rejects_non_positive_stroke_width() {
    let strokes = vec![MechanicalStroke {
        from: Point { x: 0, y: 0 },
        to: Point { x: 1, y: 1 },
        width_nm: 0,
    }];

    let err = render_rs274x_mechanical_layer(41, &[], &strokes)
        .expect_err("mechanical stroke width should fail");
    assert!(matches!(err, ExportError::InvalidTrackWidth));
}

#[test]
fn render_rs274x_silkscreen_layer_emits_text_strokes() {
    let texts = vec![BoardText {
        uuid: uuid::Uuid::nil(),
        text: "TOP".to_string(),
        position: Point {
            x: 1_000_000,
            y: 2_000_000,
        },
        rotation: 0,
        layer: 3,
        height_nm: 1_000_000,
        stroke_width_nm: 120_000,
    }];

    let gerber = render_rs274x_silkscreen_layer(3, &texts, &[]).expect("silkscreen should render");
    assert!(gerber.contains("G04 datum-eda native silkscreen layer 3*"));
    assert!(gerber.contains("%ADD10C,0.120000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("D02*"));
    assert!(gerber.contains("D01*"));
}

#[test]
fn render_rs274x_silkscreen_layer_emits_component_strokes() {
    let strokes = vec![SilkscreenStroke {
        from: Point { x: 0, y: 0 },
        to: Point { x: 1_000_000, y: 0 },
        width_nm: 150_000,
    }];

    let gerber =
        render_rs274x_silkscreen_layer(3, &[], &strokes).expect("silkscreen should render");
    assert!(gerber.contains("G04 datum-eda native silkscreen layer 3*"));
    assert!(gerber.contains("%ADD10C,0.150000*%"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
}

#[test]
fn render_rs274x_silkscreen_layer_rejects_unsupported_character() {
    let texts = vec![BoardText {
        uuid: uuid::Uuid::nil(),
        text: "@".to_string(),
        position: Point { x: 0, y: 0 },
        rotation: 0,
        layer: 3,
        height_nm: 1_000_000,
        stroke_width_nm: 120_000,
    }];

    let err = render_rs274x_silkscreen_layer(3, &texts, &[]).expect_err("unsupported character");
    assert!(matches!(
        err,
        ExportError::UnsupportedSilkscreenTextCharacter('@')
    ));
}

#[test]
fn render_excellon_drill_assigns_tools_by_drill() {
    let vias = vec![
        Via {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            position: Point {
                x: 1_000_000,
                y: 1_500_000,
            },
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 2,
        },
        Via {
            uuid: uuid::Uuid::from_u128(1),
            net: uuid::Uuid::nil(),
            position: Point {
                x: 2_000_000,
                y: 3_000_000,
            },
            drill: 350_000,
            diameter: 700_000,
            from_layer: 1,
            to_layer: 2,
        },
    ];

    let excellon = render_excellon_drill(&vias).expect("drill should render");
    assert!(excellon.contains("M48"));
    assert!(excellon.contains("METRIC,TZ"));
    assert!(excellon.contains("T01C0.300000"));
    assert!(excellon.contains("T02C0.350000"));
    assert!(excellon.contains("T01\nX1.000000Y1.500000"));
    assert!(excellon.contains("T02\nX2.000000Y3.000000"));
    assert!(excellon.ends_with("M30\n"));
}

#[test]
fn render_excellon_drill_rejects_non_positive_drill() {
    let vias = vec![Via {
        uuid: uuid::Uuid::nil(),
        net: uuid::Uuid::nil(),
        position: Point { x: 0, y: 0 },
        drill: 0,
        diameter: 600_000,
        from_layer: 1,
        to_layer: 2,
    }];

    let err = render_excellon_drill(&vias).expect_err("drill should fail");
    assert!(matches!(err, ExportError::InvalidViaDrill));
}
