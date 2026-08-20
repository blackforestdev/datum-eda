//! Renderer-owned projection of immutable TerminalCore snapshots.
//!
//! This is the production cell-rendering boundary for DTC-P23. It consumes no
//! parser state and mutates no terminal model: fixed cells, shaping clusters,
//! palette entries, decorations, selection, cursor, clipping, history, and
//! graphics placement all originate in one immutable `RenderSnapshot`.

use datum_gui_protocol::ReviewWorkspaceState;
use datum_gui_viewport::{TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX, TerminalScreenGeometry};
use datum_terminal_core::{
    CellAttribute, CellContent, CellStyle, CellWidth, Color, CursorShape, GraphicAnchorResolution,
    LogicalPoint, RenderPalette, RenderRow, RenderRowSource, RenderSnapshot, Selection,
    SelectionScope, UnderlineStyle,
};

use super::{
    HitRegion, HitTarget, PreparedTerminalGraphic, Quad, RectPx, TEXT_ACCENT, TEXT_MUTED, TextFace,
    TextRun, TextRunSpan, draw_rich_text, push_rect_border,
};
use crate::bottom_dock::terminal_block_elements::{
    render_terminal_block_elements_with_color, text_without_geometric_blocks,
};
use crate::bottom_dock::{TERMINAL_FONT_SIZE_PX, TERMINAL_SELECTION_BG, TERMINAL_SELECTION_FG};

const CURSOR_STROKE_PX: f32 = 1.0;
const CURSOR_HORIZONTAL_INSET_PX: f32 = 1.0;
const CURSOR_BAR_WIDTH_PX: f32 = 3.0;
const CURSOR_UNDERLINE_HEIGHT_PX: f32 = 3.0;

pub(super) fn prepare_terminal_graphics(
    snapshot: &RenderSnapshot,
    geometry: &TerminalScreenGeometry,
    scroll_offset: usize,
    graphics: &mut Vec<PreparedTerminalGraphic>,
) {
    let screen: RectPx = geometry.screen.into();
    let total_rows = snapshot.rows().len();
    let max_rows = usize::from(geometry.rows);
    let scroll = scroll_offset.min(total_rows.saturating_sub(max_rows));
    let first = total_rows.saturating_sub(max_rows + scroll);
    for graphic in snapshot.graphics() {
        let placement = graphic.placement();
        if placement.is_virtual() {
            continue;
        }
        let (absolute_row, column, visible_pixel_width, visible_pixel_height) =
            match graphic.resolution() {
                GraphicAnchorResolution::History { row, column } => (row, column, 0, 0),
                GraphicAnchorResolution::Screen {
                    row,
                    column,
                    visible_pixel_width,
                    visible_pixel_height,
                } => (
                    snapshot.history_row_count() + usize::from(row),
                    column,
                    visible_pixel_width,
                    visible_pixel_height,
                ),
                GraphicAnchorResolution::InactiveBuffer
                | GraphicAnchorResolution::Trimmed
                | GraphicAnchorResolution::Unknown => continue,
            };
        let Some(visible_row) = absolute_row.checked_sub(first) else {
            continue;
        };
        if visible_row >= max_rows || usize::from(column) >= usize::from(geometry.columns) {
            continue;
        }
        let cells = placement.cell_extent();
        let width = if cells.columns > 0 {
            cells.columns as f32 * TERMINAL_CELL_WIDTH_PX
        } else if visible_pixel_width > 0 {
            visible_pixel_width as f32
        } else {
            placement.width() as f32
        };
        let height = if cells.rows > 0 {
            cells.rows as f32 * TERMINAL_CELL_HEIGHT_PX
        } else if visible_pixel_height > 0 {
            visible_pixel_height as f32
        } else {
            placement.height() as f32
        };
        let offset = placement.pixel_offset();
        let rect = RectPx {
            x: screen.x + f32::from(column) * TERMINAL_CELL_WIDTH_PX + offset.x as f32,
            y: screen.y + visible_row as f32 * TERMINAL_CELL_HEIGHT_PX + offset.y as f32,
            width,
            height,
        };
        if let Some(clip) = nonempty_intersection(rect, screen) {
            graphics.push(PreparedTerminalGraphic {
                graphic: graphic.clone(),
                rect,
                clip,
            });
        }
    }
}

pub(super) fn render_terminal_core_snapshot(
    state: &ReviewWorkspaceState,
    snapshot: &RenderSnapshot,
    geometry: &TerminalScreenGeometry,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let screen: RectPx = geometry.screen.into();
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalScreen,
        rect: screen,
    });
    panel_quads.push(Quad::from_rect(
        screen,
        resolve_background(Color::Default, snapshot.palette()),
    ));

    let rows = snapshot.rows().collect::<Vec<_>>();
    let max_rows = usize::from(geometry.rows);
    let scroll = state
        .ui
        .terminal
        .scroll_offset
        .min(rows.len().saturating_sub(max_rows));
    let first = rows.len().saturating_sub(max_rows + scroll);
    for (visible_row, row) in rows.iter().skip(first).take(max_rows).enumerate() {
        let y = screen.y + visible_row as f32 * TERMINAL_CELL_HEIGHT_PX;
        render_row(
            row,
            snapshot,
            screen,
            y,
            usize::from(geometry.columns),
            panel_quads,
            text_runs,
        );
        if matches!(row.source(), RenderRowSource::Screen { row } if row == snapshot.cursor().position.row.get())
            && snapshot.cursor().visible
            && usize::from(snapshot.cursor().position.column.get()) < usize::from(geometry.columns)
        {
            render_cursor(
                snapshot,
                state.ui.focus.is_terminal(),
                screen.x,
                y,
                panel_quads,
            );
        }
    }
}

pub(super) fn render_row(
    row: &RenderRow,
    snapshot: &RenderSnapshot,
    screen: RectPx,
    y: f32,
    max_columns: usize,
    quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let mut logical_cluster = row.logical_start().cluster;
    for (column, cell) in row.cells().iter().take(max_columns).enumerate() {
        if matches!(cell.content, CellContent::Continuation { .. }) {
            continue;
        }
        let selected = snapshot.selection().is_some_and(|selection| {
            selection_contains(
                selection,
                LogicalPoint {
                    line: row.logical_start().line,
                    cluster: logical_cluster,
                },
            )
        });
        logical_cluster = logical_cluster.saturating_add(1);
        let width_cells = match &cell.content {
            CellContent::Cluster(cluster) if cluster.width() == CellWidth::Two => 2,
            _ => 1,
        };
        let cell_rect = RectPx {
            x: screen.x + column as f32 * TERMINAL_CELL_WIDTH_PX,
            y,
            width: width_cells as f32 * TERMINAL_CELL_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        };
        let (foreground, background) = style_colors(cell.style, snapshot.palette());
        if background != resolve_background(Color::Default, snapshot.palette()) {
            quads.push(Quad::from_rect(cell_rect, background));
        }
        if selected {
            quads.push(Quad::from_rect(cell_rect, TERMINAL_SELECTION_BG));
        }
        let decoration = match cell.style.underline_color {
            Color::Default => foreground,
            color => resolve_foreground(color, snapshot.palette()),
        };
        render_decorations(cell.style, cell_rect, decoration, selected, quads);
        let CellContent::Cluster(cluster) = &cell.content else {
            continue;
        };
        let display_color = if selected {
            TERMINAL_SELECTION_FG
        } else if cell.style.attributes.contains(CellAttribute::Hidden) {
            background
        } else {
            foreground
        };
        render_terminal_block_elements_with_color(
            cluster.text(),
            cell_rect.x,
            cell_rect.y,
            width_cells,
            display_color,
            quads,
        );
        let shaped = text_without_geometric_blocks(cluster.text());
        draw_rich_text(
            &shaped,
            vec![TextRunSpan {
                text: shaped.clone(),
                color: display_color,
                bold: cell.style.attributes.contains(CellAttribute::Bold),
                italic: cell.style.attributes.contains(CellAttribute::Italic),
            }],
            cell_rect.x,
            cell_rect.y,
            TERMINAL_FONT_SIZE_PX,
            display_color,
            TextFace::Terminal,
            text_runs,
        );
        if let Some(run) = text_runs.last_mut() {
            run.clip_bounds = Some(intersection(cell_rect, screen));
        }
    }
}

fn style_colors(style: CellStyle, palette: &RenderPalette) -> ([f32; 3], [f32; 3]) {
    let mut foreground = resolve_foreground(style.foreground, palette);
    let mut background = resolve_background(style.background, palette);
    if style.attributes.contains(CellAttribute::Inverse) {
        std::mem::swap(&mut foreground, &mut background);
    }
    if style.attributes.contains(CellAttribute::Faint) {
        foreground = foreground.map(|channel| channel * 0.66);
    }
    (foreground, background)
}

fn resolve_foreground(color: Color, palette: &RenderPalette) -> [f32; 3] {
    let rgb = match color {
        Color::Default => match palette.default_foreground() {
            Color::Default => return [0.90, 0.92, 0.94],
            resolved => return resolve_foreground(resolved, palette),
        },
        Color::Indexed(index) => match palette.color(index.get()) {
            Color::Default | Color::Indexed(_) => return ansi_fallback(index.get()),
            Color::Rgb(rgb) => rgb,
        },
        Color::Rgb(rgb) => rgb,
    };
    [
        f32::from(rgb.red) / 255.0,
        f32::from(rgb.green) / 255.0,
        f32::from(rgb.blue) / 255.0,
    ]
}

pub(super) fn resolve_background(color: Color, palette: &RenderPalette) -> [f32; 3] {
    match color {
        Color::Default => match palette.default_background() {
            Color::Default => [0.071, 0.082, 0.102],
            resolved => resolve_foreground(resolved, palette),
        },
        resolved => resolve_foreground(resolved, palette),
    }
}

fn ansi_fallback(index: u8) -> [f32; 3] {
    const ANSI: [[u8; 3]; 16] = [
        [64, 69, 77],
        [242, 82, 71],
        [115, 209, 122],
        [245, 199, 82],
        [107, 158, 242],
        [209, 128, 230],
        [97, 209, 224],
        [230, 235, 240],
        [122, 133, 148],
        [255, 107, 92],
        [148, 235, 143],
        [255, 219, 107],
        [133, 184, 255],
        [235, 158, 255],
        [128, 235, 245],
        [255, 255, 255],
    ];
    let value = if index < 16 {
        ANSI[usize::from(index)]
    } else if index < 232 {
        let cube = index - 16;
        let level = |component: u8| [0, 95, 135, 175, 215, 255][usize::from(component)];
        [level(cube / 36), level((cube % 36) / 6), level(cube % 6)]
    } else {
        let gray = 8 + (index - 232) * 10;
        [gray, gray, gray]
    };
    value.map(|channel| f32::from(channel) / 255.0)
}

fn render_decorations(
    style: CellStyle,
    rect: RectPx,
    foreground: [f32; 3],
    selected: bool,
    quads: &mut Vec<Quad>,
) {
    let color = if selected {
        TERMINAL_SELECTION_FG
    } else {
        foreground
    };
    match style.underline {
        UnderlineStyle::None => {}
        UnderlineStyle::Single => push_rule(quads, rect, rect.height - 2.0, 1.0, color),
        UnderlineStyle::Double => {
            push_rule(quads, rect, rect.height - 4.0, 1.0, color);
            push_rule(quads, rect, rect.height - 2.0, 1.0, color);
        }
        UnderlineStyle::Curly => {
            let mut x = 0.0;
            while x < rect.width {
                let offset = if ((x / 2.0) as usize).is_multiple_of(2) {
                    -1.0
                } else {
                    0.0
                };
                quads.push(Quad::from_rect(
                    RectPx {
                        x: rect.x + x,
                        y: rect.y + rect.height - 2.0 + offset,
                        width: 2.0_f32.min(rect.width - x),
                        height: 1.0,
                    },
                    color,
                ));
                x += 2.0;
            }
        }
        UnderlineStyle::Dotted => push_pattern(quads, rect, 1.0, 1.0, color),
        UnderlineStyle::Dashed => push_pattern(quads, rect, 3.0, 2.0, color),
    }
    if style.attributes.contains(CellAttribute::Strike) {
        push_rule(quads, rect, rect.height * 0.55, 1.0, color);
    }
    if style.attributes.contains(CellAttribute::Overline) {
        push_rule(quads, rect, 1.0, 1.0, color);
    }
}

fn push_rule(quads: &mut Vec<Quad>, rect: RectPx, offset: f32, height: f32, color: [f32; 3]) {
    quads.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y + offset,
            width: rect.width,
            height,
        },
        color,
    ));
}

fn push_pattern(quads: &mut Vec<Quad>, rect: RectPx, width: f32, gap: f32, color: [f32; 3]) {
    let mut x = 0.0;
    while x < rect.width {
        quads.push(Quad::from_rect(
            RectPx {
                x: rect.x + x,
                y: rect.y + rect.height - 2.0,
                width: width.min(rect.width - x),
                height: 1.0,
            },
            color,
        ));
        x += width + gap;
    }
}

fn selection_contains(selection: Selection, point: LogicalPoint) -> bool {
    let (start, end) = if selection.anchor() <= selection.focus() {
        (selection.anchor(), selection.focus())
    } else {
        (selection.focus(), selection.anchor())
    };
    match selection.scope() {
        SelectionScope::Block => {
            (start.line..=end.line).contains(&point.line)
                && (start.cluster.min(end.cluster)..=start.cluster.max(end.cluster))
                    .contains(&point.cluster)
        }
        _ => (start..=end).contains(&point),
    }
}

pub(super) fn render_cursor(
    snapshot: &RenderSnapshot,
    focused: bool,
    origin_x: f32,
    y: f32,
    quads: &mut Vec<Quad>,
) {
    let cursor = snapshot.cursor();
    let logical_x = origin_x
        + f32::from(cursor.position.column.get()) * TERMINAL_CELL_WIDTH_PX
        + CURSOR_HORIZONTAL_INSET_PX;
    let usable_width = TERMINAL_CELL_WIDTH_PX - 2.0 * CURSOR_HORIZONTAL_INSET_PX;
    let rect = match cursor.shape {
        CursorShape::Block => RectPx {
            x: logical_x,
            y,
            width: usable_width,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
        CursorShape::Underline => RectPx {
            x: logical_x,
            y: y + TERMINAL_CELL_HEIGHT_PX - CURSOR_UNDERLINE_HEIGHT_PX,
            width: usable_width,
            height: CURSOR_UNDERLINE_HEIGHT_PX,
        },
        CursorShape::Bar => RectPx {
            x: logical_x,
            y,
            width: CURSOR_BAR_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
    };
    let palette_cursor = resolve_foreground(snapshot.palette().cursor(), snapshot.palette());
    let color = if palette_cursor == [0.90, 0.92, 0.94] {
        if focused { TEXT_ACCENT } else { TEXT_MUTED }
    } else {
        palette_cursor
    };
    if focused {
        quads.push(Quad::from_rect(rect, color));
    } else {
        push_rect_border(quads, rect, color, CURSOR_STROKE_PX);
    }
}

fn intersection(first: RectPx, second: RectPx) -> RectPx {
    let x = first.x.max(second.x);
    let y = first.y.max(second.y);
    let right = (first.x + first.width).min(second.x + second.width);
    let bottom = (first.y + first.height).min(second.y + second.height);
    RectPx {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}

fn nonempty_intersection(first: RectPx, second: RectPx) -> Option<RectPx> {
    let intersection = intersection(first, second);
    (intersection.width > 0.0 && intersection.height > 0.0).then_some(intersection)
}

#[cfg(test)]
#[path = "terminal_core_render_tests.rs"]
mod tests;
