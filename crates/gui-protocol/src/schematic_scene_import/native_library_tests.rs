use super::*;
use std::{collections::BTreeMap, path::PathBuf};

#[test]
fn display_pose_uses_known_positive_rotation_and_reflection_before_rotation() {
    let origin = Point {
        x: 20_000_000,
        y: 0,
    };
    // Independently rounded 30-degree triangle vertices, not generated with
    // the implementation under test. Source endpoints remain integer/local.
    for (local, expected) in [
        ((-1_000_000, -1_000_000), (19_633_975, -1_366_025)),
        ((1_000_000, 0), (20_866_025, 500_000)),
        ((-1_000_000, 1_000_000), (18_633_975, 366_025)),
    ] {
        let p = Point {
            x: local.0,
            y: local.1,
        };
        for angle in [30, -330, 390] {
            assert_eq!(
                display_point(p, origin, angle, false).unwrap(),
                PointNm {
                    x: expected.0,
                    y: expected.1,
                }
            );
        }
    }
    assert_eq!(
        display_point(Point { x: 1_000_000, y: 0 }, origin, 30, true).unwrap(),
        PointNm {
            x: 19_133_975,
            y: -500_000
        }
    );
}

#[test]
fn display_cardinal_poses_retain_exact_engine_results() {
    let origin = Point {
        x: i64::MAX - 4_000_000,
        y: i64::MIN + 4_000_000,
    };
    for angle in [-450, -360, -90, 0, 90, 180, 270, 360, 450] {
        for mirrored in [false, true] {
            let p = Point {
                x: 1_000_001,
                y: -2_000_003,
            };
            assert_eq!(
                display_point(p, origin, angle, mirrored).unwrap(),
                point_nm(place_local_point(p, origin, angle, mirrored).unwrap())
            );
        }
    }
    // The exact native writer still refuses irrational electrical placements.
    assert!(place_local_point(Point { x: 1, y: 1 }, origin, 30, false).is_err());
}

#[test]
fn display_pose_refuses_overflow_instead_of_saturating_or_wrapping() {
    let zero = Point { x: 0, y: 0 };
    assert!(display_point(Point { x: i64::MIN, y: 0 }, zero, 30, true).is_err());
    assert!(
        display_point(
            Point {
                x: i64::MAX,
                y: i64::MAX
            },
            zero,
            30,
            false
        )
        .is_err()
    );
    // At 45 degrees this component rounds to 2^63 as f64, the exclusive
    // upper bound; a saturating `as i64` must not silently accept it.
    assert!(
        display_point(
            Point {
                x: i64::MAX,
                y: -3_820_445_789_094_120_000
            },
            zero,
            45,
            false
        )
        .is_err()
    );
    assert!(
        display_point(
            Point { x: 1_000_000, y: 0 },
            Point { x: i64::MAX, y: 0 },
            30,
            false
        )
        .is_err()
    );
    assert!(
        display_point(
            Point { x: 1_000_000, y: 0 },
            Point { x: 0, y: i64::MIN },
            210,
            false
        )
        .is_err()
    );
    assert!(display_point(Point { x: i64::MIN, y: 0 }, zero, 90, true).is_err());
}

#[test]
fn frozen_rotated_body_load_and_preview_preserve_pins_and_all_source_history() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-parents-v3",
    );
    let catalogue: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.parent().unwrap().parent().unwrap().join("index.json")).unwrap(),
    )
    .unwrap();
    let snapshot = || -> BTreeMap<String, Vec<u8>> {
        catalogue["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| {
                let path = row["path"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("s5a-c03-u5-parents-v3/")?;
                Some((path.to_owned(), std::fs::read(root.join(path)).unwrap()))
            })
            .collect()
    };
    let before = snapshot();
    assert_eq!(before.len(), 63);
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let editor = crate::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    let source = editor.source.as_ref().unwrap();
    let sheet = &source.sheets[&editor.sheet.unwrap()];
    let raw = project_schematic_sheet_source(source, sheet, &root.join("schematic/schematic.json"))
        .unwrap();
    let loaded = crate::native_schematic_scene::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    // This is the actual preview consumer, including existing rotated Symbols.
    let preview = editor
        .project_source(source, sheet.uuid, &loaded, &root)
        .unwrap();
    let pin_layer = |layer: &str| matches!(layer, "L212" | "L213");
    let pin_texts = |scene: &BoardReviewSceneV1| {
        let mut labels: Vec<_> = scene
            .board_texts
            .iter()
            .filter(|text| pin_layer(&text.layer_id))
            .cloned()
            .collect();
        labels.sort_by(|a, b| a.text_uuid.cmp(&b.text_uuid));
        labels
    };
    let pin_geometry = |scene: &BoardReviewSceneV1| {
        let mut geometry: Vec<_> = scene
            .board_text_geometries
            .iter()
            .filter(|text| pin_layer(&text.layer_id))
            .cloned()
            .collect();
        geometry.sort_by(|a, b| a.text_uuid.cmp(&b.text_uuid));
        geometry
    };
    assert_eq!(pin_texts(&loaded), pin_texts(&preview));
    assert_eq!(pin_geometry(&loaded), pin_geometry(&preview));
    let expected_labels = pin_texts(&raw);
    for scene in [&loaded, &preview] {
        for symbol in sheet.symbols.values() {
            let prefix = format!("schematic-symbol:{}", symbol.uuid);
            assert!(!scene.board_graphics.iter().any(|g| g.object_id == prefix));
            let lines: Vec<_> = scene
                .board_graphics
                .iter()
                .filter(|g| g.object_id.starts_with(&(prefix.clone() + ":library:")))
                .collect();
            assert_eq!(lines.len(), 3);
            assert!(
                lines
                    .iter()
                    .all(|g| g.source_object_uuid == symbol.uuid.to_string()
                        && g.object_kind == "schematic_symbol"
                        && g.primitive_kind == "line")
            );
            if symbol.reference == "U2" {
                let first = lines.iter().find(|g| g.object_id.ends_with(":0")).unwrap();
                assert_eq!(
                    first.path,
                    vec![
                        PointNm {
                            x: 19_633_975,
                            y: -1_366_025
                        },
                        PointNm {
                            x: 20_866_025,
                            y: 500_000
                        }
                    ]
                );
            }
        }
        let pins = |s: &BoardReviewSceneV1| {
            s.board_graphics
                .iter()
                .filter(|g| {
                    g.object_kind == "schematic_pin"
                        && g.object_id.starts_with("schematic-symbol-pin-terminal:")
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            pins(scene),
            pins(&raw),
            "persisted pin terminal projections remain unchanged"
        );
        assert!(!pins(scene).is_empty());
        let other_texts = |s: &BoardReviewSceneV1| {
            s.board_texts
                .iter()
                .filter(|text| !pin_layer(&text.layer_id))
                .cloned()
                .collect::<Vec<_>>()
        };
        let other_geometry = |s: &BoardReviewSceneV1| {
            s.board_text_geometries
                .iter()
                .filter(|text| !pin_layer(&text.layer_id))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(other_texts(scene), other_texts(&raw));
        assert_eq!(other_geometry(scene), other_geometry(&raw));
        let labels = pin_texts(scene);
        assert_eq!(labels.len(), expected_labels.len());
        for (actual, fallback) in labels.iter().zip(&expected_labels) {
            assert_eq!(actual.object_id, fallback.object_id);
            assert_eq!(actual.text_uuid, fallback.text_uuid);
            assert_eq!(actual.source_object_uuid, fallback.source_object_uuid);
            assert_eq!(actual.layer_id, fallback.layer_id);
            assert_eq!(actual.text, fallback.text);
            let geometry = scene
                .board_text_geometries
                .iter()
                .find(|geometry| geometry.text_uuid == actual.text_uuid)
                .unwrap();
            assert!(crate::selection_geometry_catalog::layout_polygon(geometry).is_some());
            assert!(
                !geometry.glyphs.is_empty()
                    || !geometry.fills.is_empty()
                    || !geometry.strokes.is_empty()
            );
        }
        assert_eq!(scene.bounds, raw.bounds);
    }
    assert_eq!(
        snapshot(),
        before,
        "all 63 canonical files, including history and pins"
    );
}
