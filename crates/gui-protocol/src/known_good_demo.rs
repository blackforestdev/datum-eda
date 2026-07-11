//! The known-good demo project writer (decision 022 source-health extraction —
//! behavior-preserving move out of `lib.rs`).
//!
//! Owns writing the deterministic Route-Path-Candidate demo project skeleton
//! (`project.json` + schematic/board/rules shards) to a scratch root, then
//! delegating footprint silkscreen/mechanical geometry to the `kicad_geometry`
//! child. This is the fixture the GUI's "known good" review request materializes;
//! it is not a native authoring path and is never journaled.

mod kicad_geometry;

use anyhow::{Context, Result};
use std::path::Path;

pub(crate) fn write_known_good_demo_project(root: &Path) -> Result<()> {
    let schematic_dir = root.join("schematic");
    let board_dir = root.join("board");
    let rules_dir = root.join("rules");
    std::fs::create_dir_all(&schematic_dir)
        .with_context(|| format!("failed to create {}", schematic_dir.display()))?;
    std::fs::create_dir_all(&board_dir)
        .with_context(|| format!("failed to create {}", board_dir.display()))?;
    std::fs::create_dir_all(&rules_dir)
        .with_context(|| format!("failed to create {}", rules_dir.display()))?;

    write_json_file(
        &root.join("project.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c100",
            "name": "Datum GUI Known Good",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json",
            "forward_annotation_review": {}
        }),
    )?;
    write_json_file(
        &schematic_dir.join("schematic.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c101",
            "sheets": {},
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": []
        }),
    )?;
    write_json_file(
        &rules_dir.join("rules.json"),
        &serde_json::json!({
            "schema_version": 1,
            "rules": []
        }),
    )?;
    write_json_file(
        &board_dir.join("board.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c207",
            "name": "Route Path Candidate Proposal Artifact Demo Board",
            "stackup": {
                "layers": [
                    { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                    { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                    { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                ]
            },
            "outline": {
                "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 24000000, "y": 0 },
                    { "x": 24000000, "y": 14000000 },
                    { "x": 0, "y": 14000000 }
                ],
                "closed": true
            },
            "packages": {
                "00000000-0000-0000-0000-00000000c203": {
                    "uuid": "00000000-0000-0000-0000-00000000c203",
                    "package": "10000000-0000-0000-0000-00000000c203",
                    "part": "20000000-0000-0000-0000-00000000c203",
                    "reference": "U1",
                    "value": "SOIC-8_3.9x4.9mm_P1.27mm",
                    "position": { "x": 4500000, "y": 3365000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c204": {
                    "uuid": "00000000-0000-0000-0000-00000000c204",
                    "package": "10000000-0000-0000-0000-00000000c204",
                    "part": "20000000-0000-0000-0000-00000000c204",
                    "reference": "J2",
                    "value": "PinHeader_1x03_P2.54mm_Vertical",
                    "position": { "x": 18000000, "y": 1460000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c208": {
                    "uuid": "00000000-0000-0000-0000-00000000c208",
                    "package": "10000000-0000-0000-0000-00000000c208",
                    "part": "20000000-0000-0000-0000-00000000c208",
                    "reference": "R1",
                    "value": "R_0805_2012Metric",
                    "position": { "x": 7000000, "y": 10200000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c209": {
                    "uuid": "00000000-0000-0000-0000-00000000c209",
                    "package": "10000000-0000-0000-0000-00000000c209",
                    "part": "20000000-0000-0000-0000-00000000c209",
                    "reference": "TP1",
                    "value": "TestPoint_Loop_D2.60mm_Drill1.4mm_Beaded",
                    "position": { "x": 12500000, "y": 10200000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                }
            },
            "pads": {
                "00000000-0000-0000-0000-00000000c205": {
                    "uuid": "00000000-0000-0000-0000-00000000c205",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "6",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 4000000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c212": {
                    "uuid": "00000000-0000-0000-0000-00000000c212",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c213": {
                    "uuid": "00000000-0000-0000-0000-00000000c213",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 2730000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c214": {
                    "uuid": "00000000-0000-0000-0000-00000000c214",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "3",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 4000000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c215": {
                    "uuid": "00000000-0000-0000-0000-00000000c215",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "7",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 2730000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c218": {
                    "uuid": "00000000-0000-0000-0000-00000000c218",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "8",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 6975000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c221": {
                    "uuid": "00000000-0000-0000-0000-00000000c221",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "4",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 5270000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c222": {
                    "uuid": "00000000-0000-0000-0000-00000000c222",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "5",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 5270000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c206": {
                    "uuid": "00000000-0000-0000-0000-00000000c206",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 18000000, "y": 4000000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 1700000,
                    "width": 0,
                    "height": 0,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c219": {
                    "uuid": "00000000-0000-0000-0000-00000000c219",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 18000000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1700000,
                    "height": 1700000,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c220": {
                    "uuid": "00000000-0000-0000-0000-00000000c220",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "3",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 18000000, "y": 6540000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 1700000,
                    "width": 0,
                    "height": 0,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c20a": {
                    "uuid": "00000000-0000-0000-0000-00000000c20a",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6087500, "y": 10200000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1025000,
                    "height": 1400000
                },
                "00000000-0000-0000-0000-00000000c20b": {
                    "uuid": "00000000-0000-0000-0000-00000000c20b",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 7912500, "y": 10200000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1025000,
                    "height": 1400000
                },
                "00000000-0000-0000-0000-00000000c20c": {
                    "uuid": "00000000-0000-0000-0000-00000000c20c",
                    "package": "00000000-0000-0000-0000-00000000c209",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 12500000, "y": 10200000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 2800000,
                    "width": 0,
                    "height": 0,
                    "drill": 1400000
                }
            },
            "component_silkscreen": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "from": { "x": -2060000, "y": -2560000 },
                        "to": { "x": 2060000, "y": -2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": 2560000 },
                        "to": { "x": 2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": -2560000 },
                        "to": { "x": -2060000, "y": -2465000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": 2465000 },
                        "to": { "x": -2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 2060000, "y": -2560000 },
                        "to": { "x": 2060000, "y": -2465000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 2060000, "y": 2465000 },
                        "to": { "x": 2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "from": { "x": -1380000, "y": -1380000 },
                        "to": { "x": 0, "y": -1380000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 1270000 },
                        "to": { "x": -1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 1270000 },
                        "to": { "x": 1380000, "y": 1270000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 6460000 },
                        "to": { "x": 1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 1380000, "y": 1270000 },
                        "to": { "x": 1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "from": { "x": -227064, "y": -735000 },
                        "to": { "x": 227064, "y": -735000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -227064, "y": 735000 },
                        "to": { "x": 227064, "y": 735000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "from": { "x": -900000, "y": 2200000 },
                        "to": { "x": 900000, "y": 2200000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_arcs": {},
            "component_silkscreen_circles": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "center": { "x": -2600000, "y": -2470000 },
                        "radius_nm": 70000,
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "center": { "x": 0, "y": 0 },
                        "radius_nm": 1700000,
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_polygons": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "vertices": [
                            { "x": -2600000, "y": -2470000 },
                            { "x": -2840000, "y": -2800000 },
                            { "x": -2360000, "y": -2800000 }
                        ],
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_polylines": {
                "00000000-0000-0000-0000-00000000c208": []
            },
            "component_silkscreen_texts": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "text": "SRC",
                        "position": { "x": -220000, "y": -340000 },
                        "rotation": 0,
                        "height_nm": 160000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "text": "DST",
                        "position": { "x": -220000, "y": -360000 },
                        "rotation": 0,
                        "height_nm": 160000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "text": "R1",
                        "position": { "x": 0, "y": -1200000 },
                        "rotation": 0,
                        "height_nm": 180000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "text": "TP1",
                        "position": { "x": 0, "y": 2600000 },
                        "rotation": 0,
                        "height_nm": 180000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ]
            },
            "component_mechanical_lines": {},
            "component_mechanical_arcs": {},
            "component_mechanical_circles": {
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "center": { "x": 0, "y": 0 },
                        "radius_nm": 2000000,
                        "width_nm": 50000,
                        "layer": 41
                    }
                ]
            },
            "component_mechanical_texts": {},
            "component_mechanical_polylines": {},
            "component_mechanical_polygons": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "vertices": [
                            { "x": -3700000, "y": -2700000 },
                            { "x": -2200000, "y": -2700000 },
                            { "x": -2200000, "y": -2460000 },
                            { "x": 2200000, "y": -2460000 },
                            { "x": 2200000, "y": -2700000 },
                            { "x": 3700000, "y": -2700000 },
                            { "x": 3700000, "y": 2460000 },
                            { "x": 2200000, "y": 2460000 },
                            { "x": 2200000, "y": 2700000 },
                            { "x": -2200000, "y": 2700000 },
                            { "x": -2200000, "y": 2460000 },
                            { "x": -3700000, "y": 2460000 }
                        ],
                        "layer": 41
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "vertices": [
                            { "x": -1770000, "y": -1770000 },
                            { "x": 1770000, "y": -1770000 },
                            { "x": 1770000, "y": 6850000 },
                            { "x": -1770000, "y": 6850000 }
                        ],
                        "layer": 41
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "vertices": [
                            { "x": -1680000, "y": -950000 },
                            { "x": 1680000, "y": -950000 },
                            { "x": 1680000, "y": 950000 },
                            { "x": -1680000, "y": 950000 }
                        ],
                        "layer": 41
                    }
                ]
            },
            "tracks": {
                "00000000-0000-0000-0000-00000000c20d": {
                    "uuid": "00000000-0000-0000-0000-00000000c20d",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 5200000, "y": 10200000 },
                    "to": { "x": 10200000, "y": 10200000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c20e": {
                    "uuid": "00000000-0000-0000-0000-00000000c20e",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 10200000, "y": 10200000 },
                    "to": { "x": 12600000, "y": 8900000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c20f": {
                    "uuid": "00000000-0000-0000-0000-00000000c20f",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12600000, "y": 8900000 },
                    "to": { "x": 16000000, "y": 8900000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c216": {
                    "uuid": "00000000-0000-0000-0000-00000000c216",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12000000, "y": 2400000 },
                    "to": { "x": 12000000, "y": 5600000 },
                    "width": 320000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c217": {
                    "uuid": "00000000-0000-0000-0000-00000000c217",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12000000, "y": 2400000 },
                    "to": { "x": 12000000, "y": 5600000 },
                    "width": 320000,
                    "layer": 3
                }
            },
            "vias": {
                "00000000-0000-0000-0000-00000000c210": {
                    "uuid": "00000000-0000-0000-0000-00000000c210",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 10200000, "y": 10200000 },
                    "drill": 250000,
                    "diameter": 520000,
                    "from_layer": 1,
                    "to_layer": 3
                }
            },
            "zones": {
                "00000000-0000-0000-0000-00000000c211": {
                    "uuid": "00000000-0000-0000-0000-00000000c211",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "polygon": {
                        "vertices": [
                            { "x": 3500000, "y": 8000000 },
                            { "x": 22000000, "y": 8000000 },
                            { "x": 22000000, "y": 12600000 },
                            { "x": 3500000, "y": 12600000 }
                        ],
                        "closed": true
                    },
                    "layer": 1,
                    "priority": 1,
                    "thermal_relief": false,
                    "thermal_gap": 200000,
                    "thermal_spoke_width": 200000
                }
            },
            "nets": {
                "00000000-0000-0000-0000-00000000c200": {
                    "uuid": "00000000-0000-0000-0000-00000000c200",
                    "name": "SIG",
                    "class": "00000000-0000-0000-0000-00000000c202"
                },
                "00000000-0000-0000-0000-00000000c201": {
                    "uuid": "00000000-0000-0000-0000-00000000c201",
                    "name": "GND",
                    "class": "00000000-0000-0000-0000-00000000c202"
                }
            },
            "net_classes": {
                "00000000-0000-0000-0000-00000000c202": {
                    "uuid": "00000000-0000-0000-0000-00000000c202",
                    "name": "Default",
                    "clearance": 150000,
                    "track_width": 200000,
                    "via_drill": 300000,
                    "via_diameter": 600000,
                    "diffpair_width": 0,
                    "diffpair_gap": 0
                }
            },
            "keepouts": [],
            "dimensions": [],
            "texts": []
        }),
    )?;
    kicad_geometry::apply_kicad_reference_geometry(&board_dir.join("board.json"))?;
    Ok(())
}

pub(super) fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let payload = serde_json::to_string_pretty(value).context("failed to serialize demo JSON")?;
    std::fs::write(path, format!("{payload}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}
