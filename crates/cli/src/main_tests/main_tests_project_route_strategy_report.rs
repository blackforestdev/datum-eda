use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, seed_route_proposal_profile_project,
    unique_project_root,
};
use super::*;

#[test]
fn project_route_strategy_report_recommends_default_profile_for_default_objective() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-report-default");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-report",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "route_strategy_report");
    assert_eq!(report["objective"], "default");
    assert_eq!(report["recommended_profile"], "default");
    assert_eq!(report["selected_candidate"], "route-path-candidate");
    assert!(
        report["recommendation_rule"]
            .as_str()
            .expect("recommendation_rule should be a string")
            .contains("objective default maps directly to selector profile default")
    );
}

#[test]
fn project_route_strategy_report_recommends_authored_copper_priority_profile() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-report-authored-copper");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_proposal_profile_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-report",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
        "--objective",
        "authored-copper-priority",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["objective"], "authored-copper-priority");
    assert_eq!(report["recommended_profile"], "authored-copper-priority");
    assert_eq!(report["selected_candidate"], "authored-copper-graph");
    assert_eq!(report["selected_policy"], "plain");
    assert!(
        report["explanation"]
            .as_str()
            .expect("explanation should be a string")
            .contains("prepends the accepted authored-copper-graph policy family")
    );
}

#[test]
fn project_route_strategy_report_reports_when_no_route_is_available_under_objective() {
    let root = unique_project_root("datum-eda-cli-project-route-strategy-report-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-strategy-report",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route strategy report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report["selector_status"],
        "no_route_proposal_under_current_authored_constraints"
    );
    assert!(report["selected_candidate"].is_null());
    assert!(
        report["explanation"]
            .as_str()
            .expect("explanation should be a string")
            .contains("finds no selectable route proposal")
    );
}
