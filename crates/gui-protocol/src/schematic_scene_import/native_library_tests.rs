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
                .filter(|g| g.object_kind == "schematic_pin")
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            pins(scene),
            pins(&raw),
            "persisted pin projections remain unchanged"
        );
        assert!(!pins(scene).is_empty());
        assert_eq!(scene.board_texts, raw.board_texts);
        assert_eq!(scene.board_text_geometries, raw.board_text_geometries);
        assert_eq!(scene.bounds, raw.bounds);
    }
    assert_eq!(
        snapshot(),
        before,
        "all 63 canonical files, including history and pins"
    );
}
