use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_manufacturing_plan_as_proposal_defers_create_and_delete_until_apply() {
    let root = unique_project_root("datum-eda-cli-manufacturing-plan-proposal");
    create_native_project(&root, Some("Manufacturing Plan Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let create_proposal = Uuid::new_v4().to_string();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--as-proposal",
            "--proposal",
            create_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan create proposal should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "propose_create_manufacturing_plan");

    let empty_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query should succeed");
    let empty: serde_json::Value = serde_json::from_str(&empty_output).unwrap();
    assert_eq!(empty["manufacturing_plan_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            create_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan proposal apply should succeed");
    let created_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query after apply should succeed");
    let created: serde_json::Value = serde_json::from_str(&created_output).unwrap();
    assert_eq!(created["manufacturing_plan_count"], 1);
    let manufacturing_plan = created["manufacturing_plans"][0]["id"].as_str().unwrap();
    let delete_proposal = Uuid::new_v4().to_string();

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-manufacturing-plan",
            root.to_str().unwrap(),
            "--manufacturing-plan",
            manufacturing_plan,
            "--as-proposal",
            "--proposal",
            delete_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan delete proposal should succeed");
    let delete_report: serde_json::Value = serde_json::from_str(&delete_output).unwrap();
    assert_eq!(delete_report["contract"], "proposal_create_v1");
    assert_eq!(delete_report["action"], "propose_delete_manufacturing_plan");

    let still_present_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query before delete apply should succeed");
    let still_present: serde_json::Value = serde_json::from_str(&still_present_output).unwrap();
    assert_eq!(still_present["manufacturing_plan_count"], 1);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            delete_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan delete proposal apply should succeed");
    let deleted_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query after delete apply should succeed");
    let deleted: serde_json::Value = serde_json::from_str(&deleted_output).unwrap();
    assert_eq!(deleted["manufacturing_plan_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_update_manufacturing_plan_as_proposal_defers_mutation_until_apply() {
    let root = unique_project_root("datum-eda-cli-manufacturing-plan-update-proposal");
    create_native_project(
        &root,
        Some("Manufacturing Plan Update Proposal Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let manufacturing_plan = create_report["manufacturing_plan"]["id"].as_str().unwrap();
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "update-manufacturing-plan",
            root.to_str().unwrap(),
            "--manufacturing-plan",
            manufacturing_plan,
            "--name",
            "Release A reviewed",
            "--prefix",
            "release-a-reviewed",
            "--as-proposal",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review manufacturing plan naming before apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan update proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(
        proposal_report["action"],
        "propose_update_manufacturing_plan"
    );
    assert_eq!(proposal_report["proposal_id"], proposal_id);
    assert_eq!(proposal_report["proposal"]["status"], "draft");

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["manufacturing_plans"][0]["name"], "Release A");
    assert_eq!(unchanged["manufacturing_plans"][0]["prefix"], "release-a");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan update proposal apply should succeed");

    let applied_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(
        applied["manufacturing_plans"][0]["name"],
        "Release A reviewed"
    );
    assert_eq!(
        applied["manufacturing_plans"][0]["prefix"],
        "release-a-reviewed"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_manufacturing_plan_command_defers_creation_until_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-create-manufacturing-plan-command");
    create_native_project(
        &root,
        Some("Canonical Manufacturing Plan Proposal Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A fabrication",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review canonical manufacturing plan creation",
        ])
        .expect("CLI should parse"),
    )
    .expect("canonical manufacturing plan proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(
        proposal_report["action"],
        "propose_create_manufacturing_plan"
    );
    assert_eq!(proposal_report["proposal_id"], proposal_id);

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["manufacturing_plan_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");
    let applied_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "manufacturing-plans",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["manufacturing_plan_count"], 1);
    assert_eq!(
        applied["manufacturing_plans"][0]["name"],
        "Release A fabrication"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_panel_projection_as_proposal_defers_create_and_delete_until_apply() {
    let root = unique_project_root("datum-eda-cli-panel-projection-proposal");
    create_native_project(&root, Some("Panel Projection Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let create_proposal = Uuid::new_v4().to_string();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "release-a-panel",
            "--as-proposal",
            "--proposal",
            create_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection create proposal should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "propose_create_panel_projection");

    let empty_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query should succeed");
    let empty: serde_json::Value = serde_json::from_str(&empty_output).unwrap();
    assert_eq!(empty["panel_projection_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            create_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection proposal apply should succeed");
    let created_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query after apply should succeed");
    let created: serde_json::Value = serde_json::from_str(&created_output).unwrap();
    assert_eq!(created["panel_projection_count"], 1);
    let panel_projection = created["panel_projections"][0]["id"].as_str().unwrap();
    let delete_proposal = Uuid::new_v4().to_string();

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-panel-projection",
            root.to_str().unwrap(),
            "--panel-projection",
            panel_projection,
            "--as-proposal",
            "--proposal",
            delete_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection delete proposal should succeed");
    let delete_report: serde_json::Value = serde_json::from_str(&delete_output).unwrap();
    assert_eq!(delete_report["contract"], "proposal_create_v1");
    assert_eq!(delete_report["action"], "propose_delete_panel_projection");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            delete_proposal.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection delete proposal apply should succeed");
    let deleted_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query after delete apply should succeed");
    let deleted: serde_json::Value = serde_json::from_str(&deleted_output).unwrap();
    assert_eq!(deleted["panel_projection_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_update_panel_projection_as_proposal_defers_mutation_until_apply() {
    let root = unique_project_root("datum-eda-cli-panel-projection-update-proposal");
    create_native_project(
        &root,
        Some("Panel Projection Update Proposal Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "release-a-panel",
            "--name",
            "Release A panel",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let panel_projection = create_report["panel_projection"]["id"].as_str().unwrap();
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "update-panel-projection",
            root.to_str().unwrap(),
            "--panel-projection",
            panel_projection,
            "--name",
            "Release A panel reviewed",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--rotation-deg",
            "90",
            "--as-proposal",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review panel transform before apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection update proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_update_panel_projection");
    assert_eq!(proposal_report["proposal_id"], proposal_id);
    assert_eq!(proposal_report["proposal"]["status"], "draft");

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["panel_projections"][0]["name"], "Release A panel");
    assert_eq!(
        unchanged["panel_projections"][0]["board_instances"][0]["x_nm"],
        0
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection update proposal apply should succeed");

    let applied_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(
        applied["panel_projections"][0]["name"],
        "Release A panel reviewed"
    );
    assert_eq!(
        applied["panel_projections"][0]["board_instances"][0]["x_nm"],
        1000
    );
    assert_eq!(
        applied["panel_projections"][0]["board_instances"][0]["rotation_deg"],
        90
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_panel_projection_command_defers_creation_until_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-create-panel-projection-command");
    create_native_project(
        &root,
        Some("Canonical Panel Projection Proposal Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "release-a-panel",
            "--name",
            "Release A panel",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review canonical panel projection creation",
        ])
        .expect("CLI should parse"),
    )
    .expect("canonical panel projection proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_create_panel_projection");
    assert_eq!(proposal_report["proposal_id"], proposal_id);

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["panel_projection_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");
    let applied_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "panel-projections",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["panel_projection_count"], 1);
    assert_eq!(applied["panel_projections"][0]["name"], "Release A panel");

    let _ = std::fs::remove_dir_all(&root);
}
