//! Shared terminal screen-cell geometry — T0-C02 truthful viewport geometry.
//!
//! `DATUM_NATIVE_TERMINAL_SPEC.md` §2.3/§7.1 (T0-C02) and decision 027
//! (FT-001/FT-008): the exact visible cell rectangle is the single authority
//! for BOTH the rows/columns the renderer draws AND the rows/columns the PTY
//! is resized to. The renderer (`gui-render` terminal lane) and the PTY resize
//! path (`gui-app` `resize_terminal_to_dock`) must derive their dimensions
//! from THIS one function; a second, duplicated budget is the defect that
//! produced the 76px chrome-estimate drift where the PTY was told more rows
//! than the lane could draw (bead `dat-pan-trace-terminal-pollution-0j0`).
//!
//! The screen rectangle is the PRIMARY space owner of the terminal lane: the
//! cell rectangle is computed first (integral rows/columns of the fixed cell
//! metric). Persistent terminal chrome consumes no cell rows.
//! Application summaries consume zero cell rows by construction: no summary
//! band exists in this geometry at all.
//!
//! All rectangles are in the same device-pixel space as the already-scaled
//! `ShellLayout` (`ShellLayout::for_surface` applies the HiDPI scale before
//! any consumer sees a rect), so renderer and PTY agree at every scale factor
//! by construction.

use crate::editor::ScreenRectPx;
use datum_gui_protocol::ScreenPointPx;

/// Fixed mono cell advance used by the terminal lane renderer (px).
pub const TERMINAL_CELL_WIDTH_PX: f32 = 7.9;
/// Fixed terminal line pitch used by the terminal lane renderer (px).
pub const TERMINAL_CELL_HEIGHT_PX: f32 = 16.0;
/// Minimum useful terminal height used by callers and tests.
pub const TERMINAL_MIN_ROWS: u16 = 4;

// Dock-content derivation from the bottom strip (must equal the bottom-dock
// solver's content rect: x+12, y+44, w-24, h-56).
const DOCK_CONTENT_INSET_X: f32 = 12.0;
const DOCK_CONTENT_TOP: f32 = 44.0;
const DOCK_CONTENT_BOTTOM: f32 = 12.0;
// Lane padding inside the dock content rect.
const LANE_PAD_X: f32 = 12.0;
const LANE_PAD_TOP: f32 = 8.0;
const LANE_PAD_BOTTOM: f32 = 8.0;
/// The solved terminal-lane geometry and exact visible cell rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalScreenGeometry {
    /// The dock content rectangle the lane lives in.
    pub content: ScreenRectPx,
    /// The exact visible cell rectangle: `columns x rows` whole cells. This
    /// rectangle is the terminal hit target and the PTY size authority.
    pub screen: ScreenRectPx,
    /// Columns the renderer draws and the PTY is resized to.
    pub columns: u16,
    /// Rows the renderer draws and the PTY is resized to.
    pub rows: u16,
}

impl TerminalScreenGeometry {
    /// The cell under a screen point as `(column, row)` within the visible
    /// screen rectangle, or `None` outside it. This is the coordinate seam
    /// the later text-selection phase anchors on (T0-C02: the screen hit
    /// target carries cell coordinates).
    pub fn cell_at(&self, x: f32, y: f32) -> Option<(u16, u16)> {
        if self.columns == 0 || self.rows == 0 {
            return None;
        }
        if !self.screen.contains(ScreenPointPx { x, y }) {
            return None;
        }
        let column = ((x - self.screen.x) / TERMINAL_CELL_WIDTH_PX) as u16;
        let row = ((y - self.screen.y) / TERMINAL_CELL_HEIGHT_PX) as u16;
        Some((column.min(self.columns - 1), row.min(self.rows - 1)))
    }
}

/// Solve the terminal lane geometry for the shell's bottom dock strip.
///
/// The cell rectangle consumes the full lane interior. The returned
/// `columns`/`rows` are the one authority for renderer and PTY alike.
pub fn terminal_screen_geometry(bottom_strip: ScreenRectPx) -> TerminalScreenGeometry {
    let content = ScreenRectPx {
        x: bottom_strip.x + DOCK_CONTENT_INSET_X,
        y: bottom_strip.y + DOCK_CONTENT_TOP,
        width: (bottom_strip.width - 2.0 * DOCK_CONTENT_INSET_X).max(0.0),
        height: (bottom_strip.height - DOCK_CONTENT_TOP - DOCK_CONTENT_BOTTOM).max(0.0),
    };
    let inner = ScreenRectPx {
        x: content.x + LANE_PAD_X,
        y: content.y + LANE_PAD_TOP,
        width: (content.width - 2.0 * LANE_PAD_X).max(0.0),
        height: (content.height - LANE_PAD_TOP - LANE_PAD_BOTTOM).max(0.0),
    };
    let columns = ((inner.width / TERMINAL_CELL_WIDTH_PX) as u16).max(1);
    let rows = (inner.height / TERMINAL_CELL_HEIGHT_PX) as u16;
    let rows = rows.max(1);
    let screen = ScreenRectPx {
        x: inner.x,
        y: inner.y,
        width: f32::from(columns) * TERMINAL_CELL_WIDTH_PX,
        height: f32::from(rows) * TERMINAL_CELL_HEIGHT_PX,
    };
    TerminalScreenGeometry {
        content,
        screen,
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(width: f32, height: f32) -> ScreenRectPx {
        ScreenRectPx {
            x: 0.0,
            y: 800.0 - height,
            width,
            height,
        }
    }

    fn assert_screen_within_content(geometry: &TerminalScreenGeometry) {
        let screen = geometry.screen;
        let content = geometry.content;
        assert!(screen.x >= content.x, "screen left inside content");
        assert!(screen.y >= content.y, "screen top inside content");
        assert!(
            screen.x + screen.width <= content.x + content.width + 0.001,
            "screen right must not overflow content ({} > {})",
            screen.x + screen.width,
            content.x + content.width
        );
        assert!(
            screen.y + screen.height <= content.y + content.height + 0.001,
            "screen bottom must not overflow content ({} > {})",
            screen.y + screen.height,
            content.y + content.height
        );
    }

    #[test]
    fn default_dock_has_no_header_and_uses_every_affordable_row() {
        let geometry = terminal_screen_geometry(strip(1280.0, 220.0));
        assert_eq!(
            geometry.rows, 9,
            "removing all persistent header chrome returns every affordable row"
        );
        assert!(geometry.columns >= 80, "a 1280px dock affords 80+ columns");
        assert_screen_within_content(&geometry);
    }

    #[test]
    fn screen_rect_is_exactly_columns_by_rows_whole_cells() {
        for height in [90.0_f32, 140.0, 220.0, 320.0, 480.0] {
            let geometry = terminal_screen_geometry(strip(1280.0, height));
            assert_eq!(
                geometry.screen.width,
                f32::from(geometry.columns) * TERMINAL_CELL_WIDTH_PX
            );
            assert_eq!(
                geometry.screen.height,
                f32::from(geometry.rows) * TERMINAL_CELL_HEIGHT_PX
            );
            assert_screen_within_content(&geometry);
        }
    }

    #[test]
    fn short_docks_give_every_affordable_row_to_the_screen() {
        let geometry = terminal_screen_geometry(strip(1280.0, 150.0));
        assert_eq!(geometry.rows, TERMINAL_MIN_ROWS);
        assert_screen_within_content(&geometry);
        // The lane interior itself holds under MIN_ROWS cells at 120px, and
        // every remaining row still belongs to the screen.
        let geometry = terminal_screen_geometry(strip(1280.0, 120.0));
        assert_eq!(geometry.rows, 3, "48px interior affords exactly 3 cells");
        assert_screen_within_content(&geometry);
    }

    #[test]
    fn hidpi_scaled_strip_yields_scaled_rects_and_consistent_grid() {
        // The strip arrives already scaled (ShellLayout::for_surface); the
        // geometry must stay self-consistent in that device-pixel space.
        let geometry = terminal_screen_geometry(ScreenRectPx {
            x: 0.0,
            y: 1160.0,
            width: 2560.0,
            height: 440.0,
        });
        assert!(geometry.rows >= TERMINAL_MIN_ROWS);
        assert_eq!(
            geometry.screen.width,
            f32::from(geometry.columns) * TERMINAL_CELL_WIDTH_PX
        );
        assert_screen_within_content(&geometry);
    }

    #[test]
    fn every_affordable_row_goes_to_the_screen_with_no_summary_band() {
        // T0-C04 regression boundary (DATUM_NATIVE_TERMINAL_SPEC.md §7.1):
        // application summaries/telemetry consume zero cell rows. Across dock
        // sizes and HiDPI scales, the space between the surviving chrome bands
        // and the lane bottom belongs entirely to the screen — the leftover
        // below the screen is less than one cell (no row is withheld for any
        // application band), and the screen top sits exactly under the kept
        // chrome (nothing else may displace shell cells).
        for width in [800.0_f32, 1280.0, 2560.0] {
            for height in [90.0_f32, 120.0, 150.0, 220.0, 320.0, 440.0, 600.0] {
                let geometry = terminal_screen_geometry(strip(width, height));
                let inner_x = geometry.content.x + LANE_PAD_X;
                let inner_y = geometry.content.y + LANE_PAD_TOP;
                let inner_height =
                    (geometry.content.height - LANE_PAD_TOP - LANE_PAD_BOTTOM).max(0.0);
                assert_eq!(
                    geometry.screen.y, inner_y,
                    "{width}x{height}: screen must start at the lane interior"
                );
                assert_eq!(
                    geometry.screen.x, inner_x,
                    "{width}x{height}: no band may indent the screen"
                );
                let leftover =
                    inner_y + inner_height - (geometry.screen.y + geometry.screen.height);
                assert!(
                    leftover < TERMINAL_CELL_HEIGHT_PX,
                    "{width}x{height}: {leftover}px of lane space is withheld from \
                     the grid — a full cell row was displaced"
                );
            }
        }
    }

    #[test]
    fn cell_at_maps_screen_points_to_cells_and_rejects_chrome() {
        let geometry = terminal_screen_geometry(strip(1280.0, 220.0));
        let screen = geometry.screen;
        assert_eq!(geometry.cell_at(screen.x, screen.y), Some((0, 0)));
        assert_eq!(
            geometry.cell_at(
                screen.x + 2.5 * TERMINAL_CELL_WIDTH_PX,
                screen.y + 1.5 * TERMINAL_CELL_HEIGHT_PX
            ),
            Some((2, 1))
        );
        let last = geometry.cell_at(
            screen.x + screen.width - 0.5,
            screen.y + screen.height - 0.5,
        );
        assert_eq!(last, Some((geometry.columns - 1, geometry.rows - 1)));
        // Lane padding above the screen is not a cell.
        assert_eq!(
            geometry.cell_at(screen.x + 4.0, geometry.content.y + 2.0),
            None
        );
        // Outside the lane entirely.
        assert_eq!(geometry.cell_at(-10.0, -10.0), None);
    }
}
