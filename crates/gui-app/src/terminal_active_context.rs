use datum_gui_protocol::render_terminal_command;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub(super) struct TerminalActiveContextCommands {
    artifact_list: Option<String>,
    artifact_show: Option<String>,
    artifact_files: Option<String>,
    artifact_preview: Option<String>,
    artifact_compare: Option<String>,
    artifact_validate: Option<String>,
    output_job_generate: Option<String>,
    output_job_start_run: Option<String>,
    output_job_cancel_run: Option<String>,
    proposal_list: Option<String>,
    proposal_show: Option<String>,
    proposal_preview: Option<String>,
    proposal_validate: Option<String>,
    proposal_review_accept: Option<String>,
    proposal_review_reject: Option<String>,
    proposal_defer: Option<String>,
    proposal_reject: Option<String>,
    proposal_accept_apply: Option<String>,
    proposal_apply: Option<String>,
    journal_list: Option<String>,
    journal_show_tip: Option<String>,
    journal_undo: Option<String>,
    journal_redo: Option<String>,
    source_shards: Option<String>,
    check_run: Option<String>,
    check_list: Option<String>,
    check_profiles: Option<String>,
    check_fill_zones: Option<String>,
    check_show: Option<String>,
    check_repair_standards: Option<String>,
    check_waive_finding: Option<String>,
    check_accept_deviation: Option<String>,
    library_list_objects: Option<String>,
    library_show_object: Option<String>,
    project_validate_pool: Option<String>,
    project_create_pin_pad_map: Option<String>,
    project_set_pin_pad_map: Option<String>,
    proposal_create_pin_pad_map: Option<String>,
    proposal_set_pin_pad_map: Option<String>,
}

impl TerminalActiveContextCommands {
    pub(super) fn from_focus(
        project_root: &Path,
        artifact_id: Option<&str>,
        previous_artifact_id: Option<&str>,
        artifact_file_path: Option<&str>,
        output_job_id: Option<&str>,
        output_job_run_id: Option<&str>,
        proposal_id: Option<&str>,
        _transaction_tip: Option<&str>,
        check_run_id: Option<&str>,
        selected_finding_fingerprint: Option<&str>,
    ) -> Self {
        let project_root = project_root.display().to_string();
        let project_root_binding = project_root.as_str();
        Self {
            artifact_list: render_terminal_command(
                "datum.artifact.list",
                &[("project_root", project_root_binding)],
            ),
            artifact_show: artifact_id.and_then(|artifact| {
                render_terminal_command(
                    "datum.artifact.show",
                    &[
                        ("project_root", project_root_binding),
                        ("artifact", artifact),
                    ],
                )
            }),
            artifact_files: artifact_id.and_then(|artifact| {
                render_terminal_command(
                    "datum.artifact.files",
                    &[
                        ("project_root", project_root_binding),
                        ("artifact", artifact),
                    ],
                )
            }),
            artifact_preview: artifact_id.and_then(|artifact| {
                artifact_file_path.and_then(|file| {
                    render_terminal_command(
                        "datum.artifact.preview",
                        &[
                            ("project_root", project_root_binding),
                            ("artifact", artifact),
                            ("file", file),
                        ],
                    )
                })
            }),
            artifact_compare: artifact_compare_command(
                project_root_binding,
                previous_artifact_id,
                artifact_id,
            ),
            artifact_validate: artifact_id.and_then(|artifact| {
                render_terminal_command(
                    "datum.artifact.validate",
                    &[
                        ("project_root", project_root_binding),
                        ("artifact", artifact),
                    ],
                )
            }),
            output_job_generate: output_job_id.and_then(|output_job| {
                render_terminal_command(
                    "datum.artifact.generate",
                    &[
                        ("project_root", project_root_binding),
                        ("output_job", output_job),
                    ],
                )
            }),
            output_job_start_run: output_job_id.and_then(|output_job| {
                render_terminal_command(
                    "datum.artifact.start_output_job_run",
                    &[
                        ("project_root", project_root_binding),
                        ("output_job", output_job),
                    ],
                )
            }),
            output_job_cancel_run: output_job_run_id.and_then(|run| {
                render_terminal_command(
                    "datum.artifact.cancel_output_job_run",
                    &[("project_root", project_root_binding), ("run", run)],
                )
            }),
            proposal_list: render_terminal_command(
                "datum.proposal.list",
                &[("project_root", project_root_binding)],
            ),
            proposal_show: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.show",
            ),
            proposal_preview: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.preview",
            ),
            proposal_validate: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.validate",
            ),
            proposal_review_accept: proposal_review_command(
                project_root_binding,
                proposal_id,
                "accepted",
            ),
            proposal_review_reject: proposal_review_command(
                project_root_binding,
                proposal_id,
                "rejected",
            ),
            proposal_defer: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.defer",
            ),
            proposal_reject: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.reject",
            ),
            proposal_accept_apply: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.accept_apply",
            ),
            proposal_apply: proposal_command(
                project_root_binding,
                proposal_id,
                "datum.proposal.apply",
            ),
            journal_list: None,
            journal_show_tip: None,
            journal_undo: None,
            journal_redo: None,
            source_shards: render_terminal_command(
                "datum.query.source_shards",
                &[("project_root", project_root_binding)],
            ),
            check_run: render_terminal_command(
                "datum.check.run",
                &[("project_root", project_root_binding)],
            ),
            check_list: render_terminal_command(
                "datum.check.list",
                &[("project_root", project_root_binding)],
            ),
            check_profiles: render_terminal_command(
                "datum.check.profiles",
                &[("project_root", project_root_binding)],
            ),
            check_fill_zones: render_terminal_command(
                "datum.check.fill_zones",
                &[("project_root", project_root_binding)],
            ),
            check_show: check_run_id.and_then(|check_run| {
                render_terminal_command(
                    "datum.check.show",
                    &[
                        ("project_root", project_root_binding),
                        ("check_run", check_run),
                    ],
                )
            }),
            check_repair_standards: check_run_id.and_then(|_| {
                render_terminal_command(
                    "datum.check.repair_standards",
                    &[("project_root", project_root_binding)],
                )
            }),
            check_waive_finding: selected_finding_fingerprint.and_then(|fingerprint| {
                render_terminal_command(
                    "datum.check.waive",
                    &[
                        ("project_root", project_root_binding),
                        ("fingerprint", fingerprint),
                        ("rationale", "<rationale>"),
                    ],
                )
            }),
            check_accept_deviation: selected_finding_fingerprint.and_then(|fingerprint| {
                render_terminal_command(
                    "datum.check.accept_deviation",
                    &[
                        ("project_root", project_root_binding),
                        ("fingerprint", fingerprint),
                        ("rationale", "<rationale>"),
                    ],
                )
            }),
            library_list_objects: render_terminal_command(
                "datum.library.list_objects",
                &[("project_root", project_root_binding), ("pool", "pool")],
            ),
            library_show_object: render_terminal_command(
                "datum.library.show_object",
                &[
                    ("project_root", project_root_binding),
                    ("pool", "pool"),
                    ("kind", "<kind>"),
                    ("object", "<uuid>"),
                ],
            ),
            project_validate_pool: render_terminal_command(
                "datum.project.validate",
                &[("project_root", project_root_binding)],
            ),
            project_create_pin_pad_map: pin_pad_map_create_command(
                project_root_binding,
                "datum.project.create_pool_pin_pad_map",
                "<part-uuid>",
                "<map-uuid>",
                "<pad-uuid>:<gate-uuid>:<pin-uuid>",
                None,
            ),
            project_set_pin_pad_map: pin_pad_map_set_command(
                project_root_binding,
                "datum.project.set_pool_pin_pad_map",
                "<map-uuid>",
                "merge",
                "<pad-uuid>:<gate-uuid>:<pin-uuid>",
                None,
            ),
            proposal_create_pin_pad_map: pin_pad_map_create_command(
                project_root_binding,
                "datum.proposal.create_pool_pin_pad_map",
                "<part-uuid>",
                "<map-uuid>",
                "<pad-uuid>:<gate-uuid>:<pin-uuid>",
                Some("create PinPadMap"),
            ),
            proposal_set_pin_pad_map: pin_pad_map_set_command(
                project_root_binding,
                "datum.proposal.set_pool_pin_pad_map",
                "<map-uuid>",
                "merge",
                "<pad-uuid>:<gate-uuid>:<pin-uuid>",
                Some("update PinPadMap"),
            ),
        }
    }
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
    render_terminal_command(
        "datum.artifact.compare",
        &[
            ("project_root", project_root),
            ("before", before),
            ("after", after),
        ],
    )
}

fn proposal_command(
    project_root: &str,
    proposal_id: Option<&str>,
    command_id: &str,
) -> Option<String> {
    proposal_id.and_then(|proposal| {
        render_terminal_command(
            command_id,
            &[("project_root", project_root), ("proposal", proposal)],
        )
    })
}

fn proposal_review_command(
    project_root: &str,
    proposal_id: Option<&str>,
    status: &str,
) -> Option<String> {
    proposal_id.and_then(|proposal| {
        render_terminal_command(
            "datum.proposal.review",
            &[
                ("project_root", project_root),
                ("proposal", proposal),
                ("status", status),
            ],
        )
    })
}

fn pin_pad_map_create_command(
    project_root: &str,
    command_id: &str,
    part: &str,
    map: &str,
    entry: &str,
    rationale: Option<&str>,
) -> Option<String> {
    let mut bindings = vec![
        ("project_root", project_root),
        ("pool", "pool"),
        ("map", map),
        ("part", part),
        ("entry", entry),
    ];
    if let Some(rationale) = rationale {
        bindings.push(("rationale", rationale));
    }
    render_terminal_command(command_id, &bindings)
}

fn pin_pad_map_set_command(
    project_root: &str,
    command_id: &str,
    map: &str,
    mode: &str,
    entry: &str,
    rationale: Option<&str>,
) -> Option<String> {
    let mut bindings = vec![
        ("project_root", project_root),
        ("pool", "pool"),
        ("map", map),
        ("mode", mode),
        ("entry", entry),
    ];
    if let Some(rationale) = rationale {
        bindings.push(("rationale", rationale));
    }
    render_terminal_command(command_id, &bindings)
}
