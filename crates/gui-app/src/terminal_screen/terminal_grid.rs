use datum_gui_protocol::{TerminalLaneState, TerminalStyledLine};

use super::terminal_style::{
    clear_style_range, shift_style_spans_for_delete, shift_style_spans_for_insert,
    sync_styled_lines,
};
use super::{
    clear_from_cursor_to_end, clear_row_to_cursor_with_style, clear_start_to_cursor, ensure_row_at,
    erase_chars_at_with_style, remove_char_at_with_style, row_mut_at, truncate_row_at_with_style,
};

pub(super) fn apply_insert_chars(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: Option<usize>,
) {
    if let Some(columns) = columns {
        insert_blank_chars_at_with_style_bounded(state, row_index, column, count, columns);
    } else {
        insert_blank_chars_at_with_style(state, row_index, column, count);
    }
}

pub(super) fn apply_delete_chars(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: Option<usize>,
) {
    if let Some(columns) = columns {
        delete_chars_at_with_style_bounded(state, row_index, column, count, columns);
    } else {
        for _ in 0..count {
            remove_char_at_with_style(state, row_index, column);
        }
    }
}

pub(super) fn apply_erase_chars(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: Option<usize>,
) {
    if let Some(columns) = columns {
        erase_chars_at_with_style_bounded(state, row_index, column, count, columns);
    } else {
        erase_chars_at_with_style(state, row_index, column, count);
    }
}

pub(super) fn apply_erase_line(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    mode: usize,
    columns: Option<usize>,
) {
    if let Some(columns) = columns {
        apply_erase_line_bounded(state, row_index, column, mode, columns);
        return;
    }
    match mode {
        0 => truncate_row_at_with_style(state, row_index, column),
        1 => clear_row_to_cursor_with_style(state, row_index, column),
        2 => {
            row_mut_at(state, row_index).clear();
            if let Some(styled) = state.styled_lines.get_mut(row_index) {
                styled.text.clear();
                styled.spans.clear();
            }
        }
        _ => {}
    }
}

pub(super) fn apply_erase_line_bounded(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    mode: usize,
    columns: usize,
) {
    match mode {
        0 => erase_chars_at_with_style_bounded(
            state,
            row_index,
            column,
            columns.saturating_sub(column),
            columns,
        ),
        1 => erase_chars_at_with_style_bounded(
            state,
            row_index,
            0,
            column.saturating_add(1),
            columns,
        ),
        2 => {
            row_mut_at(state, row_index).replace_range(.., &" ".repeat(columns));
            clear_style_range(state, row_index, 0, columns);
        }
        _ => {}
    }
}

pub(super) fn apply_erase_display(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    mode: usize,
    columns: Option<usize>,
    rows: Option<usize>,
) {
    if let (Some(columns), Some(rows)) = (columns, rows) {
        apply_erase_display_bounded(state, row_index, column, mode, columns, rows);
        return;
    }
    match mode {
        0 => clear_from_cursor_to_end(state, row_index, column),
        1 => clear_start_to_cursor(state, row_index, column),
        2 | 3 => {
            state.lines.clear();
            state.styled_lines.clear();
            ensure_row_at(state, row_index);
        }
        _ => {}
    }
}

pub(super) fn apply_erase_display_bounded(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    mode: usize,
    columns: usize,
    rows: usize,
) {
    if rows == 0 {
        return;
    }
    let row_index = row_index.min(rows.saturating_sub(1));
    match mode {
        0 => {
            apply_erase_line_bounded(state, row_index, column, 0, columns);
            for index in row_index.saturating_add(1)..rows {
                clear_full_row_bounded(state, index, columns);
            }
        }
        1 => {
            for index in 0..row_index {
                clear_full_row_bounded(state, index, columns);
            }
            apply_erase_line_bounded(state, row_index, column, 1, columns);
        }
        2 | 3 => {
            for index in 0..rows {
                clear_full_row_bounded(state, index, columns);
            }
        }
        _ => {}
    }
}

fn clear_full_row_bounded(state: &mut TerminalLaneState, row_index: usize, columns: usize) {
    row_mut_at(state, row_index).replace_range(.., &" ".repeat(columns));
    clear_style_range(state, row_index, 0, columns);
}

pub(super) fn insert_lines_at(
    state: &mut TerminalLaneState,
    row: usize,
    count: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) {
    let Some((_top, bottom)) = active_region_for_row(state, row, scroll_region, rows) else {
        return;
    };
    let count = count.min(bottom - row + 1);
    for target in (row + count..=bottom).rev() {
        state.lines[target] = state.lines[target - count].clone();
        state.styled_lines[target] = state.styled_lines[target - count].clone();
    }
    for target in row..row + count {
        state.lines[target].clear();
        state.styled_lines[target] = TerminalStyledLine::default();
    }
    sync_styled_lines(state);
}

pub(super) fn delete_lines_at(
    state: &mut TerminalLaneState,
    row: usize,
    count: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) {
    let Some((_top, bottom)) = active_region_for_row(state, row, scroll_region, rows) else {
        return;
    };
    let count = count.min(bottom - row + 1);
    let shift_end = bottom + 1 - count;
    for target in row..shift_end {
        state.lines[target] = state.lines[target + count].clone();
        state.styled_lines[target] = state.styled_lines[target + count].clone();
    }
    for target in shift_end..=bottom {
        state.lines[target].clear();
        state.styled_lines[target] = TerminalStyledLine::default();
    }
    sync_styled_lines(state);
}

pub(super) fn apply_line_operation(
    final_byte: u8,
    state: &mut TerminalLaneState,
    row: usize,
    count: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) {
    match final_byte {
        b'L' => insert_lines_at(state, row, count, scroll_region, rows),
        b'M' => delete_lines_at(state, row, count, scroll_region, rows),
        b'S' => scroll_active_region_up(state, count, scroll_region, rows),
        b'T' => scroll_active_region_down(state, count, scroll_region, rows),
        _ => {}
    }
}

pub(super) fn scroll_active_region_up(
    state: &mut TerminalLaneState,
    count: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) {
    let (top, bottom) = active_scroll_region(state, scroll_region, rows);
    let count = count.min(bottom - top + 1);
    for _ in 0..count {
        scroll_region_up(state, top, bottom);
    }
}

pub(super) fn scroll_active_region_down(
    state: &mut TerminalLaneState,
    count: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) {
    let (top, bottom) = active_scroll_region(state, scroll_region, rows);
    let count = count.min(bottom - top + 1);
    for _ in 0..count {
        scroll_region_down(state, top, bottom);
    }
}

pub(super) fn scroll_region_up(state: &mut TerminalLaneState, top: usize, bottom: usize) {
    ensure_row_at(state, bottom);
    for row in top..bottom {
        state.lines[row] = state.lines[row + 1].clone();
        state.styled_lines[row] = state.styled_lines[row + 1].clone();
    }
    state.lines[bottom].clear();
    state.styled_lines[bottom] = TerminalStyledLine::default();
    sync_styled_lines(state);
}

pub(super) fn scroll_region_down(state: &mut TerminalLaneState, top: usize, bottom: usize) {
    ensure_row_at(state, bottom);
    for row in (top + 1..=bottom).rev() {
        state.lines[row] = state.lines[row - 1].clone();
        state.styled_lines[row] = state.styled_lines[row - 1].clone();
    }
    state.lines[top].clear();
    state.styled_lines[top] = TerminalStyledLine::default();
    sync_styled_lines(state);
}

fn active_region_for_row(
    state: &mut TerminalLaneState,
    row: usize,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) -> Option<(usize, usize)> {
    let (top, bottom) = scroll_region.unwrap_or_else(|| visible_screen_region(state, rows));
    ensure_row_at(state, bottom);
    (top <= row && row <= bottom).then_some((top, bottom))
}

fn active_scroll_region(
    state: &mut TerminalLaneState,
    scroll_region: &Option<(usize, usize)>,
    rows: Option<usize>,
) -> (usize, usize) {
    let region = scroll_region.unwrap_or_else(|| visible_screen_region(state, rows));
    ensure_row_at(state, region.1);
    region
}

fn visible_screen_region(state: &TerminalLaneState, rows: Option<usize>) -> (usize, usize) {
    (
        0,
        rows.map(|rows| rows.saturating_sub(1))
            .unwrap_or_else(|| state.lines.len().saturating_sub(1)),
    )
}

pub(super) fn insert_blank_chars_at(row: &mut String, column: usize, count: usize) {
    if count == 0 {
        return;
    }
    let row_len = row.chars().count();
    if column > row_len {
        row.push_str(&" ".repeat(column - row_len));
    }
    let start = super::char_to_byte_pos(row, column);
    row.insert_str(start, &" ".repeat(count));
}

pub(super) fn insert_blank_chars_at_with_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    insert_blank_chars_at(row_mut_at(state, row_index), column, count);
    shift_style_spans_for_insert(state, row_index, column, count);
}

pub(super) fn insert_blank_chars_at_with_style_bounded(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: usize,
) {
    if column >= columns {
        return;
    }
    insert_blank_chars_at_with_style(state, row_index, column, count.min(columns - column));
    truncate_row_at_with_style(state, row_index, columns);
}

pub(super) fn delete_chars_at_with_style_bounded(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: usize,
) {
    if column >= columns {
        return;
    }
    let count = count.min(columns - column);
    for _ in 0..count {
        remove_char_at_with_style(state, row_index, column);
    }
    let row_len = row_mut_at(state, row_index).chars().count();
    if row_len < columns {
        row_mut_at(state, row_index).push_str(&" ".repeat(columns - row_len));
    }
    truncate_row_at_with_style(state, row_index, columns);
}

fn erase_chars_at_with_style_bounded(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
    columns: usize,
) {
    if column >= columns || count == 0 {
        return;
    }
    let end_col = column.saturating_add(count).min(columns);
    let row_len = row_mut_at(state, row_index).chars().count();
    if row_len < end_col {
        row_mut_at(state, row_index).push_str(&" ".repeat(end_col - row_len));
    }
    erase_chars_at_with_style(state, row_index, column, end_col - column);
}

pub(super) fn shift_insert_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    shift_style_spans_for_insert(state, row_index, column, count);
}

pub(super) fn shift_delete_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    shift_style_spans_for_delete(state, row_index, column, count);
}
