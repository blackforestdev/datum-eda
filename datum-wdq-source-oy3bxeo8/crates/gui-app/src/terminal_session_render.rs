//! Renderer-facing snapshot and surface-size boundary for terminal sessions.

use super::*;

pub(super) fn terminal_tab_label(
    fallback: &str,
    explicit: bool,
    terminal_title: Option<&str>,
    progress: datum_gui_protocol::TerminalProgressState,
    has_notification: bool,
) -> String {
    let mut label = if explicit {
        fallback.to_string()
    } else {
        terminal_title
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| fallback.to_string())
    };
    let suffix = match progress {
        datum_gui_protocol::TerminalProgressState::Clear => has_notification.then_some(" · !"),
        datum_gui_protocol::TerminalProgressState::Set { percent } => {
            label.push_str(&format!(" · {percent}%"));
            None
        }
        datum_gui_protocol::TerminalProgressState::Error { percent } => {
            label.push_str(&format!(" · error {percent}%"));
            None
        }
        datum_gui_protocol::TerminalProgressState::Paused { percent } => {
            label.push_str(&format!(" · paused {percent}%"));
            None
        }
        datum_gui_protocol::TerminalProgressState::Indeterminate => Some(" · …"),
    };
    if let Some(suffix) = suffix {
        label.push_str(suffix);
    }
    label
}

impl TerminalSessionRegistry {
    pub(crate) fn sync_lane_tabs(&mut self, state: &mut TerminalLaneState) {
        self.sync_terminal_tab_layouts(state);
        let active_index = self.active_index;
        state.active_session_id = self.active_pending_id.clone().or_else(|| {
            (!self.sessions[self.active_index].hidden_after_close)
                .then(|| self.active().session_id().to_string())
        });
        let tabs = self
            .terminal_tabs
            .clone()
            .into_iter()
            .filter_map(|tab| {
                let session_id = tab.focused_session_id;
                if let Some(pending) = self.pending_spawns.iter().find(|pending| {
                    !pending.canceled
                        && pending.placement.is_new_tab()
                        && pending.pending_id == session_id
                }) {
                    return Some(datum_gui_protocol::TerminalTabState {
                        session_id: pending.pending_id.clone(),
                        previous_session_id: None,
                        label: pending.label.clone(),
                        event_log_path: String::new(),
                        activity_event_count: 0,
                        activity_summary: vec!["starting terminal session".to_string()],
                        active: self.active_pending_id.as_deref() == Some(&pending.pending_id),
                        attached: true,
                        status: "starting".to_string(),
                        restart_count: 0,
                        unread_output: false,
                        unread_bell_count: 0,
                    });
                }
                let index = self
                    .sessions
                    .iter()
                    .position(|slot| slot.session.session_id() == session_id)?;
                let slot = &mut self.sessions[index];
                if slot.hidden_after_close {
                    return None;
                }
                let active = self.active_pending_id.is_none() && index == active_index;
                if active {
                    slot.status = state.status.clone();
                }
                let projected_lane = if active { &*state } else { &slot.parked_lane };
                let event_log_path = slot.session.event_log_path();
                slot.activity.refresh(&event_log_path);
                Some(datum_gui_protocol::TerminalTabState {
                    session_id: slot.session.session_id().to_string(),
                    previous_session_id: slot.previous_session_id.clone(),
                    label: terminal_tab_label(
                        &slot.label,
                        slot.label_is_explicit,
                        projected_lane.title.as_deref(),
                        projected_lane.progress,
                        projected_lane.latest_notification.is_some(),
                    ),
                    event_log_path: event_log_path.display().to_string(),
                    activity_event_count: slot.activity.event_count(),
                    activity_summary: slot.activity.summary_lines(2).unwrap_or_else(|err| {
                        vec![format!(
                            "activity summary unavailable for {}: {err}",
                            event_log_path.display()
                        )]
                    }),
                    active,
                    attached: slot.attached,
                    status: slot.status.clone(),
                    restart_count: slot.restart_count,
                    unread_output: !active && slot.unread_output,
                    unread_bell_count: if active {
                        0
                    } else {
                        projected_lane
                            .bell_count
                            .saturating_sub(slot.seen_bell_count)
                    },
                })
            })
            .collect::<Vec<_>>();
        if let Some(active_tab) = tabs.iter().find(|tab| tab.active) {
            state.activity_summary = active_tab.activity_summary.clone();
        }
        if self.active_pending_id.is_none() {
            let active_slot = &self.sessions[self.active_index];
            state.columns = active_slot.columns;
            state.rows = active_slot.rows;
        }
        state.tabs = tabs;
    }

    pub(crate) fn active_render_row_count(&self) -> usize {
        if self.active_pending_id.is_some() {
            return 0;
        }
        self.sessions[self.active_index]
            .core
            .render_row_count()
            .unwrap_or(0)
    }

    pub(crate) fn take_active_tab_render_states(
        &mut self,
        active_lane: &TerminalLaneState,
    ) -> Result<
        Vec<datum_gui_render::TerminalPaneRenderState>,
        crate::terminal_core_adapter::TerminalCoreAdapterError,
    > {
        if self.active_pending_id.is_some() {
            return Ok(Vec::new());
        }
        let active_session_id = self.active().session_id().to_string();
        let tab = self
            .terminal_tabs
            .iter()
            .find(|tab| tab.root.contains_session(&active_session_id));
        let focused_session_id = tab
            .map(|tab| tab.focused_session_id.clone())
            .unwrap_or_else(|| active_session_id.clone());
        let session_ids = tab
            .map(|tab| {
                tab.root
                    .session_ids()
                    .into_iter()
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![active_session_id]);
        let mut panes = Vec::with_capacity(session_ids.len());
        for session_id in session_ids {
            let index = self
                .sessions
                .iter()
                .position(|slot| slot.session.session_id() == session_id)
                .expect("terminal split leaf must name an owned session");
            let lane = if index == self.active_index {
                active_lane.clone()
            } else {
                self.sessions[index].parked_lane.clone()
            };
            let (snapshot, damage) = self.sessions[index].core.take_render_state()?;
            panes.push(datum_gui_render::TerminalPaneRenderState {
                focused: session_id == focused_session_id,
                session_id,
                lane,
                snapshot,
                damage,
            });
        }
        Ok(panes)
    }

    #[cfg(test)]
    pub(crate) fn resize_active(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.resize_active_surface(cols, rows, 0, 0)
    }

    pub(crate) fn resize_active_surface(
        &mut self,
        cols: u16,
        rows: u16,
        pixel_width: u32,
        pixel_height: u32,
    ) -> Result<()> {
        if self.active_pending_id.is_some() {
            return Ok(());
        }
        let slot = &mut self.sessions[self.active_index];
        let cols = cols.max(1);
        let rows = rows.max(1);
        if slot.columns != cols || slot.rows != rows {
            slot.session.resize(cols, rows)?;
            slot.columns = cols;
            slot.rows = rows;
        }
        slot.core.resize(cols, rows, pixel_width, pixel_height)?;
        Ok(())
    }

    pub(crate) fn resize_active_tab_surfaces(
        &mut self,
        panes: &[datum_gui_viewport::TerminalPaneGeometry],
    ) -> Result<()> {
        if self.active_pending_id.is_some() {
            return Ok(());
        }
        for pane in panes {
            let Some(slot) = self
                .sessions
                .iter_mut()
                .find(|slot| slot.session.session_id() == pane.session_id)
            else {
                continue;
            };
            let geometry = pane.geometry;
            let cols = geometry.columns.max(1);
            let rows = geometry.rows.max(1);
            if slot.columns != cols || slot.rows != rows {
                slot.session.resize(cols, rows)?;
                slot.columns = cols;
                slot.rows = rows;
            }
            slot.core.resize(
                cols,
                rows,
                geometry.screen.width.round() as u32,
                geometry.screen.height.round() as u32,
            )?;
        }
        Ok(())
    }
}
