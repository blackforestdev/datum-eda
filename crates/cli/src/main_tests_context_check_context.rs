use super::main_tests_project_check::{seed_board_process_aperture_fixture, unique_project_root};
use super::*;

#[test]
fn context_refresh_exposes_standards_check_context_for_agents() {
    let root = unique_project_root("datum-eda-cli-context-check-context");
    create_native_project(&root, Some("Context Check Demo".to_string()))
        .expect("native project should be created");
    seed_board_process_aperture_fixture(&root);
    let check_output = execute(
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
    .expect("standards check run should persist");
    let check_run: serde_json::Value = serde_json::from_str(&check_output).unwrap();
    let session_dir = root.join(".datum/terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    std::fs::write(
        session_dir.join("terminal-checks.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-checks",
  "context_id": "context-checks",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("terminal-checks".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let context: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        context["visible_check_run_ids"],
        serde_json::json!([check_run["check_run_id"].as_str().unwrap()])
    );
    let check_context = &context["check_context"];
    assert_eq!(check_context["contract"], "datum_check_context_v1");
    assert_eq!(check_context["visible_check_run_count"], 1);
    assert!(
        check_context["agent_commands"]["waive_finding"]
            .as_str()
            .unwrap()
            .contains("check waive")
    );
    assert!(
        check_context["agent_commands"]["accept_deviation"]
            .as_str()
            .unwrap()
            .contains("check accept-deviation")
    );
    let visible_run = &check_context["visible_check_runs"][0];
    assert_eq!(visible_run["profile_id"], "standards");
    assert_eq!(visible_run["check_run_id"], check_run["check_run_id"]);
    let finding = visible_run["active_findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["code"] == "pad_mask_expansion_missing")
        .expect("standards finding should be exposed to agent context");
    assert_eq!(finding["domain"], "standards");
    assert_eq!(
        finding["standards_basis"],
        "datum.process_aperture_and_geometry.current"
    );
    assert_eq!(finding["rule_revision"], "v1");
    assert!(finding["proposal_refs"].as_array().unwrap().is_empty());
    assert!(finding["waiver_refs"].as_array().unwrap().is_empty());
    assert!(finding["deviation_refs"].as_array().unwrap().is_empty());
    assert_eq!(
        context["check_status"]["check_run_id"],
        check_run["check_run_id"]
    );
    assert_eq!(context["check_status"]["profile_id"], "standards");
    assert_eq!(
        context["check_status"]["findings"][0]["fingerprint"],
        finding["fingerprint"]
    );
    assert!(
        context["visible_finding_fingerprints"]
            .as_array()
            .unwrap()
            .contains(&finding["fingerprint"])
    );

    let _ = std::fs::remove_dir_all(&root);
}
