//! Shared Inspector identity and section chrome, including fallible measured labels.
use super::*;

/// Generic selection-identity band (Design Book insp-title): identity primary
/// line, muted mono kind subtitle, a right-aligned accent-bordered SELECTED
/// pill, and a BORDER_SUBTLE bottom divider. This is selection chrome that
/// applies to the routing-review ACTION selection too — not the deferred
/// populated-component inspector.
pub(super) fn push_inspector_title_band(
    rect: RectPx,
    identity: &str,
    kind: &str,
    show_pill: bool,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) -> anyhow::Result<()> {
    let band_top = rect.y + 34.0;
    draw_text(
        identity,
        rect.x + 12.0,
        band_top,
        16.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text_runs,
    );
    if !kind.is_empty() {
        draw_text(
            kind,
            rect.x + 12.0,
            band_top + 18.0,
            11.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    if show_pill {
        let label = "SELECTED";
        let pad_x = 7.0_f32;
        let text_w = measured_text_run_width_px(label, 10.0, TextFace::UiMedium)?;
        let pill = RectPx {
            x: rect.x + rect.width - 12.0 - (text_w + pad_x * 2.0),
            y: band_top - 1.0,
            width: text_w + pad_x * 2.0,
            height: 16.0,
        };
        push_rect_border(panel_quads, pill, TEXT_ACCENT, 1.0);
        draw_text(
            label,
            pill.x + pad_x,
            pill.y + 3.0,
            10.0,
            TEXT_ACCENT,
            TextFace::UiMedium,
            text_runs,
        );
    }
    push_section_divider(
        panel_quads,
        rect.x,
        rect.y + 66.0,
        rect.width,
        PANEL_CARD_BORDER,
    );

    Ok(())
}

/// Uppercase section-header strip (Design Book sect-hd): a SURFACE_01 band with
/// a top BORDER_SUBTLE hairline and an ~11px semibold TEXT_MUTED label. Generic
/// chrome; the named Identity/Placement/Checks component sections stay deferred.
pub(super) fn push_section_header_strip(
    x: f32,
    y: f32,
    width: f32,
    label: &str,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let band = RectPx {
        x,
        y,
        width,
        height: 18.0,
    };
    panel_quads.push(Quad::from_rect(band, design_tokens::chrome::SURFACE_01));
    push_section_divider(panel_quads, x, y, width, PANEL_CARD_BORDER);
    draw_text(
        label,
        x + 12.0,
        y + 4.0,
        11.0,
        TEXT_MUTED,
        TextFace::UiStrong,
        text_runs,
    );
}
