//! Explicit parent-action chrome remains reachable alongside any Inspector subject.
use super::*;

pub(super) fn split(state: &ReviewWorkspaceState, rect: RectPx) -> (RectPx, Option<RectPx>) {
    if state.review.proposal_actions.is_empty() {
        return (rect, None);
    }
    let height = 118.0_f32.min(rect.height * 0.4);
    let inspector = RectPx {
        height: rect.height - height,
        ..rect
    };
    let lane = RectPx {
        y: rect.y + inspector.height,
        height,
        ..rect
    };
    (inspector, Some(lane))
}

pub(super) fn render(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let starts = (quads.len(), text.len(), hits.len());
    quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_section_divider(quads, rect.x, rect.y, rect.width, PANEL_CARD_BORDER);
    draw_text(
        "REVIEW ACTIONS",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    let rows = state.review_rows();
    let index = rows
        .iter()
        .position(|row| row.action_id == state.active_review_target_id)
        .unwrap_or(0);
    if let Some(row) = rows.get(index) {
        let row_rect = RectPx {
            x: rect.x + 8.0,
            y: rect.y + 34.0,
            width: (rect.width - 16.0).max(0.0),
            height: 46.0,
        };
        quads.push(Quad::from_rect(row_rect, PANEL_CARD_BG));
        push_rect_border(quads, row_rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            &truncate_text(&row.title, 24),
            row_rect.x + 8.0,
            row_rect.y + 7.0,
            13.0,
            TEXT_PANEL_VALUE,
            TextFace::Ui,
            text,
        );
        draw_text(
            &format!(
                "{} / {}  {}",
                index + 1,
                rows.len(),
                truncate_text(&row.action_id, 16)
            ),
            row_rect.x + 8.0,
            row_rect.y + 25.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::ReviewAction(row.action_id.clone()),
            rect: row_rect,
        });
    }
    for (x, label, target) in [
        (rect.x + 8.0, "PREV", HitTarget::ReviewPrev),
        (rect.x + 64.0, "NEXT", HitTarget::ReviewNext),
    ] {
        let button = RectPx {
            x,
            y: rect.y + 88.0,
            width: 48.0,
            height: 22.0,
        };
        push_rect_border(quads, button, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            x + 6.0,
            button.y + 5.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            target,
            rect: button,
        });
    }
    crate::hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, rect);
}

#[cfg(test)]
mod tests;
