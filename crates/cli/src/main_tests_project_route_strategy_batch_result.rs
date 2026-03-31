use std::path::{Path, PathBuf};

use super::main_tests_project_route_proposal_artifact::{
    seed_route_proposal_profile_project, unique_project_root,
};
use super::*;

fn write_batch_requests_manifest(path: &Path, requests: serde_json::Value) {
    std::fs::write(
        path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "kind": "native_route_strategy_batch_requests",
                "version": 1,
                "requests": requests,
            }))
            .expect("manifest serialization should succeed")
        ),
    )
    .expect("batch manifest should write");
}

fn write_batch_result_artifact(path: &Path, report: &serde_json::Value) {
    std::fs::write(
        path,
        format!(
            "{}\n",
            to_json_deterministic(report).expect("artifact serialization should succeed")
        ),
    )
    .expect("batch result artifact should write");
}

fn build_saved_batch_result_artifact() -> PathBuf {
    let root = unique_project_root("datum-eda-cli-route-strategy-batch-result");
    let (net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid) =
        seed_route_proposal_profile_project(&root);
    let manifest_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-result-requests-{}.json",
        Uuid::new_v4()
    ));
    write_batch_requests_manifest(
        &manifest_path,
        serde_json::json!([{
            "request_id": "request-a",
            "fixture_id": "fixture-a",
            "project_root": root,
            "net_uuid": net_uuid,
            "from_anchor_pad_uuid": from_anchor_pad_uuid,
            "to_anchor_pad_uuid": to_anchor_pad_uuid
        }]),
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-batch-evaluate",
        "--requests",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("batch evaluation should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    let artifact_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-result-artifact-{}.json",
        Uuid::new_v4()
    ));
    write_batch_result_artifact(&artifact_path, &report);
    let _ = std::fs::remove_file(&manifest_path);
    artifact_path
}

fn build_saved_batch_result_artifact_with_request_id(
    request_id: &str,
    fixture_id: &str,
) -> PathBuf {
    let root = unique_project_root("datum-eda-cli-route-strategy-batch-result-custom");
    let (net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid) =
        seed_route_proposal_profile_project(&root);
    let manifest_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-result-custom-requests-{}.json",
        Uuid::new_v4()
    ));
    write_batch_requests_manifest(
        &manifest_path,
        serde_json::json!([{
            "request_id": request_id,
            "fixture_id": fixture_id,
            "project_root": root,
            "net_uuid": net_uuid,
            "from_anchor_pad_uuid": from_anchor_pad_uuid,
            "to_anchor_pad_uuid": to_anchor_pad_uuid
        }]),
    );
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-batch-evaluate",
        "--requests",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("batch evaluation should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    let artifact_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-result-custom-artifact-{}.json",
        Uuid::new_v4()
    ));
    write_batch_result_artifact(&artifact_path, &report);
    let _ = std::fs::remove_file(&manifest_path);
    artifact_path
}

fn copy_artifact_into_dir(artifact_path: &Path, dir: &Path, name: &str) -> PathBuf {
    let target = dir.join(name);
    std::fs::copy(artifact_path, &target).expect("artifact copy should succeed");
    target
}

#[test]
fn project_inspect_route_strategy_batch_result_reports_summary_and_results() {
    let artifact_path = build_saved_batch_result_artifact();

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-strategy-batch-result",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("inspection should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "inspect_route_strategy_batch_result");
    assert_eq!(
        report["kind"],
        "native_route_strategy_batch_result_artifact"
    );
    assert_eq!(report["summary"]["total_evaluated_requests"], 1);
    assert_eq!(report["malformed_entries"].as_array().unwrap().len(), 0);
    assert_eq!(report["results"].as_array().unwrap().len(), 1);
    assert_eq!(report["results"][0]["identity"]["fixture_id"], "fixture-a");

    let _ = std::fs::remove_file(&artifact_path);
}

#[test]
fn project_validate_route_strategy_batch_result_reports_structural_match() {
    let artifact_path = build_saved_batch_result_artifact();

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
    assert_eq!(report["request_result_count_matches_summary"], true);
    assert_eq!(report["recommendation_counts_match_summary"], true);
    assert_eq!(report["delta_classification_counts_match_summary"], true);
    assert_eq!(report["outcome_counts_match_summary"], true);
    assert_eq!(report["proposal_counts_match_summary"], true);
    assert_eq!(report["malformed_entries"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_file(&artifact_path);
}

#[test]
fn project_validate_route_strategy_batch_result_reports_malformed_counts() {
    let artifact_path = build_saved_batch_result_artifact();
    let mut report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("artifact should read"),
    )
    .expect("artifact should parse");
    report["summary"]["same_outcome_count"] = serde_json::json!(0);
    report["results"][0]
        .as_object_mut()
        .expect("result should be object")
        .remove("delta_classification");
    write_batch_result_artifact(&artifact_path, &report);

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

    assert_eq!(report["structurally_valid"], false);
    assert_eq!(report["version_compatible"], true);
    assert_eq!(report["outcome_counts_match_summary"], false);
    assert_eq!(report["malformed_entries"].as_array().unwrap().len(), 1);
    assert!(
        report["malformed_entries"][0]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "missing delta_classification")
    );

    let _ = std::fs::remove_file(&artifact_path);
}

#[test]
fn project_compare_route_strategy_batch_result_reports_per_request_changes() {
    let before_artifact = build_saved_batch_result_artifact();
    let after_artifact = build_saved_batch_result_artifact();
    let mut after_report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&after_artifact).expect("artifact should read"),
    )
    .expect("artifact should parse");
    after_report["results"][0]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["results"][0]["delta_classification"] = serde_json::json!("same_outcome");
    after_report["results"][0]["route_strategy_report"]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["results"][0]["route_strategy_report"]["selected_candidate"] =
        serde_json::json!("authored-copper-graph");
    after_report["results"][0]["route_strategy_report"]["selected_policy"] =
        serde_json::json!("plain");
    after_report["summary"]["recommendation_counts_by_profile"] = serde_json::json!({
        "authored-copper-priority": 1
    });
    after_report["summary"]["delta_classification_counts"] = serde_json::json!({
        "same_outcome": 1
    });
    after_report["summary"]["same_outcome_count"] = serde_json::json!(1);
    after_report["summary"]["different_outcome_count"] = serde_json::json!(0);
    write_batch_result_artifact(&after_artifact, &after_report);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-route-strategy-batch-result",
        before_artifact.to_str().unwrap(),
        after_artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("comparison should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "compare_route_strategy_batch_result");
    assert_eq!(report["compatible_artifacts"], true);
    assert_eq!(
        report["comparison_classification"],
        "per_request_outcomes_changed"
    );
    assert_eq!(
        report["changed_common_requests"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        report["changed_common_requests"][0]["recommendation_changed"],
        true
    );
    assert_eq!(
        report["changed_common_requests"][0]["selected_live_outcome_changed"],
        true
    );

    let _ = std::fs::remove_file(&before_artifact);
    let _ = std::fs::remove_file(&after_artifact);
}

#[test]
fn project_compare_route_strategy_batch_result_reports_incompatible_artifacts() {
    let before_artifact =
        build_saved_batch_result_artifact_with_request_id("request-a", "fixture-a");
    let after_artifact =
        build_saved_batch_result_artifact_with_request_id("request-b", "fixture-b");
    let mut after_report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&after_artifact).expect("artifact should read"),
    )
    .expect("artifact should parse");
    after_report["requests_manifest_version"] = serde_json::json!(2);
    write_batch_result_artifact(&after_artifact, &after_report);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-route-strategy-batch-result",
        before_artifact.to_str().unwrap(),
        after_artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("comparison should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report["comparison_classification"],
        "incompatible_artifacts"
    );
    assert_eq!(report["compatible_artifacts"], false);

    let _ = std::fs::remove_file(&before_artifact);
    let _ = std::fs::remove_file(&after_artifact);
}

#[test]
fn project_gate_route_strategy_batch_result_strict_identical_fails_on_changes() {
    let before_artifact = build_saved_batch_result_artifact();
    let after_artifact = build_saved_batch_result_artifact();
    let mut after_report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&after_artifact).expect("artifact should read"),
    )
    .expect("artifact should parse");
    after_report["results"][0]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["results"][0]["route_strategy_report"]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["summary"]["recommendation_counts_by_profile"] = serde_json::json!({
        "authored-copper-priority": 1
    });
    write_batch_result_artifact(&after_artifact, &after_report);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "gate-route-strategy-batch-result",
        before_artifact.to_str().unwrap(),
        after_artifact.to_str().unwrap(),
        "--policy",
        "strict_identical",
    ])
    .expect("CLI should parse");

    let (output, exit_code) = execute_with_exit_code(cli).expect("gate command should execute");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(exit_code, 2);
    assert_eq!(report["action"], "gate_route_strategy_batch_result");
    assert_eq!(report["selected_gate_policy"], "strict_identical");
    assert_eq!(report["passed"], false);
    assert_eq!(report["changed_recommendations"], 1);

    let _ = std::fs::remove_file(&before_artifact);
    let _ = std::fs::remove_file(&after_artifact);
}

#[test]
fn project_gate_route_strategy_batch_result_allow_aggregate_only_passes_aggregate_change() {
    let before_artifact =
        build_saved_batch_result_artifact_with_request_id("request-a", "fixture-a");
    let after_artifact =
        build_saved_batch_result_artifact_with_request_id("request-b", "fixture-b");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "gate-route-strategy-batch-result",
        before_artifact.to_str().unwrap(),
        after_artifact.to_str().unwrap(),
        "--policy",
        "allow_aggregate_only",
    ])
    .expect("CLI should parse");

    let (output, exit_code) = execute_with_exit_code(cli).expect("gate command should execute");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(exit_code, 0);
    assert_eq!(report["selected_gate_policy"], "allow_aggregate_only");
    assert_eq!(report["passed"], true);
    assert_eq!(
        report["comparison_classification"],
        "aggregate_only_changed"
    );

    let _ = std::fs::remove_file(&before_artifact);
    let _ = std::fs::remove_file(&after_artifact);
}

#[test]
fn project_gate_route_strategy_batch_result_fail_on_recommendation_change_fails_only_on_recommendation_changes()
 {
    let before_artifact = build_saved_batch_result_artifact();
    let after_artifact = build_saved_batch_result_artifact();
    let mut after_report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&after_artifact).expect("artifact should read"),
    )
    .expect("artifact should parse");
    after_report["results"][0]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["results"][0]["route_strategy_report"]["recommended_profile"] =
        serde_json::json!("authored-copper-priority");
    after_report["summary"]["recommendation_counts_by_profile"] = serde_json::json!({
        "authored-copper-priority": 1
    });
    write_batch_result_artifact(&after_artifact, &after_report);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "gate-route-strategy-batch-result",
        before_artifact.to_str().unwrap(),
        after_artifact.to_str().unwrap(),
        "--policy",
        "fail_on_recommendation_change",
    ])
    .expect("CLI should parse");

    let (output, exit_code) = execute_with_exit_code(cli).expect("gate command should execute");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(exit_code, 2);
    assert_eq!(
        report["selected_gate_policy"],
        "fail_on_recommendation_change"
    );
    assert_eq!(report["passed"], false);
    assert_eq!(report["changed_recommendations"], 1);

    let _ = std::fs::remove_file(&before_artifact);
    let _ = std::fs::remove_file(&after_artifact);
}

#[test]
fn project_summarize_route_strategy_batch_results_reports_directory_summary_and_baseline_gate() {
    let baseline_artifact =
        build_saved_batch_result_artifact_with_request_id("request-a", "fixture-a");
    let later_artifact =
        build_saved_batch_result_artifact_with_request_id("request-b", "fixture-b");
    let dir = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-results-index-{}",
        Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).expect("index dir should create");
    let baseline_in_dir = copy_artifact_into_dir(&baseline_artifact, &dir, "run-a.json");
    let later_in_dir = copy_artifact_into_dir(&later_artifact, &dir, "run-b.json");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "summarize-route-strategy-batch-results",
        "--dir",
        dir.to_str().unwrap(),
        "--baseline",
        baseline_in_dir.to_str().unwrap(),
        "--policy",
        "allow_aggregate_only",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("summary should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["action"], "summarize_route_strategy_batch_results");
    assert_eq!(report["summary"]["total_artifacts"], 2);
    assert_eq!(report["summary"]["structurally_valid_artifacts"], 2);
    assert_eq!(
        report["baseline_artifact"],
        baseline_in_dir.to_str().unwrap()
    );
    assert_eq!(report["selected_gate_policy"], "allow_aggregate_only");
    assert_eq!(report["artifacts"].as_array().unwrap().len(), 2);
    assert!(
        report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |entry| entry["artifact_path"] == later_in_dir.to_str().unwrap()
                    && entry["baseline_gate"]["selected_gate_policy"] == "allow_aggregate_only"
            )
    );

    let _ = std::fs::remove_file(&baseline_artifact);
    let _ = std::fs::remove_file(&later_artifact);
    let _ = std::fs::remove_file(&baseline_in_dir);
    let _ = std::fs::remove_file(&later_in_dir);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn project_summarize_route_strategy_batch_results_reports_invalid_artifact_entry() {
    let valid_artifact = build_saved_batch_result_artifact();
    let invalid_artifact = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-invalid-{}.json",
        Uuid::new_v4()
    ));
    std::fs::write(&invalid_artifact, "{not-json\n").expect("invalid artifact should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "summarize-route-strategy-batch-results",
        "--artifact",
        valid_artifact.to_str().unwrap(),
        "--artifact",
        invalid_artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("summary should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["summary"]["total_artifacts"], 2);
    assert_eq!(report["summary"]["structurally_invalid_artifacts"], 1);
    assert!(
        report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |entry| entry["artifact_path"] == invalid_artifact.to_str().unwrap()
                    && entry["structurally_valid"] == false
                    && entry["validation_error"].is_string()
            )
    );

    let _ = std::fs::remove_file(&valid_artifact);
    let _ = std::fs::remove_file(&invalid_artifact);
}
