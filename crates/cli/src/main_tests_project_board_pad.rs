use super::*;
use eda_engine::board::PlacedPad;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_pads_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-pads",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_pad_query_edit_and_net_assignment_round_trip() {
    let root = unique_project_root("datum-eda-cli-project-board-pad");
    create_native_project(&root, Some("Board Pad Demo".to_string()))
        .expect("initial scaffold should succeed");

    let package_uuid = Uuid::new_v4();
    let seeded_pad_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Pad Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {
                    seeded_pad_uuid.to_string(): {
                        "uuid": seeded_pad_uuid,
                        "package": package_uuid,
                        "name": "1",
                        "net": null,
                        "position": { "x": 1000, "y": 2000 },
                        "layer": 1,
                        "diameter": 450000
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "GND",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].uuid, seeded_pad_uuid);
    assert_eq!(pads[0].package, package_uuid);
    assert_eq!(pads[0].name, "1");
    assert_eq!(pads[0].net, None);
    assert_eq!(pads[0].position.x, 1000);
    assert_eq!(pads[0].position.y, 2000);
    assert_eq!(pads[0].layer, 1);
    assert_eq!(pads[0].diameter, 450000);

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-pad",
        root.to_str().unwrap(),
        "--pad",
        &seeded_pad_uuid.to_string(),
        "--x-nm",
        "3000",
        "--y-nm",
        "4000",
        "--layer",
        "2",
        "--diameter-nm",
        "600000",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("edit board pad should succeed");
    let edit_report: serde_json::Value =
        serde_json::from_str(&edit_output).expect("edit output should parse");
    assert_eq!(edit_report["action"].as_str(), Some("edit_board_pad"));
    assert_eq!(edit_report["x_nm"].as_i64(), Some(3000));
    assert_eq!(edit_report["y_nm"].as_i64(), Some(4000));
    assert_eq!(edit_report["layer"].as_i64(), Some(2));
    assert_eq!(edit_report["diameter_nm"].as_i64(), Some(600000));

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].position.x, 3000);
    assert_eq!(pads[0].position.y, 4000);
    assert_eq!(pads[0].layer, 2);
    assert_eq!(pads[0].diameter, 600000);

    let set_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-pad-net",
        root.to_str().unwrap(),
        "--pad",
        &seeded_pad_uuid.to_string(),
        "--net",
        &net_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let set_output = execute(set_cli).expect("set board pad net should succeed");
    let set_report: serde_json::Value =
        serde_json::from_str(&set_output).expect("set output should parse");
    assert_eq!(set_report["action"].as_str(), Some("set_board_pad_net"));
    assert_eq!(
        set_report["net_uuid"].as_str(),
        Some(net_uuid.to_string().as_str())
    );

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].net, Some(net_uuid));

    let clear_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-board-pad-net",
        root.to_str().unwrap(),
        "--pad",
        &seeded_pad_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let clear_output = execute(clear_cli).expect("clear board pad net should succeed");
    assert!(clear_output.contains("action: clear_board_pad_net"));

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 1);
    assert_eq!(pads[0].net, None);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-pad",
        root.to_str().unwrap(),
        "--package",
        &package_uuid.to_string(),
        "--name",
        "2",
        "--x-nm",
        "7000",
        "--y-nm",
        "8000",
        "--layer",
        "3",
        "--diameter-nm",
        "700000",
        "--net",
        &net_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("place board pad should succeed");
    let place_report: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    let placed_pad_uuid = place_report["pad_uuid"].as_str().unwrap().to_string();
    assert_eq!(place_report["action"].as_str(), Some("place_board_pad"));

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 2);
    let created = pads
        .iter()
        .find(|pad| pad.uuid.to_string() == placed_pad_uuid)
        .expect("placed pad should be present");
    assert_eq!(created.package, package_uuid);
    assert_eq!(created.name, "2");
    assert_eq!(created.position.x, 7000);
    assert_eq!(created.position.y, 8000);
    assert_eq!(created.diameter, 700000);
    assert_eq!(created.layer, 3);
    assert_eq!(created.net, Some(net_uuid));

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-pad",
        root.to_str().unwrap(),
        "--pad",
        &placed_pad_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board pad should succeed");
    assert!(delete_output.contains("action: delete_board_pad"));

    let pads_output =
        execute(board_pads_query_cli(&root)).expect("board pads query should succeed");
    let pads: Vec<PlacedPad> =
        serde_json::from_str(&pads_output).expect("query output should parse");
    assert_eq!(pads.len(), 1);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_pads: 1"));

    let _ = std::fs::remove_dir_all(&root);
}
