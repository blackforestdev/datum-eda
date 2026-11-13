use super::*;
use eda_engine::board::{PadShape, Stackup, StackupLayer, StackupLayerType};
use eda_engine::export::ExportError;

fn pad_project(shape: PadShape, rotation: i32, ratio: u32) -> (PathBuf, Uuid) {
    let root = unique_project_root("datum-ca16-pad-shape");
    create_native_project(&root, Some("CA16 copper shape".into())).unwrap();
    let board = root.join("board/board.json");
    let mut source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board).unwrap()).unwrap();
    let id = Uuid::new_v4();
    source["schema_version"] = serde_json::json!(2);
    source["stackup"] = serde_json::to_value(Stackup {
        layers: vec![StackupLayer::new(
            1,
            "front",
            StackupLayerType::Copper,
            35_000,
        )],
    })
    .unwrap();
    source["pads"] = serde_json::json!({id.to_string(): {
        "uuid": id, "package": Uuid::new_v4(), "name": "CA16", "net": null,
        "position": {"x": 500000, "y": 250000}, "layer": 1,
        "shape": shape, "diameter": 1000000, "width": 2000000, "height": 1000000,
        "rotation": rotation, "roundrect_rratio_ppm": ratio
    }});
    // Native genesis fixture, matching the existing public copper export tests.
    std::fs::write(board, serde_json::to_vec_pretty(&source).unwrap()).unwrap();
    (root, id)
}

fn export_pad_project(root: &Path, out: &Path) -> anyhow::Result<String> {
    execute(
        Cli::try_parse_from([
            "datum-eda",
            "--format",
            "json",
            "project",
            "export-gerber-copper-layer",
            root.to_str().unwrap(),
            "--layer",
            "1",
            "--out",
            out.to_str().unwrap(),
        ])
        .unwrap(),
    )
}

fn assert_shape_refused_without_publication(shape: PadShape) {
    for rotation in [0, 90, 37, -90, 450] {
        for ratio in [0, 1, 250_000, 500_000] {
            let (root, id) = pad_project(shape, rotation, ratio);
            let board = root.join("board/board.json");
            let authored = std::fs::read(&board).unwrap();
            let out = root.join("front.gbr");
            for existing in [false, true] {
                let retained = b"existing copper artifact\0\xff\n";
                if existing {
                    std::fs::write(&out, retained).unwrap();
                }
                let error = export_pad_project(&root, &out)
                    .expect_err("unsupported pad cannot publish substituted copper");
                assert!(
                    matches!(error.downcast_ref::<ExportError>(),
                    Some(ExportError::UnsupportedPadShape { source_id, shape: actual })
                        if *source_id == id && *actual == shape),
                    "{error:#}"
                );
                assert!(format!("{error:#}").contains("output refused"));
                if existing {
                    assert_eq!(std::fs::read(&out).unwrap(), retained);
                } else {
                    assert!(!out.exists(), "refusal must not create an artifact");
                }
                assert_eq!(std::fs::read(&board).unwrap(), authored);
            }
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
fn project_gerber_oval_refusal_preserves_absent_and_existing_destinations() {
    assert_shape_refused_without_publication(PadShape::Oval);
}

#[test]
fn project_gerber_roundrect_refusal_preserves_absent_and_existing_destinations() {
    assert_shape_refused_without_publication(PadShape::RoundRect);
}

#[test]
fn project_gerber_circle_and_rectangle_keep_exact_supported_apertures() {
    for (shape, rotation, aperture) in [
        (PadShape::Circle, 0, "%ADD10C,1.000000*%"),
        (PadShape::Circle, 90, "%ADD10C,1.000000*%"),
        (PadShape::Rect, 0, "%ADD10R,2.000000X1.000000*%"),
        (PadShape::Rect, 90, "%ADD10R,1.000000X2.000000*%"),
    ] {
        let (root, _) = pad_project(shape, rotation, 250_000);
        let board = root.join("board/board.json");
        let authored = std::fs::read(&board).unwrap();
        let out = root.join("front.gbr");
        let result: serde_json::Value =
            serde_json::from_str(&export_pad_project(&root, &out).unwrap()).unwrap();
        assert_eq!(result["pad_count"], 1);
        let gerber = std::fs::read_to_string(out).unwrap();
        assert!(gerber.contains(aperture), "{gerber}");
        assert!(gerber.contains("X500000Y250000D03*"));
        assert!(gerber.ends_with("M02*\n"));
        assert_eq!(std::fs::read(board).unwrap(), authored);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn project_gerber_arbitrary_rectangle_refusal_preserves_destinations() {
    let (root, id) = pad_project(PadShape::Rect, 37, 250_000);
    let board = root.join("board/board.json");
    let authored = std::fs::read(&board).unwrap();
    let out = root.join("front.gbr");
    for existing in [false, true] {
        if existing {
            std::fs::write(&out, b"retained copper").unwrap();
        }
        let error = export_pad_project(&root, &out).unwrap_err();
        assert!(
            matches!(error.downcast_ref::<ExportError>(),
            Some(ExportError::UnsupportedPadRotation { source_id, rotation: 37 })
                if *source_id == id),
            "{error:#}"
        );
        if existing {
            assert_eq!(std::fs::read(&out).unwrap(), b"retained copper");
        } else {
            assert!(!out.exists());
        }
        assert_eq!(std::fs::read(&board).unwrap(), authored);
    }
    std::fs::remove_dir_all(root).unwrap();
}
