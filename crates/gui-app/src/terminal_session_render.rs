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
            .sessions
            .iter_mut()
            .enumerate()
            .filter(|(_, slot)| !slot.hidden_after_close)
            .map(|(index, slot)| {
                let active = self.active_pending_id.is_none() && index == active_index;
                if active {
                    slot.status = state.status.clone();
                }
                let projected_lane = if active { &*state } else { &slot.parked_lane };
                let event_log_path = slot.session.event_log_path();
                slot.activity.refresh(&event_log_path);
                datum_gui_protocol::TerminalTabState {
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
                }
            })
            .chain(
                self.pending_spawns
                    .iter()
                    .filter(|pending| !pending.canceled)
                    .map(|pending| datum_gui_protocol::TerminalTabState {
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
                    }),
            )
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

    pub(crate) fn take_active_render_state(
        &mut self,
    ) -> Result<
        (
            datum_terminal_core::RenderSnapshot,
            Vec<datum_terminal_core::Damage>,
        ),
        crate::terminal_core_adapter::TerminalCoreAdapterError,
    > {
        self.sessions[self.active_index].core.take_render_state()
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
}
