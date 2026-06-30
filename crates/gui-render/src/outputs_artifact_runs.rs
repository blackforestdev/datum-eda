use datum_gui_protocol::ReviewWorkspaceState;

use super::outputs_run_commands::{
    artifact_compare_command, artifact_validate_command,
    push_production_terminal_command_hit_region,
};
use super::{HitRegion, HitTarget, RectPx};
use super::{TEXT_MUTED, TEXT_SECONDARY, TextFace, TextRun};
use super::{draw_text, output_row_height, suffix_id, truncate_text};

const MAX_ARTIFACT_RUNS: usize = 4;

pub(super) fn estimate_artifact_runs_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.production.artifact_runs.is_empty() {
        return 0.0;
    }
    let mut rows = 1;
    if latest_artifact_compare_command(state).is_some() {
        rows += 1;
    }
    rows += state
        .production
        .artifact_runs
        .iter()
        .rev()
        .take(MAX_ARTIFACT_RUNS)
        .map(|run| 1 + usize::from(artifact_validate_command(&run.artifact_id).is_some()))
        .sum::<usize>();
    rows as f32 * output_row_height() + 6.0
}

pub(super) fn render_artifact_runs_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    if state.production.artifact_runs.is_empty() {
        return Some(y);
    }
    draw_text(
        &format!("ARTIFACT RUNS {}", state.production.artifact_run_count),
        rect.x + 12.0,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    let row_height = output_row_height();
    y += row_height;
    if let Some(command) = latest_artifact_compare_command(state) {
        if y + row_height > bottom {
            return None;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("COMPARE {}", truncate_text(&command.command, 52)),
            rect.x + 24.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    for run in state
        .production
        .artifact_runs
        .iter()
        .rev()
        .take(MAX_ARTIFACT_RUNS)
    {
        if y + row_height > bottom {
            return None;
        }
        hit_regions.push(HitRegion {
            target: HitTarget::ProductionArtifact(run.artifact_id.clone()),
            rect: RectPx {
                x: rect.x + 18.0,
                y: y - 2.0,
                width: (rect.width - 36.0).max(0.0),
                height: row_height,
            },
        });
        draw_text(
            &format!(
                "{} {} ART {} / {} / SEQ {}{}",
                run.run_source.replace('_', " ").to_uppercase(),
                suffix_id(&run.run_id).to_uppercase(),
                suffix_id(&run.artifact_id).to_uppercase(),
                run.status.to_uppercase(),
                run.run_sequence,
                run.exit_code
                    .map(|code| format!(" / EXIT {code}"))
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
        let Some(command) = artifact_validate_command(&run.artifact_id) else {
            continue;
        };
        if y + row_height > bottom {
            return None;
        }
        push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("VALIDATE {}", truncate_text(&command.command, 52)),
            rect.x + 36.0,
            y,
            10.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
    }
    Some(y + 6.0)
}

fn latest_artifact_compare_command(
    state: &ReviewWorkspaceState,
) -> Option<datum_gui_protocol::TerminalCommandHandoff> {
    let mut latest_unique_artifacts = Vec::new();
    for run in state.production.artifact_runs.iter().rev() {
        if latest_unique_artifacts
            .iter()
            .any(|artifact_id| artifact_id == &run.artifact_id)
        {
            continue;
        }
        latest_unique_artifacts.push(run.artifact_id.as_str());
        if latest_unique_artifacts.len() == 2 {
            break;
        }
    }
    let after = latest_unique_artifacts.first()?;
    let before = latest_unique_artifacts.get(1)?;
    artifact_compare_command(before, after)
}
