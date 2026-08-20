use crate::grid::GridAllocationError;
use crate::{
    Cell, CellContent, CellPoint, CellWidth, Column, DamageSet, EraseDisplay, EraseLine,
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
    LogicalLineIdExhausted,
}

impl fmt::Display for ScreenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(error) => error.fmt(formatter),
            Self::CellCountOverflow => formatter.write_str("terminal screen cell count overflowed"),
            Self::Allocation => formatter.write_str("terminal screen allocation failed"),
            Self::LogicalLineIdExhausted => {
                formatter.write_str("terminal logical line identity exhausted")
            }
        }
    }
}

impl Error for ScreenError {}

impl From<GridAllocationError> for ScreenError {
    fn from(value: GridAllocationError) -> Self {
        match value {
            GridAllocationError::Limit(error) => Self::Limit(error),
            GridAllocationError::Allocation => Self::Allocation,
            GridAllocationError::LogicalLineIdExhausted => Self::LogicalLineIdExhausted,
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

    pub(crate) const fn with_damage(damage: DamageSet) -> Self {
        Self { damage }
    }
}

impl TerminalCore {
    pub fn reduce(&mut self, action: ScreenAction) -> Result<Reduction, ScreenError> {
        let damage_plan = crate::reducer_damage::DamagePlan::capture(&action, self);
        let clear_graphics = match &action {
            ScreenAction::Reset => Some(None),
            ScreenAction::SwitchBuffer {
                buffer,
                clear: true,
                ..
            } => Some(Some(*buffer)),
            ScreenAction::EraseDisplay {
                mode: EraseDisplay::All,
                selective: false,
            } => Some(Some(self.state.active_buffer)),
            _ => None,
        };
        self.apply_action(action);
        match clear_graphics {
            Some(None) => self.state.graphics.clear(),
            Some(Some(buffer)) => self.state.graphics.clear_buffer(buffer),
            None => self.prune_graphics(),
        }
        let mut damage = DamageSet::new(self.limits.pending_damage);
        for entry in damage_plan.finish(self) {
            damage.push_coalesced(entry);
        }
        Ok(Reduction { damage })
    }

    fn apply_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::Print(cluster) => self.print(cluster),
            ScreenAction::AppendCluster { at, cluster } => self.append_cluster(at, cluster),
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
        let anchor = self.state.cursor.position;
        let cell = Cell {
            content: CellContent::Cluster(cluster),
            style: self.state.style,
            hyperlink: self.state.current_hyperlink,
            protected: self.state.protected,
        };
        self.state.active_grid_mut().set_cluster(row, column, cell);
        self.state.grapheme_anchor = Some(anchor);

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

    fn append_cluster(&mut self, at: CellPoint, cluster: crate::Cluster) {
        let row = usize::from(at.row.get());
        let column = usize::from(at.column.get());
        let Some(existing) = self
            .state
            .active_grid()
            .rows
            .get(row)
            .and_then(|row| row.cells.get(column))
            .cloned()
        else {
            self.state.grapheme_anchor = None;
            return;
        };
        if !matches!(existing.content, CellContent::Cluster(_)) {
            self.state.grapheme_anchor = None;
            return;
        }
        let (_, right) = self.horizontal_bounds_for_cursor();
        if cluster.width() == CellWidth::Two && at.column.get() >= right {
            if !self.state.modes.auto_wrap {
                return;
            }
            self.state.active_grid_mut().clear_cluster_at(row, column);
            self.state.cursor.position = at;
            self.state.cursor.pending_wrap = false;
            self.wrap_line();
        } else {
            self.state.cursor.position = at;
            self.state.cursor.pending_wrap = false;
        }
        let anchor = self.state.cursor.position;
        let cell = Cell {
            content: CellContent::Cluster(cluster.clone()),
            style: existing.style,
            hyperlink: existing.hyperlink,
            protected: existing.protected,
        };
        self.state.active_grid_mut().set_cluster(
            usize::from(anchor.row.get()),
            anchor.column,
            cell,
        );
        self.state.last_printed = Some(cluster.clone());
        self.state.grapheme_anchor = Some(anchor);
        let width = if cluster.width() == CellWidth::Two {
            2
        } else {
            1
        };
        let (_, right) = self.horizontal_bounds_for_cursor();
        let end = anchor.column.get().saturating_add(width - 1);
        if end >= right {
            self.state.cursor.position.column =
                Column::new(right, self.state.size.columns).unwrap_or(anchor.column);
            self.state.cursor.pending_wrap = self.state.modes.auto_wrap;
        } else {
            self.state.cursor.position.column =
                Column::new(end + 1, self.state.size.columns).unwrap_or(anchor.column);
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
        let source = &self.state.active_grid().rows[usize::from(row)];
        let continuation = source.soft_wrapped.then_some((
            source.logical_line,
            source.cluster_start.saturating_add(source.cluster_count()),
        ));
        if row == self.state.margins.bottom.get() {
            self.scroll(
                self.state.margins.top.get(),
                1,
                true,
                continuation.is_some(),
            );
        } else {
            self.set_cursor(
                row.saturating_add(1),
                self.state.cursor.position.column.get(),
            );
            if let Some((logical_line, cluster_start)) = continuation {
                let target = &mut self.state.active_grid_mut().rows[usize::from(row + 1)];
                target.logical_line = logical_line;
                target.cluster_start = cluster_start;
            }
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
        self.scroll(first_row, count, true, false);
    }

    fn scroll_down(&mut self, first_row: u16, count: u16) {
        self.scroll(first_row, count, false, false);
    }

    fn scroll(&mut self, first_row: u16, count: u16, up: bool, continue_bottom: bool) {
        let top = usize::from(first_row.max(self.state.margins.top.get()));
        let bottom = usize::from(self.state.margins.bottom.get());
        let left = usize::from(self.state.margins.left.get());
        let right = usize::from(self.state.margins.right.get());
        let count = usize::from(count.max(1)).min(bottom - top + 1);
        let full_width = left == 0 && right + 1 == usize::from(self.state.size.columns.get());
        if full_width {
            self.scroll_full_rows(top, bottom, count, up, continue_bottom);
            return;
        }
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

    fn scroll_full_rows(
        &mut self,
        top: usize,
        bottom: usize,
        count: usize,
        up: bool,
        continue_bottom: bool,
    ) {
        let columns = self.state.size.columns;
        let record_history = up
            && top == 0
            && bottom + 1 == usize::from(self.state.size.rows.get())
            && self.state.active_buffer == ScreenBuffer::Primary;
        for _ in 0..count {
            let fresh_line = crate::history::next_line_id(&mut self.state.next_logical_line);
            let grid = self.state.active_grid_mut();
            let displaced = if up {
                let displaced = grid.rows.remove(top);
                let previous = grid.rows.get(bottom.saturating_sub(1));
                let mut blank = crate::grid::GridRow::blank(columns, fresh_line);
                if let Some(previous) = previous.filter(|row| continue_bottom && row.soft_wrapped) {
                    blank.logical_line = previous.logical_line;
                    blank.cluster_start = previous
                        .cluster_start
                        .saturating_add(previous.cluster_count());
                }
                grid.rows.insert(bottom, blank);
                displaced
            } else {
                let displaced = grid.rows.remove(bottom);
                grid.rows
                    .insert(top, crate::grid::GridRow::blank(columns, fresh_line));
                displaced
            };
            if record_history {
                self.state.history.push(displaced);
            }
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
        if self.state.active_buffer != buffer {
            self.state.selection = None;
        }
        self.state.active_buffer = buffer;
        if clear {
            match buffer {
                ScreenBuffer::Primary => self
                    .state
                    .primary
                    .clear_with_new_lines(&mut self.state.next_logical_line),
                ScreenBuffer::Alternate => self
                    .state
                    .alternate
                    .clear_with_new_lines(&mut self.state.next_logical_line),
            }
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
        self.state
            .primary
            .clear_with_new_lines(&mut self.state.next_logical_line);
        self.state
            .alternate
            .clear_with_new_lines(&mut self.state.next_logical_line);
        self.state.active_buffer = ScreenBuffer::Primary;
        self.state.cursor = crate::CursorState::home(self.state.size);
        self.state.margins = Margins::full(self.state.size);
        self.state.modes = ModeState {
            auto_wrap: true,
            sixel_scrolling: true,
            ..ModeState::default()
        };
        self.state.tabs = crate::TabStops::every_eight(self.state.size.columns);
        self.state.charsets = crate::CharacterSetState::default();
        self.state.style = crate::CellStyle::default();
        self.state.protected = false;
        self.state.saved = None;
        self.state.synchronized_dirty = false;
        self.state.last_printed = None;
        self.state.grapheme_anchor = None;
        self.state.history.clear();
        self.state.selection = None;
        self.state.current_hyperlink = None;
        self.state.hyperlinks.clear();
        self.state.shell_mark = None;
        self.state.progress = crate::ProgressState::Clear;
        self.state.sixel_colors = crate::SixelColorRegisters::default();
    }

    pub(crate) fn prune_graphics(&mut self) {
        let history = &self.state.history;
        let primary = &self.state.primary;
        let alternate = &self.state.alternate;
        self.state.graphics.retain(|placement| {
            let rows = match placement.buffer() {
                ScreenBuffer::Primary => &primary.rows,
                ScreenBuffer::Alternate => &alternate.rows,
            };
            (placement.buffer() == ScreenBuffer::Primary && history.contains(placement.anchor()))
                || rows
                    .iter()
                    .any(|row| crate::history::row_contains(row, placement.anchor()))
        });
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
