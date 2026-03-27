use super::*;
use eda_engine::board::{StackupLayer, StackupLayerType};
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_stackup_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-stackup",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_stackup_replacement_round_trips_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-stackup");
    create_native_project(&root, Some("Board Stackup Demo".to_string()))
        .expect("initial scaffold should succeed");

    let set_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-stackup",
        root.to_str().unwrap(),
        "--layer",
        "1:Top:Copper:35000",
        "--layer",
        "2:Core:Dielectric:1600000",
        "--layer",
        "3:Bottom:Copper:35000",
    ])
    .expect("CLI should parse");

    let output = execute(set_cli).expect("set board stackup should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["layer_count"], 3);

    let stackup_output =
        execute(board_stackup_query_cli(&root)).expect("board stackup query should succeed");
    let stackup: Vec<StackupLayer> =
        serde_json::from_str(&stackup_output).expect("query output should parse");
    assert_eq!(stackup.len(), 3);
    assert_eq!(stackup[0].id, 1);
    assert_eq!(stackup[0].name, "Top");
    assert_eq!(stackup[0].layer_type, StackupLayerType::Copper);
    assert_eq!(stackup[1].layer_type, StackupLayerType::Dielectric);
    assert_eq!(stackup[2].thickness_nm, 35000);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_layers: 3"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_stackup_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-stackup-query");
    create_native_project(&root, Some("Board Stackup Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Stackup Query Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 }
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_stackup_query_cli(&root)).expect("board stackup query should succeed");
    let stackup: Vec<StackupLayer> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(stackup.len(), 2);
    assert_eq!(stackup[0].name, "Top");
    assert_eq!(stackup[1].layer_type, StackupLayerType::Dielectric);

    let _ = std::fs::remove_dir_all(&root);
}
