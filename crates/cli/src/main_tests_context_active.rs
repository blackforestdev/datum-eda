use super::*;

fn unique_context_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn context_refresh_projects_selected_check_finding_active_commands() {
    let root = unique_context_root("datum-eda-cli-context-selected-finding");
    create_native_project(&root, Some("Context Finding Demo".to_string()))
        .expect("native project should be created");
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "context-job",
            "--include",
            "gerber-set",
        ])
        .expect("create output job CLI should parse"),
    )
    .expect("output job should create");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();
    let start_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "start-output-job-run",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("start output job run CLI should parse"),
    )
    .expect("output job run should start");
    let start_report: serde_json::Value =
        serde_json::from_str(&start_output).expect("start output job run JSON");
    let output_job_run = start_report["output_job_run"]["run_id"]
        .as_str()
        .expect("output job run id should serialize")
        .to_string();
    let proposal_id = Uuid::new_v4().to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "proposal-job",
            "--include",
            "bom",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "context proposal",
        ])
        .expect("create output job proposal CLI should parse"),
    )
    .expect("proposal should create");
    let session_dir = root.join(".datum/terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    let context_path = session_dir.join("terminal-finding.json");
    std::fs::write(
        &context_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-finding",
  "context_id": "context-finding",
  "terminal_session_id": "terminal-finding",
  "datum_cli": "datum-eda",
  "previous_artifact_id": "artifact-previous",
  "focused_artifact_id": "artifact-gerber",
  "focused_artifact_file_path": "build/fab/doa2526.gbr",
  "selection_context": {
    "kind": "check_finding",
    "id": "sha256:selected-finding"
  }
}"#,
    )
    .expect("session context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("terminal-finding".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    let root_arg = root.display().to_string();
    assert_eq!(
        value["active_context_commands"]["artifact_list"],
        format!("datum-eda artifact list {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["artifact_show"],
        format!("datum-eda artifact show {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        value["active_context_commands"]["artifact_files"],
        format!("datum-eda artifact files {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        value["active_context_commands"]["artifact_preview"],
        format!(
            "datum-eda artifact preview {root_arg} --artifact artifact-gerber --file build/fab/doa2526.gbr"
        )
    );
    assert_eq!(value["previous_artifact_id"], "artifact-previous");
    assert_eq!(
        value["active_context_commands"]["artifact_compare"],
        format!(
            "datum-eda artifact compare {root_arg} --before artifact-previous --after artifact-gerber"
        )
    );
    assert_eq!(
        value["active_context_commands"]["artifact_validate"],
        format!("datum-eda artifact validate {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        value["active_context_commands"]["output_job_generate"],
        format!("datum-eda artifact generate {root_arg} --output-job {output_job}")
    );
    assert_eq!(
        value["active_context_commands"]["output_job_start_run"],
        format!("datum-eda artifact start-output-job-run {root_arg} --output-job {output_job}")
    );
    assert_eq!(
        value["active_context_commands"]["output_job_cancel_run"],
        format!("datum-eda artifact cancel-output-job-run {root_arg} --run {output_job_run}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_list"],
        format!("datum-eda proposal list {root_arg}")
    );
    assert_eq!(value["latest_proposal_id"], proposal_id);
    assert_eq!(
        value["visible_proposal_ids"],
        serde_json::json!([proposal_id.clone()])
    );
    assert_eq!(
        value["active_context_commands"]["proposal_show"],
        format!("datum-eda proposal show {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_preview"],
        format!("datum-eda proposal preview {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_validate"],
        format!("datum-eda proposal validate {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_review_accept"],
        format!("datum-eda proposal review {root_arg} --proposal {proposal_id} --status accepted")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_review_reject"],
        format!("datum-eda proposal review {root_arg} --proposal {proposal_id} --status rejected")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_defer"],
        format!("datum-eda proposal defer {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_reject"],
        format!("datum-eda proposal reject {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_accept_apply"],
        format!("datum-eda proposal accept-apply {root_arg} --proposal {proposal_id}")
    );
    assert_eq!(
        value["active_context_commands"]["proposal_apply"],
        format!("datum-eda proposal apply {root_arg} --proposal {proposal_id}")
    );
    let tip = value["accepted_transaction_tip"]
        .as_str()
        .expect("context refresh should expose accepted transaction tip");
    assert_eq!(
        value["active_context_commands"]["journal_list"],
        format!("datum-eda journal list {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["journal_show_tip"],
        format!("datum-eda journal show {root_arg} --transaction {tip}")
    );
    assert_eq!(
        value["active_context_commands"]["journal_undo"],
        format!("datum-eda journal undo {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["journal_redo"],
        format!("datum-eda journal redo {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["source_shards"],
        format!("datum-eda project query {root_arg} resolve-debug")
    );
    assert_eq!(
        value["active_context_commands"]["check_run"],
        format!("datum-eda check run {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["check_list"],
        format!("datum-eda check list {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["check_profiles"],
        format!("datum-eda check profiles {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["check_fill_zones"],
        format!("datum-eda check fill-zones {root_arg}")
    );
    assert_eq!(
        value["active_context_commands"]["check_waive_finding"],
        format!(
            "datum-eda check waive {root_arg} --fingerprint 'sha256:selected-finding' --rationale '<rationale>'"
        )
    );
    assert_eq!(
        value["active_context_commands"]["check_accept_deviation"],
        format!(
            "datum-eda check accept-deviation {root_arg} --fingerprint 'sha256:selected-finding' --rationale '<rationale>'"
        )
    );
    assert!(value["active_context_commands"]["check_show"].is_null());

    std::fs::write(
        &context_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-finding",
  "context_id": "context-finding",
  "terminal_session_id": "terminal-finding",
  "datum_cli": "datum-eda",
  "previous_artifact_id": "artifact-previous",
  "focused_artifact_id": "artifact-gerber",
  "focused_artifact_file_path": "build/fab/doa2526.gbr"
}"#,
    )
    .expect("empty session context envelope should be written");
    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("terminal-finding".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh without finding should succeed");
    let empty: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    assert_eq!(
        empty["active_context_commands"]["artifact_list"],
        format!("datum-eda artifact list {root_arg}"),
        "artifact discovery command should stay available without focused artifact"
    );
    assert!(
        empty["active_context_commands"]["check_waive_finding"].is_null(),
        "finding command should stay null without selected finding"
    );
    assert_eq!(
        empty["active_context_commands"]["artifact_preview"],
        format!(
            "datum-eda artifact preview {root_arg} --artifact artifact-gerber --file build/fab/doa2526.gbr"
        ),
        "artifact command should stay available when artifact focus remains selected"
    );
    assert_eq!(
        empty["active_context_commands"]["artifact_compare"],
        format!(
            "datum-eda artifact compare {root_arg} --before artifact-previous --after artifact-gerber"
        ),
        "artifact compare command should stay available while two artifact ids are known"
    );
    assert_eq!(
        empty["active_context_commands"]["output_job_cancel_run"],
        format!("datum-eda artifact cancel-output-job-run {root_arg} --run {output_job_run}"),
        "output-job run command should stay available when run focus remains selected"
    );
    assert_eq!(
        empty["active_context_commands"]["proposal_accept_apply"],
        format!("datum-eda proposal accept-apply {root_arg} --proposal {proposal_id}"),
        "proposal command should stay available while resolver has a proposal"
    );
    assert_eq!(
        empty["active_context_commands"]["proposal_list"],
        format!("datum-eda proposal list {root_arg}"),
        "proposal discovery command should stay available without selected proposal"
    );
    assert_eq!(
        empty["active_context_commands"]["journal_show_tip"],
        format!("datum-eda journal show {root_arg} --transaction {tip}"),
        "journal tip command should stay available while resolver has accepted transactions"
    );
    assert_eq!(
        empty["active_context_commands"]["source_shards"],
        format!("datum-eda project query {root_arg} resolve-debug"),
        "source-shard diagnostic command should stay available without selected finding or artifact"
    );
    assert_eq!(
        empty["active_context_commands"]["check_run"],
        format!("datum-eda check run {root_arg}"),
        "check run command should stay available without selected finding or latest run"
    );
    assert_eq!(
        empty["active_context_commands"]["check_list"],
        format!("datum-eda check list {root_arg}"),
        "check history command should stay available without selected finding or latest run"
    );
    assert_eq!(
        empty["active_context_commands"]["check_profiles"],
        format!("datum-eda check profiles {root_arg}"),
        "check profile discovery command should stay available without selected finding or latest run"
    );
    assert_eq!(
        empty["active_context_commands"]["check_fill_zones"],
        format!("datum-eda check fill-zones {root_arg}"),
        "zone-fill command should stay available without selected finding or latest run"
    );

    let _ = std::fs::remove_dir_all(&root);
}
