use super::main_tests_project_route_proposal_artifact::{
    rewrite_board_json, seed_route_path_candidate_project, unique_project_root,
};
use super::*;

#[test]
fn project_route_proposal_selects_first_same_layer_candidate() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-selection-direct");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route proposal should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report.get("status"),
        Some(&serde_json::Value::String(
            "deterministic_route_proposal_selected".to_string()
        ))
    );
    assert_eq!(
        report.get("selected_candidate"),
        Some(&serde_json::Value::String(
            "route-path-candidate".to_string()
        ))
    );
}

#[test]
fn project_route_proposal_reports_deterministic_candidate_order() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-selection-order");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route proposal should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report.get("attempted_candidates"),
        Some(&serde_json::Value::Number(18usize.into()))
    );
    let selection_rule = report
        .get("selection_rule")
        .and_then(serde_json::Value::as_str)
        .expect("selection_rule should be present");
    assert!(selection_rule.starts_with("select the first successful candidate in this deterministic order: route-path-candidate > route-path-candidate-orthogonal-dogleg"));
}

#[test]
fn project_route_proposal_reports_no_selection_when_all_candidates_fail() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-selection-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("route proposal should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(
        report.get("status"),
        Some(&serde_json::Value::String(
            "no_route_proposal_under_current_authored_constraints".to_string()
        ))
    );
    assert_eq!(
        report.get("selected_candidate"),
        Some(&serde_json::Value::Null)
    );
}
