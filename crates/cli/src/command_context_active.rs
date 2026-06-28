use serde_json::{Map, Value};

pub(super) fn update_active_context_commands(object: &mut Map<String, Value>) {
    let project_root = object
        .get("project_root")
        .and_then(Value::as_str)
        .unwrap_or("$DATUM_PROJECT_ROOT")
        .to_string();
    let artifact_id = object
        .get("focused_artifact_id")
        .and_then(Value::as_str)
        .or_else(|| object.get("latest_artifact_id").and_then(Value::as_str));
    let artifact_file_path = object
        .get("focused_artifact_file_path")
        .and_then(Value::as_str);
    let previous_artifact_id = object.get("previous_artifact_id").and_then(Value::as_str);
    let output_job_id = object.get("latest_output_job_id").and_then(Value::as_str);
    let output_job_run_id = object
        .get("latest_output_job_run_id")
        .and_then(Value::as_str);
    let proposal_id = object.get("latest_proposal_id").and_then(Value::as_str);
    let transaction_tip = object
        .get("accepted_transaction_tip")
        .and_then(Value::as_str);
    let check_run_id = object.get("latest_check_run_id").and_then(Value::as_str);
    let selected_finding = object
        .get("selection_context")
        .and_then(Value::as_object)
        .filter(|selection| selection.get("kind").and_then(Value::as_str) == Some("check_finding"))
        .and_then(|selection| selection.get("id").and_then(Value::as_str));

    object.insert(
        "active_context_commands".to_string(),
        serde_json::json!({
            "artifact_list": Some(render_command(&["datum-eda", "artifact", "list", project_root.as_str()])),
            "artifact_show": artifact_command("datum.artifact.show", &project_root, artifact_id, None),
            "artifact_files": artifact_command("datum.artifact.files", &project_root, artifact_id, None),
            "artifact_preview": artifact_command("datum.artifact.preview", &project_root, artifact_id, artifact_file_path),
            "artifact_compare": artifact_compare_command(&project_root, previous_artifact_id, artifact_id),
            "artifact_validate": artifact_command("datum.artifact.validate", &project_root, artifact_id, None),
            "output_job_generate": output_job_command("datum.artifact.generate", &project_root, output_job_id),
            "output_job_start_run": output_job_command("datum.artifact.start_output_job_run", &project_root, output_job_id),
            "output_job_cancel_run": output_job_run_command("datum.artifact.cancel_output_job_run", &project_root, output_job_run_id),
            "proposal_list": Some(render_command(&["datum-eda", "proposal", "list", project_root.as_str()])),
            "proposal_show": proposal_command("show", &project_root, proposal_id),
            "proposal_preview": proposal_command("preview", &project_root, proposal_id),
            "proposal_validate": proposal_command("validate", &project_root, proposal_id),
            "proposal_review_accept": proposal_review_command(&project_root, proposal_id, "accepted"),
            "proposal_review_reject": proposal_review_command(&project_root, proposal_id, "rejected"),
            "proposal_defer": proposal_command("defer", &project_root, proposal_id),
            "proposal_reject": proposal_command("reject", &project_root, proposal_id),
            "proposal_accept_apply": proposal_command("accept-apply", &project_root, proposal_id),
            "proposal_apply": proposal_command("apply", &project_root, proposal_id),
            "journal_list": Some(render_command(&["datum-eda", "journal", "list", project_root.as_str()])),
            "journal_show_tip": transaction_tip.and_then(|transaction| {
                Some(render_command(&[
                    "datum-eda",
                    "journal",
                    "show",
                    project_root.as_str(),
                    "--transaction",
                    transaction,
                ]))
            }),
            "journal_undo": Some(render_command(&["datum-eda", "journal", "undo", project_root.as_str()])),
            "journal_redo": Some(render_command(&["datum-eda", "journal", "redo", project_root.as_str()])),
            "source_shards": Some(render_command(&["datum-eda", "project", "query", project_root.as_str(), "resolve-debug"])),
            "check_run": Some(render_command(&["datum-eda", "check", "run", project_root.as_str()])),
            "check_list": Some(render_command(&["datum-eda", "check", "list", project_root.as_str()])),
            "check_profiles": Some(render_command(&["datum-eda", "check", "profiles", project_root.as_str()])),
            "check_fill_zones": Some(render_command(&["datum-eda", "check", "fill-zones", project_root.as_str()])),
            "check_show": check_run_id.and_then(|check_run| {
                Some(render_command(&[
                    "datum-eda",
                    "check",
                    "show",
                    project_root.as_str(),
                    "--check-run",
                    check_run,
                ]))
            }),
            "check_repair_standards": check_run_id.and_then(|_| {
                Some(render_command(&[
                    "datum-eda",
                    "check",
                    "repair-standards",
                    project_root.as_str(),
                ]))
            }),
            "check_waive_finding": selected_finding.and_then(|fingerprint| {
                finding_command("datum.check.waive", &project_root, fingerprint)
            }),
            "check_accept_deviation": selected_finding.and_then(|fingerprint| {
                finding_command("datum.check.accept_deviation", &project_root, fingerprint)
            }),
        }),
    );
}

fn artifact_compare_command(
    project_root: &str,
    before_artifact_id: Option<&str>,
    after_artifact_id: Option<&str>,
) -> Option<String> {
    let before = before_artifact_id?;
    let after = after_artifact_id?;
    if before == after {
        return None;
    }
    Some(render_command(&[
        "datum-eda",
        "artifact",
        "compare",
        project_root,
        "--before",
        before,
        "--after",
        after,
    ]))
}

fn artifact_command(
    command_id: &str,
    project_root: &str,
    artifact_id: Option<&str>,
    file_path: Option<&str>,
) -> Option<String> {
    let artifact = artifact_id?;
    match command_id {
        "datum.artifact.show" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "show",
            project_root,
            "--artifact",
            artifact,
        ])),
        "datum.artifact.files" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "files",
            project_root,
            "--artifact",
            artifact,
        ])),
        "datum.artifact.preview" => {
            let file = file_path?;
            Some(render_command(&[
                "datum-eda",
                "artifact",
                "preview",
                project_root,
                "--artifact",
                artifact,
                "--file",
                file,
            ]))
        }
        "datum.artifact.validate" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "validate",
            project_root,
            "--artifact",
            artifact,
        ])),
        _ => None,
    }
}

fn output_job_command(
    command_id: &str,
    project_root: &str,
    output_job_id: Option<&str>,
) -> Option<String> {
    let output_job = output_job_id?;
    match command_id {
        "datum.artifact.generate" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "generate",
            project_root,
            "--output-job",
            output_job,
        ])),
        "datum.artifact.start_output_job_run" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "start-output-job-run",
            project_root,
            "--output-job",
            output_job,
        ])),
        _ => None,
    }
}

fn output_job_run_command(
    command_id: &str,
    project_root: &str,
    run_id: Option<&str>,
) -> Option<String> {
    let run = run_id?;
    match command_id {
        "datum.artifact.cancel_output_job_run" => Some(render_command(&[
            "datum-eda",
            "artifact",
            "cancel-output-job-run",
            project_root,
            "--run",
            run,
        ])),
        _ => None,
    }
}

fn proposal_command(action: &str, project_root: &str, proposal_id: Option<&str>) -> Option<String> {
    let proposal = proposal_id?;
    Some(render_command(&[
        "datum-eda",
        "proposal",
        action,
        project_root,
        "--proposal",
        proposal,
    ]))
}

fn proposal_review_command(
    project_root: &str,
    proposal_id: Option<&str>,
    status: &str,
) -> Option<String> {
    let proposal = proposal_id?;
    Some(render_command(&[
        "datum-eda",
        "proposal",
        "review",
        project_root,
        "--proposal",
        proposal,
        "--status",
        status,
    ]))
}

fn finding_command(command_id: &str, project_root: &str, fingerprint: &str) -> Option<String> {
    let action = match command_id {
        "datum.check.waive" => "waive",
        "datum.check.accept_deviation" => "accept-deviation",
        _ => return None,
    };
    Some(render_command(&[
        "datum-eda",
        "check",
        action,
        project_root,
        "--fingerprint",
        fingerprint,
        "--rationale",
        "<rationale>",
    ]))
}

fn render_command(argv: &[&str]) -> String {
    argv.iter()
        .map(|value| shell_arg(value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_arg(value: &str) -> String {
    if value.starts_with('$') && !value.chars().any(char::is_whitespace) {
        return format!("\"{value}\"");
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
