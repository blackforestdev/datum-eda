use super::main_tests_project_route_proposal_artifact::unique_project_root;
use super::*;

#[test]
fn project_write_route_strategy_curated_fixture_suite_writes_repeatable_manifest_and_projects() {
    let suite_root = unique_project_root("datum-eda-cli-route-strategy-curated-suite");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "write-route-strategy-curated-fixture-suite",
        "--out-dir",
        suite_root.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("suite writer should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report["action"],
        "write_route_strategy_curated_fixture_suite"
    );
    assert_eq!(
        report["suite_id"],
        "m6_route_strategy_curated_fixture_suite_v1"
    );
    assert_eq!(
        report["requests_manifest_kind"],
        "native_route_strategy_batch_requests"
    );
    assert_eq!(report["requests_manifest_version"], 1);
    assert_eq!(report["total_fixtures"], 4);
    assert_eq!(report["total_requests"], 4);

    let fixtures = report["fixtures"]
        .as_array()
        .expect("fixtures should be an array");
    assert_eq!(fixtures.len(), 4);
    let manifest_path = PathBuf::from(
        report["requests_manifest_path"]
            .as_str()
            .expect("manifest path should be a string"),
    );
    assert!(manifest_path.is_file());
    assert!(
        fixtures.iter().any(|fixture| {
            fixture["fixture_id"] == "profile-divergence-authored-copper"
                && fixture["coverage_labels"].as_array().is_some_and(|labels| {
                    labels
                        .iter()
                        .any(|label| label.as_str() == Some("different_candidate_family"))
                })
        }),
        "profile divergence fixture should be present"
    );

    let batch_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-batch-evaluate",
        "--requests",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let batch_output = execute(batch_cli).expect("batch evaluation should succeed");
    let batch_report: serde_json::Value =
        serde_json::from_str(&batch_output).expect("batch output should parse");

    assert_eq!(batch_report["summary"]["total_evaluated_requests"], 4);
    assert_eq!(
        batch_report["summary"]["delta_classification_counts"]["same_outcome"],
        2
    );
    assert_eq!(
        batch_report["summary"]["delta_classification_counts"]["different_candidate_family"],
        1
    );
    assert_eq!(
        batch_report["summary"]["delta_classification_counts"]["no_proposal_under_any_profile"],
        1
    );
    assert_eq!(batch_report["summary"]["proposal_available_count"], 3);
    assert_eq!(batch_report["summary"]["no_proposal_count"], 1);
    assert_eq!(batch_report["summary"]["same_outcome_count"], 3);

    let results = batch_report["results"]
        .as_array()
        .expect("results should be an array");
    assert!(
        results.iter().any(|result| {
            result["identity"]["fixture_id"] == "via-available"
                && result["delta_classification"] == "same_outcome"
                && result["route_strategy_report"]["selected_candidate"] == "route-path-candidate"
        }),
        "cross-layer via fixture should remain a same-outcome live selector case"
    );

    let _ = std::fs::remove_dir_all(&suite_root);
}

#[test]
fn project_capture_route_strategy_curated_baseline_writes_reusable_result_artifact() {
    let suite_root = unique_project_root("datum-eda-cli-route-strategy-curated-baseline");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "capture-route-strategy-curated-baseline",
        "--out-dir",
        suite_root.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("baseline capture should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "capture_route_strategy_curated_baseline");
    assert_eq!(
        report["result_kind"],
        "native_route_strategy_batch_result_artifact"
    );
    assert_eq!(report["result_version"], 1);
    assert_eq!(report["total_requests"], 4);
    assert_eq!(report["summary"]["different_outcome_count"], 1);
    let result_path = PathBuf::from(
        report["result_artifact_path"]
            .as_str()
            .expect("result path should be a string"),
    );
    assert!(result_path.is_file());

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-strategy-batch-result",
        result_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspection should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect output should parse");
    assert_eq!(inspect_report["summary"]["total_evaluated_requests"], 4);

    let gate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "gate-route-strategy-batch-result",
        result_path.to_str().unwrap(),
        result_path.to_str().unwrap(),
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

    let _ = std::fs::remove_dir_all(&suite_root);
}
