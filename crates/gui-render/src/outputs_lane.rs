use datum_gui_protocol::{ReviewWorkspaceState, TerminalCommandHandoff};

use super::outputs_preview::render_artifact_preview_viewport;
use super::outputs_proposals::render_action_sections;
use super::outputs_run_commands::{
    artifact_files_command, artifact_list_command, artifact_preview_command, artifact_show_command,
    artifact_validate_command, check_accept_deviation_command, check_fill_zones_command,
    check_list_command, check_profiles_command, check_repair_standards_command, check_run_command,
    check_run_profile_command, check_show_command, check_waive_command, output_job_cancel_command,
    output_job_run_command, output_job_start_command, proposal_accept_apply_command,
    proposal_preview_command, proposal_show_command, push_output_job_run_hit_region,
    push_production_terminal_command_hit_region,
};
use super::{HitRegion, HitTarget};
use super::{Quad, RectPx, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun};
use super::{draw_text, suffix_id, truncate_text};

const MAX_OUTPUT_JOBS: usize = 6;
const MAX_MANUFACTURING_PLANS: usize = 4;
const MAX_PANEL_PROJECTIONS: usize = 4;
const MAX_ARTIFACTS_PER_JOB: usize = 2;
const MAX_FILES_PER_ARTIFACT: usize = 3;
const MAX_PROJECTIONS_PER_ARTIFACT: usize = 2;
const MAX_CHECK_FINDINGS: usize = 3;

pub(super) fn render_outputs_lane(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        "OUTPUT JOBS",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &format!(
            "{} JOBS / {} ARTIFACTS / {} RUNS / {}",
            state.production.output_job_count,
            state.production.artifact_count,
            state.production.artifact_run_count,
            state
                .production
                .latest_status
                .as_deref()
                .unwrap_or("never_run")
                .to_uppercase()
        ),
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let mut y = rect.y + 50.0;
    let bottom = rect.y + rect.height - 18.0;
    if let Some(command) = artifact_list_command() {
        if y + 14.0 > bottom {
            return;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("ARTIFACTS {}", truncate_text(&command.command, 52)),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 18.0;
    }
    if let Some(artifact) = &state.production.focused_artifact {
        push_artifact_hit_region(hit_regions, rect, y, &artifact.artifact_id);
        draw_text(
            &format!(
                "FOCUS ART {} {} / {}",
                artifact.kind.to_uppercase(),
                suffix_id(&artifact.artifact_id).to_uppercase(),
                artifact.validation_state.to_uppercase()
            ),
            rect.x + 12.0,
            y,
            10.5,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
        if let Some(command) = artifact_show_command(&artifact.artifact_id) {
            if y + 14.0 > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("SHOW {}", truncate_text(&command.command, 55)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        if let Some(command) = artifact_validate_command(&artifact.artifact_id) {
            if y + 14.0 > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("VALIDATE {}", truncate_text(&command.command, 52)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        if let Some(command) = artifact_files_command(&artifact.artifact_id) {
            if y + 14.0 > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("FILES {}", truncate_text(&command.command, 55)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        for projection in artifact
            .production_projections
            .iter()
            .take(MAX_PROJECTIONS_PER_ARTIFACT)
        {
            if y + 14.0 > bottom {
                return;
            }
            draw_text(
                &format!(
                    "FOCUS PROOF {} {}B {}",
                    truncate_text(&projection.projection_kind.to_uppercase(), 20),
                    projection.byte_count,
                    suffix_id(&projection.sha256).to_uppercase()
                ),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        for file in artifact.files.iter().take(MAX_FILES_PER_ARTIFACT) {
            if y + 14.0 > bottom {
                return;
            }
            push_artifact_file_hit_region(hit_regions, rect, y, &file.path);
            draw_text(
                &format!(
                    "FOCUS FILE {} {}",
                    truncate_text(&file.path, 42),
                    suffix_id(&file.sha256).to_uppercase()
                ),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        if let Some(file) = &artifact.focused_file {
            if y + 56.0 > bottom {
                return;
            }
            let viewer = generated_file_viewer(&file.path);
            draw_text(
                &format!("{} VIEW", viewer.label),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
            if let Some(command) = artifact_preview_command(&artifact.artifact_id, &file.path) {
                if y + 14.0 > bottom {
                    return;
                }
                push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
                draw_text(
                    &format!("PREVIEW {}", truncate_text(&command.command, 52)),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_SECONDARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
            }
            draw_text(
                &format!(
                    "{} / {} / {}",
                    viewer.family,
                    truncate_text(&file.path, 48),
                    suffix_id(&file.sha256).to_uppercase()
                ),
                rect.x + 36.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
            if let Some(preview) = &artifact.focused_preview {
                if y + 14.0 > bottom {
                    return;
                }
                draw_text(
                    &format!(
                        "PREVIEW {} HASH {}{}{}{}{}",
                        truncate_text(&preview.preview_kind.to_uppercase(), 18),
                        if preview.hash_matches_metadata {
                            "OK"
                        } else {
                            "DRIFT"
                        },
                        if preview.primitive_count > 0 {
                            format!(" PRIM {}", preview.primitive_count)
                        } else {
                            String::new()
                        },
                        preview
                            .geometry_count
                            .map(|count| format!(" GEO {count}"))
                            .unwrap_or_default(),
                        preview
                            .hit_count
                            .map(|count| format!(" HIT {count}"))
                            .unwrap_or_default(),
                        preview
                            .row_count
                            .map(|count| format!(" ROW {count}"))
                            .unwrap_or_default()
                    ),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
                if !preview.csv_rows.is_empty() {
                    if y + 14.0 > bottom {
                        return;
                    }
                    y = render_csv_preview_table(preview, rect, y, bottom, text_runs);
                }
                if !preview.primitives.is_empty() && rect.width >= 520.0 {
                    let preview_rect = RectPx {
                        x: rect.x + rect.width - 384.0,
                        y: rect.y + 50.0,
                        width: 360.0,
                        height: 78.0,
                    };
                    render_artifact_preview_viewport(
                        preview,
                        &state.ui.artifact_preview,
                        preview_rect,
                        panel_quads,
                    );
                    draw_text(
                        &format!(
                            "CAM VIEWPORT {}% PRIM {}",
                            state.ui.artifact_preview.zoom_ppm / 10_000,
                            preview.primitives.len()
                        ),
                        preview_rect.x + 8.0,
                        preview_rect.y + 8.0,
                        9.5,
                        TEXT_SECONDARY,
                        TextFace::Mono,
                        text_runs,
                    );
                    push_artifact_preview_controls(hit_regions, preview_rect);
                    draw_artifact_preview_controls(
                        &state.ui.artifact_preview,
                        preview_rect,
                        text_runs,
                    );
                }
            }
            if let Some(projection) =
                matching_file_projection(artifact.production_projections.as_slice(), viewer)
            {
                if y + 14.0 > bottom {
                    return;
                }
                draw_text(
                    &format!(
                        "VIEW PROOF {} {}B {}",
                        truncate_text(&projection.projection_kind.to_uppercase(), 22),
                        projection.byte_count,
                        suffix_id(&projection.sha256).to_uppercase()
                    ),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
            }
        }
        y += 6.0;
    }
    let Some(next_y) = render_checks_section(state, rect, y, bottom, text_runs, hit_regions) else {
        return;
    };
    y = next_y;
    let Some(next_y) = render_action_sections(state, rect, y, bottom, text_runs, hit_regions)
    else {
        return;
    };
    y = next_y;
    if !state.production.panel_projections.is_empty() {
        draw_text(
            &format!("PANELS {}", state.production.panel_projection_count),
            rect.x + 12.0,
            y,
            10.5,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
        for panel in state
            .production
            .panel_projections
            .iter()
            .take(MAX_PANEL_PROJECTIONS)
        {
            if y + 14.0 > bottom {
                return;
            }
            draw_text(
                &format!(
                    "PANEL {} / INST {} / REV {}",
                    truncate_text(&panel.name.to_uppercase(), 28),
                    panel.board_instance_count,
                    panel.object_revision
                ),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
            if let Some(board) = &panel.first_board {
                if y + 14.0 > bottom {
                    return;
                }
                draw_text(
                    &format!(
                        "BOARD {} X {} Y {} ROT {}",
                        suffix_id(board).to_uppercase(),
                        panel.first_x_nm.unwrap_or_default(),
                        panel.first_y_nm.unwrap_or_default(),
                        panel.first_rotation_deg.unwrap_or_default()
                    ),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
            }
        }
        y += 6.0;
    }
    if !state.production.manufacturing_plans.is_empty() {
        draw_text(
            &format!("PLANS {}", state.production.manufacturing_plan_count),
            rect.x + 12.0,
            y,
            10.5,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
        for plan in state
            .production
            .manufacturing_plans
            .iter()
            .take(MAX_MANUFACTURING_PLANS)
        {
            if y + 14.0 > bottom {
                return;
            }
            draw_text(
                &format!(
                    "PLAN {} / {} / REV {}",
                    truncate_text(&plan.name.to_uppercase(), 24),
                    truncate_text(&plan.prefix, 20),
                    plan.object_revision
                ),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        y += 6.0;
    }
    for job in state.production.output_jobs.iter().take(MAX_OUTPUT_JOBS) {
        if y + 32.0 > bottom {
            break;
        }
        let run_command = output_job_run_command(job);
        draw_text(
            &truncate_text(&job.name.to_uppercase(), 32),
            rect.x + 12.0,
            y,
            12.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            text_runs,
        );
        draw_text(
            &format!(
                "{} / {} / RUNS {} / ART {}{}{}",
                job.family,
                job.status.to_uppercase(),
                job.execution_count,
                job.artifact_count,
                job.latest_run_id
                    .as_ref()
                    .map(|id| format!(" / RUN {}", suffix_id(id).to_uppercase()))
                    .unwrap_or_default(),
                job.latest_run_artifact_id
                    .as_ref()
                    .map(|id| format!(" / LATEST ART {}", suffix_id(id).to_uppercase()))
                    .unwrap_or_default()
            ),
            rect.x + 12.0,
            y + 16.0,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += 32.0;
        if let Some(command) = &run_command {
            if y + 14.0 > bottom {
                return;
            }
            push_output_job_run_hit_region(hit_regions, rect, y, command);
            draw_text(
                &format!("RUN {}", truncate_text(&command.command, 56)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        let lifecycle_command = if job.status == "running" {
            output_job_cancel_command(job).map(|command| ("CANCEL", command))
        } else {
            output_job_start_command(job).map(|command| ("START", command))
        };
        if let Some((label, command)) = lifecycle_command {
            if y + 14.0 > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("{label} {}", truncate_text(&command.command, 54)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        for artifact in job.artifacts.iter().take(MAX_ARTIFACTS_PER_JOB) {
            if y + 14.0 > bottom {
                return;
            }
            push_artifact_hit_region(hit_regions, rect, y, &artifact.artifact_id);
            draw_text(
                &format!(
                    "ART {} {} / FILES {}",
                    artifact.kind.to_uppercase(),
                    suffix_id(&artifact.artifact_id).to_uppercase(),
                    artifact.file_count
                ),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
            for projection in artifact
                .production_projections
                .iter()
                .take(MAX_PROJECTIONS_PER_ARTIFACT)
            {
                if y + 14.0 > bottom {
                    return;
                }
                draw_text(
                    &format!(
                        "PROOF {} {}B {}",
                        truncate_text(&projection.projection_kind.to_uppercase(), 24),
                        projection.byte_count,
                        suffix_id(&projection.sha256).to_uppercase()
                    ),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
            }
            for file in artifact.files.iter().take(MAX_FILES_PER_ARTIFACT) {
                if y + 14.0 > bottom {
                    return;
                }
                push_artifact_file_hit_region(hit_regions, rect, y, &file.path);
                draw_text(
                    &format!(
                        "FILE {} {}",
                        truncate_text(&file.path, 44),
                        suffix_id(&file.sha256).to_uppercase()
                    ),
                    rect.x + 36.0,
                    y,
                    10.0,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
                y += 14.0;
            }
        }
        y += 8.0;
    }
}

fn render_checks_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    if state.checks.check_run_id.is_none() && state.checks.findings.is_empty() {
        return Some(y);
    }
    if y + 42.0 > bottom {
        return None;
    }
    draw_text(
        &format!("CHECKS {}", state.checks.finding_count),
        rect.x + 12.0,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += 14.0;
    if let Some(check_run_id) = state.checks.check_run_id.as_deref() {
        if let Some(command) = check_show_command(check_run_id) {
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        }
        draw_text(
            &format!(
                "RUN {} / {} / {}",
                suffix_id(check_run_id).to_uppercase(),
                state
                    .checks
                    .profile_id
                    .as_deref()
                    .unwrap_or("default")
                    .to_uppercase(),
                state
                    .checks
                    .status
                    .as_deref()
                    .unwrap_or("unknown")
                    .to_uppercase()
            ),
            rect.x + 24.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
    }
    for finding in state.checks.findings.iter().take(MAX_CHECK_FINDINGS) {
        if y + 28.0 > bottom {
            return None;
        }
        if !finding.fingerprint.is_empty() {
            push_check_finding_hit_region(hit_regions, rect, y, &finding.fingerprint);
        }
        draw_text(
            &format!(
                "FIND {} {} {}",
                finding.severity.to_uppercase(),
                truncate_text(&finding.rule_id.to_uppercase(), 22),
                suffix_id(
                    finding
                        .finding_id
                        .as_deref()
                        .filter(|id| !id.is_empty())
                        .unwrap_or(&finding.fingerprint)
                )
                .to_uppercase()
            ),
            rect.x + 24.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
        if let Some(standards_basis) = finding_standards_basis(finding) {
            if y + 14.0 > bottom {
                return None;
            }
            draw_text(
                &format!(
                    "BASIS {}",
                    truncate_text(&standards_basis.to_uppercase(), 48)
                ),
                rect.x + 36.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 14.0;
        }
        y = render_check_finding_commands(finding, rect, y, bottom, text_runs, hit_regions)?;
    }
    if let Some(command) = check_run_command() {
        let Some(next_y) =
            render_check_command_row("RERUN", &command, rect, y, bottom, text_runs, hit_regions)
        else {
            return Some(y + 6.0);
        };
        y = next_y;
    }
    if let Some(command) = check_profiles_command() {
        let Some(next_y) = render_check_command_row(
            "PROFILES",
            &command,
            rect,
            y,
            bottom,
            text_runs,
            hit_regions,
        ) else {
            return Some(y + 6.0);
        };
        y = next_y;
    }
    if let Some(command) = check_list_command() {
        let Some(next_y) =
            render_check_command_row("LIST", &command, rect, y, bottom, text_runs, hit_regions)
        else {
            return Some(y + 6.0);
        };
        y = next_y;
    }
    for profile in ["standards", "release"] {
        let Some(command) = check_run_profile_command(profile) else {
            continue;
        };
        let Some(next_y) =
            render_check_command_row("PROFILE", &command, rect, y, bottom, text_runs, hit_regions)
        else {
            return Some(y + 6.0);
        };
        y = next_y;
    }
    Some(y + 6.0)
}

fn render_check_command_row(
    label: &str,
    command: &TerminalCommandHandoff,
    rect: RectPx,
    y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    if y + 14.0 > bottom {
        return None;
    }
    push_production_terminal_command_hit_region(hit_regions, rect, y, command);
    draw_text(
        &format!("{label} {}", truncate_text(&command.command, 54)),
        rect.x + 24.0,
        y,
        10.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    Some(y + 14.0)
}

fn render_check_finding_commands(
    finding: &datum_gui_protocol::CheckFindingSummary,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    let mut commands = Vec::new();
    if finding.domain == "zone_fill" || finding.source == "zone_fill" {
        commands.push(check_fill_zones_command());
    }
    if finding.rule_id.contains("process")
        || finding.rule_id.contains("aperture")
        || finding.domain == "standards"
    {
        commands.push(check_repair_standards_command());
    }
    if let Some(proposal_id) = first_finding_proposal_id(finding) {
        commands.push(proposal_show_command(&proposal_id));
        commands.push(proposal_preview_command(&proposal_id));
        commands.push(proposal_accept_apply_command(&proposal_id));
    }
    if !finding.fingerprint.is_empty() {
        commands.push(check_waive_command(&finding.fingerprint));
        commands.push(check_accept_deviation_command(&finding.fingerprint));
    }
    for command in commands.into_iter().flatten().take(4) {
        if y + 14.0 > bottom {
            return None;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("ACT {}", truncate_text(&command.command, 56)),
            rect.x + 36.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
    }
    Some(y)
}

fn first_finding_proposal_id(finding: &datum_gui_protocol::CheckFindingSummary) -> Option<String> {
    finding.proposal_refs.first().cloned().or_else(|| {
        finding.proposal_links.iter().find_map(|link| {
            link.get("proposal_id")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
    })
}

fn finding_standards_basis(finding: &datum_gui_protocol::CheckFindingSummary) -> Option<String> {
    finding.evidence.iter().find_map(|entry| {
        (entry.get("evidence_kind").and_then(|value| value.as_str()) == Some("standards_basis"))
            .then(|| entry.get("basis_id").and_then(|value| value.as_str()))
            .flatten()
            .map(str::to_string)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneratedFileViewer {
    label: &'static str,
    family: &'static str,
    projection_hint: Option<&'static str>,
}

fn generated_file_viewer(path: &str) -> GeneratedFileViewer {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".drl") {
        return GeneratedFileViewer {
            label: "EXCELLON",
            family: "NC DRILL",
            projection_hint: Some("excellon"),
        };
    }
    if lower.ends_with(".gbr") || lower.ends_with(".ger") || lower.ends_with(".gtl") {
        let family = if lower.contains("copper")
            || lower.contains("f_cu")
            || lower.contains("b_cu")
            || lower.ends_with(".gtl")
        {
            "GERBER COPPER"
        } else if lower.contains("outline") || lower.contains("edge") {
            "GERBER OUTLINE"
        } else if lower.contains("mask") {
            "GERBER MASK"
        } else if lower.contains("paste") {
            "GERBER PASTE"
        } else if lower.contains("silk") {
            "GERBER SILK"
        } else {
            "GERBER"
        };
        return GeneratedFileViewer {
            label: "GERBER",
            family,
            projection_hint: Some("gerber"),
        };
    }
    if lower.ends_with(".csv") && lower.contains("drill") {
        return GeneratedFileViewer {
            label: "DRILL CSV",
            family: "DRILL TABLE",
            projection_hint: Some("drill"),
        };
    }
    if lower.ends_with(".csv") && lower.contains("bom") {
        return GeneratedFileViewer {
            label: "BOM",
            family: "PARTS LIST",
            projection_hint: None,
        };
    }
    if lower.ends_with(".csv") && (lower.contains("pnp") || lower.contains("pick")) {
        return GeneratedFileViewer {
            label: "PNP",
            family: "PLACEMENT",
            projection_hint: None,
        };
    }
    GeneratedFileViewer {
        label: "FILE",
        family: "ARTIFACT",
        projection_hint: None,
    }
}

fn matching_file_projection<'a>(
    projections: &'a [datum_gui_protocol::ProductionArtifactProjectionSummary],
    viewer: GeneratedFileViewer,
) -> Option<&'a datum_gui_protocol::ProductionArtifactProjectionSummary> {
    let hint = viewer.projection_hint?;
    projections
        .iter()
        .find(|projection| projection.projection_kind.contains(hint))
        .or_else(|| projections.first())
}

fn push_artifact_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    artifact_id: &str,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionArtifact(artifact_id.to_string()),
        rect: RectPx {
            x: rect.x + 8.0,
            y: y - 2.0,
            width: rect.width - 16.0,
            height: 14.0,
        },
    });
}

fn push_artifact_file_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    path: &str,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionArtifactFile(path.to_string()),
        rect: RectPx {
            x: rect.x + 28.0,
            y: y - 2.0,
            width: rect.width - 36.0,
            height: 14.0,
        },
    });
}

fn push_check_finding_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    fingerprint: &str,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::CheckFinding(fingerprint.to_string()),
        rect: RectPx {
            x: rect.x + 20.0,
            y: y - 2.0,
            width: rect.width - 40.0,
            height: 14.0,
        },
    });
}

fn push_artifact_preview_controls(hit_regions: &mut Vec<HitRegion>, rect: RectPx) {
    hit_regions.push(HitRegion {
        target: HitTarget::ArtifactPreviewViewport,
        rect,
    });
    for (target, x) in [
        (
            HitTarget::ArtifactPreviewZoomOut,
            rect.x + rect.width - 118.0,
        ),
        (HitTarget::ArtifactPreviewReset, rect.x + rect.width - 92.0),
        (HitTarget::ArtifactPreviewZoomIn, rect.x + rect.width - 48.0),
        (
            HitTarget::ToggleArtifactPreviewGeometry,
            rect.x + rect.width - 178.0,
        ),
        (
            HitTarget::ToggleArtifactPreviewDrills,
            rect.x + rect.width - 148.0,
        ),
    ] {
        hit_regions.push(HitRegion {
            target,
            rect: RectPx {
                x,
                y: rect.y + 5.0,
                width: 24.0,
                height: 18.0,
            },
        });
    }
}

fn draw_artifact_preview_controls(
    preview: &datum_gui_protocol::ArtifactPreviewViewportState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    for (label, x) in [
        ("-", rect.x + rect.width - 111.0),
        ("RESET", rect.x + rect.width - 88.0),
        ("+", rect.x + rect.width - 41.0),
        (
            if preview.show_geometry { "G" } else { "g" },
            rect.x + rect.width - 171.0,
        ),
        (
            if preview.show_drills { "D" } else { "d" },
            rect.x + rect.width - 141.0,
        ),
    ] {
        draw_text(
            label,
            x,
            rect.y + 9.0,
            9.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
    }
}

fn render_csv_preview_table(
    preview: &datum_gui_protocol::ProductionArtifactFilePreviewSummary,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
) -> f32 {
    draw_text(
        &format!(
            "TABLE {} ROWS",
            preview.row_count.unwrap_or(preview.csv_rows.len())
        ),
        rect.x + 36.0,
        y,
        10.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += 14.0;
    if !preview.csv_columns.is_empty() && y + 14.0 <= bottom {
        draw_text(
            &truncate_text(&preview.csv_columns.join(" | "), 80),
            rect.x + 48.0,
            y,
            9.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
    }
    for row in preview.csv_rows.iter().take(4) {
        if y + 14.0 > bottom {
            return y;
        }
        draw_text(
            &truncate_text(&row.join(" | "), 80),
            rect.x + 48.0,
            y,
            9.5,
            TEXT_PRIMARY,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
    }
    y
}
