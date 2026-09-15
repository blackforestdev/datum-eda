use super::{
    TerminalWakeGate,
    control::ControlBacklog,
    event::TerminalIoError,
    linux::{job_control, process_session},
    process_status::TerminalExitStatus,
    shutdown::{ShutdownPhase, ShutdownRequest},
};
use std::{
    process::Child,
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};

#[derive(Clone, Debug)]
pub(crate) struct ShutdownSnapshot {
    pub(crate) phase: ShutdownPhase,
    pub(crate) surviving_processes: Vec<ShutdownProcessIdentity>,
    pub(crate) failure: Option<String>,
    pub(crate) leader_reaped: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) visited_phases: Vec<ShutdownPhase>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShutdownProcessIdentity {
    pub(crate) pid: i32,
    pub(crate) process_group_id: i32,
    pub(crate) session_id: i32,
}

struct SupervisorState {
    phase: ShutdownPhase,
    request: Option<ShutdownRequest>,
    surviving_processes: Vec<process_session::ProcessIdentity>,
    failure: Option<String>,
    leader_reaped: bool,
    global_deadline: Option<Instant>,
    visited_phases: Vec<ShutdownPhase>,
    attempt_started: Option<Instant>,
}

pub(super) struct ProcessSupervisor {
    session_id: libc::pid_t,
    shared: Arc<(Mutex<SupervisorState>, Condvar)>,
    wake: TerminalWakeGate,
}

impl ProcessSupervisor {
    pub(super) fn start(
        mut child: Child,
        session_id: libc::pid_t,
        control: Arc<ControlBacklog>,
        wake: TerminalWakeGate,
    ) -> Arc<Self> {
        let supervisor = Arc::new(Self {
            session_id,
            shared: Arc::new((
                Mutex::new(SupervisorState {
                    phase: ShutdownPhase::Running,
                    request: None,
                    surviving_processes: Vec::new(),
                    failure: None,
                    leader_reaped: false,
                    global_deadline: None,
                    visited_phases: Vec::new(),
                    attempt_started: None,
                }),
                Condvar::new(),
            )),
            wake,
        });
        let waiter_supervisor = supervisor.clone();
        std::thread::spawn(move || match child.wait() {
            Ok(status) => {
                waiter_supervisor.mark_leader_reaped();
                control.child_exited(TerminalExitStatus::from_std(status));
                waiter_supervisor.request(ShutdownRequest::Graceful, None, true);
            }
            Err(error) => {
                control.wait_failed(TerminalIoError::wait(&error));
                waiter_supervisor.mark_wait_failed(error.to_string());
                waiter_supervisor.request(ShutdownRequest::Graceful, None, true);
            }
        });
        let worker = supervisor.clone();
        std::thread::spawn(move || worker.run());
        supervisor
    }

    pub(super) fn request_graceful(&self) {
        self.request(ShutdownRequest::Graceful, None, false);
    }

    pub(super) fn request_graceful_by(&self, deadline: Instant) {
        self.request(ShutdownRequest::Graceful, Some(deadline), false);
    }

    pub(super) fn request_force(&self) {
        self.request(ShutdownRequest::Force, None, false);
    }

    pub(super) fn request_retry_by(&self, deadline: Instant) {
        let (lock, changed) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
        if state.phase == ShutdownPhase::Closed {
            return;
        }
        state.request = Some(ShutdownRequest::Graceful);
        state.global_deadline = Some(deadline);
        state.attempt_started = Some(Instant::now());
        state.phase = ShutdownPhase::Hup;
        if !state.visited_phases.contains(&ShutdownPhase::Hup) {
            state.visited_phases.push(ShutdownPhase::Hup);
        }
        changed.notify_all();
        self.wake.request();
    }

    pub(super) fn snapshot(&self) -> ShutdownSnapshot {
        let state = self.shared.0.lock().unwrap_or_else(|p| p.into_inner());
        ShutdownSnapshot {
            phase: state.phase,
            surviving_processes: state
                .surviving_processes
                .iter()
                .map(|identity| ShutdownProcessIdentity {
                    pid: identity.pid,
                    process_group_id: identity.process_group_id,
                    session_id: identity.session_id,
                })
                .collect(),
            failure: state.failure.clone(),
            leader_reaped: state.leader_reaped,
            visited_phases: state.visited_phases.clone(),
        }
    }

    fn request(&self, request: ShutdownRequest, deadline: Option<Instant>, only_if_running: bool) {
        let (lock, changed) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
        if state.phase == ShutdownPhase::Closed
            || (only_if_running && state.phase != ShutdownPhase::Running)
        {
            return;
        }
        if request == ShutdownRequest::Force && state.phase == ShutdownPhase::Kill {
            return;
        }
        let new_attempt = matches!(state.phase, ShutdownPhase::Running | ShutdownPhase::Failed);
        if new_attempt {
            state.request = Some(request);
            state.global_deadline = deadline;
            state.attempt_started = Some(Instant::now());
        } else {
            if let Some(deadline) = deadline {
                state.global_deadline = Some(
                    state
                        .global_deadline
                        .map_or(deadline, |current| current.min(deadline)),
                );
            }
            if request == ShutdownRequest::Force {
                state.request = Some(ShutdownRequest::Force);
            }
        }
        if new_attempt || request == ShutdownRequest::Force {
            state.phase = match request {
                ShutdownRequest::Graceful => ShutdownPhase::Hup,
                ShutdownRequest::Force => ShutdownPhase::Kill,
            };
            let phase = state.phase;
            if !state.visited_phases.contains(&phase) {
                state.visited_phases.push(phase);
            }
        }
        changed.notify_all();
        self.wake.request();
    }

    fn mark_leader_reaped(&self) {
        let (lock, changed) = &*self.shared;
        lock.lock().unwrap_or_else(|p| p.into_inner()).leader_reaped = true;
        changed.notify_all();
        self.wake.request();
    }

    fn mark_wait_failed(&self, message: String) {
        let (lock, changed) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
        state.failure = Some(format!("terminal leader wait failed: {message}"));
        changed.notify_all();
        self.wake.request();
    }

    fn run(&self) {
        loop {
            let (request, global_deadline, origin) = {
                let (lock, changed) = &*self.shared;
                let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
                while state.request.is_none() && state.phase != ShutdownPhase::Closed {
                    state = changed.wait(state).unwrap_or_else(|p| p.into_inner());
                }
                if state.phase == ShutdownPhase::Closed {
                    return;
                }
                state.failure = None;
                state.surviving_processes.clear();
                (
                    state.request.take().expect("shutdown request"),
                    state.global_deadline,
                    state.attempt_started.unwrap_or_else(Instant::now),
                )
            };
            if request == ShutdownRequest::Force {
                let kill_deadline = self.capped_deadline(
                    Instant::now()
                        + std::time::Duration::from_millis(super::limits::KILL_VERIFY_MS),
                    global_deadline,
                );
                if self.run_phase(ShutdownPhase::Kill, libc::SIGKILL, false, kill_deadline) {
                    self.finish_after_deadline(kill_deadline);
                }
            } else {
                let hup_deadline = self.capped_deadline(
                    origin + std::time::Duration::from_millis(super::limits::HUP_GRACE_MS),
                    global_deadline,
                );
                if self.run_phase(ShutdownPhase::Hup, libc::SIGHUP, true, hup_deadline)
                    && !self.wait_or_closed(hup_deadline)
                {
                    if self.take_force_request() {
                        let kill_deadline = self.capped_deadline(
                            Instant::now()
                                + std::time::Duration::from_millis(super::limits::KILL_VERIFY_MS),
                            global_deadline,
                        );
                        if self.run_phase(ShutdownPhase::Kill, libc::SIGKILL, false, kill_deadline)
                        {
                            self.finish_after_deadline(kill_deadline);
                        }
                    } else {
                        let term_deadline = self.capped_deadline(
                            origin
                                + std::time::Duration::from_millis(
                                    super::limits::HUP_GRACE_MS + super::limits::TERM_GRACE_MS,
                                ),
                            global_deadline,
                        );
                        if self.run_phase(ShutdownPhase::Term, libc::SIGTERM, true, term_deadline)
                            && !self.wait_or_closed(term_deadline)
                        {
                            let forced = self.take_force_request();
                            let deadline = if forced {
                                Instant::now()
                                    + std::time::Duration::from_millis(
                                        super::limits::KILL_VERIFY_MS,
                                    )
                            } else {
                                origin
                                    + std::time::Duration::from_millis(
                                        super::limits::HUP_GRACE_MS
                                            + super::limits::TERM_GRACE_MS
                                            + super::limits::KILL_VERIFY_MS,
                                    )
                            };
                            let kill_deadline = self.capped_deadline(deadline, global_deadline);
                            if self.run_phase(
                                ShutdownPhase::Kill,
                                libc::SIGKILL,
                                false,
                                kill_deadline,
                            ) {
                                self.finish_after_deadline(kill_deadline);
                            }
                        }
                    }
                }
            }
            if self.snapshot().phase == ShutdownPhase::Closed {
                return;
            }
        }
    }

    fn run_phase(
        &self,
        phase: ShutdownPhase,
        signal: i32,
        resume: bool,
        deadline: Instant,
    ) -> bool {
        if phase == ShutdownPhase::Kill {
            self.begin_kill_phase();
        }
        if Instant::now() >= deadline {
            self.fail(
                "terminal teardown deadline elapsed before discovery".to_string(),
                Vec::new(),
            );
            return false;
        }
        let snapshot = match process_session::discover_owned_session(self.session_id) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let observed = error.observed_members();
                self.fail(
                    format!("terminal session discovery failed: {error}"),
                    observed,
                );
                return false;
            }
        };
        if Instant::now() >= deadline {
            self.fail(
                "terminal teardown deadline elapsed during discovery".to_string(),
                snapshot.members,
            );
            return false;
        }
        if snapshot.members.is_empty() {
            self.set_phase(phase, Vec::new());
            self.close_if_reaped();
            return true;
        }
        self.set_phase(phase, snapshot.members.clone());
        for (_, representative) in snapshot.groups {
            if !self.send(representative, signal, &snapshot.members, deadline) {
                return false;
            }
            if resume && !self.send(representative, libc::SIGCONT, &snapshot.members, deadline) {
                return false;
            }
        }
        true
    }

    fn send(
        &self,
        mut representative: process_session::ProcessIdentity,
        signal: i32,
        members: &[process_session::ProcessIdentity],
        deadline: Instant,
    ) -> bool {
        let process_group_id = representative.process_group_id;
        let mut observed = members.to_vec();
        for _ in 0..2 {
            if Instant::now() >= deadline {
                self.fail(
                    format!("terminal signal {signal} exceeded teardown deadline"),
                    observed,
                );
                return false;
            }
            match job_control::signal_owned_process_group(representative, self.session_id, signal) {
                Ok(()) => return true,
                Err(error)
                    if error.raw_os_error() == Some(libc::ESRCH)
                        || error.kind() == std::io::ErrorKind::InvalidData =>
                {
                    let snapshot = match process_session::discover_owned_session(self.session_id) {
                        Ok(snapshot) => snapshot,
                        Err(rescan_error) => {
                            let rescanned = rescan_error.observed_members();
                            self.fail(
                                format!(
                                    "terminal process-group revalidation failed: {rescan_error}"
                                ),
                                if rescanned.is_empty() {
                                    observed
                                } else {
                                    rescanned
                                },
                            );
                            return false;
                        }
                    };
                    if Instant::now() >= deadline {
                        self.fail(
                            format!(
                                "terminal signal {signal} revalidation exceeded teardown deadline"
                            ),
                            snapshot.members,
                        );
                        return false;
                    }
                    observed = snapshot.members.clone();
                    let Some((_, replacement)) = snapshot
                        .groups
                        .into_iter()
                        .find(|(group, _)| *group == process_group_id)
                    else {
                        return true;
                    };
                    representative = replacement;
                }
                Err(error) => {
                    self.fail(
                        format!("terminal signal {signal} failed: {error}"),
                        observed,
                    );
                    return false;
                }
            }
        }
        self.fail(
            format!("terminal process group {process_group_id} kept changing during signal"),
            observed,
        );
        false
    }

    fn wait_or_closed(&self, deadline: Instant) -> bool {
        loop {
            if Instant::now() >= deadline {
                return false;
            }
            if self.close_if_empty_before(deadline) {
                return true;
            }
            let (lock, changed) = &*self.shared;
            let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
            if state.request == Some(ShutdownRequest::Force) {
                return false;
            }
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let waited = changed
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .unwrap_or_else(|p| p.into_inner());
            state = waited.0;
            drop(state);
        }
    }

    fn finish_after_deadline(&self, deadline: Instant) {
        if self.wait_or_closed(deadline) {
            return;
        }
        match process_session::discover_owned_session(self.session_id) {
            Ok(snapshot) if snapshot.members.is_empty() => {
                if !self.close_if_reaped() {
                    self.fail(
                        "terminal session leader was not reaped".to_string(),
                        Vec::new(),
                    );
                }
            }
            Ok(snapshot) => self.fail(
                "terminal session still has processes after SIGKILL".to_string(),
                snapshot.members,
            ),
            Err(error) => self.fail(
                format!("terminal final verification failed: {error}"),
                error.observed_members(),
            ),
        }
    }

    fn capped_deadline(
        &self,
        phase_deadline: Instant,
        global_deadline: Option<Instant>,
    ) -> Instant {
        global_deadline.map_or(phase_deadline, |global| phase_deadline.min(global))
    }

    fn close_if_empty_before(&self, deadline: Instant) -> bool {
        let empty = matches!(
            process_session::discover_owned_session(self.session_id),
            Ok(snapshot) if snapshot.members.is_empty()
        );
        empty && Instant::now() < deadline && self.close_if_reaped()
    }

    fn close_if_reaped(&self) -> bool {
        let (lock, changed) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|p| p.into_inner());
        if !state.leader_reaped {
            return false;
        }
        state.phase = ShutdownPhase::Closed;
        state.surviving_processes.clear();
        changed.notify_all();
        self.wake.request();
        true
    }

    fn take_force_request(&self) -> bool {
        let mut state = self.shared.0.lock().unwrap_or_else(|p| p.into_inner());
        if state.request == Some(ShutdownRequest::Force) {
            state.request = None;
            true
        } else {
            false
        }
    }

    fn begin_kill_phase(&self) {
        let mut state = self.shared.0.lock().unwrap_or_else(|p| p.into_inner());
        state.request = None;
        state.phase = ShutdownPhase::Kill;
        if !state.visited_phases.contains(&ShutdownPhase::Kill) {
            state.visited_phases.push(ShutdownPhase::Kill);
        }
        drop(state);
        self.wake.request();
    }

    fn set_phase(&self, phase: ShutdownPhase, members: Vec<process_session::ProcessIdentity>) {
        let mut state = self.shared.0.lock().unwrap_or_else(|p| p.into_inner());
        state.phase = phase;
        if !state.visited_phases.contains(&phase) {
            state.visited_phases.push(phase);
        }
        state.surviving_processes = members;
        drop(state);
        self.wake.request();
    }

    fn fail(&self, message: String, members: Vec<process_session::ProcessIdentity>) {
        let mut state = self.shared.0.lock().unwrap_or_else(|p| p.into_inner());
        state.phase = ShutdownPhase::Failed;
        state.failure = Some(message);
        state.surviving_processes = members;
        drop(state);
        self.wake.request();
    }
}

#[cfg(test)]
#[path = "process_supervisor_tests.rs"]
mod tests;
