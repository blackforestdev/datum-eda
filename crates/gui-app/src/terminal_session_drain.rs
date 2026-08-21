use super::{
    TerminalEvent, TerminalSessionRegistry, TerminalSessionSlot, mark_terminal_session_exit,
};
use crate::{
    terminal_core_adapter::TerminalCoreAdapterUpdate,
    terminal_session_events::{
        record_terminal_exit_event, record_terminal_output_event,
        record_terminal_termination_failure_event,
    },
    terminal_transport::{GUI_DRAIN_BYTE_LIMIT, GUI_DRAIN_EVENT_LIMIT},
};
use datum_gui_protocol::TerminalLaneState;

#[derive(Default)]
pub(crate) struct TerminalDrainReport {
    pub(crate) events: usize,
    pub(crate) output_events: usize,
    pub(crate) output_bytes: usize,
    pub(crate) active_projection_changed: bool,
    pub(crate) tabs_changed: bool,
    pub(crate) pending: bool,
    pub(crate) notices: Vec<String>,
    pub(crate) clipboard_requests: Vec<TerminalClipboardWriteRequest>,
    pub(crate) notifications: Vec<TerminalNotificationRequest>,
    #[cfg(test)]
    serviced: Vec<(usize, &'static str, usize)>,
    #[cfg(test)]
    output_batches: usize,
}

pub(crate) struct TerminalClipboardWriteRequest {
    pub(crate) session_id: String,
    pub(crate) selection: datum_terminal_core::ClipboardSelection,
    pub(crate) encoded_contents: Vec<u8>,
}

pub(crate) struct TerminalNotificationRequest {
    pub(crate) session_id: String,
    pub(crate) text: String,
}

fn flush_output_batch(
    sessions: &mut [TerminalSessionSlot],
    active_index: Option<usize>,
    active_lane: &mut TerminalLaneState,
    pending: &mut [Vec<u8>],
    report: &mut TerminalDrainReport,
    index: usize,
) {
    let bytes = &mut pending[index];
    if bytes.is_empty() {
        return;
    }
    let slot = &mut sessions[index];
    debug_assert_eq!(slot.core.session_id(), slot.session.session_id());
    debug_assert_eq!(slot.core.context_id(), slot.session.context_id);
    let _ = record_terminal_output_event(&slot.session, bytes);
    let is_active = active_index == Some(index);
    let lane = if is_active {
        &mut *active_lane
    } else {
        &mut slot.parked_lane
    };
    lane.latest_notification = None;
    match slot.core.apply_output(lane, bytes) {
        Ok(update) => consume_core_update(&slot.session, lane, report, update),
        Err(error) => report
            .notices
            .push(format!("terminal core output failed: {error}")),
    }
    #[cfg(test)]
    report.serviced.push((index, "apply", bytes.len()));
    bytes.clear();
    report.active_projection_changed |= is_active;
    report.tabs_changed = true;
    #[cfg(test)]
    {
        report.output_batches += 1;
    }
}

fn consume_core_update(
    session: &super::TerminalSession,
    lane: &mut TerminalLaneState,
    report: &mut TerminalDrainReport,
    update: TerminalCoreAdapterUpdate,
) {
    for response in update.replies {
        if let Err(error) = session.write_bytes(&response) {
            report
                .notices
                .push(format!("terminal status response failed: {error}"));
        }
    }
    for error in update.semantic_errors {
        report
            .notices
            .push(format!("terminal core semantic limit/error: {error}"));
    }
    for event in update.events {
        match event {
            datum_terminal_core::CoreEvent::LimitReached(kind) => report
                .notices
                .push(format!("terminal core {:?} limit reached", kind).to_lowercase()),
            datum_terminal_core::CoreEvent::Notification(text) => {
                let text = text.as_str().to_owned();
                lane.latest_notification = Some(text.clone());
                report.notifications.push(TerminalNotificationRequest {
                    session_id: session.session_id().to_string(),
                    text,
                });
            }
            datum_terminal_core::CoreEvent::ClipboardRequest {
                selection,
                encoded_contents,
            } => report
                .clipboard_requests
                .push(TerminalClipboardWriteRequest {
                    session_id: session.session_id().to_string(),
                    selection,
                    encoded_contents: encoded_contents.as_slice().to_vec(),
                }),
            _ => {}
        }
    }
}

impl TerminalSessionRegistry {
    pub(crate) fn drain_all(&mut self, active_lane: &mut TerminalLaneState) -> TerminalDrainReport {
        let mut report = TerminalDrainReport::default();
        if self.sessions.is_empty() {
            return report;
        }
        let visible_active_index = self
            .active_pending_id
            .is_none()
            .then_some(self.active_index);
        for (index, slot) in self.sessions.iter_mut().enumerate() {
            let Some(snapshot) = slot.session.shutdown_snapshot() else {
                continue;
            };
            let next = match snapshot.phase {
                crate::terminal_transport::ShutdownPhase::Running => continue,
                crate::terminal_transport::ShutdownPhase::Hup => {
                    "terminating (HUP grace)".to_string()
                }
                crate::terminal_transport::ShutdownPhase::Term => {
                    "terminating (TERM grace)".to_string()
                }
                crate::terminal_transport::ShutdownPhase::Kill => {
                    "terminating (KILL verification)".to_string()
                }
                crate::terminal_transport::ShutdownPhase::Closed => slot
                    .exact_exit_status
                    .clone()
                    .unwrap_or_else(|| "closed".to_string()),
                crate::terminal_transport::ShutdownPhase::Failed => {
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
                    let message = format!(
                        "termination failed: {}{}{}",
                        snapshot.failure.as_deref().unwrap_or("unknown failure"),
                        if survivors.is_empty() {
                            ""
                        } else {
                            "; survivors: "
                        },
                        survivors
                    );
                    if !slot.termination_failure_reported {
                        match record_terminal_termination_failure_event(
                            &slot.session,
                            snapshot.failure.as_deref().unwrap_or("unknown failure"),
                            &snapshot.surviving_processes,
                        ) {
                            Ok(()) => slot.termination_failure_reported = true,
                            Err(error) => report.notices.push(format!(
                                "persist terminal termination failure evidence failed: {error}"
                            )),
                        }
                    }
                    message
                }
            };
            if slot.status != next {
                slot.status = next.clone();
                let lane = if visible_active_index == Some(index) {
                    &mut *active_lane
                } else {
                    &mut slot.parked_lane
                };
                lane.status = next;
                report.tabs_changed = true;
                report.active_projection_changed |= visible_active_index == Some(index);
            }
        }
        let mut idle_visits = 0usize;
        let mut output_events = 0usize;
        let mut pending_output = (0..self.sessions.len())
            .map(|_| Vec::new())
            .collect::<Vec<_>>();
        while output_events < GUI_DRAIN_EVENT_LIMIT && idle_visits < self.sessions.len() {
            let control = (0..self.sessions.len()).find_map(|offset| {
                let index = (self.next_drain_index + offset) % self.sessions.len();
                self.sessions[index]
                    .session
                    .try_recv_control_event()
                    .map(|event| (index, event))
            });
            let index = control
                .as_ref()
                .map_or(self.next_drain_index % self.sessions.len(), |(index, _)| {
                    *index
                });
            // A final exit/error must never overtake bytes already dequeued in
            // this turn. Flush accumulated per-session output before handling
            // any control event; ordinary tiny-chunk output remains one
            // parse/style/log operation per touched session per turn.
            if let Some((control_index, _)) = control.as_ref() {
                flush_output_batch(
                    &mut self.sessions,
                    visible_active_index,
                    active_lane,
                    &mut pending_output,
                    &mut report,
                    *control_index,
                );
            }
            let remaining = GUI_DRAIN_BYTE_LIMIT.saturating_sub(report.output_bytes);
            let event = control.map(|(_, event)| event).or_else(|| {
                (remaining > 0)
                    .then(|| self.sessions[index].session.try_recv_output(remaining))
                    .flatten()
                    .map(TerminalEvent::Output)
            });
            let Some(event) = event else {
                self.next_drain_index = (index + 1) % self.sessions.len();
                idle_visits += 1;
                continue;
            };
            idle_visits = 0;
            self.next_drain_index = (index + 1) % self.sessions.len();
            report.events += 1;
            let is_active = visible_active_index == Some(index);
            match event {
                TerminalEvent::Output(bytes) => {
                    #[cfg(test)]
                    report.serviced.push((index, "output", bytes.len()));
                    output_events += 1;
                    report.output_events += 1;
                    report.output_bytes += bytes.len();
                    pending_output[index].extend_from_slice(&bytes);
                }
                TerminalEvent::Exited(code) => {
                    let slot = &mut self.sessions[index];
                    let lane = if is_active {
                        &mut *active_lane
                    } else {
                        &mut slot.parked_lane
                    };
                    match slot.core.finish(lane) {
                        Ok(update) => consume_core_update(&slot.session, lane, &mut report, update),
                        Err(error) => report
                            .notices
                            .push(format!("terminal core finish failed: {error}")),
                    }
                    #[cfg(test)]
                    report.serviced.push((index, "control", 0));
                    let _ = mark_terminal_session_exit(&slot.session, code);
                    let _ = record_terminal_exit_event(&slot.session, code);
                    slot.status = match code {
                        crate::terminal_transport::TerminalExitStatus::Code(code) => {
                            format!("exited {code}")
                        }
                        crate::terminal_transport::TerminalExitStatus::Signal {
                            signal,
                            core_dumped,
                        } => format!(
                            "terminated by signal {signal}{}",
                            if core_dumped { " (core dumped)" } else { "" }
                        ),
                    };
                    slot.exact_exit_status = Some(slot.status.clone());
                    // Exiting the selected shell is terminal-close intent.
                    // Once the output/reader/writer and owned-session barriers
                    // complete, remove that tab without another CLOSE gesture.
                    // Keep inactive exited tabs visible so their exact outcome
                    // is not erased before the owner can review it.
                    slot.remove_when_closed = is_active;
                    lane.status = slot.status.clone();
                    if !slot.disconnected_reported {
                        report.notices.push(format!("terminal {}", slot.status));
                        slot.disconnected_reported = true;
                    }
                    report.tabs_changed = true;
                    report.active_projection_changed |= is_active;
                }
                TerminalEvent::Error(error) => {
                    let slot = &mut self.sessions[index];
                    #[cfg(test)]
                    report.serviced.push((index, "control", 0));
                    slot.status = format!("transport {:?} failed", error.stage).to_lowercase();
                    let lane = if is_active {
                        &mut *active_lane
                    } else {
                        &mut slot.parked_lane
                    };
                    lane.status = slot.status.clone();
                    report.notices.push(format!(
                        "terminal {:?} failure ({:?}, errno {:?}; accepted {}, written {}, remaining {}; {} requests / {} bytes undelivered)",
                        error.stage, error.kind, error.os_code, error.accepted_bytes,
                        error.written_bytes, error.remaining_bytes, error.undelivered_requests,
                        error.total_undelivered_bytes,
                    ));
                    report.tabs_changed = true;
                    report.active_projection_changed |= is_active;
                }
            }
        }
        for index in 0..self.sessions.len() {
            flush_output_batch(
                &mut self.sessions,
                visible_active_index,
                active_lane,
                &mut pending_output,
                &mut report,
                index,
            );
        }
        if self.remove_presented_closed(active_lane) {
            report.tabs_changed = true;
            report.active_projection_changed = true;
        }
        report.pending = self
            .sessions
            .iter()
            .any(|slot| slot.session.has_pending_event());
        if report.pending {
            self.request_output_poll();
        }
        report
    }
}

#[cfg(test)]
#[path = "terminal_session_drain_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "terminal_session_drain_projection_tests.rs"]
mod projection_tests;
