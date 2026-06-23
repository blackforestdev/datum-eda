use super::main_tests_project_route_proposal_artifact::{
    board_tracks_query_cli, seed_plus_one_gap_project, unique_project_root,
};
use super::*;
use eda_engine::board::Track;

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn export_plus_one_gap_artifact(root: &Path, artifact: &Path) -> (Uuid, Uuid, Uuid) {
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _) = seed_plus_one_gap_project(root);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-plus-one-gap",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    (target_net_uuid, anchor_a_uuid, anchor_b_uuid)
}

fn board_track_count(root: &Path) -> usize {
    let tracks_output = execute(board_tracks_query_cli(root)).expect("tracks query should succeed");
    let tracks: Vec<Track> = serde_json::from_str(&tracks_output).expect("tracks should parse");
    tracks.len()
}

fn apply_artifact(root: &Path, artifact: &Path) -> Result<String> {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
}

#[test]
fn route_proposal_artifact_apply_requires_embedded_accepted_proposal() {
    let root = unique_project_root("datum-eda-cli-route-artifact-policy-missing-proposal");
    let artifact = root.join("route-proposal.json");
    export_plus_one_gap_artifact(&root, &artifact);
    let before_tracks = board_track_count(&root);

    let mut artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).unwrap()).unwrap();
    assert_eq!(artifact_value["proposal"]["status"], "accepted");
    artifact_value
        .as_object_mut()
        .expect("artifact should be object")
        .remove("proposal");
    std::fs::write(
        &artifact,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact_value).unwrap()
        ),
    )
    .unwrap();

    let err = apply_artifact(&root, &artifact).expect_err("legacy artifact should fail");
    assert!(
        err.to_string()
            .contains("missing accepted proposal metadata")
    );
    assert_eq!(board_track_count(&root), before_tracks);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn route_proposal_artifact_apply_rejects_non_accepted_embedded_proposal() {
    let root = unique_project_root("datum-eda-cli-route-artifact-policy-draft-proposal");
    let artifact = root.join("route-proposal.json");
    export_plus_one_gap_artifact(&root, &artifact);
    let before_tracks = board_track_count(&root);

    let mut artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).unwrap()).unwrap();
    artifact_value["proposal"]["status"] = serde_json::json!("draft");
    std::fs::write(
        &artifact,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact_value).unwrap()
        ),
    )
    .unwrap();

    let err = apply_artifact(&root, &artifact).expect_err("draft proposal should fail");
    assert!(err.to_string().contains("expected accepted before apply"));
    assert_eq!(board_track_count(&root), before_tracks);

    let _ = std::fs::remove_dir_all(&root);
}
