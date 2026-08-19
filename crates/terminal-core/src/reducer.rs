use crate::grid::GridAllocationError;
use crate::{
    Cell, CellContent, CellPoint, CellWidth, Column, Damage, DamageSet, EraseDisplay, EraseLine,
    FoundationMode, LimitError, Margins, ModeState, SavedCursorState, ScreenAction, ScreenBuffer,
    TerminalCore,
};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreenError {
    Limit(LimitError),
    CellCountOverflow,
    Allocation,
}

impl fmt::Display for ScreenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(error) => error.fmt(formatter),
            Self::CellCountOverflow => formatter.write_str("terminal screen cell count overflowed"),
            Self::Allocation => formatter.write_str("terminal screen allocation failed"),
        }
    }
}

impl Error for ScreenError {}

impl From<GridAllocationError> for ScreenError {
    fn from(value: GridAllocationError) -> Self {
        match value {
            GridAllocationError::Limit(error) => Self::Limit(error),
            GridAllocationError::Allocation => Self::Allocation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reduction {
    damage: DamageSet,
}

impl Reduction {
    pub fn damage(&self) -> &DamageSet {
        &self.damage
    }
}

impl TerminalCore {
    pub fn reduce(&mut self, action: ScreenAction) -> Result<Reduction, ScreenError> {
        let cursor_only = matches!(
            action,
            ScreenAction::Backspace
                | ScreenAction::CarriageReturn
                | ScreenAction::HorizontalTab
                | ScreenAction::SetCursor { .. }
                | ScreenAction::MoveCursor { .. }
                | ScreenAction::SaveCursor
                | ScreenAction::RestoreCursor
        );
        self.apply_action(action);
        let mut damage = DamageSet::new(self.limits.pending_damage);
        damage
            .push(if cursor_only {
                Damage::Cursor
            } else {
                Damage::Full
            })
            .map_err(ScreenError::Limit)?;
        Ok(Reduction { damage })
    }

    fn apply_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::Print(cluster) => self.print(cluster),
            ScreenAction::Backspace => self.backspace(),
            ScreenAction::CarriageReturn => self.carriage_return(),
            ScreenAction::LineFeed => self.line_feed(),
            ScreenAction::ReverseIndex => self.reverse_index(),
            ScreenAction::HorizontalTab => self.horizontal_tab(),
            ScreenAction::SetCursor { row, column } => self.set_cursor(row, column),
            ScreenAction::MoveCursor { rows, columns } => self.move_cursor(rows, columns),
            ScreenAction::SetMargins(margins) => self.set_margins(margins),
            ScreenAction::ResetMargins => self.set_margins(Margins::full(self.state.size)),
            ScreenAction::InsertCells(count) => self.insert_cells(count),
            ScreenAction::DeleteCells(count) => self.delete_cells(count),
            ScreenAction::EraseCells(count) => self.erase_cells(count, false),
            ScreenAction::InsertLines(count) => self.insert_lines(count),
            ScreenAction::DeleteLines(count) => self.delete_lines(count),
            ScreenAction::ScrollUp(count) => self.scroll_up(self.state.margins.top.get(), count),
            ScreenAction::ScrollDown(count) => {
                self.scroll_down(self.state.margins.top.get(), count)
            }
            ScreenAction::EraseLine { mode, selective } => self.erase_line(mode, selective),
            ScreenAction::EraseDisplay { mode, selective } => self.erase_display(mode, selective),
            ScreenAction::SwitchBuffer {
                buffer,
                clear,
                home,
            } => self.switch_buffer(buffer, clear, home),
            ScreenAction::SaveCursor => self.save_cursor(),
            ScreenAction::RestoreCursor => self.restore_cursor(),
            ScreenAction::SetMode { mode, enabled } => self.set_mode(mode, enabled),
            ScreenAction::SetStyle(style) => self.state.style = style,
            ScreenAction::SetProtection(protected) => self.state.protected = protected,
            ScreenAction::Reset => self.reset(),
        }
    }

    fn print(&mut self, cluster: crate::Cluster) {
        self.state.last_printed = Some(cluster.clone());
        if self.state.cursor.pending_wrap {
            self.wrap_line();
        }
        let width = match cluster.width() {
            CellWidth::One => 1,
            CellWidth::Two => 2,
        };
        let (_, right) = self.horizontal_bounds_for_cursor();
        let column = self.state.cursor.position.column.get();
        if width == 2 && column >= right {
            if self.state.modes.auto_wrap {
                self.wrap_line();
            } else {
                return;
            }
        }
        if self.state.modes.insert {
            self.insert_cells(width);
        }
        let row = usize::from(self.state.cursor.position.row.get());
        let column = self.state.cursor.position.column;
        let cell = Cell {
            content: CellContent::Cluster(cluster),
            style: self.state.style,
            hyperlink: None,
            protected: self.state.protected,
        };
        self.state.active_grid_mut().set_cluster(row, column, cell);

        let end = column.get().saturating_add(width - 1);
        if end >= right {
            self.state.cursor.position.column = Column::new(right, self.state.size.columns)
                .unwrap_or(self.state.cursor.position.column);
            self.state.cursor.pending_wrap = self.state.modes.auto_wrap;
        } else {
            self.state.cursor.position.column = Column::new(end + 1, self.state.size.columns)
                .unwrap_or(self.state.cursor.position.column);
        }
    }

    fn wrap_line(&mut self) {
        let row = usize::from(self.state.cursor.position.row.get());
        self.state.active_grid_mut().rows[row].soft_wrapped = true;
        self.state.cursor.pending_wrap = false;
        self.carriage_return();
        self.line_feed();
    }

    fn backspace(&mut self) {
        let (left, _) = self.horizontal_bounds_for_cursor();
        let column = self
            .state
            .cursor
            .position
            .column
            .get()
            .saturating_sub(1)
            .max(left);
        self.set_cursor(self.state.cursor.position.row.get(), column);
    }

    fn carriage_return(&mut self) {
        let (left, _) = self.horizontal_bounds_for_cursor();
        self.set_cursor(self.state.cursor.position.row.get(), left);
    }

    fn line_feed(&mut self) {
        self.state.cursor.pending_wrap = false;
        let row = self.state.cursor.position.row.get();
        if row == self.state.margins.bottom.get() {
            self.scroll_up(self.state.margins.top.get(), 1);
        } else {
            self.set_cursor(
                row.saturating_add(1),
                self.state.cursor.position.column.get(),
            );
        }
        if self.state.modes.newline {
            self.carriage_return();
        }
    }

    fn reverse_index(&mut self) {
        self.state.cursor.pending_wrap = false;
        let row = self.state.cursor.position.row.get();
        if row == self.state.margins.top.get() {
            self.scroll_down(self.state.margins.top.get(), 1);
        } else {
            self.set_cursor(
                row.saturating_sub(1),
                self.state.cursor.position.column.get(),
            );
        }
    }

    fn horizontal_tab(&mut self) {
        let current = self.state.cursor.position.column.get();
        let (_, right) = self.horizontal_bounds_for_cursor();
        let next = self
            .state
            .tabs
            .iter()
            .map(Column::get)
            .find(|column| *column > current && *column <= right)
            .unwrap_or(right);
        self.set_cursor(self.state.cursor.position.row.get(), next);
    }

    fn set_cursor(&mut self, row: u16, column: u16) {
        let (top, bottom, left, right) = if self.state.modes.origin {
            (
                self.state.margins.top.get(),
                self.state.margins.bottom.get(),
                self.state.margins.left.get(),
                self.state.margins.right.get(),
            )
        } else {
            (
                0,
                self.state.size.rows.get() - 1,
                0,
                self.state.size.columns.get() - 1,
            )
        };
        let row = row.clamp(top, bottom);
        let column = column.clamp(left, right);
        if let Ok(position) = CellPoint::new(row, column, self.state.size) {
            self.state.cursor.position = position;
            self.state.cursor.pending_wrap = false;
        }
    }

    fn move_cursor(&mut self, rows: i32, columns: i32) {
        let row = offset(self.state.cursor.position.row.get(), rows);
        let column = offset(self.state.cursor.position.column.get(), columns);
        self.set_cursor(row, column);
    }

    fn set_margins(&mut self, margins: Margins) {
        self.state.margins = margins;
        self.set_cursor(margins.top.get(), margins.left.get());
    }

    fn insert_cells(&mut self, count: u16) {
        let row = usize::from(self.state.cursor.position.row.get());
        let start = usize::from(self.state.cursor.position.column.get());
        let (_, right) = self.horizontal_bounds_for_cursor();
        let right = usize::from(right);
        if start > right {
            return;
        }
        let count = usize::from(count.max(1)).min(right - start + 1);
        let grid = self.state.active_grid_mut();
        for column in (start..=right).rev() {
            grid.rows[row].cells[column] = if column >= start + count {
                grid.rows[row].cells[column - count].clone()
            } else {
                Cell::default()
            };
        }
        grid.repair_row(row);
    }

    fn delete_cells(&mut self, count: u16) {
        let row = usize::from(self.state.cursor.position.row.get());
        let start = usize::from(self.state.cursor.position.column.get());
        let (_, right) = self.horizontal_bounds_for_cursor();
        let right = usize::from(right);
        if start > right {
            return;
        }
        let count = usize::from(count.max(1)).min(right - start + 1);
        let grid = self.state.active_grid_mut();
        for column in start..=right {
            grid.rows[row].cells[column] = if column + count <= right {
                grid.rows[row].cells[column + count].clone()
            } else {
                Cell::default()
            };
        }
        grid.repair_row(row);
    }

    fn erase_cells(&mut self, count: u16, selective: bool) {
        let row = usize::from(self.state.cursor.position.row.get());
        let start = usize::from(self.state.cursor.position.column.get());
        let (_, right) = self.horizontal_bounds_for_cursor();
        let right = usize::from(right);
        if start > right {
            return;
        }
        let end = start
            .saturating_add(usize::from(count.max(1)) - 1)
            .min(right);
        self.erase_range(row, start, end, selective);
    }

    fn insert_lines(&mut self, count: u16) {
        let row = self.state.cursor.position.row.get();
        if row >= self.state.margins.top.get() && row <= self.state.margins.bottom.get() {
            self.scroll_down(row, count);
        }
    }

    fn delete_lines(&mut self, count: u16) {
        let row = self.state.cursor.position.row.get();
        if row >= self.state.margins.top.get() && row <= self.state.margins.bottom.get() {
            self.scroll_up(row, count);
        }
    }

    fn scroll_up(&mut self, first_row: u16, count: u16) {
        self.scroll(first_row, count, true);
    }

    fn scroll_down(&mut self, first_row: u16, count: u16) {
        self.scroll(first_row, count, false);
    }

    fn scroll(&mut self, first_row: u16, count: u16, up: bool) {
        let top = usize::from(first_row.max(self.state.margins.top.get()));
        let bottom = usize::from(self.state.margins.bottom.get());
        let left = usize::from(self.state.margins.left.get());
        let right = usize::from(self.state.margins.right.get());
        let count = usize::from(count.max(1)).min(bottom - top + 1);
        let grid = self.state.active_grid_mut();
        if up {
            for row in top..=bottom {
                for column in left..=right {
                    grid.rows[row].cells[column] = if row + count <= bottom {
                        grid.rows[row + count].cells[column].clone()
                    } else {
                        Cell::default()
                    };
                }
            }
        } else {
            for row in (top..=bottom).rev() {
                for column in left..=right {
                    grid.rows[row].cells[column] = if row >= top + count {
                        grid.rows[row - count].cells[column].clone()
                    } else {
                        Cell::default()
                    };
                }
            }
        }
        for row in top..=bottom {
            grid.rows[row].soft_wrapped = false;
            grid.repair_row(row);
        }
    }

    fn erase_line(&mut self, mode: EraseLine, selective: bool) {
        let row = usize::from(self.state.cursor.position.row.get());
        let column = usize::from(self.state.cursor.position.column.get());
        let right = usize::from(self.state.size.columns.get() - 1);
        let (start, end) = match mode {
            EraseLine::Right => (column, right),
            EraseLine::Left => (0, column),
            EraseLine::All => (0, right),
        };
        self.erase_range(row, start, end, selective);
    }

    fn erase_display(&mut self, mode: EraseDisplay, selective: bool) {
        let cursor_row = usize::from(self.state.cursor.position.row.get());
        let cursor_column = usize::from(self.state.cursor.position.column.get());
        let last_row = usize::from(self.state.size.rows.get() - 1);
        let last_column = usize::from(self.state.size.columns.get() - 1);
        for row in 0..=last_row {
            let range = match mode {
                EraseDisplay::Below if row < cursor_row => None,
                EraseDisplay::Below if row == cursor_row => Some((cursor_column, last_column)),
                EraseDisplay::Below => Some((0, last_column)),
                EraseDisplay::Above if row > cursor_row => None,
                EraseDisplay::Above if row == cursor_row => Some((0, cursor_column)),
                EraseDisplay::Above => Some((0, last_column)),
                EraseDisplay::All => Some((0, last_column)),
            };
            if let Some((start, end)) = range {
                self.erase_range(row, start, end, selective);
            }
        }
    }

    fn erase_range(&mut self, row: usize, start: usize, end: usize, selective: bool) {
        let grid = self.state.active_grid_mut();
        for column in start..=end {
            if !selective || !grid.rows[row].cells[column].protected {
                grid.rows[row].cells[column] = Cell::default();
            }
        }
        grid.repair_row(row);
        if grid.rows[row]
            .cells
            .iter()
            .all(|cell| cell == &Cell::default())
        {
            grid.rows[row].soft_wrapped = false;
        }
    }

    fn switch_buffer(&mut self, buffer: ScreenBuffer, clear: bool, home: bool) {
        self.state.active_buffer = buffer;
        if clear {
            self.state.active_grid_mut().clear();
        }
        if home {
            self.state.cursor = crate::CursorState::home(self.state.size);
        } else {
            self.state.cursor.pending_wrap = false;
        }
    }

    fn save_cursor(&mut self) {
        self.state.saved = Some(SavedCursorState {
            cursor: self.state.cursor,
            style: self.state.style,
            modes: self.state.modes,
            charsets: self.state.charsets,
            protected: self.state.protected,
        });
    }

    fn restore_cursor(&mut self) {
        if let Some(saved) = self.state.saved.clone() {
            self.state.cursor = saved.cursor;
            self.state.style = saved.style;
            self.state.modes = saved.modes;
            self.state.charsets = saved.charsets;
            self.state.protected = saved.protected;
        }
    }

    fn set_mode(&mut self, mode: FoundationMode, enabled: bool) {
        match mode {
            FoundationMode::AutoWrap => self.state.modes.auto_wrap = enabled,
            FoundationMode::Origin => self.state.modes.origin = enabled,
            FoundationMode::Insert => self.state.modes.insert = enabled,
            FoundationMode::Newline => self.state.modes.newline = enabled,
        }
        self.state.cursor.pending_wrap = false;
    }

    fn reset(&mut self) {
        self.state.primary.clear();
        self.state.alternate.clear();
        self.state.active_buffer = ScreenBuffer::Primary;
        self.state.cursor = crate::CursorState::home(self.state.size);
        self.state.margins = Margins::full(self.state.size);
        self.state.modes = ModeState {
            auto_wrap: true,
            ..ModeState::default()
        };
        self.state.tabs = crate::TabStops::every_eight(self.state.size.columns);
        self.state.charsets = crate::CharacterSetState::default();
        self.state.style = crate::CellStyle::default();
        self.state.protected = false;
        self.state.saved = None;
        self.state.synchronized_dirty = false;
        self.state.last_printed = None;
    }

    fn horizontal_bounds_for_cursor(&self) -> (u16, u16) {
        let column = self.state.cursor.position.column.get();
        let left = self.state.margins.left.get();
        let right = self.state.margins.right.get();
        if (left..=right).contains(&column) {
            (left, right)
        } else {
            (0, self.state.size.columns.get() - 1)
        }
    }
}

fn offset(value: u16, delta: i32) -> u16 {
    let value = i64::from(value) + i64::from(delta);
    value.clamp(0, i64::from(u16::MAX)) as u16
}
