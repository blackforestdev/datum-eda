use super::*;
use eda_engine::board::NetClass;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_net_classes_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-net-classes",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_net_class_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-net-class");
    create_native_project(&root, Some("Board Net Class Demo".to_string()))
        .expect("initial scaffold should succeed");

    let place_cli = Cli::try_parse_from([
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
        "--diffpair-width-nm",
        "180000",
        "--diffpair-gap-nm",
        "170000",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board net class should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let net_class_uuid = placed["net_class_uuid"].as_str().unwrap().to_string();

    let classes_output = execute(board_net_classes_query_cli(&root))
        .expect("board net classes query should succeed");
    let classes: Vec<NetClass> =
        serde_json::from_str(&classes_output).expect("query output should parse");
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].uuid.to_string(), net_class_uuid);
    assert_eq!(classes[0].name, "Default");
    assert_eq!(classes[0].clearance, 150000);
    assert_eq!(classes[0].diffpair_gap, 170000);

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-net-class",
        root.to_str().unwrap(),
        "--net-class",
        &net_class_uuid,
        "--name",
        "HighSpeed",
        "--track-width-nm",
        "250000",
        "--diffpair-gap-nm",
        "190000",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board net class should succeed");

    let classes_output = execute(board_net_classes_query_cli(&root))
        .expect("board net classes query should succeed");
    let classes: Vec<NetClass> =
        serde_json::from_str(&classes_output).expect("query output should parse");
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].name, "HighSpeed");
    assert_eq!(classes[0].track_width, 250000);
    assert_eq!(classes[0].diffpair_gap, 190000);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-net-class",
        root.to_str().unwrap(),
        "--net-class",
        &net_class_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board net class should succeed");
    assert!(delete_output.contains("action: delete_board_net_class"));

    let classes_output = execute(board_net_classes_query_cli(&root))
        .expect("board net classes query should succeed");
    let classes: Vec<NetClass> =
        serde_json::from_str(&classes_output).expect("query output should parse");
    assert!(classes.is_empty());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_net_classes: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_net_classes_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-net-class-query");
    create_native_project(&root, Some("Board Net Class Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_class_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Net Class Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {
                    net_class_uuid.to_string(): {
                        "uuid": net_class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 180000,
                        "diffpair_gap": 170000
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_net_classes_query_cli(&root))
        .expect("board net classes query should succeed");
    let classes: Vec<NetClass> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].uuid, net_class_uuid);
    assert_eq!(classes[0].name, "Default");
    assert_eq!(classes[0].via_diameter, 600000);

    let _ = std::fs::remove_dir_all(&root);
}
