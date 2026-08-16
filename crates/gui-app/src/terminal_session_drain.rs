use super::{TerminalEvent, TerminalSessionRegistry, mark_terminal_session_exit};
use crate::{
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
    #[cfg(test)]
    serviced: Vec<(usize, &'static str, usize)>,
}

impl TerminalSessionRegistry {
    pub(crate) fn drain_all(&mut self, active_lane: &mut TerminalLaneState) -> TerminalDrainReport {
        let mut report = TerminalDrainReport::default();
        if self.sessions.is_empty() {
            return report;
        }
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
                let lane = if index == self.active_index {
                    &mut *active_lane
                } else {
                    &mut slot.parked_lane
                };
                lane.status = next;
                report.tabs_changed = true;
                report.active_projection_changed |= index == self.active_index;
            }
        }
        let mut idle_visits = 0usize;
        let mut output_events = 0usize;
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
            let is_active = index == self.active_index;
            let slot = &mut self.sessions[index];
            match event {
                TerminalEvent::Output(bytes) => {
                    #[cfg(test)]
                    report.serviced.push((index, "output", bytes.len()));
                    output_events += 1;
                    report.output_events += 1;
                    report.output_bytes += bytes.len();
                    let _ = record_terminal_output_event(&slot.session, &bytes);
                    let lane = if is_active {
                        &mut *active_lane
                    } else {
                        &mut slot.parked_lane
                    };
                    let responses = slot.screen.apply_bytes_with_responses(lane, &bytes);
                    for response in responses {
                        if let Err(error) = slot.session.write_bytes(&response) {
                            report
                                .notices
                                .push(format!("terminal status response failed: {error}"));
                        }
                    }
                    report.active_projection_changed |= is_active;
                    report.tabs_changed = true;
                }
                TerminalEvent::Exited(code) => {
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
                    let lane = if is_active {
                        &mut *active_lane
                    } else {
                        &mut slot.parked_lane
                    };
                    lane.status = slot.status.clone();
                    if !slot.disconnected_reported {
                        report.notices.push(format!("terminal {}", slot.status));
                        slot.disconnected_reported = true;
                    }
                    report.tabs_changed = true;
                    report.active_projection_changed |= is_active;
                }
                TerminalEvent::Error(error) => {
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
mod tests {
    use super::*;
    use crate::{
        terminal_activity_snapshot::TerminalActivitySummaryCache,
        terminal_screen::TerminalScreen,
        terminal_session::{TerminalLaunchContext, TerminalSession, TerminalSessionSlot},
        terminal_transport::{TerminalTransportSession, TerminalWakeGate},
    };
    use std::time::{Duration, Instant};
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
    };

    #[test]
    fn seventeenth_session_is_refused_by_preallocation_guard() {
        assert!(super::super::ensure_session_capacity(15).is_ok());
        assert!(super::super::ensure_session_capacity(16).is_err());
    }

    #[test]
    fn one_gui_turn_never_exceeds_owner_ratified_output_limits() {
        let root =
            std::env::temp_dir().join(format!("datum-terminal-drain-limit-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let context = TerminalLaunchContext::for_project_root(&root);
        let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
        let mut lane = TerminalLaneState::default();
        registry
            .active()
            .write_bytes(b"head -c 200000 /dev/zero | tr '\\0' x\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !registry.active().has_pending_event() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        let report = registry.drain_all(&mut lane);
        assert!(report.output_events <= GUI_DRAIN_EVENT_LIMIT);
        assert!(report.output_bytes <= GUI_DRAIN_BYTE_LIMIT);
        let _ = std::fs::remove_dir_all(root);
    }

    fn synthetic_registry(session_count: usize) -> TerminalSessionRegistry {
        let wake = TerminalWakeGate::new(None);
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-synthetic-drain-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let sessions = (0..session_count)
            .map(|index| {
                let id = format!("synthetic-{index}");
                TerminalSessionSlot {
                    session: TerminalSession {
                        transport: TerminalTransportSession::synthetic(wake.clone()),
                        context_path: root.join(format!("{id}-context.json")),
                        latest_context_path: root.join("latest.json"),
                        session_path: root.join(format!("{id}-session.json")),
                        session_id: id.clone(),
                        context_id: format!("context-{index}"),
                        active_execution_id: Arc::new(Mutex::new(None)),
                        finished_scan_offset: Cell::new(0),
                    },
                    screen: TerminalScreen::default(),
                    label: id,
                    status: "running".to_string(),
                    attached: index == 0,
                    previous_session_id: None,
                    restart_count: 0,
                    columns: 80,
                    rows: 24,
                    activity: TerminalActivitySummaryCache::default(),
                    parked_lane: TerminalLaneState::default(),
                    disconnected_reported: false,
                    termination_failure_reported: false,
                    close_confirmation_armed: false,
                    close_confirmation_input: String::new(),
                    pending_restart: false,
                    remove_when_closed: false,
                    hidden_after_close: false,
                    exact_exit_status: None,
                }
            })
            .collect();
        TerminalSessionRegistry {
            sessions,
            active_index: 0,
            terminal_wake: wake,
            next_drain_index: 0,
            projection_managed: true,
        }
    }

    #[test]
    fn control_priority_round_robin_cursor_and_exact_global_caps_are_literal() {
        let mut registry = synthetic_registry(3);
        registry.sessions[1]
            .session
            .transport
            .push_synthetic_error();
        for round in 0..43 {
            for index in 0..3 {
                registry.sessions[index]
                    .session
                    .transport
                    .push_synthetic_output(&vec![b'a' + index as u8; 512]);
            }
            assert!(round < 43);
        }
        let mut lane = TerminalLaneState::default();
        let first = registry.drain_all(&mut lane);
        assert_eq!(first.serviced[0], (1, "control", 0));
        assert_eq!(first.output_events, GUI_DRAIN_EVENT_LIMIT);
        assert_eq!(first.output_bytes, GUI_DRAIN_BYTE_LIMIT);
        assert!(first.pending);
        assert_eq!(
            first
                .serviced
                .iter()
                .filter(|(_, kind, _)| *kind == "output")
                .take(6)
                .map(|(index, _, _)| *index)
                .collect::<Vec<_>>(),
            vec![2, 0, 1, 2, 0, 1]
        );
        assert_eq!(registry.next_drain_index, 1);

        let second = registry.drain_all(&mut lane);
        assert_eq!(second.serviced[0].0, 1);
        assert_eq!(second.serviced[0].1, "output");
        assert_eq!(second.output_events, 1);
        assert_eq!(second.output_bytes, 512);
        assert!(!second.pending);
    }
}

#[cfg(test)]
#[path = "terminal_session_drain_projection_tests.rs"]
mod projection_tests;
