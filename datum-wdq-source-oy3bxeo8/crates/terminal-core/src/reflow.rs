use crate::grid::{GridBuffer, GridRow};
use crate::{
    Cell, CellContent, CellPoint, CellWidth, Column, Damage, DamageSet, LimitError, LimitKind,
    LogicalPoint, Margins, Reduction, ScreenBuffer, ScreenError, TerminalCore, TerminalSize,
};

impl TerminalCore {
    pub fn resize(&mut self, size: TerminalSize) -> Result<Reduction, ScreenError> {
        if size == self.state.size {
            return Ok(Reduction::with_damage(DamageSet::new(
                self.limits.pending_damage,
            )));
        }
        let cells = size
            .cell_count()
            .and_then(|value| value.checked_mul(2))
            .ok_or(ScreenError::CellCountOverflow)?;
        self.limits
            .screen_cells
            .check(cells)
            .map_err(ScreenError::Limit)?;

        let work = self
            .state
            .history
            .rows()
            .len()
            .checked_add(self.state.primary.rows.len())
            .and_then(|rows| rows.checked_mul(usize::from(self.state.size.columns.get())))
            .ok_or(ScreenError::Limit(LimitError::ArithmeticOverflow {
                kind: LimitKind::ReflowWork,
            }))?;
        self.limits
            .reflow_work
            .check(work)
            .map_err(ScreenError::Limit)?;

        let cursor_anchor = (self.state.active_buffer == ScreenBuffer::Primary)
            .then(|| logical_point_for_cursor(&self.state.primary, self.state.cursor.position));
        let mut source = self
            .state
            .history
            .rows()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        source.extend(self.state.primary.rows.iter().cloned());
        let mut primary = reflow_rows(&source, size.columns, cursor_anchor);
        let visible_start = primary.len().saturating_sub(usize::from(size.rows.get()));
        let history = primary.drain(..visible_start).collect::<Vec<_>>();
        while primary.len() < usize::from(size.rows.get()) {
            let line = crate::history::next_line_id(&mut self.state.next_logical_line);
            primary.push(GridRow::blank(size.columns, line));
        }
        self.state.history.replace_rows(history);
        self.state.primary = GridBuffer { rows: primary };
        self.state.alternate = resize_without_reflow(
            &self.state.alternate,
            size,
            &mut self.state.next_logical_line,
        );
        self.state.size = size;
        self.state.margins = Margins::full(size);
        self.state.tabs.resize(size.columns);
        self.state.saved = None;
        self.state.grapheme_anchor = None;

        self.state.cursor.position = match (self.state.active_buffer, cursor_anchor) {
            (ScreenBuffer::Primary, Some(anchor)) => {
                resolve_visible_point(&self.state.primary, anchor, size)
            }
            _ => CellPoint::new(
                self.state
                    .cursor
                    .position
                    .row
                    .get()
                    .min(size.rows.get() - 1),
                self.state
                    .cursor
                    .position
                    .column
                    .get()
                    .min(size.columns.get() - 1),
                size,
            )
            .expect("clamped resize cursor is valid"),
        };
        self.state.cursor.pending_wrap = false;
        self.prune_graphics();

        let mut damage = DamageSet::new(self.limits.pending_damage);
        damage.push(Damage::Full).map_err(ScreenError::Limit)?;
        Ok(Reduction::with_damage(damage))
    }
}

fn logical_point_for_cursor(grid: &GridBuffer, point: CellPoint) -> LogicalPoint {
    let row = &grid.rows[usize::from(point.row.get())];
    let before = row.cells[..usize::from(point.column.get())]
        .iter()
        .filter(|cell| !matches!(cell.content, CellContent::Continuation { .. }))
        .count()
        .min(u32::MAX as usize) as u32;
    LogicalPoint {
        line: row.logical_line,
        cluster: row.cluster_start.saturating_add(before),
    }
}

fn reflow_rows(
    source: &[GridRow],
    columns: crate::Columns,
    cursor_anchor: Option<LogicalPoint>,
) -> Vec<GridRow> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < source.len() {
        let line = source[index].logical_line;
        let mut logical_cells = Vec::new();
        let mut last_soft = false;
        while index < source.len() && source[index].logical_line == line {
            let row = &source[index];
            let mut cells = row
                .cells
                .iter()
                .filter(|cell| !matches!(cell.content, CellContent::Continuation { .. }))
                .cloned()
                .collect::<Vec<_>>();
            if !row.soft_wrapped {
                let minimum_cells = cursor_anchor
                    .filter(|point| point.line == row.logical_line)
                    .map(|point| point.cluster.saturating_sub(row.cluster_start) as usize)
                    .unwrap_or(0)
                    .min(cells.len());
                while cells.len() > minimum_cells
                    && cells.last().is_some_and(|cell| cell == &Cell::default())
                {
                    cells.pop();
                }
            }
            logical_cells.extend(cells);
            last_soft = row.soft_wrapped;
            index += 1;
        }
        if logical_cells.is_empty() {
            output.push(GridRow::blank(columns, line));
            continue;
        }
        let mut row = GridRow::blank(columns, line);
        let mut column = 0usize;
        let mut cluster_start = 0u32;
        for cell in logical_cells {
            let width = cell_width(&cell);
            if column + width > usize::from(columns.get()) {
                row.soft_wrapped = true;
                output.push(row);
                cluster_start = cluster_start
                    .saturating_add(output.last().expect("reflow row exists").cluster_count());
                row = GridRow::blank(columns, line);
                row.cluster_start = cluster_start;
                column = 0;
            }
            let target = Column::new(column as u16, columns).expect("reflow column fits");
            set_cell(&mut row, target, cell);
            column += width;
        }
        row.soft_wrapped = last_soft;
        output.push(row);
    }
    output
}

fn set_cell(row: &mut GridRow, column: Column, cell: Cell) {
    let index = usize::from(column.get());
    let width = cell_width(&cell);
    row.cells[index] = cell.clone();
    if width == 2 {
        row.cells[index + 1] = Cell {
            content: CellContent::Continuation { lead: column },
            style: cell.style,
            hyperlink: cell.hyperlink,
            protected: cell.protected,
        };
    }
}

fn cell_width(cell: &Cell) -> usize {
    match &cell.content {
        CellContent::Cluster(cluster) if cluster.width() == CellWidth::Two => 2,
        _ => 1,
    }
}

fn resize_without_reflow(grid: &GridBuffer, size: TerminalSize, next_line: &mut u64) -> GridBuffer {
    let mut rows = Vec::with_capacity(usize::from(size.rows.get()));
    for row_index in 0..usize::from(size.rows.get()) {
        let mut row = grid.rows.get(row_index).cloned().unwrap_or_else(|| {
            GridRow::blank(size.columns, crate::history::next_line_id(next_line))
        });
        row.cells
            .resize(usize::from(size.columns.get()), Cell::default());
        row.cells.truncate(usize::from(size.columns.get()));
        row.soft_wrapped = false;
        rows.push(row);
    }
    let mut resized = GridBuffer { rows };
    for row in 0..resized.rows.len() {
        resized.repair_row(row);
    }
    resized
}

fn resolve_visible_point(grid: &GridBuffer, point: LogicalPoint, size: TerminalSize) -> CellPoint {
    for (row_index, row) in grid.rows.iter().enumerate() {
        if row.logical_line != point.line {
            continue;
        }
        let end = row.cluster_start.saturating_add(row.cluster_count());
        if point.cluster > end {
            continue;
        }
        let wanted = point.cluster.saturating_sub(row.cluster_start) as usize;
        let mut seen = 0usize;
        for (column, cell) in row.cells.iter().enumerate() {
            if matches!(cell.content, CellContent::Continuation { .. }) {
                continue;
            }
            if seen == wanted {
                return CellPoint::new(row_index as u16, column as u16, size)
                    .expect("resolved point fits");
            }
            seen += 1;
        }
        return CellPoint::new(row_index as u16, size.columns.get().saturating_sub(1), size)
            .expect("resolved row fits");
    }
    CellPoint::new(0, 0, size).expect("terminal size is nonzero")
}
