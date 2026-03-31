use std::path::Path;

use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, seed_route_proposal_profile_project,
    unique_project_root,
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

#[test]
fn project_route_strategy_batch_evaluate_reports_per_request_results_and_summary() {
    let root_a = unique_project_root("datum-eda-cli-route-strategy-batch-profile");
    let (net_a, from_a, to_a) = seed_route_proposal_profile_project(&root_a);
    let root_b = unique_project_root("datum-eda-cli-route-strategy-batch-none");
    let (net_b, from_b, to_b) = seed_route_path_candidate_project(&root_b);
    rewrite_board_json(&root_b, |board| {
        board["nets"][net_b.to_string()]["class"] = serde_json::Value::Null;
    });
    let manifest_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-{}.json",
        Uuid::new_v4()
    ));
    write_batch_requests_manifest(
        &manifest_path,
        serde_json::json!([
            {
                "request_id": "request-a",
                "fixture_id": "fixture-a",
                "project_root": root_a,
                "net_uuid": net_a,
                "from_anchor_pad_uuid": from_a,
                "to_anchor_pad_uuid": to_a
            },
            {
                "request_id": "request-b",
                "fixture_id": "fixture-b",
                "project_root": root_b,
                "net_uuid": net_b,
                "from_anchor_pad_uuid": from_b,
                "to_anchor_pad_uuid": to_b
            }
        ]),
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

    assert_eq!(report["action"], "route_strategy_batch_evaluate");
    assert_eq!(
        report["kind"],
        "native_route_strategy_batch_result_artifact"
    );
    assert_eq!(report["version"], 1);
    assert_eq!(report["summary"]["total_evaluated_requests"], 2);
    assert_eq!(
        report["summary"]["recommendation_counts_by_profile"]["default"],
        2
    );
    assert_eq!(
        report["summary"]["delta_classification_counts"]["different_candidate_family"],
        1
    );
    assert_eq!(
        report["summary"]["delta_classification_counts"]["no_proposal_under_any_profile"],
        1
    );
    assert_eq!(report["summary"]["same_outcome_count"], 1);
    assert_eq!(report["summary"]["different_outcome_count"], 1);
    assert_eq!(report["summary"]["proposal_available_count"], 1);
    assert_eq!(report["summary"]["no_proposal_count"], 1);

    let results = report["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["identity"]["fixture_id"], "fixture-a");
    assert_eq!(results[0]["recommended_profile"], "default");
    assert_eq!(
        results[0]["delta_classification"],
        "different_candidate_family"
    );
    assert_eq!(
        results[0]["route_strategy_compare"]["recommended_profile"],
        "default"
    );
    assert_eq!(
        results[0]["route_strategy_delta"]["delta_classification"],
        "different_candidate_family"
    );
    assert_eq!(results[1]["identity"]["fixture_id"], "fixture-b");
    assert_eq!(
        results[1]["delta_classification"],
        "no_proposal_under_any_profile"
    );

    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn project_route_strategy_batch_evaluate_rejects_empty_manifest() {
    let manifest_path = std::env::temp_dir().join(format!(
        "datum-eda-route-strategy-batch-empty-{}.json",
        Uuid::new_v4()
    ));
    write_batch_requests_manifest(&manifest_path, serde_json::json!([]));

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

    let err = execute(cli).expect_err("empty batch manifest should fail");
    assert!(
        err.to_string()
            .contains("must contain at least one request")
    );

    let _ = std::fs::remove_file(&manifest_path);
}
