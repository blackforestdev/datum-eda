use datum_gui_protocol::{DockTab, ReviewWorkspaceState};
use datum_gui_viewport::{
    TERMINAL_CELL_WIDTH_PX, TerminalScreenGeometry, terminal_screen_geometry,
};

use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TextRun,
    push_rect_border,
};
use crate::design_tokens;
use crate::terminal_tab_strip::render_terminal_tab_strip;
use taffy::prelude::*;

#[path = "terminal_block_elements.rs"]
pub(super) mod terminal_block_elements;

/// Keep JetBrains Mono's ink comfortably inside the cell, then use the shaping
/// engine's explicit letter spacing to preserve the exact governed 7.9 px
/// advance. Enlarging the raw 0.6-em glyph advance to the full cell made
/// adjacent contrasting glyphs and the cursor visually fuse at raster scale.
pub(super) const TERMINAL_FONT_SIZE_PX: f32 = 12.0;
pub(super) const TERMINAL_LETTER_SPACING_EM: f32 =
    (TERMINAL_CELL_WIDTH_PX - TERMINAL_FONT_SIZE_PX * 0.6) / TERMINAL_FONT_SIZE_PX;
pub(super) const TERMINAL_SELECTION_BG: [f32; 3] = design_tokens::chrome::TERMINAL_SELECTION;
pub(super) const TERMINAL_SELECTION_FG: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;

#[derive(Debug, Clone, Copy)]
struct BottomDockLayout {
    // Retained for the solver contract test; the seated tab is now sized to its
    // measured label directly in render_bottom_tabs.
    #[allow(dead_code)]
    terminal_tab: RectPx,
    handle: RectPx,
    content: RectPx,
}

pub(super) struct TerminalRenderInput<'a> {
    pub(super) snapshot: &'a datum_terminal_core::RenderSnapshot,
    pub(super) damage: &'a [datum_terminal_core::Damage],
    pub(super) cache: Option<&'a mut crate::TerminalRenderCache>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomDockNode {
    Terminal,
}

fn solve_bottom_dock_layout_with_taffy(layout: &ShellLayout) -> Option<BottomDockLayout> {
    let strip = layout.bottom_strip;
    let tab_height = (strip.height - 16.0).max(1.0);
    let tab_width = 120.0_f32;
    let tab_gap = 8.0_f32;
    let row_x = strip.x + 12.0;
    let row_y = strip.y + 8.0;

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_tab = |kind: BottomDockNode| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(tab_width),
                    height: length(tab_height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };
    add_tab(BottomDockNode::Terminal)?;

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                gap: Size {
                    width: length(tab_gap),
                    height: length(0.0),
                },
                size: Size {
                    width: length((strip.width - 24.0).max(1.0)),
                    height: length(tab_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;

    let rect_for = |kind: BottomDockNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let solved = taffy.layout(node).ok()?;
        Some(RectPx {
            x: row_x + solved.location.x,
            y: row_y + solved.location.y,
            width: solved.size.width,
            height: solved.size.height,
        })
    };

    Some(BottomDockLayout {
        terminal_tab: rect_for(BottomDockNode::Terminal)?,
        handle: RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 6.0,
        },
        content: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 44.0,
            width: (strip.width - 24.0).max(1.0),
            height: (strip.height - 56.0).max(0.0),
        },
    })
}

fn fallback_bottom_dock_layout(layout: &ShellLayout) -> BottomDockLayout {
    let strip = layout.bottom_strip;
    BottomDockLayout {
        terminal_tab: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 8.0,
            width: 120.0,
            height: strip.height - 16.0,
        },
        handle: RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 6.0,
        },
        content: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 44.0,
            width: strip.width - 24.0,
            height: (strip.height - 56.0).max(0.0),
        },
    }
}

pub(super) fn render_bottom_tabs(
    state: &ReviewWorkspaceState,
    terminal_render: Option<TerminalRenderInput<'_>>,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let dock_layout = solve_bottom_dock_layout_with_taffy(layout)
        .unwrap_or_else(|| fallback_bottom_dock_layout(layout));
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
    render_terminal_tab_strip(state, strip, panel_quads, text_runs, hit_regions);

    let Some(active_tab) = state.ui.active_dock_tab else {
        return;
    };
    let handle_rect = dock_layout.handle;
    panel_quads.push(Quad::from_rect(handle_rect, PANEL_CARD_BORDER));
    hit_regions.push(HitRegion {
        target: HitTarget::DockResizeHandle,
        rect: handle_rect,
    });
    let content_rect = dock_layout.content;
    panel_quads.push(Quad::from_rect(content_rect, PANEL_BG));
    push_rect_border(panel_quads, content_rect, PANEL_CARD_BORDER, 1.0);
    match active_tab {
        DockTab::Terminal => {
            // T0-C02: the exact visible cell rectangle is the space and
            // row/column authority — the same shared geometry the PTY resize
            // path uses (datum_gui_viewport::terminal_screen_geometry), so the
            // rows drawn here always equal the rows the PTY was told.
            let geometry = terminal_screen_geometry(layout.bottom_strip.into());
            if let Some(terminal_render) = terminal_render {
                if let Some(cache) = terminal_render.cache {
                    cache.render(
                        state,
                        terminal_render.snapshot,
                        terminal_render.damage,
                        &geometry,
                        (panel_quads, text_runs, hit_regions),
                    );
                } else {
                    crate::terminal_core_render::render_terminal_core_snapshot(
                        state,
                        terminal_render.snapshot,
                        &geometry,
                        panel_quads,
                        text_runs,
                        hit_regions,
                    );
                }
            } else {
                render_empty_terminal_surface(&geometry, panel_quads, hit_regions);
            }
        }
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
    fn bottom_dock_tabs_are_solver_backed_and_non_overlapping() {
        let shell = ShellLayout::for_window(1280, 800, Some(220));
        let layout =
            solve_bottom_dock_layout_with_taffy(&shell).expect("bottom dock layout should solve");

        assert!(layout.content.y > layout.terminal_tab.y);
        assert!(layout.content.x >= shell.bottom_strip.x);
        assert!(
            layout.content.x + layout.content.width
                <= shell.bottom_strip.x + shell.bottom_strip.width
        );
    }

    #[test]
    fn shared_terminal_geometry_agrees_with_dock_content_rect() {
        // T0-C02 guard: the shared geometry derives the dock content rect from
        // the bottom strip with the same constants the dock solver uses; if
        // either side drifts, renderer and PTY budgets diverge again.
        for dock_height in [120, 220, 320] {
            let shell = ShellLayout::for_window(1280, 800, Some(dock_height));
            let solved = solve_bottom_dock_layout_with_taffy(&shell)
                .expect("bottom dock layout should solve");
            let geometry = terminal_screen_geometry(shell.bottom_strip.into());
            let content: RectPx = geometry.content.into();
            assert_eq!(content, solved.content, "dock {dock_height}px");
        }
    }
}
