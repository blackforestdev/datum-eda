use super::main_tests_project_check::{build_native_check_fixture, unique_project_root};
use super::main_tests_project_check_run::{
    assert_fingerprint_ref_count, assert_fingerprint_status,
};
use super::main_tests_project_drc::seed_board_drc_fixture;
use super::*;

fn drc_unrouted_fingerprint(root: &std::path::Path) -> String {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("check-run should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("JSON should parse");
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "drc" && entry["code"] == "connectivity_unrouted_net")
        .expect("target DRC finding should exist")["fingerprint"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn project_waive_finding_commits_journaled_drc_fingerprint_waiver() {
    let root = unique_project_root("datum-eda-cli-project-waive-drc-finding");
    create_native_project(&root, Some("Journaled DRC Waiver Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);
    seed_board_drc_fixture(&root);
    let fingerprint = drc_unrouted_fingerprint(&root);

    let waive_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "waive-finding",
            root.to_str().unwrap(),
            "--fingerprint",
            &fingerprint,
            "--rationale",
            "Intentional unrouted board net during layout",
            "--created-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("DRC waive-finding should succeed");
    let waive_report: serde_json::Value =
        serde_json::from_str(&waive_output).expect("waive report JSON should parse");
    assert_eq!(waive_report["contract"], "project_waive_finding_v1");
    assert_eq!(waive_report["status"], "applied");
    assert_eq!(waive_report["domain"], "drc");
    assert_eq!(waive_report["fingerprint"], fingerprint);

    assert_fingerprint_status(&root, &fingerprint, "waived");
    assert_fingerprint_ref_count(&root, &fingerprint, "waiver_refs", 1);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "active");
    assert_fingerprint_ref_count(&root, &fingerprint, "waiver_refs", 0);

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "waived");
    assert_fingerprint_ref_count(&root, &fingerprint, "waiver_refs", 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_deviation_commits_journaled_drc_fingerprint_deviation() {
    let root = unique_project_root("datum-eda-cli-project-accept-drc-deviation");
    create_native_project(&root, Some("Journaled DRC Deviation Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);
    seed_board_drc_fixture(&root);
    let fingerprint = drc_unrouted_fingerprint(&root);

    let deviation_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "accept-deviation",
            root.to_str().unwrap(),
            "--fingerprint",
            &fingerprint,
            "--rationale",
            "Accepted DRC deviation during routed-layout staging",
            "--accepted-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("DRC accept-deviation should succeed");
    let deviation_report: serde_json::Value =
        serde_json::from_str(&deviation_output).expect("deviation report JSON should parse");
    assert_eq!(deviation_report["contract"], "project_accept_deviation_v1");
    assert_eq!(deviation_report["status"], "applied");
    assert_eq!(deviation_report["domain"], "drc");
    assert_eq!(deviation_report["fingerprint"], fingerprint);

    assert_fingerprint_status(&root, &fingerprint, "accepted_deviation");
    assert_fingerprint_ref_count(&root, &fingerprint, "deviation_refs", 1);

    let _ = std::fs::remove_dir_all(&root);
}
