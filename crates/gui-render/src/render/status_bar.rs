// Segmented status-bar renderer, extracted from `scene.rs` to keep it under its
// source-health ceiling (decision 022) while `scene.rs` takes on the S4
// interaction-overlay wiring. A real `#[path] mod` child of the crate root
// (declared in `scene.rs`), so it reaches the crate-root render helpers, colour
// constants, and layout types via `use super::*` exactly as the inline code did.
// Behaviour is unchanged — a verbatim move.

use super::*;

/// Segmented status bar (Design Book .status): labelled key/value segments with
/// full-height dividers, a flex gap, and a right-aligned build/version run. The
/// focus value reads accent; a DRC segment reads STATUS_WARN and is hidden at
/// zero findings.
pub(crate) fn render_status_bar(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let sb = layout.status_bar;
    // Single top-edge hairline (no boxed 4-side border).
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: sb.x,
            y: sb.y,
            width: sb.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    let text_y = sb.y + design_tokens::spacing::SP_02 + 1.0;
    let lab_size = design_tokens::typography::CAPTION_SIZE;
    let val_size = design_tokens::typography::DATA_SIZE;
    let gap = design_tokens::spacing::SP_02 + 2.0;
    let seg_pad = design_tokens::spacing::SP_04;
    let text_w = |text: &str, size: f32| estimated_text_run_width_px(text, size, TextFace::Mono) - 16.0;
    let divider = |panel_quads: &mut Vec<Quad>, x: f32| {
        panel_quads.push(Quad::from_rect(
            RectPx {
                x,
                y: sb.y,
                width: 1.0,
                height: sb.height,
            },
            PANEL_CARD_BORDER,
        ));
    };

    let sel = match &state.selection {
        SelectionTarget::None => "none".to_string(),
        SelectionTarget::ReviewAction(id)
        | SelectionTarget::AuthoredObject(id)
        | SelectionTarget::CheckFinding(id) => truncate_text(suffix_id(id), 8),
    };
    let tool = workspace_tool_label(state.tool);
    let layers = state.scene.layers.len().to_string();
    // Reflect the actually-focused document, not a hardcoded value — focusing the
    // Schematic pane must read "Schematic" here (context-follows-focus).
    let focus_label = match state.ui.layout.focused_content() {
        datum_gui_protocol::PaneContent::Board => "Board",
        datum_gui_protocol::PaneContent::Schematic => "Schematic",
    };
    let left: [(&str, &str, [f32; 3]); 4] = [
        ("focus", focus_label, TEXT_ACCENT),
        ("Tool", tool, TEXT_SECONDARY),
        ("Sel", sel.as_str(), TEXT_SECONDARY),
        ("Layers", layers.as_str(), TEXT_SECONDARY),
    ];
    let mut x = sb.x + seg_pad;
    for (i, (label, value, color)) in left.iter().enumerate() {
        if i > 0 {
            divider(panel_quads, x - seg_pad * 0.5);
        }
        draw_text(label, x, text_y, lab_size, TEXT_MUTED, TextFace::Mono, text_runs);
        let lw = text_w(label, lab_size) + gap;
        draw_text(value, x + lw, text_y, val_size, *color, TextFace::Mono, text_runs);
        x += lw + text_w(value, val_size) + seg_pad;
    }

    // Right cluster (right-to-left): version, rev, DRC.
    let version = "Datum EDA \u{2014} design pass";
    let mut rx = sb.x + sb.width - 13.0 - text_w(version, val_size);
    draw_text(version, rx, text_y, val_size, TEXT_MUTED, TextFace::Mono, text_runs);

    let short_rev: String = state.scene.source_revision.chars().take(6).collect();
    if !short_rev.is_empty() {
        let lw = text_w("rev", lab_size) + gap;
        rx -= seg_pad + lw + text_w(&short_rev, val_size);
        divider(panel_quads, rx - seg_pad * 0.5);
        draw_text("rev", rx, text_y, lab_size, TEXT_MUTED, TextFace::Mono, text_runs);
        draw_text(&short_rev, rx + lw, text_y, val_size, TEXT_SECONDARY, TextFace::Mono, text_runs);
    }

    let findings = state.supervision.checks.finding_count;
    if findings > 0 {
        let drc = format!("DRC {}", findings);
        rx -= seg_pad + text_w(&drc, val_size);
        divider(panel_quads, rx - seg_pad * 0.5);
        draw_text(
            &drc,
            rx,
            text_y,
            val_size,
            design_tokens::chrome::STATUS_WARN,
            TextFace::Mono,
            text_runs,
        );
    }
}
