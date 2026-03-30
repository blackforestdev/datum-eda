use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, unique_project_root,
};
use super::*;

#[test]
fn project_route_proposal_explain_reports_selected_candidate_reason() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-explain");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-proposal-explain",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route proposal explain should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "route_proposal_explain");
    assert_eq!(report["selected_candidate"], "route-path-candidate");
    let explanation = report["explanation"]
        .as_str()
        .expect("explanation should be a string");
    assert!(explanation.contains("first candidate in deterministic order"));
}

#[test]
fn project_route_proposal_explain_reports_no_candidate_when_all_fail() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-explain-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-proposal-explain",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route proposal explain should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report["status"],
        "no_route_proposal_under_current_authored_constraints"
    );
    assert!(report["selected_candidate"].is_null());
    assert_eq!(
        report["explanation"],
        "no candidate produced a valid proposal action set under current authored constraints"
    );
}
