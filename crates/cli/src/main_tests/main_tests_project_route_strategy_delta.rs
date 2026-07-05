use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, seed_route_proposal_profile_project,
    unique_project_root,
};
use super::*;

#[test]
fn project_route_strategy_delta_reports_different_candidate_family_when_profiles_diverge() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-delta-diverge");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_proposal_profile_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-delta",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy delta should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "route_strategy_delta");
    assert_eq!(report["outcomes_match"], false);
    assert_eq!(report["outcome_relation"], "different");
    assert_eq!(report["delta_classification"], "different_candidate_family");
    let profiles = report["profiles"]
        .as_array()
        .expect("profiles should be an array");
    assert_eq!(profiles[0]["profile"], "default");
    assert_eq!(profiles[0]["selected_candidate"], "route-path-candidate");
    assert_eq!(profiles[1]["profile"], "authored-copper-priority");
    assert_eq!(profiles[1]["selected_candidate"], "authored-copper-graph");
}

#[test]
fn project_route_strategy_delta_reports_same_outcome_when_profiles_converge() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-delta-same");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-delta",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy delta should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["outcomes_match"], true);
    assert_eq!(report["outcome_relation"], "identical");
    assert_eq!(report["delta_classification"], "same_outcome");
}

#[test]
fn project_route_strategy_delta_reports_no_proposal_when_neither_profile_succeeds() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-delta-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-delta",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy delta should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report["delta_classification"],
        "no_proposal_under_any_profile"
    );
    assert!(
        report["material_difference"]
            .as_str()
            .expect("material_difference should be a string")
            .contains("neither accepted profile currently yields")
    );
}
