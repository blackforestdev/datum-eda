use datum_gui_protocol::{DockTab, ReviewWorkspaceState, TerminalTabState};

use super::{
    HitRegion, HitTarget, Quad, RectPx, TEXT_ACCENT, TEXT_MUTED, TEXT_PRIMARY, TextFace, TextRun,
    design_tokens, draw_text, estimated_text_run_width_px, truncate_text,
};

const TAB_GAP_PX: f32 = 8.0;
const PLUS_WIDTH_PX: f32 = 20.0;
const MIN_TAB_WIDTH_PX: f32 = 44.0;
const MAX_TAB_WIDTH_PX: f32 = 152.0;

pub(super) fn render_terminal_tab_strip(
    state: &ReviewWorkspaceState,
    strip: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let hint = "Ctrl+Shift+T new terminal   \u{00B7}   Ctrl+K palette";
    let hint_width = estimated_text_run_width_px(hint, 11.5, TextFace::Mono) - 16.0;
    let hint_x = strip.x + strip.width - 12.0 - hint_width;
    let tab_y = strip.y + 6.0;
    let tab_height = (strip.height - 6.0).max(1.0);
    let mut x = strip.x + 12.0;
    let tabs = &state.ui.terminal.tabs;

    if tabs.is_empty() {
        let label = state
            .ui
            .terminal
            .title
            .as_deref()
            .map(|title| truncate_text(title, 16))
            .unwrap_or_else(|| "terminal".to_string());
        let width = (estimated_text_run_width_px(&label, 12.5, TextFace::Ui) - 16.0
            + design_tokens::spacing::SP_04 * 2.0)
            .clamp(MIN_TAB_WIDTH_PX, MAX_TAB_WIDTH_PX);
        let rect = RectPx {
            x,
            y: tab_y,
            width,
            height: tab_height,
        };
        render_tab(label.as_str(), true, rect, panel_quads, text_runs);
        hit_regions.push(HitRegion {
            target: HitTarget::TerminalTab,
            rect,
        });
        x += width + TAB_GAP_PX;
    } else {
        let available = (hint_x - x - PLUS_WIDTH_PX - TAB_GAP_PX).max(MIN_TAB_WIDTH_PX);
        let tab_width =
            (available / tabs.len() as f32 - TAB_GAP_PX).clamp(MIN_TAB_WIDTH_PX, MAX_TAB_WIDTH_PX);
        for tab in tabs {
            let rect = RectPx {
                x,
                y: tab_y,
                width: tab_width,
                height: tab_height,
            };
            let label = top_tab_label(tab, tab_width);
            render_tab(
                label.as_str(),
                tab.active && matches!(state.ui.active_dock_tab, Some(DockTab::Terminal)),
                rect,
                panel_quads,
                text_runs,
            );
            hit_regions.push(HitRegion {
                target: HitTarget::TerminalSessionTab(tab.session_id.clone()),
                rect,
            });
            x += tab_width + TAB_GAP_PX;
        }
    }

    let plus = RectPx {
        x,
        y: tab_y,
        width: PLUS_WIDTH_PX,
        height: tab_height,
    };
    draw_text(
        "+",
        plus.x + 6.0,
        plus.y + 7.0,
        14.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalSessionNew,
        rect: plus,
    });
    draw_text(
        hint,
        hint_x,
        tab_y + 8.0,
        11.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

fn top_tab_label(tab: &TerminalTabState, tab_width: f32) -> String {
    let max_chars = (((tab_width - 16.0) / 7.0) as usize).max(1);
    truncate_text(&tab.label, max_chars)
}

fn render_tab(
    label: &str,
    active: bool,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    if active {
        panel_quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_01));
        panel_quads.push(Quad::from_rect(
            RectPx {
                height: 2.0,
                ..rect
            },
            TEXT_ACCENT,
        ));
    }
    draw_text(
        label,
        rect.x + design_tokens::spacing::SP_04,
        rect.y + 8.0,
        12.5,
        if active { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
}
