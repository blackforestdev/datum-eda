use super::{TerminalSessionRegistry, terminate_terminal_session};
use anyhow::Result;
use datum_gui_protocol::TerminalLaneState;
use std::time::Instant;

impl TerminalSessionRegistry {
    pub(crate) fn terminate_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        self.sessions[self.active_index].close_confirmation_armed = false;
        self.sessions[self.active_index]
            .close_confirmation_input
            .clear();
        terminate_terminal_session(self.active(), state)?;
        self.sessions[self.active_index].status = state.status.clone();
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(crate) fn close_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        let can_remove = self.active().presentation_complete();
        if !can_remove {
            let slot = &mut self.sessions[self.active_index];
            if !slot.close_confirmation_armed {
                slot.close_confirmation_armed = true;
                slot.close_confirmation_input.clear();
                state.status = "close terminal? type yes + Enter, click TERMINATE, or repeat Ctrl+Shift+W; Escape cancels".to_string();
                slot.status = state.status.clone();
                self.sync_lane_tabs(state);
                return Ok(());
            }
            return Ok(());
        }
        if self.sessions.len() == 1 {
            self.sessions[0].hidden_after_close = true;
            let mut discarded = TerminalLaneState::default();
            state.swap_session_projection(&mut discarded);
            state.status = "no terminal session; use +NEW".to_string();
            self.sync_lane_tabs(state);
            return Ok(());
        }
        self.sessions.remove(self.active_index);
        if self.active_index >= self.sessions.len() {
            self.active_index = self.sessions.len() - 1;
        }
        self.sessions[self.active_index].attached = true;
        if self.projection_managed {
            let mut discarded = TerminalLaneState::default();
            state.swap_session_projection(&mut discarded);
            state.swap_session_projection(&mut self.sessions[self.active_index].parked_lane);
        } else {
            state.status = self.sessions[self.active_index].status.clone();
        }
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(crate) fn active_close_confirmation_armed(&self) -> bool {
        self.sessions[self.active_index].close_confirmation_armed
    }

    pub(crate) fn handle_close_confirmation_input(
        &mut self,
        bytes: &[u8],
        state: &mut TerminalLaneState,
    ) {
        let mut confirm = false;
        {
            let slot = &mut self.sessions[self.active_index];
            match bytes {
                b"\x1b" => {
                    slot.close_confirmation_armed = false;
                    slot.close_confirmation_input.clear();
                    state.status = "running".to_string();
                    slot.status = state.status.clone();
                }
                b"\x7f" => {
                    slot.close_confirmation_input.pop();
                }
                b"\r" | b"\n" => {
                    if slot.close_confirmation_input == "yes" {
                        confirm = true;
                    } else {
                        slot.close_confirmation_input.clear();
                    }
                }
                bytes if bytes.len() <= 16 => {
                    if let Ok(text) = std::str::from_utf8(bytes)
                        && text
                            .chars()
                            .all(|character| character.is_ascii_alphabetic())
                        && slot.close_confirmation_input.len() + text.len() <= 16
                    {
                        slot.close_confirmation_input.push_str(text);
                    }
                }
                _ => {}
            }
            if slot.close_confirmation_armed && !confirm {
                state.status = format!(
                    "close terminal? type yes + Enter [{}]; Escape cancels",
                    slot.close_confirmation_input
                );
                slot.status = state.status.clone();
            }
        }
        if confirm {
            let _ = self.confirm_close_active(state);
        }
    }

    pub(crate) fn confirm_close_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        self.sessions[self.active_index].remove_when_closed = true;
        self.terminate_active(state)
    }

    pub(crate) fn force_kill_active(&self) {
        self.active().force_kill();
    }

    pub(crate) fn terminate_all_by(&mut self, deadline: Instant) {
        for slot in &mut self.sessions {
            slot.close_confirmation_armed = false;
            slot.close_confirmation_input.clear();
            let _ = slot.session.terminate_by(deadline);
        }
    }

    pub(crate) fn retry_failed_terminations(&mut self) {
        for slot in &mut self.sessions {
            if slot.session.shutdown_snapshot().is_some_and(|snapshot| {
                snapshot.phase == crate::terminal_transport::ShutdownPhase::Failed
            }) {
                slot.termination_failure_reported = false;
                let _ = slot.session.terminate();
            }
        }
    }

    pub(crate) fn retry_nonclosed_terminations_by(&mut self, deadline: Instant) {
        for slot in &mut self.sessions {
            if slot.session.shutdown_snapshot().is_some_and(|snapshot| {
                snapshot.phase != crate::terminal_transport::ShutdownPhase::Closed
            }) {
                slot.termination_failure_reported = false;
                slot.session.retry_termination_by(deadline);
            }
        }
    }

    pub(crate) fn all_sessions_closed(&self) -> bool {
        self.sessions.iter().all(|slot| {
            slot.session.shutdown_snapshot().is_some_and(|snapshot| {
                snapshot.phase == crate::terminal_transport::ShutdownPhase::Closed
                    && snapshot.leader_reaped
                    && slot.session.presentation_complete()
            })
        })
    }

    pub(crate) fn shutdown_failure_summary(&self) -> String {
        self.sessions
            .iter()
            .filter_map(|slot| {
                let snapshot = slot.session.shutdown_snapshot()?;
                (snapshot.phase != crate::terminal_transport::ShutdownPhase::Closed).then(|| {
                    let survivors = snapshot
                        .surviving_processes
                        .iter()
                        .map(|identity| {
                            format!(
                                "pid={} pgid={} sid={}",
                                identity.pid, identity.process_group_id, identity.session_id
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{}: {}{}{}",
                        slot.label,
                        snapshot
                            .failure
                            .as_deref()
                            .unwrap_or("teardown deadline exceeded"),
                        if survivors.is_empty() { "" } else { "; " },
                        survivors
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    pub(crate) fn remove_presented_closed(&mut self, state: &mut TerminalLaneState) -> bool {
        let targets = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.remove_when_closed && slot.session.presentation_complete())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return false;
        }
        for index in targets.into_iter().rev() {
            if self.sessions.len() == 1 {
                self.sessions[0].remove_when_closed = false;
                self.sessions[0].hidden_after_close = true;
                let mut discarded = TerminalLaneState::default();
                state.swap_session_projection(&mut discarded);
                state.status = "no terminal session; use +NEW".to_string();
                continue;
            }
            let was_active = index == self.active_index;
            self.sessions.remove(index);
            if index < self.active_index || self.active_index >= self.sessions.len() {
                self.active_index = self
                    .active_index
                    .saturating_sub(1)
                    .min(self.sessions.len() - 1);
            }
            if was_active {
                let mut discarded = TerminalLaneState::default();
                state.swap_session_projection(&mut discarded);
                state.swap_session_projection(&mut self.sessions[self.active_index].parked_lane);
            }
        }
        self.sync_lane_tabs(state);
        true
    }
}
