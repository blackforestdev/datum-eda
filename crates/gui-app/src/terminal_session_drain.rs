use super::{
    TerminalEvent, TerminalSessionRegistry, TerminalSessionSlot, mark_terminal_session_exit,
};
use crate::{
    terminal_session_events::{
        record_terminal_exit_event, record_terminal_output_event,
        record_terminal_termination_failure_event,
    },
    terminal_transport::{GUI_DRAIN_BYTE_LIMIT, GUI_DRAIN_EVENT_LIMIT},
};
use datum_gui_protocol::TerminalLaneState;
use std::time::{Duration, Instant};

const APPLY_BATCH_BYTES: usize = 4 * 1024;
const DISPATCH_BUDGET: Duration = Duration::from_millis(1);

#[path = "terminal_session_core_events.rs"]
mod core_events;
use core_events::consume_core_update;

#[derive(Default)]
pub(crate) struct TerminalDrainReport {
    pub(crate) events: usize,
    pub(crate) output_events: usize,
    pub(crate) output_bytes: usize,
    applied_bytes: usize,
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
    report: &mut TerminalDrainReport,
    index: usize,
) {
    if sessions[index].pending_drain_output.is_empty() {
        return;
    }
    let slot = &mut sessions[index];
    let count = slot
        .pending_drain_output
        .len()
        .min(APPLY_BATCH_BYTES)
        .min(GUI_DRAIN_BYTE_LIMIT.saturating_sub(report.applied_bytes));
    if count == 0 {
        return;
    }
    report.applied_bytes += count;
    let bytes: Vec<_> = slot.pending_drain_output.drain(..count).collect();
    debug_assert_eq!(slot.core.session_id(), slot.session.session_id());
    debug_assert_eq!(slot.core.context_id(), slot.session.context_id);
    let _ = record_terminal_output_event(&slot.session, &bytes);
    let is_active = active_index == Some(index);
    if !is_active {
        slot.unread_output = true;
    }
    let lane = if is_active {
        &mut *active_lane
    } else {
        &mut slot.parked_lane
    };
    lane.latest_notification = None;
    match slot.core.apply_output(lane, &bytes) {
        Ok(update) => consume_core_update(&slot.session, lane, report, update),
        Err(error) => report
            .notices
            .push(format!("terminal core output failed: {error}")),
    }
    #[cfg(test)]
    report.serviced.push((index, "apply", bytes.len()));

    report.active_projection_changed |= is_active;
    report.tabs_changed = true;
    #[cfg(test)]
    {
        report.output_batches += 1;
    }
}

impl TerminalSessionRegistry {
    /// Use available dispatch time without letting a busy session monopolize it.
    /// One byte cap spans both retained and newly dequeued application passes.
    fn apply_pending_output(
        &mut self,
        active_lane: &mut TerminalLaneState,
        report: &mut TerminalDrainReport,
        active_index: Option<usize>,
        started: Instant,
        now: &mut impl FnMut() -> Instant,
    ) {
        let mut idle_visits = 0;
        while idle_visits < self.sessions.len()
            && report.applied_bytes < GUI_DRAIN_BYTE_LIMIT
            && now().saturating_duration_since(started) < DISPATCH_BUDGET
        {
            let index = self.next_apply_index % self.sessions.len();
            self.next_apply_index = (index + 1) % self.sessions.len();
            if self.sessions[index].pending_drain_output.is_empty() {
                idle_visits += 1;
                continue;
            }
            idle_visits = 0;
            flush_output_batch(&mut self.sessions, active_index, active_lane, report, index);
        }
    }

    pub(crate) fn drain_all(&mut self, active_lane: &mut TerminalLaneState) -> TerminalDrainReport {
        self.drain_with_clock(active_lane, Instant::now)
    }

    fn drain_with_clock(
        &mut self,
        active_lane: &mut TerminalLaneState,
        mut now: impl FnMut() -> Instant,
    ) -> TerminalDrainReport {
        let started = now();
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
        // Finish retained batches before dequeuing more bytes. This keeps the
        // application staging bound at one existing 64 KiB dispatch, even when
        // a slow parser/log write makes a single batch exceed the time budget.
        self.apply_pending_output(
            active_lane,
            &mut report,
            visible_active_index,
            started,
            &mut now,
        );
        let retained = self
            .sessions
            .iter()
            .any(|slot| !slot.pending_drain_output.is_empty());
        while !retained
            && report.output_events < GUI_DRAIN_EVENT_LIMIT
            && report.events < 256
            && idle_visits < self.sessions.len()
            && now().saturating_duration_since(started) < DISPATCH_BUDGET
        {
            let control = (0..self.sessions.len()).find_map(|offset| {
                let index = (self.next_drain_index + offset) % self.sessions.len();
                self.sessions[index]
                    .pending_drain_output
                    .is_empty()
                    .then(|| self.sessions[index].session.try_recv_control_event())
                    .flatten()
                    .map(|event| (index, event))
            });
            let index = control
                .as_ref()
                .map_or(self.next_drain_index % self.sessions.len(), |(index, _)| {
                    *index
                });
            // Never dequeue a final exit/error ahead of retained bytes.
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
                    report.output_events += 1;
                    report.output_bytes += bytes.len();
                    self.sessions[index]
                        .pending_drain_output
                        .extend_from_slice(&bytes);
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
        self.apply_pending_output(
            active_lane,
            &mut report,
            visible_active_index,
            started,
            &mut now,
        );
        if self.remove_presented_closed(active_lane) {
            report.tabs_changed = true;
            report.active_projection_changed = true;
        }
        report.pending = self
            .sessions
            .iter()
            .any(|slot| !slot.pending_drain_output.is_empty() || slot.session.has_pending_event());
        if report.pending {
            self.request_output_poll();
        }
        let elapsed = now().saturating_duration_since(started);
        if elapsed > DISPATCH_BUDGET {
            crate::gui_runtime_support::append_gui_verbose_diagnostic_line(|| {
                format!(
                    "terminal dispatch over budget elapsed_us={} events={} bytes={} pending={}",
                    elapsed.as_micros(),
                    report.events,
                    report.output_bytes,
                    report.pending
                )
            });
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
