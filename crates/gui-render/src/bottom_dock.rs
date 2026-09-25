use datum_gui_protocol::{DockTab, ReviewWorkspaceState};
use datum_gui_viewport::{
    TERMINAL_CELL_WIDTH_PX, TerminalScreenGeometry, terminal_screen_geometry_with_chrome_scale,
    terminal_split_dividers, terminal_split_geometries,
};

use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TextRun,
    push_rect_border,
};
use crate::design_tokens;
use crate::terminal_tab_strip::render_terminal_tab_strip;

#[path = "terminal_block_elements.rs"]
pub(super) mod terminal_block_elements;

/// Keep JetBrains Mono's ink comfortably inside the cell, then use the shaping
/// engine's explicit letter spacing to preserve the exact governed 7.9 px
/// advance. Enlarging the raw 0.6-em glyph advance to the full cell made
/// adjacent contrasting glyphs and the cursor visually fuse at raster scale.
pub(super) const TERMINAL_FONT_SIZE_PX: f32 = datum_gui_viewport::TERMINAL_FONT_SIZE_PX;
pub(super) const TERMINAL_LETTER_SPACING_EM: f32 =
    (TERMINAL_CELL_WIDTH_PX - TERMINAL_FONT_SIZE_PX * 0.6) / TERMINAL_FONT_SIZE_PX;
pub(super) const TERMINAL_SELECTION_BG: [f32; 3] = design_tokens::chrome::TERMINAL_SELECTION;
pub(super) const TERMINAL_SELECTION_FG: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
pub(super) const TERMINAL_SEARCH_BG: [f32; 3] = [0.34, 0.25, 0.08];
pub(super) const TERMINAL_SEARCH_ALL_BG: [f32; 3] = [0.23, 0.19, 0.08];

pub(super) struct TerminalRenderInput<'a> {
    pub(super) panes: &'a [crate::TerminalPaneRenderState<'a>],
    pub(super) cache: Option<&'a mut crate::TerminalRenderCache>,
}

pub(super) fn render_bottom_tabs(
    state: &ReviewWorkspaceState,
    terminal_render: Option<TerminalRenderInput<'_>>,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let strip = layout.bottom_strip;
    // Single top-edge hairline on the dock strip.
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    let scale = crate::text_presentation::chrome_scale::Scale::for_layout(layout);
    let starts = (panel_quads.len(), text_runs.len(), hit_regions.len());
    render_terminal_tab_strip(
        state,
        scale.logical_layout(layout.clone()).bottom_strip,
        scale.factor(),
        panel_quads,
        text_runs,
        hit_regions,
    );
    scale.quads(&mut panel_quads[starts.0..]);
    scale.text_geometry(&mut text_runs[starts.1..]);
    scale.hits(&mut hit_regions[starts.2..]);

    let Some(active_tab) = state.ui.active_dock_tab else {
        return;
    };
    let root_geometry = terminal_screen_geometry_with_chrome_scale(
        strip.into(),
        state.ui.terminal.font_scale_millis,
        scale.factor(),
    );
    let handle_rect = RectPx {
        height: 6.0,
        ..strip
    };
    panel_quads.push(Quad::from_rect(handle_rect, PANEL_CARD_BORDER));
    hit_regions.push(HitRegion {
        target: HitTarget::DockResizeHandle,
        rect: handle_rect,
    });
    let content_rect: RectPx = root_geometry.content.into();
    panel_quads.push(Quad::from_rect(content_rect, PANEL_BG));
    push_rect_border(panel_quads, content_rect, PANEL_CARD_BORDER, 1.0);
    match active_tab {
        DockTab::Terminal => {
            // T0-C02: the exact visible cell rectangle is the space and
            // row/column authority — the same shared geometry the PTY resize
            // path uses (datum_gui_viewport::terminal_screen_geometry), so the
            // rows drawn here always equal the rows the PTY was told.
            if let Some(terminal_render) = terminal_render {
                if let Some(cache) = terminal_render.cache {
                    let active_layout =
                        state
                            .ui
                            .terminal
                            .active_tab_id
                            .as_deref()
                            .and_then(|tab_id| {
                                state
                                    .ui
                                    .terminal
                                    .tab_layouts
                                    .iter()
                                    .find(|tab| tab.tab_id == tab_id)
                            });
                    let geometries = active_layout
                        .map(|tab| terminal_split_geometries(root_geometry, tab))
                        .unwrap_or_default();
                    cache.retain_sessions(
                        terminal_render
                            .panes
                            .iter()
                            .map(|pane| pane.session_id.as_str()),
                    );
                    for pane in terminal_render.panes {
                        let geometry = geometries
                            .iter()
                            .find(|candidate| candidate.session_id == pane.session_id)
                            .map(|candidate| candidate.geometry)
                            .unwrap_or(root_geometry);
                        let target = if pane.focused {
                            HitTarget::TerminalScreen
                        } else {
                            HitTarget::TerminalPaneScreen(pane.session_id.clone())
                        };
                        cache.render_pane(
                            &pane.session_id,
                            pane.lane,
                            pane.focused && state.ui.focus.is_terminal(),
                            &pane.snapshot,
                            &pane.damage,
                            &geometry,
                            target,
                            (panel_quads, text_runs, hit_regions),
                        );
                    }
                    if let Some(active_layout) = active_layout {
                        render_terminal_split_dividers(
                            root_geometry,
                            active_layout,
                            panel_quads,
                            hit_regions,
                        );
                    }
                } else if let Some(pane) = terminal_render.panes.iter().find(|pane| pane.focused) {
                    crate::terminal_core_render::render_terminal_core_snapshot(
                        state,
                        &pane.snapshot,
                        &root_geometry,
                        panel_quads,
                        text_runs,
                        hit_regions,
                    );
                }
            } else {
                render_empty_terminal_surface(&root_geometry, panel_quads, hit_regions);
            }
        }
    }
}

fn render_terminal_split_dividers(
    root_geometry: TerminalScreenGeometry,
    active_layout: &datum_gui_protocol::TerminalTabLayout,
    panel_quads: &mut Vec<Quad>,
    hit_regions: &mut Vec<HitRegion>,
) {
    for divider in terminal_split_dividers(root_geometry, active_layout) {
        let gutter: RectPx = divider.gutter.into();
        panel_quads.push(Quad::from_rect(gutter, PANEL_CARD_BORDER));
        hit_regions.push(HitRegion {
            target: HitTarget::TerminalSplitDivider(divider.path),
            rect: gutter,
        });
    }
}

fn render_empty_terminal_surface(
    geometry: &TerminalScreenGeometry,
    panel_quads: &mut Vec<Quad>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let screen: RectPx = geometry.screen.into();
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalScreen,
        rect: screen,
    });
    panel_quads.push(Quad::from_rect(screen, PANEL_BG));
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_dock_has_no_content_or_resize_hits() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_dock_tab = None;
        let shell = ShellLayout::for_window(1280, 800, None);
        let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
        render_bottom_tabs(&state, None, &shell, &mut quads, &mut text, &mut hits);
        assert!(!hits.iter().any(|hit| matches!(
            hit.target,
            HitTarget::DockResizeHandle | HitTarget::TerminalScreen
        )));
    }

    #[test]
    fn rendered_dock_content_uses_shared_terminal_geometry() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_dock_tab = Some(DockTab::Terminal);
        for width in [1, 24, 1280] {
            for dock_height in [120, 220, 320] {
                for scale in [1.0, 1.5, 2.0] {
                    for font_scale in [750, 1000, 2000] {
                        state.ui.terminal.font_scale_millis = font_scale;
                        let shell = ShellLayout::for_surface(width, 800, scale, Some(dock_height));
                        let geometry = terminal_screen_geometry_with_chrome_scale(
                            shell.bottom_strip.into(),
                            font_scale,
                            scale,
                        );
                        let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
                        render_bottom_tabs(&state, None, &shell, &mut quads, &mut text, &mut hits);
                        let expected = Quad::from_rect(geometry.content.into(), PANEL_BG);
                        assert!(
                            quads.iter().any(|quad| quad.points == expected.points
                                && quad.color == expected.color),
                            "painted content disagrees with shared geometry: width={width} dock={dock_height} scale={scale} font={font_scale}"
                        );
                        assert!(
                            hits.iter()
                                .any(|hit| hit.target == HitTarget::TerminalScreen
                                    && hit.rect == geometry.screen.into())
                        );
                        let handle = hits
                            .iter()
                            .find(|hit| hit.target == HitTarget::DockResizeHandle)
                            .unwrap()
                            .rect;
                        assert_eq!(
                            handle,
                            RectPx {
                                height: 6.0,
                                ..shell.bottom_strip
                            }
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn split_divider_renders_one_path_stable_resize_target_in_its_gutter() {
        use datum_gui_protocol::{TerminalSplitDirection, TerminalSplitNode, TerminalTabLayout};

        let shell = ShellLayout::for_window(1280, 800, Some(320));
        let root = datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into());
        let tab = TerminalTabLayout {
            tab_id: "split-tab".to_string(),
            focused_session_id: "right".to_string(),
            root: TerminalSplitNode::Split {
                direction: TerminalSplitDirection::SideBySide,
                ratio_millis: 500,
                first: Box::new(TerminalSplitNode::session("left")),
                second: Box::new(TerminalSplitNode::session("right")),
            },
        };
        let mut quads = Vec::new();
        let mut hits = Vec::new();
        render_terminal_split_dividers(root, &tab, &mut quads, &mut hits);

        assert_eq!(quads.len(), 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].target, HitTarget::TerminalSplitDivider(Vec::new()));
        assert!((hits[0].rect.width - datum_gui_viewport::TERMINAL_SPLIT_GUTTER_PX).abs() < 0.001);
        assert_eq!(hits[0].rect.height, root.screen.height);
    }
}
