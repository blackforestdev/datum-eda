use super::main_tests_project_route_proposal_artifact::{
    seed_route_path_candidate_authored_copper_graph_obstacle_aware_project,
    seed_route_path_candidate_orthogonal_dogleg_project,
    seed_route_path_candidate_orthogonal_two_bend_project, seed_route_path_candidate_project,
    seed_route_path_candidate_via_project, unique_project_root,
};
use super::*;
use clap::CommandFactory;

fn route_path_candidate_explain_query_cli(
    root: &Path,
    net_uuid: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    candidate: Option<&str>,
    policy: Option<&str>,
) -> Cli {
    let mut args = vec![
        "eda".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "project".to_string(),
        "query".to_string(),
        root.to_str().unwrap().to_string(),
        "route-path-candidate-explain".to_string(),
        "--net".to_string(),
        net_uuid.to_string(),
        "--from-anchor".to_string(),
        from_anchor.to_string(),
        "--to-anchor".to_string(),
        to_anchor.to_string(),
    ];
    if let Some(candidate) = candidate {
        args.push("--candidate".to_string());
        args.push(candidate.to_string());
    }
    if let Some(policy) = policy {
        args.push("--policy".to_string());
        args.push(policy.to_string());
    }
    Cli::try_parse_from(args).expect("CLI should parse")
}

#[test]
fn project_query_route_path_candidate_explain_generic_surface_defaults_to_single_layer_contract() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-explain-generic-default");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        None,
        None,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_route_path_candidate_explain_v1");
    assert_eq!(report["status"], "deterministic_path_found");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_explain_generic_surface_supports_policy_selected_authored_copper_graph()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-explain-generic-policy-query",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _) =
        seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(&root);

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        Some("authored-copper-graph"),
        Some("plain"),
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_explain_v1"
    );
    assert_eq!(report["policy"], "plain");
    assert_eq!(report["status"], "deterministic_path_found");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_explain_generic_surface_supports_orthogonal_dogleg_candidate()
{
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-explain-generic-dogleg");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) =
        seed_route_path_candidate_orthogonal_dogleg_project(&root);

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        Some("route-path-candidate-orthogonal-dogleg"),
        None,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_orthogonal_dogleg_explain_v1"
    );
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["selected_dogleg"]["corner"]["x"], 100000);
    assert_eq!(report["selected_dogleg"]["corner"]["y"], 900000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_explain_generic_surface_supports_orthogonal_two_bend_candidate()
{
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-explain-generic-two-bend",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) =
        seed_route_path_candidate_orthogonal_two_bend_project(&root);

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        Some("route-path-candidate-orthogonal-two-bend"),
        None,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_orthogonal_two_bend_explain_v1"
    );
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["selected_path"]["detour_coordinate"], 0);
    assert_eq!(report["selected_path"]["points"].as_array().unwrap().len(), 4);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn legacy_route_path_candidate_via_explain_help_marks_command_deprecated() {
    let mut project_command = Cli::command()
        .find_subcommand_mut("project")
        .expect("project command should exist")
        .clone();
    let mut query_command = project_command
        .find_subcommand_mut("query")
        .expect("query command should exist")
        .clone();
    let mut legacy = query_command
        .find_subcommand_mut("route-path-candidate-via-explain")
        .expect("legacy command should exist")
        .clone();
    let help = legacy.render_long_help().to_string();
    assert!(help.contains("Deprecated compatibility wrapper"));
    assert!(help.contains("route-path-candidate-explain --candidate route-path-candidate-via"));
}

#[test]
fn legacy_route_path_candidate_via_explain_text_output_includes_deprecation_note() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-via-explain-legacy-text");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _) =
        seed_route_path_candidate_via_project(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-path-candidate-via-explain",
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query should succeed");
    assert!(output.contains("note: deprecated compatibility wrapper"));
    assert!(output.contains("route-path-candidate-explain --candidate route-path-candidate-via"));

    let _ = std::fs::remove_dir_all(&root);
}
