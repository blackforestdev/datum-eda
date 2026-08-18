use super::{
    HitRegion, HitTarget, PANEL_CARD_BG, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TEXT_ACCENT,
    TEXT_MUTED, TEXT_PRIMARY, TextFace, TextRun, draw_text, push_rect_border,
};
use datum_gui_protocol::{DockTab, ReviewWorkspaceState};
use datum_gui_viewport::terminal_screen_geometry;

const MENU_WIDTH_PX: f32 = 216.0;
const ITEM_HEIGHT_PX: f32 = 30.0;
const MENU_MARGIN_PX: f32 = 4.0;

pub(super) fn render_terminal_clipboard_menu(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    overlay_quads: &mut Vec<Quad>,
    overlay_text: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let Some(menu) = state.ui.terminal_clipboard_menu else {
        return;
    };
    if state.ui.active_dock_tab != Some(DockTab::Terminal) {
        return;
    }
    let screen: RectPx = terminal_screen_geometry(layout.bottom_strip.into())
        .screen
        .into();
    let menu_height = ITEM_HEIGHT_PX * 2.0;
    let x = menu.anchor_x.clamp(
        screen.x + MENU_MARGIN_PX,
        (screen.x + screen.width - MENU_WIDTH_PX - MENU_MARGIN_PX).max(screen.x + MENU_MARGIN_PX),
    );
    let y = menu.anchor_y.clamp(
        screen.y + MENU_MARGIN_PX,
        (screen.y + screen.height - menu_height - MENU_MARGIN_PX).max(screen.y + MENU_MARGIN_PX),
    );
    let card = RectPx {
        x,
        y,
        width: MENU_WIDTH_PX,
        height: menu_height,
    };
    overlay_quads.push(Quad::from_rect(card, PANEL_CARD_BG));
    push_rect_border(overlay_quads, card, PANEL_CARD_BORDER, 1.0);

    for (index, (label, shortcut, target)) in [
        ("COPY", "CTRL+SHIFT+C", HitTarget::TerminalClipboardCopy),
        ("PASTE", "CTRL+SHIFT+V", HitTarget::TerminalClipboardPaste),
    ]
    .into_iter()
    .enumerate()
    {
        let item = RectPx {
            x,
            y: y + index as f32 * ITEM_HEIGHT_PX,
            width: MENU_WIDTH_PX,
            height: ITEM_HEIGHT_PX,
        };
        if index == 1 {
            overlay_quads.push(Quad::from_rect(
                RectPx {
                    x: item.x + 1.0,
                    y: item.y,
                    width: item.width - 2.0,
                    height: 1.0,
                },
                PANEL_CARD_BORDER,
            ));
        }
        draw_text(
            label,
            item.x + 10.0,
            item.y + 7.0,
            12.5,
            TEXT_PRIMARY,
            TextFace::Ui,
            overlay_text,
        );
        draw_text(
            shortcut,
            item.x + 92.0,
            item.y + 7.0,
            11.5,
            if index == 0 { TEXT_MUTED } else { TEXT_ACCENT },
            TextFace::Mono,
            overlay_text,
        );
        hit_regions.push(HitRegion { target, rect: item });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_context_menu_is_clamped_and_exposes_copy_paste_actions() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_dock_tab = Some(DockTab::Terminal);
        state.ui.terminal_clipboard_menu = Some(datum_gui_protocol::TerminalClipboardMenuState {
            anchor_x: f32::MAX,
            anchor_y: f32::MAX,
        });
        let layout = ShellLayout::for_window(1280, 800, Some(260));
        let screen: RectPx = terminal_screen_geometry(layout.bottom_strip.into())
            .screen
            .into();
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();

        render_terminal_clipboard_menu(&state, &layout, &mut quads, &mut text, &mut hits);

        assert!(text.iter().any(|run| run.text == "COPY"));
        assert!(text.iter().any(|run| run.text == "PASTE"));
        for target in [
            HitTarget::TerminalClipboardCopy,
            HitTarget::TerminalClipboardPaste,
        ] {
            let rect = hits
                .iter()
                .find(|region| region.target == target)
                .expect("clipboard action hit target")
                .rect;
            assert!(rect.x >= screen.x && rect.y >= screen.y);
            assert!(rect.x + rect.width <= screen.x + screen.width);
            assert!(rect.y + rect.height <= screen.y + screen.height);
        }
    }
}
