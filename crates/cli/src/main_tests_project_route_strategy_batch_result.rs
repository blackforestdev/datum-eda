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
