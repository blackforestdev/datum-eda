//! Initial transfer uses the existing shared form and button primitives.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter, button};
use datum_gui_protocol::board_transfer::BoardTransferAction as Action;
use eda_engine::api::native_write::schematic_board_transfer::TransferStatus;
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let Some(review) = &state.ui.board_transfer else {
        return;
    };
    let mut painter = ControlPainter::new(quads, controls, scale);
    let starts = (painter.len(), text.len(), hits.len());
    let area = layout.viewport.scale_by(1.0 / scale);
    let card = RectPx {
        x: area.x + 12.0,
        y: area.y + 12.0,
        width: (area.width - 24.0).max(1.0),
        height: (area.height - 24.0).max(1.0),
    };
    painter.push(Quad::from_rect(card, PANEL_BG));
    // The review intercepts its own surface; ordinary application chrome remains available.
    hits.push(HitRegion {
        rect: card,
        target: HitTarget::BoardTransfer(Action::Surface),
    });
    let x = card.x + 14.0;
    let width = (card.width - 28.0).max(1.0);
    let mut y = card.y + 14.0;
    let mut line = |message: &str, y: f32| {
        draw_text_clipped(message, x, y, 14.0, TEXT_PRIMARY, TextFace::Ui, card, text)
    };
    line("Transfer root schematic to PCB", y);
    y += 25.0;
    line(
        "Choose each footprint/map explicitly. Existing counterparts are unchanged.",
        y,
    );
    y += 23.0;
    line(
        &format!(
            "{} ready / {} components — initial placement at 5 mm spacing",
            review.plan.ready_count(),
            review.plan.components.len()
        ),
        y,
    );
    y += 26.0;
    draw_text_clipped(
        "Preparation is a separate saved change. Closing this review does not undo it.",
        x,
        y,
        12.0,
        TEXT_MUTED,
        TextFace::Ui,
        card,
        text,
    );
    y += 20.0;
    let visible = (((card.height - 210.0).max(0.0) / 70.0) as usize).max(1);
    for (index, row) in review
        .plan
        .components
        .iter()
        .enumerate()
        .skip(review.first_row)
        .take(visible)
    {
        let status = match row.status {
            TransferStatus::Ready => "Ready",
            TransferStatus::Existing => "Already on PCB",
            TransferStatus::Unresolved => "Unresolved",
        };
        draw_text_clipped(
            &format!("{} — {}", row.reference, status),
            x,
            y,
            14.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            card,
            text,
        );
        y += 20.0;
        let selected = row
            .pin_pad_map
            .and_then(|id| row.map_options.iter().find(|o| o.pin_pad_map == id));
        let label = selected.map_or_else(
            || "Choose footprint / pin-pad map…".to_string(),
            |o| format!("Footprint: {} — click to cycle maps", o.footprint_name),
        );
        let rect = RectPx {
            x,
            y,
            width,
            height: 25.0,
        };
        let available = row.component_instance.is_some()
            && row.existing_packages.is_empty()
            && !row.map_options.is_empty();
        button(&label, rect, false, available, &mut painter, text);
        if available {
            hits.push(HitRegion {
                rect,
                target: HitTarget::BoardTransfer(Action::ChooseMap(index)),
            });
        }
        y += 28.0;
        if let Some(reason) = &row.reason {
            draw_text_clipped(reason, x, y, 12.0, TEXT_MUTED, TextFace::Ui, card, text);
        }
        y += 22.0;
    }
    y = card.y + card.height - 90.0;
    if let Some(reason) = &review.refusal {
        draw_text_clipped(reason, x, y, 13.0, TEXT_PRIMARY, TextFace::Ui, card, text);
    }
    y += 25.0;
    let actions = [
        ("Previous", Action::Previous, review.first_row > 0),
        (
            "Next",
            Action::Next,
            review.first_row + visible < review.plan.components.len(),
        ),
        (
            if review.plan.requires_profile_adoption {
                "Prepare footprint sources"
            } else {
                "Transfer ready components"
            },
            if review.plan.requires_profile_adoption {
                Action::PrepareSources
            } else {
                Action::Accept
            },
            review.plan.requires_profile_adoption || review.plan.ready_count() > 0,
        ),
        ("Close review", Action::Cancel, true),
    ];
    let widths = [75.0, 60.0, (width - 260.0).max(50.0), 110.0];
    let mut left = x;
    for ((label, action, available), w) in actions.into_iter().zip(widths) {
        let rect = RectPx {
            x: left,
            y,
            width: w,
            height: 30.0,
        };
        button(label, rect, false, available, &mut painter, text);
        if available {
            hits.push(HitRegion {
                rect,
                target: HitTarget::BoardTransfer(action),
            });
        }
        left += w + 5.0;
    }
    crate::global_preferences_dialog::dialog_coordinates::scale_output(
        &mut painter,
        text,
        hits,
        starts,
        scale,
    );
}
