use crate::{Cell, CellContent, CellPoint, CellWidth, Column, Damage, DamageSet, TerminalCore};

impl TerminalCore {
    pub(crate) fn print_ascii_run(
        &mut self,
        bytes: &[u8],
        damage: &mut DamageSet,
    ) -> Result<(), crate::ClusterError> {
        debug_assert!(bytes.iter().all(|byte| (0x20..=0x7e).contains(byte)));
        let damage_plan = crate::reducer_damage::DamagePlan::capture_metadata(self);
        let mut last = None;
        for &byte in bytes {
            let mapped = self.state.charsets.map(char::from(byte));
            let width = if mapped.is_ascii() {
                CellWidth::One
            } else {
                let mut encoded = [0; 4];
                crate::terminal_cluster_width(mapped.encode_utf8(&mut encoded))
            };
            let cluster = crate::Cluster::from_char(mapped, width, self.limits.cluster_bytes)?;
            self.print_cluster(cluster);
            last = Some((mapped, width));
        }
        if let Some((mapped, width)) = last {
            self.state.last_printed = Some(crate::Cluster::from_char(
                mapped,
                width,
                self.limits.cluster_bytes,
            )?);
        }
        self.prune_graphics();
        damage.push_coalesced(Damage::Full);
        damage_plan.finish(self, damage);
        Ok(())
    }

    pub(crate) fn print(&mut self, cluster: crate::Cluster) {
        self.state.last_printed = Some(cluster.clone());
        self.print_cluster(cluster);
    }

    fn print_cluster(&mut self, cluster: crate::Cluster) {
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

    pub(crate) fn append_cluster(&mut self, at: CellPoint, cluster: crate::Cluster) {
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
}
