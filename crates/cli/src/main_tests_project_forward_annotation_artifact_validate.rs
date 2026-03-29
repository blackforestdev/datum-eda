use super::main_tests_project_forward_annotation_support::unique_project_root;
use super::*;

#[test]
fn project_validate_forward_annotation_proposal_artifact_reports_match_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-validate-artifact");

    create_native_project(&root, Some("Forward Annotation Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-forward-annotation-proposal",
            root.to_str().unwrap(),
            "--out",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");

    let ok = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-forward-annotation-proposal-artifact",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should run");
    let ok_report: serde_json::Value = serde_json::from_str(&ok.0).expect("report JSON");
    assert_eq!(ok.1, 0);
    assert_eq!(
        ok_report["action"],
        "validate_forward_annotation_proposal_artifact"
    );
    assert_eq!(ok_report["matches_expected"], true);
    assert_eq!(ok_report["canonical_bytes_match"], true);

    let drifted = std::fs::read_to_string(&artifact_path).expect("artifact should read");
    std::fs::write(&artifact_path, format!(" \n{}", drifted)).expect("drifted artifact should write");

    let drift = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-forward-annotation-proposal-artifact",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should run");
    let drift_report: serde_json::Value = serde_json::from_str(&drift.0).expect("report JSON");
    assert_eq!(drift.1, 1);
    assert_eq!(drift_report["matches_expected"], false);
    assert_eq!(drift_report["canonical_bytes_match"], false);

    let _ = std::fs::remove_dir_all(&root);
}
