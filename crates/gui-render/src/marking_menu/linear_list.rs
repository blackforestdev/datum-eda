//! Shared clipped linear marking list for Select and declared overflow actions.
use super::*;
pub(super) struct Row {
    pub label: String,
    pub slot: String,
    pub unavailable: Option<String>,
}
pub(super) fn render(
    preview: &MarkingMenuState,
    layout: &ShellLayout,
    title: &str,
    rows: &[Row],
    focus: usize,
    scroll: usize,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let bounds = layout.viewport;
    let width = 360.0_f32.min(bounds.width.max(0.0));
    let row = 30.0;
    let visible = ((bounds.height - 40.0).max(0.0) / row).floor() as usize;
    let mut start = scroll.min(rows.len().saturating_sub(1));
    if visible > 0 {
        if focus < start {
            start = focus;
        } else if focus >= start + visible {
            start = focus + 1 - visible;
        }
    }
    let height = (40.0 + row * visible.min(rows.len().saturating_sub(start)) as f32)
        .min(bounds.height.max(0.0));
    let x = (preview.anchor_x_px as f32)
        .clamp(bounds.x, (bounds.x + bounds.width - width).max(bounds.x));
    let y = (preview.anchor_y_px as f32)
        .clamp(bounds.y, (bounds.y + bounds.height - height).max(bounds.y));
    let panel = RectPx {
        x,
        y,
        width,
        height,
    };
    // Before individual rows: their actionable targets retain hit precedence.
    hits.push(HitRegion {
        rect: panel,
        target: HitTarget::MarkingMenuSurface {
            menu_key: preview.menu_key.clone(),
            anchor_x_px: preview.anchor_x_px,
            anchor_y_px: preview.anchor_y_px,
        },
    });
    quads.push(Quad::from_rect(panel, PANEL_BG));
    push_rect_border(quads, panel, PANEL_CARD_BORDER, 1.0);
    draw_text(
        title,
        x + 8.0,
        y + 10.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    for (index, item) in rows.iter().enumerate().skip(start).take(visible) {
        let rect = RectPx {
            x: x + 4.0,
            y: y + 36.0 + row * (index - start) as f32,
            width: (width - 8.0).max(0.0),
            height: row,
        };
        let focused = focus == index;
        quads.push(Quad::from_rect(
            rect,
            if focused {
                REVIEW_ROW_ACTIVE_BG
            } else {
                PANEL_BG
            },
        ));
        if focused {
            push_rect_border(quads, rect, TEXT_ACCENT, 1.0);
        }
        let label = if item.unavailable.is_some() {
            format!("Unavailable: {}", item.label)
        } else {
            item.label.clone()
        };
        let reason = focused.then_some(item.unavailable.as_ref()).flatten();
        let columns = ((rect.width - 12.0).max(0.0)
            / (design_tokens::typography::CAPTION_SIZE * 0.58)) as usize;
        draw_text_clipped(
            &truncate_text(&label, columns),
            rect.x + 6.0,
            rect.y + if reason.is_some() { 2.0 } else { 8.0 },
            design_tokens::typography::CAPTION_SIZE,
            if item.unavailable.is_some() {
                TEXT_MUTED
            } else {
                TEXT_PRIMARY
            },
            TextFace::Ui,
            rect,
            text,
        );
        if let Some(reason) = reason {
            // The captured cause also remains exact in the menu/accessibility
            // model. Reuse the Inspector's readable preview, without resolving
            // again or changing this disabled command's authority.
            // SelectionResolutionError::Engine wraps the existing electrical
            // cause in its Debug presentation; the shared preview labels that
            // cause, while the captured reason itself remains untouched.
            let cause = reason
                .strip_prefix("Engine(")
                .and_then(|cause| cause.strip_suffix(')'))
                .unwrap_or(reason);
            let reason =
                datum_gui_protocol::selection_view::InspectionValue::Unavailable(cause.into())
                    .display_preview(columns);
            draw_text_clipped(
                &reason,
                rect.x + 6.0,
                rect.y + 16.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_MUTED,
                TextFace::Ui,
                rect,
                text,
            );
        }
        hits.push(HitRegion {
            rect,
            target: HitTarget::MarkingMenuItem {
                menu_key: preview.menu_key.clone(),
                slot: item.slot.clone(),
                label: item.label.clone(),
            },
        });
    }
}
