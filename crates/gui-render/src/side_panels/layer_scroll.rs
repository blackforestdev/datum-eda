//! Scroll affordance for the bounded physical-layer row viewport.

use super::*;

pub(super) fn render_layer_scroll_affordance(
    state: &ReviewWorkspaceState,
    filters_rect: RectPx,
    layer_rows: &[RectPx],
    layer_count: usize,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> usize {
    let visible_capacity = layer_rows.len();
    let max_start = layer_count.saturating_sub(visible_capacity);
    let start = state.ui.filters.layer_scroll_offset.min(max_start);
    let (Some(first), Some(last)) = (layer_rows.first(), layer_rows.last()) else {
        return start;
    };
    let scroll_region = RectPx {
        x: filters_rect.x,
        y: first.y - 4.0,
        width: filters_rect.width,
        height: last.y + last.height - first.y + 8.0,
    };
    hit_regions.push(HitRegion {
        target: HitTarget::LayerScrollRegion,
        rect: scroll_region,
    });
    if layer_count <= visible_capacity {
        return start;
    }
    let track = RectPx {
        x: filters_rect.x + filters_rect.width - 5.0,
        y: scroll_region.y + 2.0,
        width: 2.0,
        height: (scroll_region.height - 4.0).max(1.0),
    };
    panel_quads.push(Quad::from_rect(track, PANEL_CARD_BORDER));
    let thumb_height = (track.height * visible_capacity as f32 / layer_count as f32)
        .max(16.0)
        .min(track.height);
    let travel = (track.height - thumb_height).max(0.0);
    let progress = if max_start == 0 {
        0.0
    } else {
        start as f32 / max_start as f32
    };
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: track.x - 1.0,
            y: track.y + travel * progress,
            width: 4.0,
            height: thumb_height,
        },
        TEXT_MUTED,
    ));
    draw_text(
        &format!(
            "{}–{} / {}",
            start + 1,
            (start + visible_capacity).min(layer_count),
            layer_count
        ),
        filters_rect.x + filters_rect.width - 72.0,
        first.y - 16.0,
        9.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    start
}
