use super::main_tests_project_route_proposal_artifact::unique_project_root;
use super::*;

fn repo_route_strategy_baseline_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/quality/route_strategy_curated_baseline_v1")
}

#[test]
fn project_validate_repo_route_strategy_baseline_artifact() {
    let artifact_path = repo_route_strategy_baseline_dir().join("route-strategy-batch-result.json");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-route-strategy-batch-result",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("validation should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["action"], "validate_route_strategy_batch_result");
    assert_eq!(report["structurally_valid"], true);
    assert_eq!(report["version_compatible"], true);
}

#[test]
fn project_repo_route_strategy_baseline_matches_fresh_curated_capture() {
    let baseline_artifact =
        repo_route_strategy_baseline_dir().join("route-strategy-batch-result.json");
    let fresh_root = unique_project_root("datum-eda-cli-route-strategy-repo-baseline-fresh");
    let capture_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "capture-route-strategy-curated-baseline",
        "--out-dir",
        fresh_root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let capture_output = execute(capture_cli).expect("capture should succeed");
    let capture_report: serde_json::Value =
        serde_json::from_str(&capture_output).expect("capture output should parse");
    let fresh_artifact = PathBuf::from(
        capture_report["result_artifact_path"]
            .as_str()
            .expect("result path should be a string"),
    );

    let gate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "gate-route-strategy-batch-result",
        baseline_artifact.to_str().unwrap(),
        fresh_artifact.to_str().unwrap(),
        "--policy",
        "strict_identical",
    ])
    .expect("CLI should parse");
    let (gate_output, gate_status) =
        execute_with_exit_code(gate_cli).expect("gate command should succeed");
    let gate_report: serde_json::Value =
        serde_json::from_str(&gate_output).expect("gate output should parse");

    assert_eq!(gate_status, 0);
    assert_eq!(gate_report["passed"], true);
    assert_eq!(gate_report["comparison_classification"], "identical");

    let _ = std::fs::remove_dir_all(&fresh_root);
}
