use super::main_tests_project_check::{
    build_native_check_fixture, seed_board_process_aperture_fixture, unique_project_root,
};
use super::*;

#[test]
fn check_list_and_show_report_persisted_check_run_evidence() {
    let root = unique_project_root("datum-eda-cli-check-list-show");
    create_native_project(&root, Some("Check History Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let run: serde_json::Value =
        serde_json::from_str(&run_output).expect("check-run JSON should parse");
    let check_run_id = run["check_run_id"].as_str().unwrap().to_string();

    let list_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check list should succeed");
    let list: serde_json::Value = serde_json::from_str(&list_output).unwrap();
    assert_eq!(list["contract"], "check_run_list_v1");
    assert_eq!(list["check_run_count"], 1);
    assert_eq!(list["check_runs"][0]["check_run_id"], check_run_id);
    assert_eq!(list["check_runs"][0]["profile_id"], "native-combined");
    assert_eq!(list["check_runs"][0]["status"], run["status"]);
    assert_eq!(list["check_runs"][0]["finding_count"], run["finding_count"]);

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "show",
            root.to_str().unwrap(),
            "--check-run",
            &check_run_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("check show should succeed");
    let show: serde_json::Value = serde_json::from_str(&show_output).unwrap();
    assert_eq!(show["contract"], "check_run_record_v1");
    assert_eq!(show["check_run"]["check_run_id"], check_run_id);
    assert_eq!(show["check_run"]["model_revision"], run["model_revision"]);
    assert_eq!(show["check_run"]["finding_count"], run["finding_count"]);
    assert_eq!(show["check_run"]["raw_report"]["domain"], "combined");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_list_reports_latest_profile_history() {
    let root = unique_project_root("datum-eda-cli-check-list-history");
    create_native_project(&root, Some("Check History Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let combined_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "native-combined",
        ])
        .expect("CLI should parse"),
    )
    .expect("combined check run should succeed");
    let combined: serde_json::Value =
        serde_json::from_str(&combined_output).expect("combined check run should parse");
    let standards_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "standards",
        ])
        .expect("CLI should parse"),
    )
    .expect("standards check run should succeed");
    let standards: serde_json::Value =
        serde_json::from_str(&standards_output).expect("standards check run should parse");

    let list_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check list should succeed");
    let list: serde_json::Value =
        serde_json::from_str(&list_output).expect("check list should parse");

    assert_eq!(list["contract"], "check_run_list_v1");
    assert_eq!(list["check_run_count"], 2);
    assert_eq!(list["latest_check_run_id"], standards["check_run_id"]);
    assert_eq!(list["latest_profile_id"], "standards");
    let latest_profiles = list["profile_latest_check_runs"].as_array().unwrap();
    assert!(latest_profiles.iter().any(|entry| {
        entry["profile_id"] == "native-combined"
            && entry["check_run_id"] == combined["check_run_id"]
            && entry["finding_count"].as_u64().is_some()
    }));
    assert!(latest_profiles.iter().any(|entry| {
        entry["profile_id"] == "standards"
            && entry["check_run_id"] == standards["check_run_id"]
            && entry["finding_count"].as_u64().is_some()
    }));
    assert!(list["check_runs"].as_array().unwrap().iter().all(|entry| {
        entry["latest_for_profile"] == true
            && entry["coverage_count"].as_u64().is_some()
            && entry["proposal_refs"].as_array().is_some()
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_show_enriches_persisted_record_with_current_repair_proposal_links() {
    let root = unique_project_root("datum-eda-cli-check-show-repair-links");
    create_native_project(&root, Some("Check Show Repair Links Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let run: serde_json::Value =
        serde_json::from_str(&run_output).expect("check-run JSON should parse");
    let check_run_id = run["check_run_id"].as_str().unwrap().to_string();
    assert_eq!(run["proposal_refs"], serde_json::json!([]));

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "repair-standards",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "show",
            root.to_str().unwrap(),
            "--check-run",
            &check_run_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("check show should succeed");
    let show: serde_json::Value = serde_json::from_str(&show_output).unwrap();
    let shown_run = &show["check_run"];
    assert_eq!(shown_run["check_run_id"], check_run_id);
    assert_eq!(shown_run["proposal_refs"].as_array().unwrap().len(), 2);
    assert_eq!(shown_run["proposal_links"].as_array().unwrap().len(), 2);
    assert!(
        shown_run["proposal_links"]
            .as_array()
            .unwrap()
            .iter()
            .all(|link| link["command_templates"]["preview"]
                .as_str()
                .unwrap()
                .contains("datum-eda proposal preview "))
    );
    assert!(
        shown_run["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| {
                if finding["code"] == "pad_mask_expansion_missing"
                    || finding["code"] == "pad_paste_reduction_missing"
                {
                    !finding["proposal_refs"].as_array().unwrap().is_empty()
                        && !finding["proposal_links"].as_array().unwrap().is_empty()
                        && finding["proposal_links"][0]["command_templates"]["preview"]
                            .as_str()
                            .unwrap()
                            .contains("datum-eda proposal preview ")
                } else {
                    true
                }
            })
    );

    let persisted_path = root.join(format!(".datum/check_runs/{check_run_id}.json"));
    let persisted: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&persisted_path).expect("persisted check run should read"),
    )
    .expect("persisted check run should parse");
    assert_eq!(persisted["proposal_refs"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}
