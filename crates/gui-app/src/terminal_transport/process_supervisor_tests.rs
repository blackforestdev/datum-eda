use super::*;

fn supervisor() -> ProcessSupervisor {
    ProcessSupervisor {
        session_id: 42,
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
        wake: TerminalWakeGate::new(None),
    }
}

#[test]
fn force_and_natural_exit_never_regress_phase_or_erase_global_deadline() {
    let supervisor = supervisor();
    let deadline = Instant::now() + std::time::Duration::from_secs(6);
    supervisor.request_graceful_by(deadline);
    supervisor.request_force();
    supervisor.request(ShutdownRequest::Graceful, None, true);
    let state = supervisor.shared.0.lock().unwrap();
    assert_eq!(state.phase, ShutdownPhase::Kill);
    assert_eq!(state.request, Some(ShutdownRequest::Force));
    assert_eq!(state.global_deadline, Some(deadline));
}

#[test]
fn explicit_retry_replaces_an_expired_attempt_deadline() {
    let supervisor = supervisor();
    let expired = Instant::now();
    supervisor.request_graceful_by(expired);
    {
        let mut state = supervisor.shared.0.lock().unwrap();
        state.phase = ShutdownPhase::Failed;
        state.request = None;
    }
    let replacement = Instant::now() + std::time::Duration::from_secs(6);
    supervisor.request_retry_by(replacement);
    let state = supervisor.shared.0.lock().unwrap();
    assert_eq!(state.phase, ShutdownPhase::Hup);
    assert_eq!(state.global_deadline, Some(replacement));
    assert_eq!(state.request, Some(ShutdownRequest::Graceful));
}

#[test]
fn redundant_force_during_kill_cannot_queue_an_unauthorized_retry() {
    let supervisor = supervisor();
    {
        let mut state = supervisor.shared.0.lock().unwrap();
        state.phase = ShutdownPhase::Term;
        state.request = Some(ShutdownRequest::Force);
    }
    supervisor.begin_kill_phase();
    supervisor.request_force();
    let state = supervisor.shared.0.lock().unwrap();
    assert_eq!(state.phase, ShutdownPhase::Kill);
    assert_eq!(state.request, None);
}
