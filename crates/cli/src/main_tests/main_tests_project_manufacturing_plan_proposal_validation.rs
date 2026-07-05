use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn proposal_create_manufacturing_plan_rejects_missing_variant_before_draft() {
    let root = unique_project_root("datum-eda-cli-proposal-manufacturing-plan-missing-variant");
    create_native_project(
        &root,
        Some("Manufacturing Plan Missing Variant".to_string()),
    )
    .expect("initial scaffold should succeed");
    let missing_variant = Uuid::new_v4().to_string();

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--variant",
            missing_variant.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing variant should be rejected before proposal creation");
    assert!(
        err.to_string()
            .contains("manufacturing plan references missing variant")
    );

    let proposals = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal list should succeed");
    let proposals: serde_json::Value = serde_json::from_str(&proposals).unwrap();
    assert_eq!(proposals["proposal_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_panel_projection_rejects_missing_board_before_draft() {
    let root = unique_project_root("datum-eda-cli-proposal-panel-missing-board");
    create_native_project(&root, Some("Panel Missing Board".to_string()))
        .expect("initial scaffold should succeed");
    let missing_board = Uuid::new_v4().to_string();

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "invalid-panel",
            "--board",
            missing_board.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing board should be rejected before proposal creation");
    assert!(
        err.to_string()
            .contains("panel projection board instance references missing board")
    );

    let proposals = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal list should succeed");
    let proposals: serde_json::Value = serde_json::from_str(&proposals).unwrap();
    assert_eq!(proposals["proposal_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
