use crate::grid::GridRow;
use crate::{Cell, CellContent, HistoryBytesLimit, HistoryLinesLimit, LogicalLineId, LogicalPoint};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorResolution {
    History { row: usize, column: u16 },
    Screen { row: u16, column: u16 },
    Trimmed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryRowSnapshot {
    logical_start: LogicalPoint,
    cells: Vec<Cell>,
    soft_wrapped: bool,
}

impl HistoryRowSnapshot {
    pub const fn logical_start(&self) -> LogicalPoint {
        self.logical_start
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub const fn soft_wrapped(&self) -> bool {
        self.soft_wrapped
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistorySnapshot {
    rows: Vec<HistoryRowSnapshot>,
    payload_bytes: usize,
    trimmed_rows: u64,
}

impl HistorySnapshot {
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &HistoryRowSnapshot> {
        self.rows.iter()
    }

    pub const fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }

    pub const fn trimmed_rows(&self) -> u64 {
        self.trimmed_rows
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HistoryStore {
    rows: VecDeque<GridRow>,
    payload_bytes: usize,
    trimmed_rows: u64,
    line_limit: HistoryLinesLimit,
    byte_limit: HistoryBytesLimit,
}

impl HistoryStore {
    pub(crate) fn new(line_limit: HistoryLinesLimit, byte_limit: HistoryBytesLimit) -> Self {
        Self {
            rows: VecDeque::new(),
            payload_bytes: 0,
            trimmed_rows: 0,
            line_limit,
            byte_limit,
        }
    }

    pub(crate) fn push(&mut self, row: GridRow) {
        self.payload_bytes = self.payload_bytes.saturating_add(row.payload_bytes());
        self.rows.push_back(row);
        self.trim_to_limits();
    }

    pub(crate) fn clear(&mut self) {
        self.rows.clear();
        self.payload_bytes = 0;
        self.trimmed_rows = 0;
    }

    pub(crate) fn rows(&self) -> &VecDeque<GridRow> {
        &self.rows
    }

    pub(crate) fn replace_rows(&mut self, rows: Vec<GridRow>) {
        self.rows = rows.into();
        self.payload_bytes = self.rows.iter().map(GridRow::payload_bytes).sum();
        self.trim_to_limits();
    }

    pub(crate) fn snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            rows: self
                .rows
                .iter()
                .map(|row| HistoryRowSnapshot {
                    logical_start: LogicalPoint {
                        line: row.logical_line,
                        cluster: row.cluster_start,
                    },
                    cells: row.cells.clone(),
                    soft_wrapped: row.soft_wrapped,
                })
                .collect(),
            payload_bytes: self.payload_bytes,
            trimmed_rows: self.trimmed_rows,
        }
    }

    pub(crate) fn contains(&self, point: LogicalPoint) -> bool {
        self.rows.iter().any(|row| row_contains(row, point))
    }

    pub(crate) fn resolve(&self, point: LogicalPoint) -> Option<(usize, u16)> {
        self.rows
            .iter()
            .enumerate()
            .find_map(|(index, row)| column_for_point(row, point).map(|column| (index, column)))
    }

    pub(crate) fn oldest_line(&self) -> Option<LogicalLineId> {
        self.rows.front().map(|row| row.logical_line)
    }

    fn trim_to_limits(&mut self) {
        while logical_line_count(&self.rows) > self.line_limit.get()
            || self.payload_bytes > self.byte_limit.get()
        {
            let Some(front) = self.rows.front() else {
                break;
            };
            let line = front.logical_line;
            while self
                .rows
                .front()
                .is_some_and(|row| row.logical_line == line)
            {
                let removed = self.rows.pop_front().expect("front row exists");
                self.payload_bytes = self.payload_bytes.saturating_sub(removed.payload_bytes());
                self.trimmed_rows = self.trimmed_rows.saturating_add(1);
            }
        }
    }
}

fn logical_line_count(rows: &VecDeque<GridRow>) -> usize {
    let mut count = 0usize;
    let mut previous = None;
    for row in rows {
        if previous != Some(row.logical_line) {
            count = count.saturating_add(1);
            previous = Some(row.logical_line);
        }
    }
    count
}

pub(crate) fn row_contains(row: &GridRow, point: LogicalPoint) -> bool {
    column_for_point(row, point).is_some()
}

pub(crate) fn column_for_point(row: &GridRow, point: LogicalPoint) -> Option<u16> {
    if row.logical_line != point.line || point.cluster < row.cluster_start {
        return None;
    }
    let wanted = point.cluster - row.cluster_start;
    let mut seen = 0u32;
    for (column, cell) in row.cells.iter().enumerate() {
        if matches!(cell.content, CellContent::Continuation { .. }) {
            continue;
        }
        if seen == wanted {
            return u16::try_from(column).ok();
        }
        seen = seen.saturating_add(1);
    }
    None
}

pub(crate) fn next_line_id(value: &mut u64) -> LogicalLineId {
    let current = LogicalLineId::new(*value);
    *value = value.saturating_add(1);
    current
}
