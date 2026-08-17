use datum_gui_protocol::{DockTab, ReviewWorkspaceState, TerminalTabState};

use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BORDER, Quad, RectPx, TEXT_ACCENT, TEXT_MUTED,
    TEXT_PRIMARY, TextFace, TextRun, design_tokens, draw_text, estimated_text_run_width_px,
    push_rect_border, truncate_text,
};
use crate::terminal_session_chrome::{
    render_terminal_lifecycle_controls, terminal_lifecycle_controls_width,
};

pub(super) const TAB_GAP_PX: f32 = 8.0;
const CLOSE_WIDTH_PX: f32 = 28.0;
const PLUS_WIDTH_PX: f32 = 20.0;
const MIN_TAB_WIDTH_PX: f32 = 56.0;
const MAX_TAB_WIDTH_PX: f32 = 152.0;

pub(super) fn render_terminal_tab_strip(
    state: &ReviewWorkspaceState,
    strip: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let routine_hint = "Ctrl+Shift+T new terminal   \u{00B7}   Ctrl+K palette";
    let lifecycle_label = state
        .ui
        .terminal
        .application_shutdown_blocked
        .as_deref()
        .or_else(|| {
            (state.ui.terminal.status != "running").then_some(state.ui.terminal.status.as_str())
        });
    let trailing_width = lifecycle_label.map_or_else(
        || estimated_text_run_width_px(routine_hint, 11.5, TextFace::Mono) - 16.0,
        |_| (strip.width * 0.46).clamp(320.0, 620.0),
    );
    let trailing_x = strip.x + strip.width - 12.0 - trailing_width;
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
        let available = (trailing_x - x - PLUS_WIDTH_PX - TAB_GAP_PX).max(MIN_TAB_WIDTH_PX);
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
                rect: RectPx {
                    width: rect.width - CLOSE_WIDTH_PX,
                    ..rect
                },
            });
            let close = RectPx {
                x: rect.x + rect.width - CLOSE_WIDTH_PX,
                width: CLOSE_WIDTH_PX,
                ..rect
            };
            draw_text(
                "×",
                close.x + 7.0,
                close.y + 6.0,
                15.5,
                if tab.active { TEXT_PRIMARY } else { TEXT_MUTED },
                TextFace::Ui,
                text_runs,
            );
            hit_regions.push(HitRegion {
                target: HitTarget::TerminalSessionClose(tab.session_id.clone()),
                rect: close,
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
    if let Some(label) = lifecycle_label {
        let chrome = RectPx {
            x: trailing_x,
            y: tab_y + 2.0,
            width: trailing_width,
            height: 22.0,
        };
        panel_quads.push(Quad::from_rect(chrome, PANEL_BG));
        push_rect_border(panel_quads, chrome, PANEL_CARD_BORDER, 1.0);
        let controls_width = terminal_lifecycle_controls_width(
            &state.ui.terminal.status,
            state.ui.terminal.application_shutdown_blocked.as_deref(),
        );
        let label_columns = (((chrome.width - controls_width - 20.0) / 7.0) as usize).max(1);
        let label = truncate_text(label, label_columns);
        draw_text(
            &label,
            chrome.x + 8.0,
            chrome.y + 4.0,
            11.0,
            TEXT_PRIMARY,
            TextFace::Mono,
            text_runs,
        );
        render_terminal_lifecycle_controls(
            chrome,
            chrome.y + 4.0,
            &state.ui.terminal.status,
            state.ui.terminal.application_shutdown_blocked.as_deref(),
            text_runs,
            hit_regions,
        );
    } else {
        draw_text(
            routine_hint,
            trailing_x,
            tab_y + 8.0,
            11.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
}

fn top_tab_label(tab: &TerminalTabState, tab_width: f32) -> String {
    let max_chars = (((tab_width - CLOSE_WIDTH_PX - 16.0) / 7.0) as usize).max(1);
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
