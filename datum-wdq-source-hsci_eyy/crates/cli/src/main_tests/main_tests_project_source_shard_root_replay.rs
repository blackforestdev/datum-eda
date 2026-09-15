use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_resolve_debug_reports_missing_materialized_board_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-board-shard");
    create_native_project(
        &root,
        Some("Resolve Debug Missing Board Shard Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_path = root.join("board/board.json");
    let original_board_bytes = std::fs::read(&board_path).expect("board should read");
    let board: serde_json::Value =
        serde_json::from_slice(&original_board_bytes).expect("board should parse");
    let board_id =
        Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist")).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before board name update");
    let batch_path = root.join("commit-batch-missing-board-shard.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "create missing promoted board shard".to_string(),
        },
        operations: vec![Operation::SetBoardName {
            board_id,
            name: "Journaled Missing Board Name".to_string(),
        }],
    };
    std::fs::write(
        &batch_path,
        to_json_deterministic(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
            "--commit-batch",
            batch_path.to_str().unwrap(),
            "--apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug --apply should succeed");
    std::fs::remove_file(&board_path).expect("promoted board shard should remove");

    let output = execute(
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
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| {
                diagnostic["code"] == "missing_required_shard"
                    && diagnostic["path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with("board/board.json"))
            }),
        "resolve-debug should expose missing promoted board shard as a required-shard diagnostic"
    );
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| {
                diagnostic["code"] == "journal_replay_failed"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("could not replay"))
            }),
        "resolve-debug should expose journal replay failure as a diagnostic"
    );

    let _ = std::fs::remove_dir_all(&root);
}
