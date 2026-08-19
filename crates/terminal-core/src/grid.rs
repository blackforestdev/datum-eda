use crate::{
    Cell, CellContent, CellWidth, Column, LimitError, LimitKind, ScreenCellsLimit, TerminalSize,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GridRow {
    pub cells: Vec<Cell>,
    pub soft_wrapped: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GridBuffer {
    pub rows: Vec<GridRow>,
}

impl GridBuffer {
    pub fn new(size: TerminalSize, limit: ScreenCellsLimit) -> Result<Self, GridAllocationError> {
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
            let mut cells = Vec::new();
            cells
                .try_reserve_exact(usize::from(size.columns.get()))
                .map_err(|_| GridAllocationError::Allocation)?;
            cells.resize(usize::from(size.columns.get()), Cell::default());
            rows.push(GridRow {
                cells,
                soft_wrapped: false,
            });
        }
        Ok(Self { rows })
    }

    pub fn clear(&mut self) {
        for row in &mut self.rows {
            row.cells.fill(Cell::default());
            row.soft_wrapped = false;
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
        let Some(content) = cells.get(column).map(|cell| cell.content.clone()) else {
            return;
        };
        match content {
            CellContent::Continuation { lead } => {
                let lead = usize::from(lead.get());
                if let Some(cell) = cells.get_mut(lead) {
                    *cell = Cell::default();
                }
                cells[column] = Cell::default();
            }
            CellContent::Cluster(cluster) if cluster.width() == CellWidth::Two => {
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
        self.rows[row].cells[column_index] = cell.clone();
        if width == CellWidth::Two {
            self.rows[row].cells[column_index + 1] = Cell {
                content: CellContent::Continuation { lead: column },
                style: cell.style,
                hyperlink: cell.hyperlink,
                protected: cell.protected,
            };
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GridAllocationError {
    Limit(LimitError),
    Allocation,
}
