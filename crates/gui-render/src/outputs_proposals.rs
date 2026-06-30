use datum_gui_protocol::{ReviewWorkspaceState, TerminalCommandHandoff};

use super::outputs_artifact_runs::{
    estimate_artifact_runs_section_height, render_artifact_runs_section,
};
use super::outputs_run_commands::{
    proposal_accept_apply_command, proposal_defer_command, proposal_list_command,
    proposal_preview_command, proposal_reject_command, proposal_show_command,
    proposal_validate_command, push_production_terminal_command_hit_region,
};
use super::{HitRegion, draw_text, output_row_height, suffix_id, truncate_text};
use super::{RectPx, TEXT_MUTED, TEXT_SECONDARY, TextFace, TextRun};

const MAX_PROPOSALS: usize = 4;

pub(super) fn estimate_action_sections_height(state: &ReviewWorkspaceState) -> f32 {
    estimate_artifact_runs_section_height(state) + estimate_proposals_section_height(state)
}

pub(super) fn render_action_sections(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    let y = render_artifact_runs_section(state, rect, y, bottom, text_runs, hit_regions)?;
    render_proposals_section(state, rect, y, bottom, text_runs, hit_regions)
}

fn estimate_proposals_section_height(state: &ReviewWorkspaceState) -> f32 {
    if state.production.proposals.is_empty() {
        return 0.0;
    }
    let mut rows = 1;
    rows += usize::from(proposal_list_command().is_some());
    for proposal in state.production.proposals.iter().take(MAX_PROPOSALS) {
        rows += 2;
        rows += usize::from(!proposal.blocker_codes.is_empty());
        rows += usize::from(proposal.preview.is_some());
        rows += usize::from(proposal_accept_apply_command(&proposal.proposal_id).is_some());
        rows += usize::from(proposal_show_command(&proposal.proposal_id).is_some());
        rows += usize::from(proposal_preview_command(&proposal.proposal_id).is_some());
        rows += usize::from(proposal_validate_command(&proposal.proposal_id).is_some());
        rows += usize::from(proposal_defer_command(&proposal.proposal_id).is_some());
        rows += usize::from(proposal_reject_command(&proposal.proposal_id).is_some());
    }
    rows as f32 * output_row_height()
        + state.production.proposals.len().min(MAX_PROPOSALS) as f32 * 4.0
        + 2.0
}

pub(super) fn render_proposals_section(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    mut y: f32,
    bottom: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    if state.production.proposals.is_empty() {
        return Some(y);
    }
    draw_text(
        &format!("PROPOSALS {}", state.production.proposal_count),
        rect.x + 12.0,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    let row_height = output_row_height();
    y += row_height;
    y = render_proposal_command(
        rect,
        y,
        bottom,
        "LIST",
        proposal_list_command(),
        text_runs,
        hit_regions,
    )?;
    for proposal in state.production.proposals.iter().take(MAX_PROPOSALS) {
        if y + row_height > bottom {
            return None;
        }
        draw_text(
            &format!(
                "PROP {} / {} / {} OPS{}",
                suffix_id(&proposal.proposal_id).to_uppercase(),
                proposal.status.to_uppercase(),
                proposal.operation_count,
                proposal
                    .can_apply
                    .map(|can_apply| if can_apply {
                        " / APPLY OK"
                    } else {
                        " / BLOCKED"
                    })
                    .unwrap_or("")
            ),
            rect.x + 24.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
        if y + row_height > bottom {
            return None;
        }
        draw_text(
            &format!(
                "{} / {}",
                truncate_text(&proposal.source.to_uppercase(), 14),
                truncate_text(&proposal.rationale, 42)
            ),
            rect.x + 36.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += row_height;
        if !proposal.blocker_codes.is_empty() {
            if y + row_height > bottom {
                return None;
            }
            draw_text(
                &format!(
                    "BLOCKERS {}",
                    truncate_text(&proposal.blocker_codes.join(","), 50)
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
        if let Some(preview) = &proposal.preview {
            if y + row_height > bottom {
                return None;
            }
            draw_text(
                &format!(
                    "PREVIEW DIFF +{} ~{} -{} / {} OBJ",
                    preview.created_count,
                    preview.modified_count,
                    preview.deleted_count,
                    preview.affected_object_count
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
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "ACCEPT+APPLY",
            proposal_accept_apply_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "SHOW",
            proposal_show_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "PREVIEW",
            proposal_preview_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "VALIDATE",
            proposal_validate_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "DEFER",
            proposal_defer_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y = render_proposal_command(
            rect,
            y,
            bottom,
            "REJECT",
            proposal_reject_command(&proposal.proposal_id),
            text_runs,
            hit_regions,
        )?;
        y += 4.0;
    }
    Some(y + 2.0)
}

fn render_proposal_command(
    rect: RectPx,
    y: f32,
    bottom: f32,
    label: &str,
    command: Option<TerminalCommandHandoff>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    let command = command?;
    let row_height = output_row_height();
    if y + row_height > bottom {
        return None;
    }
    push_production_terminal_command_hit_region(hit_regions, rect, y, &command);
    draw_text(
        &format!("{label} {}", truncate_text(&command.command, 48)),
        rect.x + 36.0,
        y,
        10.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    Some(y + row_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_proposal_preview_diff_summary() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.production.proposal_count = 1;
        state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "00000000-0000-0000-0000-00000000aa01".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "preview render".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: Vec::new(),
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 1,
                modified_count: 2,
                deleted_count: 0,
                affected_object_count: 3,
                affected_objects: vec![
                    "00000000-0000-0000-0000-00000000bb01".to_string(),
                    "00000000-0000-0000-0000-00000000bb02".to_string(),
                    "00000000-0000-0000-0000-00000000bb03".to_string(),
                ],
                render_deltas: Vec::new(),
            }),
        }];
        let mut text_runs = Vec::new();
        let mut hit_regions = Vec::new();

        let rendered = render_proposals_section(
            &state,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 640.0,
                height: 480.0,
            },
            10.0,
            480.0,
            &mut text_runs,
            &mut hit_regions,
        );

        assert!(rendered.is_some());
        assert!(
            text_runs
                .iter()
                .any(|run| run.text.contains("PREVIEW DIFF +1 ~2 -0 / 3 OBJ"))
        );
    }
}
