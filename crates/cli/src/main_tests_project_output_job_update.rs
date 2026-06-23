use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

#[test]
fn project_update_output_job_round_trips_through_journal_and_undo() {
    let root = unique_project_root("datum-eda-cli-project-update-output-job");
    create_native_project(&root, Some("Output Job Update Demo".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();
    let variant = Uuid::new_v4().to_string();

    let update_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "update-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
            "--name",
            "Release A CAM package",
            "--output-dir",
            root.join("fab-rev-b").to_str().unwrap(),
            "--variant",
            variant.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job update should succeed");
    let update_report: serde_json::Value =
        serde_json::from_str(&update_output).expect("update output job JSON");
    assert_eq!(update_report["contract"], "output_job_mutation_v1");
    assert_eq!(update_report["action"], "update_output_job");
    assert_eq!(update_report["created"], false);
    assert_eq!(
        update_report["output_job"]["id"],
        serde_json::json!(output_job)
    );
    assert_eq!(update_report["output_job"]["name"], "Release A CAM package");
    assert_eq!(update_report["output_job"]["prefix"], "release-a");
    assert_eq!(
        update_report["output_job"]["include"],
        serde_json::json!(["gerber_set"])
    );
    assert_eq!(
        update_report["output_job"]["output_dir"],
        root.join("fab-rev-b").display().to_string()
    );
    assert_eq!(update_report["output_job"]["variant"], variant);
    assert_eq!(update_report["output_job"]["object_revision"], 1);

    let output_job_path = root.join(format!(".datum/output_jobs/{output_job}.json"));
    std::fs::remove_file(&output_job_path).expect("promoted output job shard should remove");
    let query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should replay journal");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("output jobs query JSON");
    assert_eq!(query_report["output_job_count"], 1);
    assert_eq!(
        query_report["output_jobs"][0]["name"],
        "Release A CAM package"
    );
    assert_eq!(query_report["output_jobs"][0]["prefix"], "release-a");
    assert_eq!(
        query_report["output_jobs"][0]["include"],
        serde_json::json!(["gerber_set"])
    );
    assert_eq!(
        query_report["output_jobs"][0]["output_dir"],
        root.join("fab-rev-b").display().to_string()
    );
    assert_eq!(query_report["output_jobs"][0]["variant"], variant);
    assert_eq!(query_report["output_jobs"][0]["object_revision"], 1);

    let undo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job update undo should succeed");
    let undo_report: serde_json::Value =
        serde_json::from_str(&undo_output).expect("undo output job JSON");
    assert_eq!(undo_report["action"], "undo");

    let undone_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after undo should succeed");
    let undone_report: serde_json::Value =
        serde_json::from_str(&undone_output).expect("output jobs after undo JSON");
    assert_eq!(undone_report["output_jobs"][0]["name"], "Release A");
    assert_eq!(undone_report["output_jobs"][0]["prefix"], "release-a");
    assert_eq!(
        undone_report["output_jobs"][0]["include"],
        serde_json::json!(["gerber_set"])
    );
    assert_eq!(
        undone_report["output_jobs"][0]["output_dir"],
        serde_json::Value::Null
    );
    assert_eq!(
        undone_report["output_jobs"][0]["variant"],
        serde_json::Value::Null
    );
    assert_eq!(undone_report["output_jobs"][0]["object_revision"], 0);

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job delete should succeed");
    let delete_report: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete output job JSON");
    assert_eq!(delete_report["contract"], "output_job_mutation_v1");
    assert_eq!(delete_report["action"], "delete_output_job");
    assert_eq!(delete_report["created"], false);
    assert_eq!(
        delete_report["output_job"]["id"],
        serde_json::json!(output_job)
    );
    assert!(
        !output_job_path.exists(),
        "promoted output job shard should be deleted"
    );

    let deleted_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after delete should succeed");
    let deleted_report: serde_json::Value =
        serde_json::from_str(&deleted_output).expect("output jobs after delete JSON");
    assert_eq!(deleted_report["output_job_count"], 0);

    let delete_undo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job delete undo should succeed");
    let delete_undo_report: serde_json::Value =
        serde_json::from_str(&delete_undo_output).expect("delete undo JSON");
    assert_eq!(delete_undo_report["action"], "undo");
    let restored_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after delete undo should succeed");
    let restored_report: serde_json::Value =
        serde_json::from_str(&restored_output).expect("output jobs restored JSON");
    assert_eq!(restored_report["output_job_count"], 1);
    assert_eq!(restored_report["output_jobs"][0]["id"], output_job);

    let delete_redo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "redo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job delete redo should succeed");
    let delete_redo_report: serde_json::Value =
        serde_json::from_str(&delete_redo_output).expect("delete redo JSON");
    assert_eq!(delete_redo_report["action"], "redo");
    let redone_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after delete redo should succeed");
    let redone_report: serde_json::Value =
        serde_json::from_str(&redone_output).expect("output jobs redone JSON");
    assert_eq!(redone_report["output_job_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_update_output_job_as_proposal_defers_mutation_until_apply() {
    let root = unique_project_root("datum-eda-cli-project-update-output-job-proposal");
    create_native_project(&root, Some("Output Job Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let output_job = create_report["output_job"]["id"].as_str().unwrap();
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "update-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
            "--name",
            "Release A reviewed",
            "--as-proposal",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review output job naming before apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_update_output_job");
    assert_eq!(proposal_report["proposal_id"], proposal_id);
    assert_eq!(proposal_report["proposal"]["status"], "draft");

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["output_jobs"][0]["name"], "Release A");

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
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["output_jobs"][0]["name"], "Release A reviewed");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_output_job_as_proposal_defers_creation_until_apply() {
    let root = unique_project_root("datum-eda-cli-project-create-output-job-proposal");
    create_native_project(&root, Some("Output Job Create Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--include",
            "drill",
            "--name",
            "Release A drill",
            "--as-proposal",
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_create_output_job");
    assert_eq!(proposal_report["proposal_id"], proposal_id);

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["output_job_count"], 0);

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
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["output_job_count"], 1);
    assert_eq!(applied["output_jobs"][0]["name"], "Release A drill");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_output_job_command_defers_creation_until_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-create-output-job-command");
    create_native_project(
        &root,
        Some("Canonical Output Job Proposal Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
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
            "--name",
            "Release A drill",
            "--proposal",
            proposal_id.as_str(),
            "--rationale",
            "review canonical output job creation",
        ])
        .expect("CLI should parse"),
    )
    .expect("canonical output job create proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_create_output_job");
    assert_eq!(proposal_report["proposal_id"], proposal_id);

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["output_job_count"], 0);

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
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["output_job_count"], 1);
    assert_eq!(applied["output_jobs"][0]["name"], "Release A drill");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_delete_output_job_as_proposal_defers_deletion_until_apply() {
    let root = unique_project_root("datum-eda-cli-project-delete-output-job-proposal");
    create_native_project(&root, Some("Output Job Delete Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let output_job = create_report["output_job"]["id"].as_str().unwrap();
    let proposal_id = Uuid::new_v4().to_string();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
            "--as-proposal",
            "--proposal",
            proposal_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job delete proposal should succeed");
    let proposal_report: serde_json::Value = serde_json::from_str(&proposal_output).unwrap();
    assert_eq!(proposal_report["contract"], "proposal_create_v1");
    assert_eq!(proposal_report["action"], "propose_delete_output_job");
    assert_eq!(proposal_report["proposal_id"], proposal_id);

    let unchanged_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should succeed");
    let unchanged: serde_json::Value = serde_json::from_str(&unchanged_output).unwrap();
    assert_eq!(unchanged["output_job_count"], 1);

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
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query after apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&applied_output).unwrap();
    assert_eq!(applied["output_job_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
