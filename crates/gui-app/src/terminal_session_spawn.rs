use super::*;
use std::{sync::mpsc, thread};

impl TerminalSessionRegistry {
    #[allow(dead_code)]
    pub(super) fn spawn_and_activate(&mut self, context: &TerminalLaunchContext) -> Result<&str> {
        ensure_session_capacity(self.sessions.len() + self.pending_spawns.len())?;
        let session = spawn_terminal_session_with_wake(context, self.terminal_wake.clone())?;
        let label = self.reserve_session_label();
        self.sessions.push(new_session_slot(session, label));
        self.active_index = self.sessions.len() - 1;
        mark_terminal_session_lifecycle(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        record_terminal_lifecycle_event(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        Ok(self.active().session_id())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn spawn_and_activate_with_lane(
        &mut self,
        context: &TerminalLaunchContext,
        lane: &mut TerminalLaneState,
    ) -> Result<String> {
        let previous = self.active_index;
        let session_id = self.spawn_and_activate(context)?.to_string();
        lane.swap_session_projection(&mut self.sessions[previous].parked_lane);
        self.projection_managed = true;
        debug_assert!(
            self.sessions[self.active_index]
                .parked_lane
                .grid_lines()
                .is_empty()
        );
        Ok(session_id)
    }

    /// Reserve and project a new tab without running filesystem or PTY setup
    /// on the GUI event thread. Completion is delivered through the existing
    /// coalesced terminal wake.
    pub(crate) fn begin_spawn_and_activate(
        &mut self,
        context: &TerminalLaunchContext,
    ) -> Result<String> {
        self.begin_spawn_and_activate_using(context, |context, wake| {
            spawn_terminal_session_with_wake(&context, wake)
        })
    }

    fn begin_spawn_and_activate_using<F>(
        &mut self,
        context: &TerminalLaunchContext,
        spawn: F,
    ) -> Result<String>
    where
        F: FnOnce(TerminalLaunchContext, TerminalWakeGate) -> Result<TerminalSession>
            + Send
            + 'static,
    {
        ensure_session_capacity(self.sessions.len() + self.pending_spawns.len())?;
        let label = self.reserve_session_label();
        let pending_id = format!("pending-{}", label.replace(' ', "-"));
        let (sender, result) = mpsc::channel();
        let worker_context = context.clone();
        let worker_wake = self.terminal_wake.clone();
        let completion_wake = self.terminal_wake.clone();
        thread::Builder::new()
            .name(format!("terminal-spawn-{pending_id}"))
            .spawn(move || {
                let result = spawn(worker_context, worker_wake)
                    .and_then(|session| {
                        mark_terminal_session_lifecycle(
                            &session,
                            DatumToolSessionLifecycle::Attached,
                            None,
                        )?;
                        record_terminal_lifecycle_event(
                            &session,
                            DatumToolSessionLifecycle::Attached,
                            None,
                        )?;
                        Ok(session)
                    })
                    .map_err(|error| format!("{error:#}"));
                let _ = sender.send(result);
                completion_wake.request();
            })
            .map_err(|error| anyhow::anyhow!("start terminal session worker: {error}"))?;
        self.pending_spawns.push(PendingTerminalSpawn {
            pending_id: pending_id.clone(),
            label,
            result,
        });
        Ok(pending_id)
    }

    /// Install completed spawns in reservation order. Later tabs cannot jump
    /// ahead merely because their shell happened to initialize first.
    pub(crate) fn complete_pending_spawns(&mut self, lane: &mut TerminalLaneState) -> Vec<String> {
        let mut notices = Vec::new();
        loop {
            let completion = match self.pending_spawns.first() {
                Some(pending) => match pending.result.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                        "terminal session worker disconnected before completion".to_string(),
                    )),
                },
                None => None,
            };
            let Some(completion) = completion else {
                break;
            };
            let pending = self.pending_spawns.remove(0);
            match completion {
                Ok(session) => {
                    let previous = self.active_index;
                    let session_id = session.session_id().to_string();
                    self.sessions.push(new_session_slot(session, pending.label));
                    self.active_index = self.sessions.len() - 1;
                    lane.swap_session_projection(&mut self.sessions[previous].parked_lane);
                    self.projection_managed = true;
                    notices.push(format!("opened terminal session {session_id}"));
                }
                Err(error) => {
                    lane.status = format!("terminal session open failed: {error}");
                    notices.push(lane.status.clone());
                }
            }
        }
        notices
    }

    fn reserve_session_label(&mut self) -> String {
        let label = format!("shell {}", self.next_session_ordinal);
        self.next_session_ordinal += 1;
        label
    }
}

fn new_session_slot(session: TerminalSession, label: String) -> TerminalSessionSlot {
    TerminalSessionSlot {
        session,
        screen: TerminalScreen::default(),
        label,
        status: "running".to_string(),
        attached: true,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::mpsc, time::Duration};

    #[test]
    fn pending_tab_is_projected_before_spawn_work_finishes() {
        let root =
            std::env::temp_dir().join(format!("datum-terminal-async-spawn-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let context = TerminalLaunchContext::for_project_root(&root);
        let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
        let (entered_sender, entered_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();

        registry
            .begin_spawn_and_activate_using(&context, move |context, wake| {
                entered_sender.send(()).unwrap();
                release_receiver
                    .recv_timeout(Duration::from_secs(1))
                    .expect("GUI path must return before background spawn is released");
                spawn_terminal_session_with_wake(&context, wake)
            })
            .unwrap();
        entered_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("background spawn should begin");

        let mut lane = TerminalLaneState::default();
        registry.sync_lane_tabs(&mut lane);
        assert_eq!(lane.tabs.len(), 2);
        assert_eq!(lane.tabs[1].label, "shell 2");
        assert_eq!(lane.tabs[1].status, "starting");
        assert_eq!(registry.sessions.len(), 1);

        release_sender.send(()).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !registry.pending_spawns.is_empty() && std::time::Instant::now() < deadline {
            registry.complete_pending_spawns(&mut lane);
            std::thread::yield_now();
        }
        assert!(registry.pending_spawns.is_empty());
        assert_eq!(registry.sessions.len(), 2);
        assert_eq!(registry.sessions[registry.active_index].label, "shell 2");
        let _ = fs::remove_dir_all(root);
    }
}
