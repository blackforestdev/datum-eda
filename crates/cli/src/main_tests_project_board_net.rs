use super::*;
use eda_engine::board::Net;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_nets_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-nets",
    ])
    .expect("CLI should parse")
}

fn board_net_query_cli(root: &Path, net_uuid: Uuid) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-net",
        "--net",
        &net_uuid.to_string(),
    ])
    .expect("CLI should parse")
}

fn journal_list(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    serde_json::from_str(&output).expect("journal-list JSON should parse")
}

#[test]
fn project_board_net_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-net");
    create_native_project(&root, Some("Board Net Demo".to_string()))
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
    let default_class_uuid = class_report["net_class_uuid"].as_str().unwrap().to_string();

    let second_class_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net-class",
        root.to_str().unwrap(),
        "--name",
        "Power",
        "--clearance-nm",
        "200000",
        "--track-width-nm",
        "300000",
        "--via-drill-nm",
        "350000",
        "--via-diameter-nm",
        "700000",
    ])
    .expect("CLI should parse");
    let second_class_output =
        execute(second_class_cli).expect("place second board net class should succeed");
    let second_class_report: serde_json::Value =
        serde_json::from_str(&second_class_output).expect("second class output should parse");
    let second_class_uuid = second_class_report["net_class_uuid"]
        .as_str()
        .unwrap()
        .to_string();
    let second_class_uuid_value =
        Uuid::parse_str(&second_class_uuid).expect("second class uuid should parse");

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net",
        root.to_str().unwrap(),
        "--name",
        "GND",
        "--class",
        &default_class_uuid,
        "--impedance-target-ohms",
        "50.0",
        "--impedance-tolerance-pct",
        "10",
        "--controlled-dielectric-layer",
        "2",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board net should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let net_uuid = placed["net_uuid"].as_str().unwrap().to_string();

    let nets_output =
        execute(board_nets_query_cli(&root)).expect("board nets query should succeed");
    let nets: Vec<Net> = serde_json::from_str(&nets_output).expect("query output should parse");
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].uuid.to_string(), net_uuid);
    assert_eq!(nets[0].name, "GND");
    assert_eq!(nets[0].class.to_string(), default_class_uuid);
    let impedance = nets[0]
        .controlled_impedance
        .as_ref()
        .expect("controlled impedance should be authored");
    assert_eq!(impedance.target_ohms.to_string(), "50.0");
    assert_eq!(impedance.tolerance_pct.to_string(), "10");
    assert_eq!(impedance.controlled_dielectric, Some(2));
    let journal = journal_list(&root);
    assert_eq!(journal["transactions"][2]["reason"], "place board net");

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-net",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--name",
        "PWR_GND",
        "--class",
        &second_class_uuid,
        "--impedance-tolerance-pct",
        "7.5",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board net should succeed");

    let nets_output =
        execute(board_nets_query_cli(&root)).expect("board nets query should succeed");
    let nets: Vec<Net> = serde_json::from_str(&nets_output).expect("query output should parse");
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "PWR_GND");
    assert_eq!(nets[0].class, second_class_uuid_value);
    let impedance = nets[0]
        .controlled_impedance
        .as_ref()
        .expect("controlled impedance should be retained after partial edit");
    assert_eq!(impedance.target_ohms.to_string(), "50.0");
    assert_eq!(impedance.tolerance_pct.to_string(), "7.5");
    let journal = journal_list(&root);
    assert_eq!(journal["transactions"][3]["reason"], "edit board net");

    let clear_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-net",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--clear-controlled-impedance",
    ])
    .expect("CLI should parse");
    let _ = execute(clear_cli).expect("clear controlled impedance should succeed");
    let net_output = execute(board_net_query_cli(
        &root,
        Uuid::parse_str(&net_uuid).expect("net UUID should parse"),
    ))
    .expect("single board net query should succeed");
    let net: Net = serde_json::from_str(&net_output).expect("single net output should parse");
    assert!(net.controlled_impedance.is_none());

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-net",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board net should succeed");
    assert!(delete_output.contains("action: delete_board_net"));

    let nets_output =
        execute(board_nets_query_cli(&root)).expect("board nets query should succeed");
    let nets: Vec<Net> = serde_json::from_str(&nets_output).expect("query output should parse");
    assert!(nets.is_empty());
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 6);
    assert_eq!(journal["transactions"][5]["reason"], "delete board net");

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_nets: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_nets_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-net-query");
    create_native_project(&root, Some("Board Net Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let class_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Net Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
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
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_nets_query_cli(&root)).expect("board nets query should succeed");
    let nets: Vec<Net> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].uuid, net_uuid);
    assert_eq!(nets[0].name, "GND");
    assert_eq!(nets[0].class, class_uuid);
    assert!(nets[0].controlled_impedance.is_none());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_net_reads_one_existing_native_board_net() {
    let root = unique_project_root("datum-eda-cli-project-board-net-single-query");
    create_native_project(&root, Some("Board Net Single Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let class_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Net Single Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "AGND",
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
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output =
        execute(board_net_query_cli(&root, net_uuid)).expect("board net query should succeed");
    let net: Net = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(net.uuid, net_uuid);
    assert_eq!(net.name, "AGND");
    assert_eq!(net.class, class_uuid);

    let _ = std::fs::remove_dir_all(&root);
}
