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
//! metric) and chrome fits around it, collapsing — sessions row first, then
//! the header — before the screen drops below [`TERMINAL_MIN_ROWS`].
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
/// Chrome collapses before the screen drops below this many rows.
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
/// Header chrome band: lane title line + session status/meta line.
const HEADER_BAND_PX: f32 = 34.0;
/// Sessions chrome band: session controls + tabs (+ inline rename hint).
const SESSIONS_BAND_PX: f32 = 18.0;

/// The solved terminal-lane geometry: the exact visible cell rectangle plus
/// the chrome bands that survived around it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalScreenGeometry {
    /// The dock content rectangle the lane lives in.
    pub content: ScreenRectPx,
    /// Header chrome band (title + session meta), when it fits.
    pub header: Option<ScreenRectPx>,
    /// Sessions chrome band (controls + tabs), when it fits.
    pub sessions_row: Option<ScreenRectPx>,
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
/// Screen-first space budget: the cell rectangle is computed against the full
/// lane interior and chrome only keeps the bands the remaining space affords
/// without pushing the screen below [`TERMINAL_MIN_ROWS`] (sessions row is
/// dropped first, then the header). The returned `columns`/`rows` are the one
/// authority for renderer and PTY alike.
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
    let rows_with = |chrome: f32| ((inner.height - chrome).max(0.0) / TERMINAL_CELL_HEIGHT_PX) as u16;
    let min_rows_px = f32::from(TERMINAL_MIN_ROWS) * TERMINAL_CELL_HEIGHT_PX;
    let (keep_header, keep_sessions) =
        if inner.height - (HEADER_BAND_PX + SESSIONS_BAND_PX) >= min_rows_px {
            (true, true)
        } else if inner.height - HEADER_BAND_PX >= min_rows_px {
            (true, false)
        } else {
            (false, false)
        };
    let mut chrome_bottom = inner.y;
    let header = keep_header.then(|| {
        let band = ScreenRectPx {
            x: inner.x,
            y: chrome_bottom,
            width: inner.width,
            height: HEADER_BAND_PX,
        };
        chrome_bottom += HEADER_BAND_PX;
        band
    });
    let sessions_row = keep_sessions.then(|| {
        let band = ScreenRectPx {
            x: inner.x,
            y: chrome_bottom,
            width: inner.width,
            height: SESSIONS_BAND_PX,
        };
        chrome_bottom += SESSIONS_BAND_PX;
        band
    });
    let rows = rows_with(chrome_bottom - inner.y).max(1);
    let screen = ScreenRectPx {
        x: inner.x,
        y: chrome_bottom,
        width: f32::from(columns) * TERMINAL_CELL_WIDTH_PX,
        height: f32::from(rows) * TERMINAL_CELL_HEIGHT_PX,
    };
    TerminalScreenGeometry {
        content,
        header,
        sessions_row,
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
    fn default_dock_keeps_full_chrome_and_at_least_min_rows() {
        let geometry = terminal_screen_geometry(strip(1280.0, 220.0));
        assert!(geometry.header.is_some(), "default dock keeps the header");
        assert!(
            geometry.sessions_row.is_some(),
            "default dock keeps the sessions row"
        );
        assert!(geometry.rows >= TERMINAL_MIN_ROWS);
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
    fn chrome_collapses_before_screen_drops_below_min_rows() {
        // 150px dock: content 94px, inner 78px — full chrome (52px) would
        // leave 1 row, so the sessions row must be surrendered first.
        let geometry = terminal_screen_geometry(strip(1280.0, 150.0));
        assert!(geometry.sessions_row.is_none(), "sessions row collapses first");
        assert!(geometry.header.is_none() || geometry.rows >= TERMINAL_MIN_ROWS);
        assert!(geometry.rows >= TERMINAL_MIN_ROWS);
        assert_screen_within_content(&geometry);
        // 120px dock: the lane interior itself holds under MIN_ROWS cells, so
        // ALL chrome is surrendered and every remaining row goes to the screen.
        let geometry = terminal_screen_geometry(strip(1280.0, 120.0));
        assert!(geometry.header.is_none(), "header collapses when space demands");
        assert!(geometry.sessions_row.is_none());
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
        // Chrome above the screen is not a cell.
        let header = geometry.header.expect("default dock has a header");
        assert_eq!(geometry.cell_at(header.x + 4.0, header.y + 4.0), None);
        // Outside the lane entirely.
        assert_eq!(geometry.cell_at(-10.0, -10.0), None);
    }
}
