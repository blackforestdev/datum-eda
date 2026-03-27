use super::*;
use eda_engine::board::Via;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_vias_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-vias",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_via_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-via");
    create_native_project(&root, Some("Board Via Demo".to_string()))
        .expect("initial scaffold should succeed");

    let class_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net-class",
        root.to_str().unwrap(),
        "--name",
        "Default",
        "--clearance-nm",
        "150000",
        "--track-width-nm",
        "200000",
        "--via-drill-nm",
        "300000",
        "--via-diameter-nm",
        "600000",
    ])
    .expect("CLI should parse");
    let class_output = execute(class_cli).expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");

    let net_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net",
        root.to_str().unwrap(),
        "--name",
        "GND",
        "--class",
        class_report["net_class_uuid"].as_str().unwrap(),
    ])
    .expect("CLI should parse");
    let net_output = execute(net_cli).expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    let net_uuid = net_report["net_uuid"].as_str().unwrap().to_string();

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-via",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--x-nm",
        "5000",
        "--y-nm",
        "6000",
        "--drill-nm",
        "300000",
        "--diameter-nm",
        "600000",
        "--from-layer",
        "1",
        "--to-layer",
        "2",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("place board via should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    let via_uuid = placed["via_uuid"].as_str().unwrap().to_string();

    let vias_output = execute(board_vias_query_cli(&root)).expect("board vias query should succeed");
    let vias: Vec<Via> = serde_json::from_str(&vias_output).expect("query output should parse");
    assert_eq!(vias.len(), 1);
    assert_eq!(vias[0].uuid.to_string(), via_uuid);
    assert_eq!(vias[0].net.to_string(), net_uuid);
    assert_eq!(vias[0].position.x, 5000);
    assert_eq!(vias[0].position.y, 6000);
    assert_eq!(vias[0].drill, 300000);
    assert_eq!(vias[0].diameter, 600000);
    assert_eq!(vias[0].from_layer, 1);
    assert_eq!(vias[0].to_layer, 2);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-via",
        root.to_str().unwrap(),
        "--via",
        &via_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board via should succeed");
    assert!(delete_output.contains("action: delete_board_via"));

    let vias_output = execute(board_vias_query_cli(&root)).expect("board vias query should succeed");
    let vias: Vec<Via> = serde_json::from_str(&vias_output).expect("query output should parse");
    assert!(vias.is_empty());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_vias: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_vias_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-via-query");
    create_native_project(&root, Some("Board Via Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Via Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 50, "y": 60 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 2
                    }
                },
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

    let output = execute(board_vias_query_cli(&root)).expect("board vias query should succeed");
    let vias: Vec<Via> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(vias.len(), 1);
    assert_eq!(vias[0].uuid, via_uuid);
    assert_eq!(vias[0].net, net_uuid);
    assert_eq!(vias[0].position.x, 50);
    assert_eq!(vias[0].position.y, 60);
    assert_eq!(vias[0].drill, 300000);
    assert_eq!(vias[0].diameter, 600000);
    assert_eq!(vias[0].from_layer, 1);
    assert_eq!(vias[0].to_layer, 2);

    let _ = std::fs::remove_dir_all(&root);
}
