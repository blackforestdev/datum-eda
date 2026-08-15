use datum_gui_protocol::{
    TerminalLaneState, TerminalStyleSpan, TerminalStyledLine, TerminalTextStyle,
};

pub(super) fn sync_styled_lines(state: &mut TerminalLaneState) {
    let grid = state.pty_grid_mut();
    while grid.styled_lines.len() < grid.lines.len() {
        grid.styled_lines.push(TerminalStyledLine::default());
    }
    if grid.styled_lines.len() > grid.lines.len() {
        grid.styled_lines.truncate(grid.lines.len());
    }
    for (index, line) in grid.lines.iter().enumerate() {
        let styled = &mut grid.styled_lines[index];
        // Cheap win (terminal performance slice): this runs on the per-byte
        // hot path, so unchanged rows must not re-allocate — compare before
        // copying and skip span pruning entirely for unstyled rows.
        if styled.text != *line {
            styled.text.clone_from(line);
        }
        if !styled.spans.is_empty() {
            let line_len = line.chars().count();
            styled
                .spans
                .retain(|span| span.start < span.end && span.end <= line_len);
        }
    }
}

/// Keep the style-row topology aligned while PTY bytes are being applied.
/// Text projection is deliberately deferred to one `sync_styled_lines` call
/// at the end of the batch; cloning every accumulated row for every byte was
/// the dominant T0-C03A output-lag defect.
pub(super) fn ensure_styled_line_count(state: &mut TerminalLaneState) {
    let grid = state.pty_grid_mut();
    grid.styled_lines
        .resize_with(grid.lines.len(), TerminalStyledLine::default);
    grid.styled_lines.truncate(grid.lines.len());
}

pub(super) fn set_style_at(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    style: &TerminalTextStyle,
) {
    ensure_styled_line_count(state);
    if style == &TerminalTextStyle::default() {
        clear_style_range(state, row_index, column, column.saturating_add(1));
        return;
    }
    clear_style_range(state, row_index, column, column.saturating_add(1));
    let Some(row) = state.pty_grid_mut().styled_lines.get_mut(row_index) else {
        return;
    };
    row.spans.push(TerminalStyleSpan {
        start: column,
        end: column.saturating_add(1),
        fg: style.fg.clone(),
        bg: style.bg.clone(),
        bold: style.bold,
        dim: style.dim,
        italic: style.italic,
        underline: style.underline,
        overline: style.overline,
        blink: style.blink,
        strikethrough: style.strikethrough,
        conceal: style.conceal,
        inverse: style.inverse,
    });
    coalesce_style_spans(row);
}

pub(super) fn clear_style_range(
    state: &mut TerminalLaneState,
    row_index: usize,
    start: usize,
    end: usize,
) {
    if start >= end {
        return;
    }
    let Some(row) = state.pty_grid_mut().styled_lines.get_mut(row_index) else {
        return;
    };
    // Cheap win (terminal performance slice): unstyled rows are the common
    // case on the per-byte hot path — nothing to clear, no allocation.
    if row.spans.is_empty() {
        return;
    }
    let mut retained = Vec::new();
    for span in row.spans.drain(..) {
        if span.end <= start || span.start >= end {
            retained.push(span);
            continue;
        }
        if span.start < start {
            retained.push(TerminalStyleSpan {
                start: span.start,
                end: start,
                fg: span.fg.clone(),
                bg: span.bg.clone(),
                bold: span.bold,
                dim: span.dim,
                italic: span.italic,
                underline: span.underline,
                overline: span.overline,
                blink: span.blink,
                strikethrough: span.strikethrough,
                conceal: span.conceal,
                inverse: span.inverse,
            });
        }
        if span.end > end {
            retained.push(TerminalStyleSpan {
                start: end,
                end: span.end,
                fg: span.fg,
                bg: span.bg,
                bold: span.bold,
                dim: span.dim,
                italic: span.italic,
                underline: span.underline,
                overline: span.overline,
                blink: span.blink,
                strikethrough: span.strikethrough,
                conceal: span.conceal,
                inverse: span.inverse,
            });
        }
    }
    row.spans = retained;
    coalesce_style_spans(row);
}

pub(super) fn shift_style_spans_for_insert(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    let Some(row) = state.pty_grid_mut().styled_lines.get_mut(row_index) else {
        return;
    };
    for span in &mut row.spans {
        if span.start >= column {
            span.start += count;
            span.end += count;
        } else if span.end > column {
            span.end += count;
        }
    }
}

pub(super) fn shift_style_spans_for_delete(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    let end = column.saturating_add(count);
    clear_style_range(state, row_index, column, end);
    let Some(row) = state.pty_grid_mut().styled_lines.get_mut(row_index) else {
        return;
    };
    for span in &mut row.spans {
        if span.start >= end {
            span.start -= count;
            span.end -= count;
        } else if span.end > end {
            span.end -= count;
        }
    }
    coalesce_style_spans(row);
}

fn coalesce_style_spans(row: &mut TerminalStyledLine) {
    // Zero or one span coalesces to itself; skip the sort and re-collection.
    if row.spans.len() <= 1 {
        return;
    }
    row.spans.sort_by_key(|span| (span.start, span.end));
    let mut merged: Vec<TerminalStyleSpan> = Vec::new();
    for span in row.spans.drain(..) {
        if let Some(previous) = merged.last_mut()
            && previous.end == span.start
            && previous.fg == span.fg
            && previous.bg == span.bg
            && previous.bold == span.bold
            && previous.dim == span.dim
            && previous.italic == span.italic
            && previous.underline == span.underline
            && previous.overline == span.overline
            && previous.blink == span.blink
            && previous.strikethrough == span.strikethrough
            && previous.conceal == span.conceal
            && previous.inverse == span.inverse
        {
            previous.end = span.end;
            continue;
        }
        merged.push(span);
    }
    row.spans = merged;
}
