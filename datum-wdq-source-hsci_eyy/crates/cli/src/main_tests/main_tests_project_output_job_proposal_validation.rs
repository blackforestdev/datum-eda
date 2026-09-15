use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

#[test]
fn proposal_create_output_job_rejects_missing_manufacturing_plan_before_draft() {
    let root = unique_project_root("datum-eda-cli-proposal-output-job-missing-plan");
    create_native_project(
        &root,
        Some("Output Job Missing Manufacturing Plan".to_string()),
    )
    .expect("initial scaffold should succeed");
    let missing_plan = Uuid::new_v4().to_string();

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--include",
            "drill",
            "--manufacturing-plan",
            missing_plan.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing manufacturing plan should be rejected before proposal creation");
    assert!(
        err.to_string()
            .contains("output job references missing manufacturing plan")
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
fn proposal_create_output_job_rejects_missing_variant_before_draft() {
    let root = unique_project_root("datum-eda-cli-proposal-output-job-missing-variant");
    create_native_project(&root, Some("Output Job Missing Variant".to_string()))
        .expect("initial scaffold should succeed");
    let missing_variant = Uuid::new_v4().to_string();

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--include",
            "drill",
            "--variant",
            missing_variant.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing variant should be rejected before proposal creation");
    assert!(
        err.to_string()
            .contains("output job references missing variant")
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
