use super::{HitRegion, HitTarget, RectPx};
use datum_gui_protocol::{TerminalCommandHandoff, render_terminal_command_handoff};

pub(super) fn output_job_run_command(
    job: &datum_gui_protocol::ProductionOutputJobSummary,
) -> Option<TerminalCommandHandoff> {
    if job.id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.generate",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("output_job", &job.id),
        ],
    )
}

pub(super) fn output_job_start_command(
    job: &datum_gui_protocol::ProductionOutputJobSummary,
) -> Option<TerminalCommandHandoff> {
    if job.id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.start_output_job_run",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("output_job", &job.id),
        ],
    )
}

pub(super) fn output_job_cancel_command(
    job: &datum_gui_protocol::ProductionOutputJobSummary,
) -> Option<TerminalCommandHandoff> {
    let run_id = job.latest_run_id.as_deref()?;
    if run_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.cancel_output_job_run",
        &[("project_root", "$DATUM_PROJECT_ROOT"), ("run", run_id)],
    )
}

pub(super) fn artifact_list_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.artifact.list",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn artifact_validate_command(artifact_id: &str) -> Option<TerminalCommandHandoff> {
    if artifact_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.validate",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("artifact", artifact_id),
        ],
    )
}

pub(super) fn artifact_files_command(artifact_id: &str) -> Option<TerminalCommandHandoff> {
    if artifact_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.files",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("artifact", artifact_id),
        ],
    )
}

pub(super) fn artifact_show_command(artifact_id: &str) -> Option<TerminalCommandHandoff> {
    if artifact_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.show",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("artifact", artifact_id),
        ],
    )
}

pub(super) fn artifact_preview_command(
    artifact_id: &str,
    file: &str,
) -> Option<TerminalCommandHandoff> {
    if artifact_id.is_empty() || file.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.preview",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("artifact", artifact_id),
            ("file", file),
        ],
    )
}

pub(super) fn artifact_compare_command(
    before_artifact_id: &str,
    after_artifact_id: &str,
) -> Option<TerminalCommandHandoff> {
    if before_artifact_id.is_empty()
        || after_artifact_id.is_empty()
        || before_artifact_id == after_artifact_id
    {
        return None;
    }
    render_terminal_command_handoff(
        "datum.artifact.compare",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("before", before_artifact_id),
            ("after", after_artifact_id),
        ],
    )
}

pub(super) fn check_run_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.check.run",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn check_run_profile_command(profile: &str) -> Option<TerminalCommandHandoff> {
    if profile.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.check.run_profile",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("profile", profile),
        ],
    )
}

pub(super) fn check_profiles_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.check.profiles",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn check_list_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.check.list",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn check_show_command(check_run_id: &str) -> Option<TerminalCommandHandoff> {
    if check_run_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.check.show",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("check_run", check_run_id),
        ],
    )
}

pub(super) fn check_fill_zones_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.check.fill_zones",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn check_repair_standards_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.check.repair_standards",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn check_waive_command(fingerprint: &str) -> Option<TerminalCommandHandoff> {
    check_finding_disposition_command("datum.check.waive", fingerprint)
}

pub(super) fn check_accept_deviation_command(fingerprint: &str) -> Option<TerminalCommandHandoff> {
    check_finding_disposition_command("datum.check.accept_deviation", fingerprint)
}

fn check_finding_disposition_command(
    command_id: &str,
    fingerprint: &str,
) -> Option<TerminalCommandHandoff> {
    if fingerprint.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        command_id,
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("fingerprint", fingerprint),
            ("rationale", "document rationale"),
        ],
    )
}

pub(super) fn proposal_list_command() -> Option<TerminalCommandHandoff> {
    render_terminal_command_handoff(
        "datum.proposal.list",
        &[("project_root", "$DATUM_PROJECT_ROOT")],
    )
}

pub(super) fn proposal_show_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    if proposal_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        "datum.proposal.show",
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("proposal", proposal_id),
        ],
    )
}

pub(super) fn proposal_preview_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    proposal_command("preview", proposal_id)
}

pub(super) fn proposal_reject_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    proposal_command("reject", proposal_id)
}

pub(super) fn proposal_validate_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    proposal_command("validate", proposal_id)
}

pub(super) fn proposal_defer_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    proposal_command("defer", proposal_id)
}

pub(super) fn proposal_accept_apply_command(proposal_id: &str) -> Option<TerminalCommandHandoff> {
    proposal_command("accept-apply", proposal_id)
}

fn proposal_command(verb: &str, proposal_id: &str) -> Option<TerminalCommandHandoff> {
    if proposal_id.is_empty() {
        return None;
    }
    render_terminal_command_handoff(
        match verb {
            "accept-apply" => "datum.proposal.accept_apply",
            "defer" => "datum.proposal.defer",
            "preview" => "datum.proposal.preview",
            "reject" => "datum.proposal.reject",
            "validate" => "datum.proposal.validate",
            _ => return None,
        },
        &[
            ("project_root", "$DATUM_PROJECT_ROOT"),
            ("proposal", proposal_id),
        ],
    )
}

pub(super) fn push_output_job_run_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    handoff: &TerminalCommandHandoff,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionOutputJobRun(handoff.clone()),
        rect: RectPx {
            x: rect.x + 18.0,
            y: y - 2.0,
            width: (rect.width - 36.0).max(0.0),
            height: 14.0,
        },
    });
}

pub(super) fn push_production_terminal_command_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    handoff: &TerminalCommandHandoff,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionTerminalCommand(handoff.clone()),
        rect: RectPx {
            x: rect.x + 18.0,
            y: y - 2.0,
            width: (rect.width - 36.0).max(0.0),
            height: 14.0,
        },
    });
}
