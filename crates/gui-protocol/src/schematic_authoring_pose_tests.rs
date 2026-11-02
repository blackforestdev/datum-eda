//! An existing asymmetric native symbol exercises body/pin agreement in the real preview.
use super::*;
use PlacementTransform::{FlipX, FlipY, Rotate};

struct FixtureRoot(std::path::PathBuf);
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn isolated_fixture() -> FixtureRoot {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-parents-v3",
    );
    let catalogue: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            source
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("index.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let fixture =
        FixtureRoot(std::env::temp_dir().join(format!("datum-symbol-pose-{}", Uuid::new_v4())));
    // Copy the pinned source/history files only, never another test's lock or
    // resolver cache. resolve() takes a recovery lease even on immutable input.
    for row in catalogue["files"].as_array().unwrap() {
        let Some(relative) = row["path"]
            .as_str()
            .unwrap()
            .strip_prefix("s5a-c03-u5-parents-v3/")
        else {
            continue;
        };
        let destination = fixture.0.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative), destination).unwrap();
    }
    fixture
}

#[test]
fn placement_pose_transforms_asymmetric_body_and_pins_and_resets_on_cancel() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let original = SchematicAuthoring::from_model(&model).unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    let choice = original
        .choices
        .iter()
        .position(|c| c.name == "Datum single op amp symbol")
        .unwrap();
    let terminal_id = original.choices[choice]
        .pins
        .iter()
        .find(|p| {
            p.position
                == eda_engine::ir::geometry::Point {
                    x: -1_500_000,
                    y: -500_000,
                }
        })
        .unwrap()
        .uuid
        .to_string();
    let ghost = Uuid::nil();
    // Coordinates below are independently specified, in half-millimetre units.
    // Origin is (80,80); the first triangle edge is (-2,-2)->(2,0),
    // and the asymmetric input terminal is (-3,-1).
    for (steps, pose, edge, terminal) in [
        (vec![], (0, false), [(78, 78), (82, 80)], (77, 79)),
        (vec![Rotate], (90, false), [(82, 78), (80, 82)], (81, 77)),
        (vec![FlipX], (0, true), [(82, 78), (78, 80)], (83, 79)),
        (vec![FlipY], (180, true), [(78, 82), (82, 80)], (77, 81)),
        (
            vec![Rotate, FlipX],
            (270, true),
            [(78, 78), (80, 82)],
            (79, 77),
        ),
        (
            vec![Rotate, FlipY],
            (90, true),
            [(82, 82), (80, 78)],
            (81, 83),
        ),
        (
            vec![Rotate, Rotate],
            (180, false),
            [(82, 82), (78, 80)],
            (83, 81),
        ),
        (
            vec![Rotate, Rotate, Rotate],
            (270, false),
            [(78, 82), (80, 78)],
            (79, 83),
        ),
    ] {
        let mut draft = original.clone();
        assert!(draft.select(choice));
        draft.picker = false;
        draft.position = Some(PointNm {
            x: 40_000_000,
            y: 40_000_000,
        });
        draft.base = Some(base.clone());
        for step in steps {
            assert!(draft.transform_placement(step));
        }
        assert_eq!((draft.rotation_deg, draft.mirrored), pose);
        let scene = draft.preview_scene(&base, &root).unwrap();
        let line = scene
            .board_graphics
            .iter()
            .find(|g| g.object_id == format!("schematic-symbol:{ghost}:library:0"))
            .unwrap();
        let point = |(x, y)| PointNm {
            x: i64::from(x) * 500_000,
            y: i64::from(y) * 500_000,
        };
        assert_eq!(line.path, edge.map(point).to_vec(), "body at pose {pose:?}");
        let pin = scene
            .board_graphics
            .iter()
            .find(|g| {
                g.object_id
                    .starts_with(&format!("schematic-symbol-pin-terminal:{ghost}:"))
                    && g.source_object_uuid == terminal_id
            })
            .unwrap();
        let center = point(terminal);
        let radius = 90_000i64;
        assert_eq!(
            (
                pin.path.iter().map(|p| p.x).min(),
                pin.path.iter().map(|p| p.x).max(),
                pin.path.iter().map(|p| p.y).min(),
                pin.path.iter().map(|p| p.y).max()
            ),
            (
                Some(center.x - radius),
                Some(center.x + radius),
                Some(center.y - radius),
                Some(center.y + radius)
            ),
            "terminal ring must be centered at the exact attachment point for pose {pose:?}"
        );
        for vertex in &pin.path {
            let dx = i128::from(vertex.x - center.x);
            let dy = i128::from(vertex.y - center.y);
            assert!(
                (dx * dx + dy * dy - i128::from(radius).pow(2)).abs() <= i128::from(2 * radius + 1),
                "terminal vertex must lie on the 90 µm ring within integer rounding"
            );
        }
        assert_eq!(
            draft.source, original.source,
            "preview never edits the source"
        );
        let request = SchematicEditRequest::PlaceSymbol {
            sheet: draft.sheet.unwrap(),
            library_symbol: draft.choices[choice].id,
            reference: draft.reference.clone(),
            value: draft.value.clone(),
            x_nm: 40_000_000,
            y_nm: 40_000_000,
            rotation_deg: draft.rotation_deg,
            mirrored: draft.mirrored,
        };
        let (verb, params) = request.verb_params();
        assert_eq!(verb, "datum.schematic.place_symbol");
        assert_eq!(params["rotation_deg"], pose.0);
        assert_eq!(params["mirrored"], pose.1);
        assert_eq!(draft.cancel(), Some(base.clone()));
        assert_eq!((draft.rotation_deg, draft.mirrored), (0, false));
        assert!(draft.pending_request.is_none());
        assert_eq!(draft.source, original.source);
    }
    let mut draft = original;
    assert!(draft.select(choice));
    draft.picker = true;
    assert!(
        !draft.transform_placement(Rotate),
        "text-form input cannot rotate a symbol"
    );
    draft.picker = false;
    for _ in 0..4 {
        assert!(draft.transform_placement(Rotate));
    }
    assert_eq!((draft.rotation_deg, draft.mirrored), (0, false));
    draft.wire = true;
    assert!(
        !draft.transform_placement(FlipX),
        "wire gestures do not acquire symbol poses"
    );
}

#[test]
fn placed_symbol_pose_preview_keeps_asymmetric_body_pins_wire_and_payload_together() {
    use crate::footprint_move_preview::native::NativeMovePreview;
    use crate::native_edit_client::NativeEditAction;
    use eda_engine::api::native_write::{
        WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    use serde_json::json;
    fn write(
        model: &mut eda_engine::substrate::DesignModel,
        root: &std::path::Path,
        verb: &str,
        params: serde_json::Value,
    ) {
        let prepared = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model,
                project_root: root,
            },
            WriteProvenance::new(
                "test",
                eda_engine::substrate::CommitSource::Test,
                "connected pose preview",
            ),
            params,
        )
        .unwrap();
        commit_prepared(model, root, prepared).unwrap();
    }
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let sheet = authoring.sheet.unwrap();
    let library = authoring
        .choices
        .iter()
        .find(|c| c.name == "Datum single op amp symbol")
        .unwrap()
        .id;
    write(
        &mut model,
        root,
        "datum.schematic.place_symbol",
        json!({
            "sheet":sheet,"library_symbol":library,"reference":"U3","value":"pose preview",
            "x_nm":40_000_000,"y_nm":40_000_000,"rotation_deg":0,"mirrored":false
        }),
    );
    write(
        &mut model,
        root,
        "datum.schematic.draw_wire",
        json!({
            "sheet":sheet,"points":[{"x":41_500_000,"y":40_000_000},{"x":60_000_000,"y":40_000_000}]
        }),
    );
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let owner = authoring.source.as_ref().unwrap().sheets[&sheet]
        .symbols
        .values()
        .find(|s| s.reference == "U3")
        .unwrap()
        .uuid;
    write(
        &mut model,
        root,
        "datum.schematic.set_symbol_field",
        json!({
            "symbol":owner,"instance_path":[],"field":"custom","key":"Note","value":"field pose"
        }),
    );
    let field = SchematicAuthoring::from_model(&model)
        .unwrap()
        .source
        .unwrap()
        .sheets[&sheet]
        .symbols[&owner]
        .fields[0]
        .uuid;
    write(
        &mut model,
        root,
        "datum.schematic.set_symbol_field",
        json!({
            "symbol":owner,"instance_path":[],"field":"custom","field_id":field,"visible":true,
            "position":{"x":42_000_000,"y":41_000_000}
        }),
    );
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let symbol = source.sheets[&sheet]
        .symbols
        .values()
        .find(|s| s.reference == "U3")
        .unwrap();
    let wire = source.sheets[&sheet]
        .wires
        .values()
        .find(|w| w.to.x == 60_000_000)
        .unwrap();
    let terminal = symbol.pins.iter().find(|p| p.name == "IN-").unwrap().uuid;
    let id = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::Symbol,
        id: symbol.uuid,
        instance_path: authoring.move_paths[&symbol.uuid].clone(),
    };
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let origin = PointNm {
        x: 40_000_000,
        y: 40_000_000,
    };
    let snap = crate::AuthoringSnapState {
        enabled: false,
        grid_nm: 100_000,
    };
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let point = |(x, y)| PointNm {
        x: i64::from(x) * 500_000,
        y: i64::from(y) * 500_000,
    };
    for (steps, pose, edge, input, output) in [
        (
            vec![Rotate],
            (90, false),
            [(82, 78), (80, 82)],
            (81, 77),
            (80, 83),
        ),
        (
            vec![Rotate, FlipX],
            (270, true),
            [(78, 78), (80, 82)],
            (79, 77),
            (80, 83),
        ),
        (
            vec![Rotate, FlipY],
            (90, true),
            [(82, 82), (80, 78)],
            (81, 83),
            (80, 77),
        ),
        (
            vec![Rotate, Rotate, Rotate, Rotate],
            (0, false),
            [(78, 78), (82, 80)],
            (77, 79),
            (83, 80),
        ),
    ] {
        let mut preview =
            NativeMovePreview::symbol(&base, &authoring, &id, origin, snap, root.clone()).unwrap();
        for step in steps {
            assert!(preview.transform_symbol(step, origin).unwrap());
        }
        let scene = preview.scene_at(origin).unwrap();
        let field_text = scene
            .board_texts
            .iter()
            .find(|t| t.text == "field pose")
            .unwrap();
        let expected = match pose {
            (90, false) => PointNm {
                x: 39_000_000,
                y: 42_000_000,
            },
            (270, true) => PointNm {
                x: 41_000_000,
                y: 42_000_000,
            },
            (90, true) => PointNm {
                x: 39_000_000,
                y: 38_000_000,
            },
            (0, false) => PointNm {
                x: 42_000_000,
                y: 41_000_000,
            },
            _ => unreachable!(),
        };
        assert_eq!(
            field_text.position, expected,
            "owned field follows preview pose"
        );
        let original_field = base
            .board_texts
            .iter()
            .find(|t| t.text == "field pose")
            .unwrap();
        assert_eq!(field_text.object_id, original_field.object_id);
        assert_eq!(field_text.source_object_uuid, Some(id.id.to_string()));
        let body = scene
            .board_graphics
            .iter()
            .find(|g| g.object_id == format!("schematic-symbol:{}:library:0", id.id))
            .unwrap();
        assert_eq!(body.path, edge.map(point).to_vec(), "body {pose:?}");
        let pin = scene
            .board_graphics
            .iter()
            .find(|g| {
                g.object_id
                    .starts_with(&format!("schematic-symbol-pin-terminal:{}:", id.id))
                    && g.source_object_uuid == terminal.to_string()
            })
            .unwrap();
        let center = point(input);
        assert_eq!(
            (
                pin.path.iter().map(|p| p.x).min(),
                pin.path.iter().map(|p| p.x).max(),
                pin.path.iter().map(|p| p.y).min(),
                pin.path.iter().map(|p| p.y).max()
            ),
            (
                Some(center.x - 90_000),
                Some(center.x + 90_000),
                Some(center.y - 90_000),
                Some(center.y + 90_000)
            ),
            "pin {pose:?}"
        );
        let segment = scene
            .board_graphics
            .iter()
            .find(|g| g.source_object_uuid == wire.uuid.to_string() && g.path.len() == 2)
            .unwrap();
        assert_eq!(
            segment.path,
            vec![
                point(output),
                PointNm {
                    x: 60_000_000,
                    y: 40_000_000
                }
            ],
            "attached wire {pose:?}"
        );
        let NativeEditAction::MoveSymbol {
            symbol: target,
            x_nm,
            y_nm,
            rotation_deg,
            mirrored,
        } = preview.action(origin).unwrap()
        else {
            panic!("symbol action")
        };
        assert_eq!(
            (target, x_nm, y_nm, rotation_deg, mirrored),
            (id.id, origin.x, origin.y, Some(pose.0), Some(pose.1))
        );
        assert_eq!(preview.base(), &base, "immutable cancel source");
    }
    let mut refused =
        NativeMovePreview::symbol(&base, &authoring, &id, origin, snap, root.clone()).unwrap();
    assert!(
        refused
            .transform_symbol(
                Rotate,
                PointNm {
                    x: 60_000_000,
                    y: 38_500_000
                }
            )
            .is_err(),
        "collapsing attached wire is refused"
    );
    let NativeEditAction::MoveSymbol {
        rotation_deg,
        mirrored,
        ..
    } = refused.action(origin).unwrap()
    else {
        panic!("symbol action")
    };
    assert_eq!(
        (rotation_deg, mirrored),
        (None, None),
        "refusal retains the prior pose"
    );
    assert_eq!(authoring.source.as_ref().unwrap(), source);
    for (path, before) in bytes {
        assert_eq!(
            std::fs::read(path).unwrap(),
            before,
            "preview/cancel writes no source bytes"
        );
    }
}

#[path = "schematic_label_move_tests.rs"]
mod label_move_tests;

#[path = "schematic_marker_tests.rs"]
mod marker_tests;

#[path = "schematic_label_edit_tests.rs"]
mod label_edit_tests;

#[path = "schematic_marking_context_tests.rs"]
mod marking_context_tests;

#[path = "schematic_bus_tests.rs"]
mod bus_tests;

#[path = "schematic_custom_field_tests.rs"]
mod custom_field_tests;

#[path = "schematic_delete_review_tests.rs"]
mod delete_review_tests;

#[path = "schematic_group_move_tests.rs"]
mod group_move_tests;

#[path = "schematic_text_tests.rs"]
mod text_tests;
