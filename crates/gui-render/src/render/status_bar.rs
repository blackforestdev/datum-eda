// Segmented status-bar renderer, extracted from `scene.rs` to keep it under its
// source-health ceiling (decision 022) while `scene.rs` takes on the S4
// interaction-overlay wiring. A real `#[path] mod` child of the crate root
// (declared in `scene.rs`), so it reaches the crate-root render helpers, colour
// constants, and layout types via `use super::*` exactly as the inline code did.
// Paint is clipped through the shared ancestor owner after segment layout.

use super::*;

pub(crate) fn application_focus_label(state: &ReviewWorkspaceState) -> &'static str {
    match state.ui.focus {
        datum_gui_protocol::ApplicationFocus::Terminal => "Terminal",
        datum_gui_protocol::ApplicationFocus::Overlay => "Overlay",
        datum_gui_protocol::ApplicationFocus::Editor(pane) => match state
            .ui
            .layout
            .content_for(pane)
            .unwrap_or_else(|| state.ui.layout.focused_content())
        {
            datum_gui_protocol::PaneContent::Board => "Board",
            datum_gui_protocol::PaneContent::Schematic => "Schematic",
            datum_gui_protocol::PaneContent::Revision(_) => "Revision",
        },
    }
}

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
    let scale = crate::text_presentation::chrome_scale::Scale::for_layout(layout);
    let starts = (panel_quads.len(), text_runs.len());
    render_status_bar_logical(
        state,
        &scale.logical_layout(layout.clone()),
        panel_quads,
        text_runs,
    );
    scale.quads(&mut panel_quads[starts.0..]);
    scale.text_geometry(&mut text_runs[starts.1..]);
}

pub(crate) fn render_status_bar_logical(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let starts = (panel_quads.len(), text_runs.len());
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
    let text_w =
        |text: &str, size: f32| estimated_text_run_width_px(text, size, TextFace::Mono) - 16.0;
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
    let focus_label = application_focus_label(state);
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
        draw_text(
            label,
            x,
            text_y,
            lab_size,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        let lw = text_w(label, lab_size) + gap;
        draw_text(
            value,
            x + lw,
            text_y,
            val_size,
            *color,
            TextFace::Mono,
            text_runs,
        );
        x += lw + text_w(value, val_size) + seg_pad;
    }

    // Right cluster (right-to-left): version, rev, DRC.
    let version = "Datum EDA \u{2014} design pass";
    let short_rev: String = state.scene.source_revision.chars().take(6).collect();
    let findings = state.supervision.checks.finding_count;
    let drc = if findings > 0 {
        format!("DRC {}", findings)
    } else {
        String::new()
    };
    let revision_width = if short_rev.is_empty() {
        0.0
    } else {
        seg_pad + text_w("rev", lab_size) + gap + text_w(&short_rev, val_size)
    };
    let findings_width = if drc.is_empty() {
        0.0
    } else {
        seg_pad + text_w(&drc, val_size)
    };
    let mut rx = sb.x + sb.width - 13.0;
    // Decoration yields to actual document/check status at constrained widths.
    if rx - text_w(version, val_size) - revision_width - findings_width >= x {
        rx -= text_w(version, val_size);
        draw_text(
            version,
            rx,
            text_y,
            val_size,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }

    if !short_rev.is_empty() && rx - revision_width - findings_width >= x {
        let lw = text_w("rev", lab_size) + gap;
        rx -= seg_pad + lw + text_w(&short_rev, val_size);
        divider(panel_quads, rx - seg_pad * 0.5);
        draw_text(
            "rev",
            rx,
            text_y,
            lab_size,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &short_rev,
            rx + lw,
            text_y,
            val_size,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
    }

    if findings > 0 && rx - findings_width >= x {
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
    crate::hit_clipping::clip_content(
        panel_quads,
        text_runs,
        &mut Vec::new(),
        starts.0,
        starts.1,
        0,
        sb,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_status_segments_share_strip_bounds_at_constrained_extents() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.supervision.checks.finding_count = 17;
        for width in [1, 50, 320, 1280] {
            let layout = ShellLayout::for_surface(width, 800, 1.0, None);
            let mut quads = Vec::new();
            let mut text = Vec::new();
            render_status_bar(&state, &layout, &mut quads, &mut text);
            let bounds = layout.status_bar;
            for quad in &quads {
                for &(x, y) in &quad.points {
                    assert!(bounds.contains(x, y));
                }
            }
            for run in &text {
                let clip = run.clip_bounds.expect("status ancestor clip");
                assert!(clip.x >= bounds.x && clip.x + clip.width <= bounds.x + bounds.width);
                assert!(clip.y >= bounds.y && clip.y + clip.height <= bounds.y + bounds.height);
            }
            if width == 1280 {
                assert!(text.iter().any(|run| run.text == "DRC 17"));
                assert!(text.iter().any(|run| run.text == "focus"));
            }
        }
    }
}
