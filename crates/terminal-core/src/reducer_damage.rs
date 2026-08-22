use crate::{
    CellContent, CellPoint, CellWidth, Damage, EraseDisplay, Margins, Row, ScreenAction,
    ScrollDirection, TerminalCore, TerminalSize,
};

pub(crate) struct DamagePlan {
    entries: [Option<Damage>; 5],
    len: usize,
    history: HistoryFingerprint,
    graphic_count: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct HistoryFingerprint {
    rows: usize,
    payload_bytes: usize,
    trimmed_rows: u64,
}

impl DamagePlan {
    pub(crate) fn capture(action: &ScreenAction, core: &TerminalCore) -> Self {
        let state = &core.state;
        let mut plan = Self {
            entries: [None; 5],
            len: 0,
            history: history_fingerprint(core),
            graphic_count: state.graphics.iter().len(),
        };
        classify(action, core, &mut plan);
        plan
    }

    pub(crate) fn capture_metadata(core: &TerminalCore) -> Self {
        Self {
            entries: [None; 5],
            len: 0,
            history: history_fingerprint(core),
            graphic_count: core.state.graphics.iter().len(),
        }
    }

    fn push(&mut self, damage: Damage) {
        debug_assert!(self.len < self.entries.len());
        self.entries[self.len] = Some(damage);
        self.len += 1;
    }

    pub(crate) fn finish(mut self, core: &TerminalCore, damage: &mut crate::DamageSet) {
        if self.history != history_fingerprint(core) {
            self.push(Damage::History);
        }
        if self.graphic_count != core.state.graphics.iter().len() {
            self.push(Damage::Graphics);
        }
        for entry in self.entries.into_iter().take(self.len).flatten() {
            damage.push_coalesced(entry);
        }
    }
}

fn history_fingerprint(core: &TerminalCore) -> HistoryFingerprint {
    let (rows, payload_bytes, trimmed_rows) = core.state.history.fingerprint();
    HistoryFingerprint {
        rows,
        payload_bytes,
        trimmed_rows,
    }
}

fn classify(action: &ScreenAction, core: &TerminalCore, damage: &mut DamagePlan) {
    let state = &core.state;
    let cursor = state.cursor.position;
    let margins = state.margins;
    let size = state.size;
    match action {
        ScreenAction::Print(cluster) => {
            damage.push(print_damage(core, cursor, cluster.width()));
            damage.push(Damage::Cursor);
        }
        ScreenAction::AppendCluster { at, cluster } => {
            damage.push(print_damage(core, *at, cluster.width()));
            damage.push(Damage::Cursor);
        }
        ScreenAction::Backspace
        | ScreenAction::CarriageReturn
        | ScreenAction::HorizontalTab
        | ScreenAction::SetCursor { .. }
        | ScreenAction::MoveCursor { .. }
        | ScreenAction::SetMargins(_)
        | ScreenAction::ResetMargins
        | ScreenAction::RestoreCursor => damage.push(Damage::Cursor),
        ScreenAction::LineFeed => {
            if cursor.row == margins.bottom {
                damage.push(scroll_damage(margins, ScrollDirection::Up, 1));
            }
            damage.push(Damage::Cursor);
        }
        ScreenAction::ReverseIndex => {
            if cursor.row == margins.top {
                damage.push(scroll_damage(margins, ScrollDirection::Down, 1));
            }
            damage.push(Damage::Cursor);
        }
        ScreenAction::InsertCells(_)
        | ScreenAction::DeleteCells(_)
        | ScreenAction::EraseCells(_)
        | ScreenAction::EraseLine { .. } => damage.push(Damage::Row(cursor.row)),
        ScreenAction::InsertLines(count) => damage.push(scroll_damage(
            Margins {
                top: cursor.row,
                ..margins
            },
            ScrollDirection::Down,
            *count,
        )),
        ScreenAction::DeleteLines(count) => damage.push(scroll_damage(
            Margins {
                top: cursor.row,
                ..margins
            },
            ScrollDirection::Up,
            *count,
        )),
        ScreenAction::ScrollUp(count) => {
            damage.push(scroll_damage(margins, ScrollDirection::Up, *count));
        }
        ScreenAction::ScrollDown(count) => {
            damage.push(scroll_damage(margins, ScrollDirection::Down, *count));
        }
        ScreenAction::EraseDisplay { mode, .. } => match mode {
            EraseDisplay::All => damage.push(Damage::Full),
            EraseDisplay::Below => damage.push(Damage::Rows {
                first: cursor.row,
                last: Row::new(size.rows.get() - 1, size.rows).expect("last row is valid"),
            }),
            EraseDisplay::Above => damage.push(Damage::Rows {
                first: Row::new(0, size.rows).expect("first row is valid"),
                last: cursor.row,
            }),
        },
        ScreenAction::SwitchBuffer { .. } | ScreenAction::Reset => damage.push(Damage::Full),
        ScreenAction::SaveCursor
        | ScreenAction::SetMode { .. }
        | ScreenAction::SetStyle(_)
        | ScreenAction::SetProtection(_) => {}
    }
}

fn print_damage(core: &TerminalCore, at: CellPoint, new_width: CellWidth) -> Damage {
    let state = &core.state;
    if state.cursor.pending_wrap || state.modes.insert {
        return Damage::Full;
    }
    let right = horizontal_right_margin(at, state.margins, state.size);
    if new_width == CellWidth::Two && at.column.get() >= right && state.modes.auto_wrap {
        return Damage::Full;
    }
    let existing_is_wide = state
        .active_grid()
        .rows
        .get(usize::from(at.row.get()))
        .and_then(|row| row.cells.get(usize::from(at.column.get())))
        .is_some_and(|cell| match &cell.content {
            CellContent::Cluster(cluster) => cluster.width() == CellWidth::Two,
            CellContent::Continuation { .. } => true,
            CellContent::Empty => false,
        });
    if new_width == CellWidth::Two || existing_is_wide {
        Damage::Row(at.row)
    } else {
        Damage::Cell(at)
    }
}

fn horizontal_right_margin(at: CellPoint, margins: Margins, size: TerminalSize) -> u16 {
    if at.row >= margins.top && at.row <= margins.bottom {
        margins.right.get()
    } else {
        size.columns.get() - 1
    }
}

fn scroll_damage(margins: Margins, direction: ScrollDirection, count: u16) -> Damage {
    let height = margins
        .bottom
        .get()
        .saturating_sub(margins.top.get())
        .saturating_add(1);
    Damage::Scroll {
        first_row: margins.top,
        last_row: margins.bottom,
        first_column: margins.left,
        last_column: margins.right,
        direction,
        count: count.max(1).min(height),
    }
}
