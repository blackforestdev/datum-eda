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

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
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

    let output =
        execute(board_stackup_query_cli(&root)).expect("board stackup query should succeed");
    let stackup: Vec<StackupLayer> =
        serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(stackup.len(), 2);
    assert_eq!(stackup[0].name, "Top");
    assert_eq!(stackup[1].layer_type, StackupLayerType::Dielectric);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_default_top_stackup_retrofits_missing_seed_layers() {
    let root = unique_project_root("datum-eda-cli-project-board-stackup-defaults");
    create_native_project(&root, Some("Board Stackup Defaults Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Stackup Defaults Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
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

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-default-top-stackup",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("add default top stackup should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["action"], "add_default_top_stackup");
    assert_eq!(report["layer_count"], 5);

    let stackup_output =
        execute(board_stackup_query_cli(&root)).expect("board stackup query should succeed");
    let stackup: Vec<StackupLayer> =
        serde_json::from_str(&stackup_output).expect("query output should parse");
    assert_eq!(stackup.len(), 5);
    assert_eq!(stackup[0].id, 1);
    assert_eq!(stackup[1].id, 2);
    assert_eq!(stackup[1].layer_type, StackupLayerType::SolderMask);
    assert_eq!(stackup[2].id, 3);
    assert_eq!(stackup[3].id, 4);
    assert_eq!(stackup[3].layer_type, StackupLayerType::Paste);
    assert_eq!(stackup[4].id, 41);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_default_top_stackup_rejects_conflicting_default_layer_ids() {
    let root = unique_project_root("datum-eda-cli-project-board-stackup-default-conflict");
    create_native_project(&root, Some("Board Stackup Conflict Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Stackup Conflict Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
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

    let cli = Cli::try_parse_from([
        "eda",
        "project",
        "add-default-top-stackup",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let error = execute(cli).expect_err("conflicting stackup retrofit should fail");
    assert!(
        error
            .to_string()
            .contains("layer id 2 already exists with conflicting definition")
    );

    let _ = std::fs::remove_dir_all(&root);
}
