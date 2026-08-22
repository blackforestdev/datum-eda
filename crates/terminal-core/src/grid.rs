use crate::{
    Cell, CellContent, CellWidth, Column, LimitError, LimitKind, LogicalLineId, ScreenCellsLimit,
    TerminalSize,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GridRow {
    pub cells: Vec<Cell>,
    pub soft_wrapped: bool,
    pub logical_line: LogicalLineId,
    pub cluster_start: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GridBuffer {
    pub rows: Vec<GridRow>,
}

impl GridBuffer {
    pub fn new(
        size: TerminalSize,
        limit: ScreenCellsLimit,
        next_line: &mut u64,
    ) -> Result<Self, GridAllocationError> {
        let cells = size.cell_count().ok_or(GridAllocationError::Limit(
            LimitError::ArithmeticOverflow {
                kind: LimitKind::ScreenCells,
            },
        ))?;
        limit.check(cells).map_err(GridAllocationError::Limit)?;

        let mut rows = Vec::new();
        rows.try_reserve_exact(usize::from(size.rows.get()))
            .map_err(|_| GridAllocationError::Allocation)?;
        for _ in 0..size.rows.get() {
            rows.push(GridRow::blank(size.columns, take_line_id(next_line)?));
        }
        Ok(Self { rows })
    }

    pub(crate) fn clear_with_new_lines(&mut self, next_line: &mut u64) {
        for row in &mut self.rows {
            row.cells.fill(Cell::default());
            row.soft_wrapped = false;
            row.cluster_start = 0;
            row.logical_line = LogicalLineId::new(*next_line);
            *next_line = next_line.saturating_add(1);
        }
    }

    pub fn repair_row(&mut self, row: usize) {
        let cells = &mut self.rows[row].cells;
        let mut column = 0;
        while column < cells.len() {
            match &cells[column].content {
                CellContent::Cluster(cluster) if cluster.width() == CellWidth::Two => {
                    let valid = cells.get(column + 1).is_some_and(|cell| {
                        matches!(cell.content, CellContent::Continuation { lead } if usize::from(lead.get()) == column)
                    });
                    if valid {
                        column += 2;
                    } else {
                        cells[column] = Cell::default();
                        column += 1;
                    }
                }
                CellContent::Continuation { lead } => {
                    let lead = usize::from(lead.get());
                    let valid = lead.checked_add(1) == Some(column)
                        && matches!(
                            cells.get(lead).map(|cell| &cell.content),
                            Some(CellContent::Cluster(cluster)) if cluster.width() == CellWidth::Two
                        );
                    if !valid {
                        cells[column] = Cell::default();
                    }
                    column += 1;
                }
                _ => column += 1,
            }
        }
    }

    pub fn clear_cluster_at(&mut self, row: usize, column: usize) {
        let Some(cells) = self.rows.get_mut(row).map(|row| &mut row.cells) else {
            return;
        };
        let Some(content) = cells.get(column).map(|cell| match &cell.content {
            CellContent::Continuation { lead } => (1_u8, Some(*lead)),
            CellContent::Cluster(cluster) if cluster.width() == CellWidth::Two => (2, None),
            _ => (0, None),
        }) else {
            return;
        };
        match content {
            (1, Some(lead)) => {
                let lead = usize::from(lead.get());
                if let Some(cell) = cells.get_mut(lead) {
                    *cell = Cell::default();
                }
                cells[column] = Cell::default();
            }
            (2, _) => {
                cells[column] = Cell::default();
                if column + 1 < cells.len() {
                    cells[column + 1] = Cell::default();
                }
            }
            _ => cells[column] = Cell::default(),
        }
    }

    pub fn set_cluster(&mut self, row: usize, column: Column, cell: Cell) {
        let column_index = usize::from(column.get());
        let Some(row_length) = self.rows.get(row).map(|row| row.cells.len()) else {
            return;
        };
        let width = match &cell.content {
            CellContent::Cluster(cluster) => cluster.width(),
            _ => CellWidth::One,
        };
        if column_index >= row_length || (width == CellWidth::Two && column_index + 1 >= row_length)
        {
            return;
        }
        self.clear_cluster_at(row, column_index);
        if width == CellWidth::Two {
            self.clear_cluster_at(row, column_index + 1);
        }
        if width == CellWidth::Two {
            self.rows[row].cells[column_index] = cell.clone();
            self.rows[row].cells[column_index + 1] = Cell {
                content: CellContent::Continuation { lead: column },
                style: cell.style,
                hyperlink: cell.hyperlink,
                protected: cell.protected,
            };
        } else {
            self.rows[row].cells[column_index] = cell;
        }
    }
}

impl GridRow {
    pub(crate) fn blank(columns: crate::Columns, logical_line: LogicalLineId) -> Self {
        Self {
            cells: vec![Cell::default(); usize::from(columns.get())],
            soft_wrapped: false,
            logical_line,
            cluster_start: 0,
        }
    }

    pub(crate) fn payload_bytes(&self) -> usize {
        self.cells
            .iter()
            .filter_map(|cell| match &cell.content {
                CellContent::Cluster(cluster) => Some(cluster.text().len()),
                _ => None,
            })
            .sum()
    }

    pub(crate) fn cluster_count(&self) -> u32 {
        self.cells
            .iter()
            .filter(|cell| !matches!(cell.content, CellContent::Continuation { .. }))
            .count()
            .min(u32::MAX as usize) as u32
    }
}

fn take_line_id(next_line: &mut u64) -> Result<LogicalLineId, GridAllocationError> {
    let line = LogicalLineId::new(*next_line);
    *next_line = next_line
        .checked_add(1)
        .ok_or(GridAllocationError::LogicalLineIdExhausted)?;
    Ok(line)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GridAllocationError {
    Limit(LimitError),
    Allocation,
    LogicalLineIdExhausted,
}
