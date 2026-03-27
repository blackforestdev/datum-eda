use super::*;
use eda_engine::board::Track;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_tracks_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-tracks",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_track_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-track");
    create_native_project(&root, Some("Board Track Demo".to_string()))
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

    let draw_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "draw-board-track",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--from-x-nm",
        "1000",
        "--from-y-nm",
        "2000",
        "--to-x-nm",
        "3000",
        "--to-y-nm",
        "4000",
        "--width-nm",
        "250000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let draw_output = execute(draw_cli).expect("draw board track should succeed");
    let drawn: serde_json::Value =
        serde_json::from_str(&draw_output).expect("draw output should parse");
    let track_uuid = drawn["track_uuid"].as_str().unwrap().to_string();

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("query output should parse");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].uuid.to_string(), track_uuid);
    assert_eq!(tracks[0].net.to_string(), net_uuid);
    assert_eq!(tracks[0].from.x, 1000);
    assert_eq!(tracks[0].from.y, 2000);
    assert_eq!(tracks[0].to.x, 3000);
    assert_eq!(tracks[0].to.y, 4000);
    assert_eq!(tracks[0].width, 250000);
    assert_eq!(tracks[0].layer, 1);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-track",
        root.to_str().unwrap(),
        "--track",
        &track_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board track should succeed");
    assert!(delete_output.contains("action: delete_board_track"));

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("query output should parse");
    assert!(tracks.is_empty());

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_tracks: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_tracks_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-track-query");
    create_native_project(&root, Some("Board Track Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Track Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {
                    track_uuid.to_string(): {
                        "uuid": track_uuid,
                        "net": net_uuid,
                        "from": { "x": 10, "y": 20 },
                        "to": { "x": 30, "y": 40 },
                        "width": 150000,
                        "layer": 1
                    }
                },
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

    let output = execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].uuid, track_uuid);
    assert_eq!(tracks[0].net, net_uuid);
    assert_eq!(tracks[0].from.x, 10);
    assert_eq!(tracks[0].from.y, 20);
    assert_eq!(tracks[0].to.x, 30);
    assert_eq!(tracks[0].to.y, 40);
    assert_eq!(tracks[0].width, 150000);
    assert_eq!(tracks[0].layer, 1);

    let _ = std::fs::remove_dir_all(&root);
}
