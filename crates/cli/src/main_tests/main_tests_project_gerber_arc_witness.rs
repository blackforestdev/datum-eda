use super::*;
use eda_engine::board::{Stackup, StackupLayer, StackupLayerType, Track};
use eda_engine::ir::geometry::Point;

#[test]
fn project_gerber_arc_exact_output_and_rational_refusal_preserve_existing_artifact() {
    let root = std::env::temp_dir().join(format!("datum-m1-cli-{}", Uuid::new_v4()));
    create_native_project(&root, Some("M1 native output proof".into())).unwrap();
    let board = root.join("board/board.json");
    let mut source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board).unwrap()).unwrap();
    let net = Uuid::new_v4();
    let id = Uuid::new_v4();
    let mut arc = Track::straight(
        id,
        net,
        Point::new(0, 0),
        Point::new(10_000_000, 0),
        200_001,
        1,
    );
    arc.midpoint = Some(Point::new(5_000_000, 5_000_000));
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
    source["nets"] =
        serde_json::json!({net.to_string():{"uuid":net,"name":"proof","class":Uuid::nil()}});
    source["tracks"] = serde_json::json!({id.to_string():serde_json::to_value(&arc).unwrap()});
    std::fs::write(&board, serde_json::to_vec_pretty(&source).unwrap()).unwrap();
    let out = root.join("front.gbr");
    let run = || {
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
    };
    let authored = std::fs::read(&board).unwrap();
    let result: serde_json::Value = serde_json::from_str(&run().unwrap()).unwrap();
    let exact = std::fs::read_to_string(&out).unwrap();
    assert!(exact.contains("G75*\nG02*\nX10000000Y0I5000000J0D01*"));
    assert!(exact.contains("%ADD10C,0.200001*%"));
    assert!(exact.contains(&format!("Track {id} certified zero-deviation")));
    assert_eq!(result["track_count"], 1);
    assert_eq!(std::fs::read(&board).unwrap(), authored);
    arc.to = Point::new(3_000_000, 0);
    arc.midpoint = Some(Point::new(1_000_000, 3_000_000));
    source["tracks"][id.to_string()] = serde_json::to_value(&arc).unwrap();
    // This second native genesis recipe isolates rejection; no direct mutation
    // writer is implemented or adopted by this test.
    std::fs::write(&board, serde_json::to_vec_pretty(&source).unwrap()).unwrap();
    let authored = std::fs::read(&board).unwrap();
    let failure = run().unwrap_err();
    let message = format!("{failure:#}");
    assert!(message.contains(&id.to_string()));
    assert!(message.contains("AllowanceRequired"));
    assert!(message.contains("numerator: 2, denominator: 3"));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), exact);
    assert_eq!(std::fs::read(&board).unwrap(), authored);
    std::fs::remove_dir_all(root).unwrap();
}
