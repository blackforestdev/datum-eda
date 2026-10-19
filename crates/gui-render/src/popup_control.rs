//! Generic popup paint, clipping and hit ownership; clients supply typed items.
use super::*;
use crate::global_preferences_primitives::{
    ControlMeshCache, ControlPainter, push_rounded_rect_with_border,
};
use datum_gui_protocol::popup_menu::PopupMenu;
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint<A: Clone>(
    menu: &PopupMenu<A>,
    viewport: RectPx,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    target: impl Fn(usize) -> HitTarget,
) -> anyhow::Result<()> {
    let width = menu
        .items
        .iter()
        .try_fold(230.0_f32, |w, item| -> anyhow::Result<f32> {
            Ok(w.max(
                measured_text_run_width_px(&item.label, 12.5, TextFace::Ui)?
                    + item
                        .shortcut
                        .as_ref()
                        .map(|s| measured_text_run_width_px(s, 10.5, TextFace::Mono))
                        .transpose()?
                        .unwrap_or(0.0)
                    + 44.0,
            ))
        })?
        .min(viewport.width.max(1.0));
    let reasons = menu
        .items
        .iter()
        .map(|item| {
            item.unavailable
                .as_deref()
                .map(|reason| crate::control_text::wrap_text(reason, width - 24.0))
                .transpose()
                .map(|r| r.unwrap_or_default())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let height = 10.0
        + menu.items.len() as f32 * 28.5
        + reasons
            .iter()
            .map(|r| {
                if r.is_empty() {
                    0.0
                } else {
                    r.len() as f32 * 14.0 + 6.0
                }
            })
            .sum::<f32>();
    let card = RectPx {
        x: menu.anchor.0.clamp(
            viewport.x,
            (viewport.x + viewport.width - width).max(viewport.x),
        ),
        y: menu.anchor.1.clamp(
            viewport.y,
            (viewport.y + viewport.height - height).max(viewport.y),
        ),
        width,
        height,
    };
    let starts = (quads.len(), text.len(), hits.len());
    let mut painter = ControlPainter::new(quads, controls, scale);
    push_rounded_rect_with_border(
        &mut painter,
        card,
        design_tokens::chrome::SURFACE_03,
        design_tokens::chrome::BORDER_STRONG,
        1.0,
        6.0,
    );
    let mut y = card.y + 5.0;
    for (index, (item, reason)) in menu.items.iter().zip(reasons).enumerate() {
        let row = RectPx {
            x: card.x + 1.0,
            y,
            width: card.width - 2.0,
            height: 28.5,
        };
        if menu.focused == Some(index) && item.unavailable.is_none() {
            painter.push(Quad::from_rect(row, design_tokens::chrome::ACCENT_TINT));
            push_rect_border(&mut painter, row, design_tokens::chrome::ACCENT, 1.0);
        }
        draw_text(
            &item.label,
            row.x + 12.0,
            row.y + 7.0,
            12.5,
            if item.unavailable.is_none() {
                TEXT_PRIMARY
            } else {
                TEXT_MUTED
            },
            TextFace::Ui,
            text,
        );
        if let Some(shortcut) = &item.shortcut {
            let w = measured_text_run_width_px(shortcut, 10.5, TextFace::Mono)?;
            draw_text(
                shortcut,
                row.x + row.width - 12.0 - w,
                row.y + 8.0,
                10.5,
                TEXT_MUTED,
                TextFace::Mono,
                text,
            );
        }
        hits.push(HitRegion {
            target: target(index),
            rect: row,
        });
        y += row.height;
        for line in &reason {
            draw_text(line, row.x + 12.0, y, 10.5, TEXT_MUTED, TextFace::Ui, text);
            y += 14.0;
        }
        if !reason.is_empty() {
            y += 6.0;
        }
    }
    drop(painter);
    crate::hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, viewport);
    Ok(())
}
