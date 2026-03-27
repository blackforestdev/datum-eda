use super::*;
use eda_engine::board::Zone;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_zones_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-zones",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_zone_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-zone");
    create_native_project(&root, Some("Board Zone Demo".to_string()))
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
        "place-board-zone",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--vertex",
        "0:0",
        "--vertex",
        "1000:0",
        "--vertex",
        "1000:1000",
        "--layer",
        "1",
        "--priority",
        "2",
        "--thermal-relief",
        "true",
        "--thermal-gap-nm",
        "250000",
        "--thermal-spoke-width-nm",
        "200000",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("place board zone should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    let zone_uuid = placed["zone_uuid"].as_str().unwrap().to_string();

    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("query output should parse");
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].uuid.to_string(), zone_uuid);
    assert_eq!(zones[0].net.to_string(), net_uuid);
    assert_eq!(zones[0].layer, 1);
    assert_eq!(zones[0].priority, 2);
    assert!(zones[0].thermal_relief);
    assert_eq!(zones[0].thermal_gap, 250000);
    assert_eq!(zones[0].thermal_spoke_width, 200000);
    assert_eq!(zones[0].polygon.vertices.len(), 3);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-zone",
        root.to_str().unwrap(),
        "--zone",
        &zone_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board zone should succeed");
    assert!(delete_output.contains("action: delete_board_zone"));

    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("query output should parse");
    assert!(zones.is_empty());

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_zones: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_zones_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-zone-query");
    create_native_project(&root, Some("Board Zone Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Zone Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 0, "y": 0 },
                                { "x": 10, "y": 0 },
                                { "x": 10, "y": 10 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 3,
                        "thermal_relief": true,
                        "thermal_gap": 250000,
                        "thermal_spoke_width": 200000
                    }
                },
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

    let output = execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].uuid, zone_uuid);
    assert_eq!(zones[0].net, net_uuid);
    assert_eq!(zones[0].layer, 1);
    assert_eq!(zones[0].priority, 3);
    assert!(zones[0].thermal_relief);
    assert_eq!(zones[0].thermal_gap, 250000);
    assert_eq!(zones[0].thermal_spoke_width, 200000);
    assert_eq!(zones[0].polygon.vertices.len(), 3);

    let _ = std::fs::remove_dir_all(&root);
}
