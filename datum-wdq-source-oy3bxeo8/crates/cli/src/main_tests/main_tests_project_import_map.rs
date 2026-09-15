use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

#[test]
fn project_query_import_map_is_resolver_backed_and_non_mutating() {
    let root = unique_project_root("datum-eda-cli-project-query-import-map");
    create_native_project(&root, Some("Import Map Query Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board_path = root.join("board/board.json");
    let board_before = std::fs::read(&board_path).expect("board should read");
    let board: serde_json::Value = serde_json::from_slice(&board_before).unwrap();
    let board_id = Uuid::parse_str(board["uuid"].as_str().unwrap()).unwrap();
    let resolve_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("resolve-debug should succeed");
    let resolve_report: serde_json::Value = serde_json::from_str(&resolve_output).unwrap();
    let board_shard_id = resolve_report["source_shards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|shard| shard["path"] == "board/board.json")
        .and_then(|shard| shard["shard_id"].as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("board shard should be discoverable");

    let import_map_path = root.join(".datum/import_map/kicad.json");
    std::fs::create_dir_all(import_map_path.parent().unwrap()).unwrap();
    std::fs::write(
        &import_map_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "entries": [{
                    "import_key": "kicad:board:root",
                    "object_id": board_id,
                    "source_shard_id": board_shard_id,
                    "source_hash": "fixture-source-hash"
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "import-map",
        ])
        .expect("CLI should parse"),
    )
    .expect("import-map query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "import_map_query_v1");
    assert_eq!(report["import_map_count"], 1);
    assert_eq!(
        report["entries"]["kicad:board:root"]["source_hash"],
        "fixture-source-hash"
    );
    assert_eq!(report["entries"]["kicad:board:root"]["status"], "active");
    assert_eq!(std::fs::read(board_path).unwrap(), board_before);
}
