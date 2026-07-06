use datum_gui_protocol::{ReviewWorkspaceState, TerminalCommandHandoff};

use super::outputs_lane_layout::{
    OutputsBodySectionKind, OutputsBodySectionLayout, OutputsBodySectionSpec, OutputsLaneLayout,
    solve_outputs_body_sections_with_taffy, solve_outputs_lane_layout_with_taffy,
};
use super::outputs_lane_sections::render_lower_output_sections;
use super::outputs_preview::render_artifact_preview_viewport;
use super::outputs_preview_controls::{
    draw_artifact_preview_controls, push_artifact_preview_controls,
};
use super::outputs_proposals::{estimate_action_sections_height, render_action_sections};
use super::outputs_run_commands::{
    artifact_files_command, artifact_list_command, artifact_preview_command, artifact_show_command,
    artifact_validate_command, check_accept_deviation_command, check_fill_zones_command,
    check_list_command, check_profiles_command, check_repair_standards_command, check_run_command,
    check_run_profile_command, check_show_command, check_waive_command,
    proposal_accept_apply_command, proposal_preview_command, proposal_show_command,
    push_production_terminal_command_hit_region,
};
use super::{HitRegion, HitTarget};
use super::{Quad, RectPx, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun};
use super::{draw_text, output_row_height, suffix_id, truncate_text};

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
    let layout = solve_outputs_lane_layout_with_taffy(rect).unwrap_or(OutputsLaneLayout {
        title: RectPx {
            x: rect.x + 12.0,
            y: rect.y + 12.0,
            width: rect.width - 24.0,
            height: 16.0,
        },
        status: RectPx {
            x: rect.x + 12.0,
            y: rect.y + 28.0,
            width: rect.width - 24.0,
            height: 18.0,
        },
        artifact_command: RectPx {
            x: rect.x + 12.0,
            y: rect.y + 50.0,
            width: rect.width - 24.0,
            height: 18.0,
        },
        body: RectPx {
            x: rect.x + 12.0,
            y: rect.y + 68.0,
            width: rect.width - 24.0,
            height: (rect.height - 86.0).max(1.0),
        },
    });
    draw_text(
        "OUTPUT JOBS",
        layout.title.x,
        layout.title.y,
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
        layout.status.x,
        layout.status.y,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let bottom = layout.body.y + layout.body.height;
    if let Some(command) = artifact_list_command() {
        push_production_terminal_command_hit_region(
            hit_regions,
            RectPx {
                x: layout.artifact_command.x,
                y: layout.artifact_command.y,
                width: layout.artifact_command.width,
                height: layout.artifact_command.height,
            },
            layout.artifact_command.y,
            &command,
        );
        draw_text(
            &format!("ARTIFACTS {}", truncate_text(&command.command, 52)),
            layout.artifact_command.x,
            layout.artifact_command.y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
    }
    let upper_sections = solve_upper_output_sections(state, layout.body);
    for section in &upper_sections {
        match section.kind {
            OutputsBodySectionKind::Supervision => {
                render_engine_supervision_section(state, section.rect, text_runs)
            }
            OutputsBodySectionKind::FocusedArtifact => render_focused_artifact_section(
                state,
                section.rect,
                panel_quads,
                text_runs,
                hit_regions,
            ),
            OutputsBodySectionKind::Checks => {
                let _ = render_checks_section(
                    state,
                    section.rect,
                    section.rect.y,
                    section.rect.y + section.rect.height,
                    text_runs,
                    hit_regions,
                );
            }
            OutputsBodySectionKind::Actions => {
                let _ = render_action_sections(
                    state,
                    section.rect,
                    section.rect.y,
                    section.rect.y + section.rect.height,
                    text_runs,
                    hit_regions,
                );
            }
            OutputsBodySectionKind::Panels
            | OutputsBodySectionKind::Plans
            | OutputsBodySectionKind::Jobs => {}
        }
    }
    let y = upper_sections
        .last()
        .map(|section| section.rect.y + section.rect.height)
        .unwrap_or(layout.body.y);
    let remaining_body = RectPx {
        x: layout.body.x,
        y,
        width: layout.body.width,
        height: (bottom - y).max(0.0),
    };
    render_lower_output_sections(state, remaining_body, text_runs, hit_regions);
}

fn solve_upper_output_sections(
    state: &ReviewWorkspaceState,
    body: RectPx,
) -> Vec<OutputsBodySectionLayout> {
    solve_outputs_body_sections_with_taffy(
        body,
        &[
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Supervision,
                height: engine_supervision_section_height(state),
            },
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::FocusedArtifact,
                height: focused_artifact_section_height(state),
            },
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Checks,
                height: checks_section_height(state),
            },
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Actions,
                height: estimate_action_sections_height(state),
            },
        ],
    )
    .unwrap_or_default()
}

fn engine_supervision_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.supervision.contract.is_empty() {
        return 0.0;
    }
    5.0 * output_row_height() + 6.0
}

fn render_engine_supervision_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let snapshot = &state.supervision;
    let mut y = rect.y;
    let bottom = rect.y + rect.height;
    let row_height = output_row_height();
    if y + row_height > bottom {
        return;
    }
    draw_text(
        "ENGINE SUPERVISION",
        rect.x,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    if y + row_height > bottom {
        return;
    }
    let tip = snapshot
        .journal
        .accepted_transaction_tip
        .as_deref()
        .map(|tip| suffix_id(tip).to_uppercase())
        .unwrap_or_else(|| "NONE".to_string());
    draw_text(
        &format!(
            "REV {} / JOURNAL {} / TIP {}",
            suffix_id(&snapshot.model_revision).to_uppercase(),
            snapshot.journal.applied_transaction_count,
            tip
        ),
        rect.x + 12.0,
        y,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    if y + row_height > bottom {
        return;
    }
    draw_text(
        &format!(
            "SHARDS {} / ATTENTION {} / READ ONLY {}",
            snapshot.source_shards.total,
            snapshot.source_shards.attention_count(),
            if snapshot.read_only { "YES" } else { "NO" }
        ),
        rect.x + 12.0,
        y,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    if y + row_height > bottom {
        return;
    }
    draw_text(
        &format!(
            "{} C{} P{} T{} V{} Z{} TXT{} L{}",
            truncate_text(&snapshot.scene_kind.to_uppercase(), 22),
            snapshot.scene.component_count,
            snapshot.scene.pad_count,
            snapshot.scene.track_count,
            snapshot.scene.via_count,
            snapshot.scene.zone_count,
            snapshot.scene.board_text_count,
            snapshot.scene.layer_count
        ),
        rect.x + 12.0,
        y,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    if y + row_height > bottom {
        return;
    }
    draw_text(
        &format!(
            "CHECK {} FIND {} / DATA JOB{} ART{} PROP{}",
            snapshot
                .checks
                .status
                .as_deref()
                .unwrap_or("not_run")
                .to_uppercase(),
            snapshot.checks.finding_count,
            snapshot.data.output_job_count,
            snapshot.data.artifact_count,
            snapshot.data.proposal_count
        ),
        rect.x + 12.0,
        y,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

fn focused_artifact_section_height(state: &ReviewWorkspaceState) -> f32 {
    let Some(artifact) = state.production.focused_artifact.as_ref() else {
        return 0.0;
    };
    let mut rows = 1;
    rows += usize::from(artifact_show_command(&artifact.artifact_id).is_some());
    rows += usize::from(artifact_validate_command(&artifact.artifact_id).is_some());
    rows += usize::from(artifact_files_command(&artifact.artifact_id).is_some());
    rows += artifact
        .production_projections
        .iter()
        .take(MAX_PROJECTIONS_PER_ARTIFACT)
        .count();
    rows += artifact.files.iter().take(MAX_FILES_PER_ARTIFACT).count();
    if let Some(file) = &artifact.focused_file {
        rows += 2;
        rows += usize::from(artifact_preview_command(&artifact.artifact_id, &file.path).is_some());
        if let Some(preview) = &artifact.focused_preview {
            rows += 1;
            if !preview.csv_rows.is_empty() {
                rows += 1;
                rows += usize::from(!preview.csv_columns.is_empty());
                rows += preview.csv_rows.len().min(4);
            }
        }
        let viewer = generated_file_viewer(&file.path);
        rows += usize::from(
            matching_file_projection(artifact.production_projections.as_slice(), viewer).is_some(),
        );
    }
    rows as f32 * output_row_height() + 6.0
}

fn checks_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.checks.check_run_id.is_none() && state.checks.findings.is_empty() {
        return 0.0;
    }
    let mut rows = 1;
    rows += usize::from(state.checks.check_run_id.is_some());
    for finding in state.checks.findings.iter().take(MAX_CHECK_FINDINGS) {
        rows += 1;
        rows += usize::from(finding_standards_basis(finding).is_some());
        rows += check_finding_command_count(finding).min(4);
    }
    rows += usize::from(check_run_command().is_some());
    rows += usize::from(check_profiles_command().is_some());
    rows += usize::from(check_list_command().is_some());
    rows += ["standards", "release"]
        .into_iter()
        .filter(|profile| check_run_profile_command(profile).is_some())
        .count();
    rows as f32 * output_row_height() + 6.0
}

fn check_finding_command_count(finding: &datum_gui_protocol::CheckFindingSummary) -> usize {
    let mut count = 0;
    if finding.domain == "zone_fill" || finding.source == "zone_fill" {
        count += usize::from(check_fill_zones_command().is_some());
    }
    if finding.rule_id.contains("process")
        || finding.rule_id.contains("aperture")
        || finding.domain == "standards"
    {
        count += usize::from(check_repair_standards_command().is_some());
    }
    if let Some(proposal_id) = first_finding_proposal_id(finding) {
        count += usize::from(proposal_show_command(&proposal_id).is_some());
        count += usize::from(proposal_preview_command(&proposal_id).is_some());
        count += usize::from(proposal_accept_apply_command(&proposal_id).is_some());
    }
    if !finding.fingerprint.is_empty() {
        count += usize::from(check_waive_command(&finding.fingerprint).is_some());
        count += usize::from(check_accept_deviation_command(&finding.fingerprint).is_some());
    }
    count
}

fn render_focused_artifact_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let Some(artifact) = &state.production.focused_artifact else {
        return;
    };
    let mut y = rect.y;
    let bottom = rect.y + rect.height;
    let row_height = output_row_height();
    push_artifact_hit_region(hit_regions, rect, y, &artifact.artifact_id);
    draw_text(
        &format!(
            "FOCUS ART {} {} / {}",
            artifact.kind.to_uppercase(),
            suffix_id(&artifact.artifact_id).to_uppercase(),
            artifact.validation_state.to_uppercase()
        ),
        rect.x,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    if let Some(command) = artifact_show_command(&artifact.artifact_id) {
        if y + row_height > bottom {
            return;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("SHOW {}", truncate_text(&command.command, 55)),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    if let Some(command) = artifact_validate_command(&artifact.artifact_id) {
        if y + row_height > bottom {
            return;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("VALIDATE {}", truncate_text(&command.command, 52)),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    if let Some(command) = artifact_files_command(&artifact.artifact_id) {
        if y + row_height > bottom {
            return;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("FILES {}", truncate_text(&command.command, 55)),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    for projection in artifact
        .production_projections
        .iter()
        .take(MAX_PROJECTIONS_PER_ARTIFACT)
    {
        if y + row_height > bottom {
            return;
        }
        draw_text(
            &format!(
                "FOCUS PROOF {} {}B {}",
                truncate_text(&projection.projection_kind.to_uppercase(), 20),
                projection.byte_count,
                suffix_id(&projection.sha256).to_uppercase()
            ),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    for file in artifact.files.iter().take(MAX_FILES_PER_ARTIFACT) {
        if y + row_height > bottom {
            return;
        }
        push_artifact_file_hit_region(hit_regions, rect, y, &file.path);
        draw_text(
            &format!(
                "FOCUS FILE {} {}",
                truncate_text(&file.path, 42),
                suffix_id(&file.sha256).to_uppercase()
            ),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    if let Some(file) = &artifact.focused_file {
        if y + row_height * 3.0 > bottom {
            return;
        }
        let viewer = generated_file_viewer(&file.path);
        draw_text(
            &format!("{} VIEW", viewer.label),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
        if let Some(command) = artifact_preview_command(&artifact.artifact_id, &file.path) {
            if y + row_height > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("PREVIEW {}", truncate_text(&command.command, 52)),
                rect.x + 24.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
        }
        draw_text(
            &format!(
                "{} / {} / {}",
                viewer.family,
                truncate_text(&file.path, 48),
                suffix_id(&file.sha256).to_uppercase()
            ),
            rect.x + 24.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
        if let Some(preview) = &artifact.focused_preview {
            if y + row_height > bottom {
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
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
            if !preview.csv_rows.is_empty() {
                y = render_csv_preview_table(preview, rect, y, bottom, text_runs);
            }
            if !preview.primitives.is_empty() && rect.width >= 520.0 {
                let preview_rect = RectPx {
                    x: rect.x + rect.width - 384.0,
                    y: rect.y,
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
                draw_artifact_preview_controls(&state.ui.artifact_preview, preview_rect, text_runs);
            }
        }
        if let Some(projection) =
            matching_file_projection(artifact.production_projections.as_slice(), viewer)
        {
            if y + row_height > bottom {
                return;
            }
            draw_text(
                &format!(
                    "VIEW PROOF {} {}B {}",
                    truncate_text(&projection.projection_kind.to_uppercase(), 22),
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
        }
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
    let row_height = output_row_height();
    if y + row_height * 3.0 > bottom {
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
    y += row_height;
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
        y += row_height;
    }
    for finding in state.checks.findings.iter().take(MAX_CHECK_FINDINGS) {
        if y + row_height * 2.0 > bottom {
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
        y += row_height;
        if let Some(standards_basis) = finding_standards_basis(finding) {
            if y + row_height > bottom {
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
            y += row_height;
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
    let row_height = output_row_height();
    if y + row_height > bottom {
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
    Some(y + row_height)
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
    let row_height = output_row_height();
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
        if y + row_height > bottom {
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
        y += row_height;
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
    let row_height = output_row_height();
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionArtifact(artifact_id.to_string()),
        rect: RectPx {
            x: rect.x + 8.0,
            y: y - 2.0,
            width: rect.width - 16.0,
            height: row_height,
        },
    });
}

fn push_artifact_file_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    path: &str,
) {
    let row_height = output_row_height();
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionArtifactFile(path.to_string()),
        rect: RectPx {
            x: rect.x + 28.0,
            y: y - 2.0,
            width: rect.width - 36.0,
            height: row_height,
        },
    });
}

fn push_check_finding_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    fingerprint: &str,
) {
    let row_height = output_row_height();
    hit_regions.push(HitRegion {
        target: HitTarget::CheckFinding(fingerprint.to_string()),
        rect: RectPx {
            x: rect.x + 20.0,
            y: y - 2.0,
            width: rect.width - 40.0,
            height: row_height,
        },
    });
}

fn render_csv_preview_table(
    preview: &datum_gui_protocol::ProductionArtifactFilePreviewSummary,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
) -> f32 {
    let row_height = output_row_height();
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
    y += row_height;
    if !preview.csv_columns.is_empty() && y + row_height <= bottom {
        draw_text(
            &truncate_text(&preview.csv_columns.join(" | "), 80),
            rect.x + 48.0,
            y,
            9.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    for row in preview.csv_rows.iter().take(4) {
        if y + row_height > bottom {
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
        y += row_height;
    }
    y
}
