use datum_gui_protocol::ReviewWorkspaceState;

use super::outputs_lane_layout::{
    OutputsBodySectionKind, OutputsBodySectionLayout, OutputsBodySectionSpec,
    solve_outputs_body_sections_with_taffy,
};
use super::outputs_run_commands::{
    output_job_cancel_command, output_job_run_command, output_job_start_command,
    push_output_job_run_hit_region, push_production_terminal_command_hit_region,
};
use super::{
    HitRegion, HitTarget, RectPx, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun,
    draw_text, output_row_height, suffix_id, truncate_text,
};

const MAX_OUTPUT_JOBS: usize = 6;
const MAX_MANUFACTURING_PLANS: usize = 4;
const MAX_PANEL_PROJECTIONS: usize = 4;
const MAX_ARTIFACTS_PER_JOB: usize = 2;
const MAX_FILES_PER_ARTIFACT: usize = 3;
const MAX_PROJECTIONS_PER_ARTIFACT: usize = 2;

pub(super) fn render_lower_output_sections(
    state: &ReviewWorkspaceState,
    body: RectPx,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    for section in solve_lower_output_sections(state, body) {
        match section.kind {
            OutputsBodySectionKind::Panels => {
                render_panel_projection_section(state, section.rect, text_runs)
            }
            OutputsBodySectionKind::Plans => {
                render_manufacturing_plan_section(state, section.rect, text_runs)
            }
            OutputsBodySectionKind::Jobs => {
                render_output_jobs_section(state, section.rect, text_runs, hit_regions)
            }
            OutputsBodySectionKind::FocusedArtifact
            | OutputsBodySectionKind::Supervision
            | OutputsBodySectionKind::Checks
            | OutputsBodySectionKind::Actions => {}
        }
    }
}

fn solve_lower_output_sections(
    state: &ReviewWorkspaceState,
    body: RectPx,
) -> Vec<OutputsBodySectionLayout> {
    solve_outputs_body_sections_with_taffy(
        body,
        &[
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Panels,
                height: panel_projection_section_height(state),
            },
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Plans,
                height: manufacturing_plan_section_height(state),
            },
            OutputsBodySectionSpec {
                kind: OutputsBodySectionKind::Jobs,
                height: output_jobs_section_height(state),
            },
        ],
    )
    .unwrap_or_default()
}

fn panel_projection_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.production.panel_projections.is_empty() {
        return 0.0;
    }
    let rows = 1 + state
        .production
        .panel_projections
        .iter()
        .take(MAX_PANEL_PROJECTIONS)
        .map(|panel| 1 + usize::from(panel.first_board.is_some()))
        .sum::<usize>();
    rows as f32 * output_row_height() + 6.0
}

fn manufacturing_plan_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.production.manufacturing_plans.is_empty() {
        return 0.0;
    }
    let rows = 1 + state
        .production
        .manufacturing_plans
        .iter()
        .take(MAX_MANUFACTURING_PLANS)
        .count();
    rows as f32 * output_row_height() + 6.0
}

fn output_jobs_section_height(state: &ReviewWorkspaceState) -> f32 {
    state
        .production
        .output_jobs
        .iter()
        .take(MAX_OUTPUT_JOBS)
        .map(|job| {
            let mut height = 32.0 + 8.0;
            let row_height = output_row_height();
            if output_job_run_command(job).is_some() {
                height += row_height;
            }
            if job.status == "running" {
                if output_job_cancel_command(job).is_some() {
                    height += row_height;
                }
            } else if output_job_start_command(job).is_some() {
                height += row_height;
            }
            for artifact in job.artifacts.iter().take(MAX_ARTIFACTS_PER_JOB) {
                height += row_height;
                height += artifact
                    .production_projections
                    .iter()
                    .take(MAX_PROJECTIONS_PER_ARTIFACT)
                    .count() as f32
                    * row_height;
                height +=
                    artifact.files.iter().take(MAX_FILES_PER_ARTIFACT).count() as f32 * row_height;
            }
            height
        })
        .sum()
}

fn render_panel_projection_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let mut y = rect.y;
    let bottom = rect.y + rect.height;
    let row_height = output_row_height();
    if y + row_height > bottom {
        return;
    }
    draw_text(
        &format!("PANELS {}", state.production.panel_projection_count),
        rect.x,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    for panel in state
        .production
        .panel_projections
        .iter()
        .take(MAX_PANEL_PROJECTIONS)
    {
        if y + row_height > bottom {
            return;
        }
        draw_text(
            &format!(
                "PANEL {} / INST {} / REV {}",
                truncate_text(&panel.name.to_uppercase(), 28),
                panel.board_instance_count,
                panel.object_revision
            ),
            rect.x + 12.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
        if let Some(board) = &panel.first_board {
            if y + row_height > bottom {
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
                rect.x + 24.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
        }
    }
}

fn render_manufacturing_plan_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let mut y = rect.y;
    let bottom = rect.y + rect.height;
    let row_height = output_row_height();
    if y + row_height > bottom {
        return;
    }
    draw_text(
        &format!("PLANS {}", state.production.manufacturing_plan_count),
        rect.x,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    y += row_height;
    for plan in state
        .production
        .manufacturing_plans
        .iter()
        .take(MAX_MANUFACTURING_PLANS)
    {
        if y + row_height > bottom {
            return;
        }
        draw_text(
            &format!(
                "PLAN {} / {} / REV {}",
                truncate_text(&plan.name.to_uppercase(), 24),
                truncate_text(&plan.prefix, 20),
                plan.object_revision
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
}

fn render_output_jobs_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let mut y = rect.y;
    let bottom = rect.y + rect.height;
    let row_height = output_row_height();
    for job in state.production.output_jobs.iter().take(MAX_OUTPUT_JOBS) {
        if y + 32.0 > bottom {
            break;
        }
        let run_command = output_job_run_command(job);
        draw_text(
            &truncate_text(&job.name.to_uppercase(), 32),
            rect.x,
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
            rect.x,
            y + 16.0,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += 32.0;
        if let Some(command) = &run_command {
            if y + row_height > bottom {
                return;
            }
            push_output_job_run_hit_region(hit_regions, rect, y, command);
            draw_text(
                &format!("RUN {}", truncate_text(&command.command, 56)),
                rect.x + 12.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
        }
        let lifecycle_command = if job.status == "running" {
            output_job_cancel_command(job).map(|command| ("CANCEL", command))
        } else {
            output_job_start_command(job).map(|command| ("START", command))
        };
        if let Some((label, command)) = lifecycle_command {
            if y + row_height > bottom {
                return;
            }
            push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
            draw_text(
                &format!("{label} {}", truncate_text(&command.command, 54)),
                rect.x + 12.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
        }
        for artifact in job.artifacts.iter().take(MAX_ARTIFACTS_PER_JOB) {
            if y + row_height > bottom {
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
                rect.x + 12.0,
                y,
                10.0,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            y += row_height;
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
                        "PROOF {} {}B {}",
                        truncate_text(&projection.projection_kind.to_uppercase(), 24),
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
                y += row_height;
            }
            for file in artifact.files.iter().take(MAX_FILES_PER_ARTIFACT) {
                if y + row_height > bottom {
                    return;
                }
                push_artifact_file_hit_region(hit_regions, rect, y, &file.path);
                draw_text(
                    &format!(
                        "FILE {} {}",
                        truncate_text(&file.path, 44),
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
            }
        }
        y += 8.0;
    }
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
            height: output_row_height(),
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
            height: output_row_height(),
        },
    });
}
