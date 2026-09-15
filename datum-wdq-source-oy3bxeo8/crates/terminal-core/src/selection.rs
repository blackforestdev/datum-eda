use crate::grid::GridRow;
use crate::{
    AnchorResolution, CellContent, ClipboardBytesLimit, LimitError, LogicalPoint, ScreenBuffer,
    TerminalCore,
};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionScope {
    Grapheme,
    Word,
    WrappedLine,
    LogicalLine,
    Block,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    anchor: LogicalPoint,
    focus: LogicalPoint,
    scope: SelectionScope,
}

impl Selection {
    pub const fn new(anchor: LogicalPoint, focus: LogicalPoint, scope: SelectionScope) -> Self {
        Self {
            anchor,
            focus,
            scope,
        }
    }

    pub const fn anchor(self) -> LogicalPoint {
        self.anchor
    }

    pub const fn focus(self) -> LogicalPoint {
        self.focus
    }

    pub const fn scope(self) -> SelectionScope {
        self.scope
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionState {
    Active,
    Trimmed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionError {
    Missing,
    Trimmed,
    UnknownEndpoint,
    Limit(LimitError),
}

impl fmt::Display for SelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("terminal selection is missing"),
            Self::Trimmed => formatter.write_str("terminal selection endpoint was trimmed"),
            Self::UnknownEndpoint => formatter.write_str("terminal selection endpoint is unknown"),
            Self::Limit(error) => error.fmt(formatter),
        }
    }
}

impl Error for SelectionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopiedText(String);

impl CopiedText {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TerminalCore {
    pub fn set_selection(&mut self, selection: Selection) -> Result<(), SelectionError> {
        endpoint_state(self, selection.anchor)?;
        endpoint_state(self, selection.focus)?;
        self.state.selection = Some(selection);
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.state.selection = None;
    }

    pub const fn selection(&self) -> Option<Selection> {
        self.state.selection
    }

    pub fn selection_state(&self) -> Result<Option<SelectionState>, SelectionError> {
        let Some(selection) = self.state.selection else {
            return Ok(None);
        };
        let anchor = endpoint_state(self, selection.anchor);
        let focus = endpoint_state(self, selection.focus);
        if matches!(anchor, Err(SelectionError::Trimmed))
            || matches!(focus, Err(SelectionError::Trimmed))
        {
            return Ok(Some(SelectionState::Trimmed));
        }
        anchor?;
        focus?;
        Ok(Some(SelectionState::Active))
    }

    pub fn copy_selection(&self) -> Result<CopiedText, SelectionError> {
        let selection = self.state.selection.ok_or(SelectionError::Missing)?;
        endpoint_state(self, selection.anchor)?;
        endpoint_state(self, selection.focus)?;
        let rows = document_rows(self);
        let text = match selection.scope {
            SelectionScope::Block => copy_block(&rows, selection)?,
            SelectionScope::WrappedLine => copy_wrapped_lines(&rows, selection)?,
            SelectionScope::LogicalLine => copy_logical_lines(&rows, selection),
            SelectionScope::All => copy_all(&rows),
            SelectionScope::Word => copy_linear(&rows, expand_words(&rows, selection)?),
            SelectionScope::Grapheme => copy_linear(&rows, normalized(selection)),
        };
        bounded_copy(text, self.limits.clipboard_bytes)
    }
}

#[derive(Clone, Copy)]
struct DocumentRow<'a> {
    row: &'a GridRow,
}

fn document_rows(core: &TerminalCore) -> Vec<DocumentRow<'_>> {
    let mut rows = Vec::new();
    if core.state.active_buffer == ScreenBuffer::Primary {
        rows.extend(
            core.state
                .history
                .rows()
                .iter()
                .map(|row| DocumentRow { row }),
        );
    }
    rows.extend(
        core.state
            .active_grid()
            .rows
            .iter()
            .map(|row| DocumentRow { row }),
    );
    rows
}

fn endpoint_state(core: &TerminalCore, point: LogicalPoint) -> Result<(), SelectionError> {
    match core.state.resolve_logical_point(point) {
        AnchorResolution::History { .. } | AnchorResolution::Screen { .. } => Ok(()),
        AnchorResolution::Trimmed => Err(SelectionError::Trimmed),
        AnchorResolution::Unknown => Err(SelectionError::UnknownEndpoint),
    }
}

fn normalized(selection: Selection) -> Selection {
    if selection.anchor <= selection.focus {
        selection
    } else {
        Selection::new(selection.focus, selection.anchor, selection.scope)
    }
}

fn copy_linear(rows: &[DocumentRow<'_>], selection: Selection) -> String {
    let selection = normalized(selection);
    let mut output = String::new();
    let mut copied_any_row = false;
    let mut prior_logical_line = None;
    for document in rows {
        let row = document.row;
        let mut selected = Vec::new();
        for (column, cell) in row.cells.iter().enumerate() {
            if matches!(cell.content, CellContent::Continuation { .. }) {
                continue;
            }
            let point = point_at(row, column);
            if point < selection.anchor || point > selection.focus {
                continue;
            }
            selected.push(cell);
        }
        if selected.is_empty() {
            continue;
        }
        if copied_any_row && prior_logical_line.is_some_and(|line| line != row.logical_line) {
            output.push('\n');
        }
        if row.logical_line != selection.focus.line {
            while selected
                .last()
                .is_some_and(|cell| matches!(cell.content, CellContent::Empty))
            {
                selected.pop();
            }
        }
        for cell in selected {
            push_cell(&mut output, cell);
        }
        copied_any_row = true;
        prior_logical_line = Some(row.logical_line);
    }
    output
}

fn copy_logical_lines(rows: &[DocumentRow<'_>], selection: Selection) -> String {
    let selection = normalized(selection);
    let mut output = String::new();
    let mut copied_line = None;
    for document in rows {
        let row = document.row;
        if row.logical_line < selection.anchor.line || row.logical_line > selection.focus.line {
            continue;
        }
        if copied_line.is_some_and(|line| line != row.logical_line) {
            output.push('\n');
        }
        copied_line = Some(row.logical_line);
        push_row(&mut output, row, 0, row.cells.len().saturating_sub(1));
    }
    output
}

fn copy_wrapped_lines(
    rows: &[DocumentRow<'_>],
    selection: Selection,
) -> Result<String, SelectionError> {
    let (first_row, _) = locate(rows, selection.anchor).ok_or(SelectionError::UnknownEndpoint)?;
    let (last_row, _) = locate(rows, selection.focus).ok_or(SelectionError::UnknownEndpoint)?;
    let (first_row, last_row) = ordered_pair(first_row, last_row);
    let mut output = String::new();
    for (index, document) in rows[first_row..=last_row].iter().enumerate() {
        push_row(
            &mut output,
            document.row,
            0,
            document.row.cells.len().saturating_sub(1),
        );
        if index + first_row < last_row {
            output.push('\n');
        }
    }
    Ok(output)
}

fn copy_block(rows: &[DocumentRow<'_>], selection: Selection) -> Result<String, SelectionError> {
    let (first_row, first_column) =
        locate(rows, selection.anchor).ok_or(SelectionError::UnknownEndpoint)?;
    let (last_row, last_column) =
        locate(rows, selection.focus).ok_or(SelectionError::UnknownEndpoint)?;
    let (first_row, last_row) = ordered_pair(first_row, last_row);
    let (first_column, last_column) = ordered_pair(first_column, last_column);
    let mut output = String::new();
    for (offset, document) in rows[first_row..=last_row].iter().enumerate() {
        push_row_exact(&mut output, document.row, first_column, last_column);
        if offset + first_row < last_row {
            output.push('\n');
        }
    }
    Ok(output)
}

fn copy_all(rows: &[DocumentRow<'_>]) -> String {
    let row_count = rows
        .iter()
        .rposition(|document| {
            document
                .row
                .cells
                .iter()
                .any(|cell| !matches!(cell.content, CellContent::Empty))
        })
        .map_or(0, |index| index + 1);
    let mut output = String::new();
    for (index, document) in rows[..row_count].iter().enumerate() {
        push_row(
            &mut output,
            document.row,
            0,
            document.row.cells.len().saturating_sub(1),
        );
        if !document.row.soft_wrapped && index + 1 < row_count {
            output.push('\n');
        }
    }
    output
}

fn expand_words(
    rows: &[DocumentRow<'_>],
    selection: Selection,
) -> Result<Selection, SelectionError> {
    let mut selection = normalized(selection);
    selection.anchor = expand_word(rows, selection.anchor, false)?;
    selection.focus = expand_word(rows, selection.focus, true)?;
    Ok(selection)
}

fn expand_word(
    rows: &[DocumentRow<'_>],
    point: LogicalPoint,
    toward_end: bool,
) -> Result<LogicalPoint, SelectionError> {
    let mut cells = rows
        .iter()
        .flat_map(|document| logical_cells(document.row))
        .filter(|(candidate, _)| candidate.line == point.line)
        .collect::<Vec<_>>();
    cells.sort_by_key(|(point, _)| *point);
    let index = cells
        .iter()
        .position(|(candidate, _)| *candidate == point)
        .ok_or(SelectionError::UnknownEndpoint)?;
    let class = word_class(cells[index].1);
    let mut boundary = index;
    if toward_end {
        while boundary + 1 < cells.len() && word_class(cells[boundary + 1].1) == class {
            boundary += 1;
        }
    } else {
        while boundary > 0 && word_class(cells[boundary - 1].1) == class {
            boundary -= 1;
        }
    }
    Ok(cells[boundary].0)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum WordClass {
    Space,
    Word,
    Punctuation,
}

fn word_class(cell: &crate::Cell) -> WordClass {
    match &cell.content {
        CellContent::Empty => WordClass::Space,
        CellContent::Cluster(cluster) => {
            let mut characters = cluster.text().chars();
            let first = characters.next().unwrap_or(' ');
            if cluster
                .text()
                .chars()
                .all(|character| character == ' ' || character == '\t')
            {
                WordClass::Space
            } else if !first.is_ascii() || first.is_ascii_alphanumeric() || first == '_' {
                WordClass::Word
            } else {
                WordClass::Punctuation
            }
        }
        CellContent::Continuation { .. } => WordClass::Punctuation,
    }
}

fn locate(rows: &[DocumentRow<'_>], point: LogicalPoint) -> Option<(usize, usize)> {
    rows.iter().enumerate().find_map(|(row_index, document)| {
        crate::history::column_for_point(document.row, point)
            .map(|column| (row_index, usize::from(column)))
    })
}

fn logical_cells(row: &GridRow) -> impl Iterator<Item = (LogicalPoint, &crate::Cell)> {
    row.cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| !matches!(cell.content, CellContent::Continuation { .. }))
        .enumerate()
        .map(|(offset, (_, cell))| {
            (
                LogicalPoint {
                    line: row.logical_line,
                    cluster: row.cluster_start.saturating_add(offset as u32),
                },
                cell,
            )
        })
}

fn point_at(row: &GridRow, column: usize) -> LogicalPoint {
    let before = row.cells[..column]
        .iter()
        .filter(|cell| !matches!(cell.content, CellContent::Continuation { .. }))
        .count()
        .min(u32::MAX as usize) as u32;
    LogicalPoint {
        line: row.logical_line,
        cluster: row.cluster_start.saturating_add(before),
    }
}

fn push_row(output: &mut String, row: &GridRow, first: usize, last: usize) {
    let cells = row
        .cells
        .iter()
        .take(last.saturating_add(1))
        .skip(first)
        .filter(|cell| !matches!(cell.content, CellContent::Continuation { .. }))
        .collect::<Vec<_>>();
    push_cells_without_padding(output, &cells);
}

fn push_row_exact(output: &mut String, row: &GridRow, first: usize, last: usize) {
    for cell in row.cells.iter().take(last.saturating_add(1)).skip(first) {
        push_cell(output, cell);
    }
}

fn push_cells_without_padding(output: &mut String, cells: &[&crate::Cell]) {
    let meaningful = cells
        .iter()
        .rposition(|cell| !matches!(cell.content, CellContent::Empty))
        .map_or(0, |index| index + 1);
    for cell in &cells[..meaningful] {
        push_cell(output, cell);
    }
}

fn push_cell(output: &mut String, cell: &crate::Cell) {
    match &cell.content {
        CellContent::Cluster(cluster) => output.push_str(cluster.text()),
        CellContent::Empty => output.push(' '),
        CellContent::Continuation { .. } => {}
    }
}

fn bounded_copy(text: String, limit: ClipboardBytesLimit) -> Result<CopiedText, SelectionError> {
    limit.check(text.len()).map_err(SelectionError::Limit)?;
    Ok(CopiedText(text))
}

fn ordered_pair<T: Ord>(left: T, right: T) -> (T, T) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}
