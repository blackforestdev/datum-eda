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
