use datum_gui_protocol::{
    TERMINAL_COMMAND_CATALOG_VERSION, TerminalCommandCatalogEntry, terminal_command_catalog,
};

#[test]
fn production_handoff_catalog_uses_canonical_aliases_and_stable_templates() {
    let catalog = terminal_command_catalog();
    assert_eq!(
        TERMINAL_COMMAND_CATALOG_VERSION,
        "datum.terminal_command_catalog.v1"
    );
    assert_eq!(
        catalog.keys().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "datum.artifact.cancel_output_job_run",
            "datum.artifact.compare",
            "datum.artifact.export_manufacturing_set",
            "datum.artifact.files",
            "datum.artifact.generate",
            "datum.artifact.list",
            "datum.artifact.preview",
            "datum.artifact.show",
            "datum.artifact.start_output_job_run",
            "datum.artifact.validate",
            "datum.artifact.validate_manufacturing_set",
            "datum.check.accept_deviation",
            "datum.check.fill_zones",
            "datum.check.list",
            "datum.check.profiles",
            "datum.check.repair_standards",
            "datum.check.run",
            "datum.check.run_profile",
            "datum.check.show",
            "datum.check.waive",
            "datum.journal.list",
            "datum.journal.redo",
            "datum.journal.show",
            "datum.journal.undo",
            "datum.proposal.accept_apply",
            "datum.proposal.apply",
            "datum.proposal.create_manufacturing_plan",
            "datum.proposal.create_output_job",
            "datum.proposal.create_panel_projection",
            "datum.proposal.defer",
            "datum.proposal.delete_manufacturing_plan",
            "datum.proposal.delete_output_job",
            "datum.proposal.delete_panel_projection",
            "datum.proposal.list",
            "datum.proposal.preview",
            "datum.proposal.reject",
            "datum.proposal.review",
            "datum.proposal.show",
            "datum.proposal.update_manufacturing_plan",
            "datum.proposal.update_output_job",
            "datum.proposal.update_panel_projection",
            "datum.proposal.validate",
        ]
    );

    assert_catalog_entry(
        catalog
            .get("datum.artifact.cancel_output_job_run")
            .expect("artifact cancel output-job run handoff command"),
        &[
            "datum-eda",
            "artifact",
            "cancel-output-job-run",
            "{project_root}",
            "--run",
            "{run}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.compare")
            .expect("artifact compare handoff command"),
        &[
            "datum-eda",
            "artifact",
            "compare",
            "{project_root}",
            "--before",
            "{before}",
            "--after",
            "{after}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.files")
            .expect("artifact files handoff command"),
        &[
            "datum-eda",
            "artifact",
            "files",
            "{project_root}",
            "--artifact",
            "{artifact}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.export_manufacturing_set")
            .expect("artifact manufacturing-set export handoff command"),
        &[
            "datum-eda",
            "artifact",
            "export-manufacturing-set",
            "{project_root}",
            "--output-dir",
            "{output_dir}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.generate")
            .expect("artifact generation handoff command"),
        &[
            "datum-eda",
            "artifact",
            "generate",
            "{project_root}",
            "--output-job",
            "{output_job}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.list")
            .expect("artifact list handoff command"),
        &["datum-eda", "artifact", "list", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.start_output_job_run")
            .expect("artifact start output-job run handoff command"),
        &[
            "datum-eda",
            "artifact",
            "start-output-job-run",
            "{project_root}",
            "--output-job",
            "{output_job}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.validate")
            .expect("artifact validation handoff command"),
        &[
            "datum-eda",
            "artifact",
            "validate",
            "{project_root}",
            "--artifact",
            "{artifact}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.validate_manufacturing_set")
            .expect("artifact manufacturing-set validation handoff command"),
        &[
            "datum-eda",
            "artifact",
            "validate-manufacturing-set",
            "{project_root}",
            "--output-dir",
            "{output_dir}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.preview")
            .expect("artifact preview handoff command"),
        &[
            "datum-eda",
            "artifact",
            "preview",
            "{project_root}",
            "--artifact",
            "{artifact}",
            "--file",
            "{file}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.artifact.show")
            .expect("artifact show handoff command"),
        &[
            "datum-eda",
            "artifact",
            "show",
            "{project_root}",
            "--artifact",
            "{artifact}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.run")
            .expect("check run handoff command"),
        &["datum-eda", "check", "run", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.run_profile")
            .expect("profile-specific check run handoff command"),
        &[
            "datum-eda",
            "check",
            "run",
            "{project_root}",
            "--profile",
            "{profile}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.list")
            .expect("check list handoff command"),
        &["datum-eda", "check", "list", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.show")
            .expect("check show handoff command"),
        &[
            "datum-eda",
            "check",
            "show",
            "{project_root}",
            "--check-run",
            "{check_run}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.profiles")
            .expect("check profiles handoff command"),
        &["datum-eda", "check", "profiles", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.repair_standards")
            .expect("check repair-standards handoff command"),
        &["datum-eda", "check", "repair-standards", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.fill_zones")
            .expect("check fill-zones handoff command"),
        &["datum-eda", "check", "fill-zones", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.waive")
            .expect("check waiver handoff command"),
        &[
            "datum-eda",
            "check",
            "waive",
            "{project_root}",
            "--fingerprint",
            "{fingerprint}",
            "--rationale",
            "{rationale}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.check.accept_deviation")
            .expect("check accept-deviation handoff command"),
        &[
            "datum-eda",
            "check",
            "accept-deviation",
            "{project_root}",
            "--fingerprint",
            "{fingerprint}",
            "--rationale",
            "{rationale}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.proposal.preview")
            .expect("proposal preview handoff command"),
        &[
            "datum-eda",
            "proposal",
            "preview",
            "{project_root}",
            "--proposal",
            "{proposal}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.proposal.review")
            .expect("proposal review handoff command"),
        &[
            "datum-eda",
            "proposal",
            "review",
            "{project_root}",
            "--proposal",
            "{proposal}",
            "--status",
            "{status}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.proposal.accept_apply")
            .expect("proposal accept/apply handoff command"),
        &[
            "datum-eda",
            "proposal",
            "accept-apply",
            "{project_root}",
            "--proposal",
            "{proposal}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.proposal.reject")
            .expect("proposal rejection handoff command"),
        &[
            "datum-eda",
            "proposal",
            "reject",
            "{project_root}",
            "--proposal",
            "{proposal}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.journal.list")
            .expect("journal list handoff command"),
        &["datum-eda", "journal", "list", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.journal.show")
            .expect("journal show handoff command"),
        &[
            "datum-eda",
            "journal",
            "show",
            "{project_root}",
            "--transaction",
            "{transaction}",
        ],
    );
    assert_catalog_entry(
        catalog
            .get("datum.journal.undo")
            .expect("journal undo handoff command"),
        &["datum-eda", "journal", "undo", "{project_root}"],
    );
    assert_catalog_entry(
        catalog
            .get("datum.journal.redo")
            .expect("journal redo handoff command"),
        &["datum-eda", "journal", "redo", "{project_root}"],
    );
}

fn assert_catalog_entry(entry: &TerminalCommandCatalogEntry, expected_template: &[&str]) {
    assert!(
        entry.command_id.starts_with("datum."),
        "terminal command ID must use canonical datum.* namespace: {}",
        entry.command_id
    );
    assert_eq!(
        entry.mcp_alias.as_deref(),
        Some(entry.command_id.as_str()),
        "terminal command mcp_alias must match canonical command_id"
    );
    assert_eq!(entry.cli_argv_template, expected_template);
}
