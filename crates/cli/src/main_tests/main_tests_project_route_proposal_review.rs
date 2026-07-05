use super::main_tests_project_route_proposal_artifact::{
    seed_route_path_candidate_project, unique_project_root,
};
use super::*;

#[test]
fn project_review_route_proposal_reviews_live_selected_route() {
    let root = unique_project_root("datum-eda-cli-project-review-route-proposal-live");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "review-route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("live review should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "review_route_proposal");
    assert_eq!(report["review_source"], "selected_route_proposal");
    assert_eq!(report["selection_profile"], "default");
    assert_eq!(report["selected_candidate"], "route-path-candidate");
    assert_eq!(report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(report["actions"], 1);
    assert_eq!(report["draw_track_actions"], 1);
    assert_eq!(report["proposal_actions"].as_array().unwrap().len(), 1);
}

#[test]
fn project_review_route_proposal_reviews_saved_artifact() {
    let root = unique_project_root("datum-eda-cli-project-review-route-proposal-artifact");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);
    let artifact = root.join("route-proposal.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("export should succeed");

    let review_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "review-route-proposal",
        "--artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(review_cli).expect("artifact review should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "review_route_proposal");
    assert_eq!(report["review_source"], "route_proposal_artifact");
    assert_eq!(report["artifact_path"], artifact.display().to_string());
    assert_eq!(report["kind"], "native_route_proposal_artifact");
    assert_eq!(report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(report["actions"], 1);
    assert_eq!(report["draw_track_actions"], 1);
    assert_eq!(report["proposal_actions"].as_array().unwrap().len(), 1);
}
