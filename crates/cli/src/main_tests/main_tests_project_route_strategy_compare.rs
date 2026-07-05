use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, seed_route_proposal_profile_project,
    unique_project_root,
};
use super::*;

#[test]
fn project_route_strategy_compare_reports_all_accepted_profiles() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-compare");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_proposal_profile_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-compare",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "route_strategy_compare");
    assert_eq!(report["recommended_profile"], "default");
    let entries = report["entries"]
        .as_array()
        .expect("entries should be an array");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["profile"], "default");
    assert_eq!(entries[0]["proposal_available"], true);
    assert_eq!(entries[0]["selected_candidate"], "route-path-candidate");
    assert_eq!(entries[1]["profile"], "authored-copper-priority");
    assert_eq!(entries[1]["proposal_available"], true);
    assert_eq!(entries[1]["selected_candidate"], "authored-copper-graph");
}

#[test]
fn project_route_strategy_compare_recommends_default_when_no_profile_yields_proposal() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-compare-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-compare",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["recommended_profile"], "default");
    assert!(
        report["recommendation_reason"]
            .as_str()
            .expect("recommendation_reason should be a string")
            .contains("no accepted profile yields a proposal")
    );
    let entries = report["entries"]
        .as_array()
        .expect("entries should be an array");
    assert!(
        entries
            .iter()
            .all(|entry| entry["proposal_available"] == false)
    );
}

#[test]
fn project_route_strategy_compare_reports_deterministic_profile_distinctions() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-compare-distinction");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-compare",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    let entries = report["entries"]
        .as_array()
        .expect("entries should be an array");

    assert_eq!(
        entries[0]["distinction"],
        "baseline profile: preserves the accepted selector family order exactly"
    );
    assert_eq!(
        entries[1]["distinction"],
        "reuse-priority profile: prepends the accepted authored-copper-graph policy family ahead of the unchanged default order"
    );
}
