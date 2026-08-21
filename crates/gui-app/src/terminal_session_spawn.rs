use super::*;
use std::{sync::mpsc, thread};

impl TerminalSessionRegistry {
    #[allow(dead_code)]
    pub(super) fn spawn_and_activate(&mut self, context: &TerminalLaunchContext) -> Result<&str> {
        ensure_session_capacity(self.sessions.len() + self.pending_spawns.len())?;
        let session = spawn_terminal_session_with_wake(context, self.terminal_wake.clone())?;
        let label = self.reserve_session_label();
        let session_id = session.session_id().to_string();
        self.sessions.push(new_session_slot(session, label)?);
        self.add_standalone_terminal_tab(session_id);
        self.active_index = self.sessions.len() - 1;
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
        Ok(session_id)
    }

    /// Reserve and project a new tab without running filesystem or PTY setup
    /// on the GUI event thread. Completion is delivered through the existing
    /// coalesced terminal wake.
    pub(crate) fn begin_spawn_and_activate(
        &mut self,
        context: &TerminalLaunchContext,
        lane: &mut TerminalLaneState,
    ) -> Result<String> {
        self.begin_spawn_and_activate_using(context, lane, |context, wake| {
            spawn_terminal_session_with_wake(&context, wake)
        })
    }

    pub(crate) fn begin_split_and_activate(
        &mut self,
        context: &TerminalLaunchContext,
        lane: &mut TerminalLaneState,
        direction: datum_gui_protocol::TerminalSplitDirection,
    ) -> Result<String> {
        let source_session_id = self.active().session_id().to_string();
        self.begin_spawn_using(
            context,
            lane,
            PendingTerminalPlacement::Split {
                source_session_id,
                direction,
            },
            |context, wake| spawn_terminal_session_with_wake(&context, wake),
        )
    }

    fn begin_spawn_and_activate_using<F>(
        &mut self,
        context: &TerminalLaunchContext,
        lane: &mut TerminalLaneState,
        spawn: F,
    ) -> Result<String>
    where
        F: FnOnce(TerminalLaunchContext, TerminalWakeGate) -> Result<TerminalSession>
            + Send
            + 'static,
    {
        self.begin_spawn_using(context, lane, PendingTerminalPlacement::NewTab, spawn)
    }

    fn begin_spawn_using<F>(
        &mut self,
        context: &TerminalLaunchContext,
        lane: &mut TerminalLaneState,
        placement: PendingTerminalPlacement,
        spawn: F,
    ) -> Result<String>
    where
        F: FnOnce(TerminalLaunchContext, TerminalWakeGate) -> Result<TerminalSession>
            + Send
            + 'static,
    {
        ensure_session_capacity(self.sessions.len() + self.pending_spawns.len())?;
        if let PendingTerminalPlacement::Split {
            source_session_id, ..
        } = &placement
            && !self
                .terminal_tabs
                .iter()
                .any(|tab| tab.root.contains_session(source_session_id))
        {
            anyhow::bail!("terminal split source not found: {source_session_id}");
        }
        let label = self.reserve_session_label();
        let pending_id = format!("pending-{}", label.replace(' ', "-"));
        let (sender, result) = mpsc::channel();
        let worker_context = context.clone();
        let worker_wake = self.terminal_wake.clone();
        let completion_wake = self.terminal_wake.clone();
        thread::Builder::new()
            .name(format!("terminal-spawn-{pending_id}"))
            .spawn(move || {
                let result =
                    spawn(worker_context, worker_wake).map_err(|error| format!("{error:#}"));
                let _ = sender.send(result);
                completion_wake.request();
            })
            .map_err(|error| anyhow::anyhow!("start terminal session worker: {error}"))?;
        self.pending_spawns.push(PendingTerminalSpawn {
            pending_id: pending_id.clone(),
            label,
            result,
            canceled: false,
            placement: placement.clone(),
        });
        match placement {
            PendingTerminalPlacement::NewTab => {
                self.add_standalone_terminal_tab(pending_id.clone());
                if self.active_pending_id.is_none() {
                    lane.swap_session_projection(&mut self.sessions[self.active_index].parked_lane);
                }
                self.active_pending_id = Some(pending_id.clone());
                lane.status = "starting terminal session".to_string();
            }
            PendingTerminalPlacement::Split {
                source_session_id,
                direction,
            } => {
                if let Err(error) =
                    self.split_terminal_session(&source_session_id, pending_id.clone(), direction)
                {
                    let pending = self
                        .pending_spawns
                        .pop()
                        .expect("split spawn reservation was just appended");
                    drop(pending);
                    return Err(error);
                }
                if let Some(tab) = self
                    .terminal_tabs
                    .iter_mut()
                    .find(|tab| tab.root.contains_session(&source_session_id))
                {
                    tab.focused_session_id = source_session_id;
                }
            }
        }
        self.projection_managed = true;
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
            let was_active = self.active_pending_id.as_deref() == Some(&pending.pending_id);
            if pending.canceled {
                self.remove_pending_terminal_tab(&pending);
                drop(completion);
                notices.push(format!("canceled terminal session {}", pending.label));
                continue;
            }
            match completion {
                Ok(session) => {
                    let session_id = session.session_id().to_string();
                    let slot = match new_session_slot(session, pending.label.clone()) {
                        Ok(slot) => slot,
                        Err(error) => {
                            self.remove_pending_terminal_tab(&pending);
                            if was_active {
                                self.active_pending_id = None;
                                lane.swap_session_projection(
                                    &mut self.sessions[self.active_index].parked_lane,
                                );
                            }
                            lane.status = format!("terminal core start failed: {error:#}");
                            notices.push(lane.status.clone());
                            continue;
                        }
                    };
                    self.sessions.push(slot);
                    self.replace_pending_terminal_tab(&pending.pending_id, session_id.clone());
                    let split_completion = !pending.placement.is_new_tab();
                    if was_active {
                        self.active_index = self.sessions.len() - 1;
                        self.active_pending_id = None;
                        lane.status = "running".to_string();
                    } else if split_completion {
                        let previous = self.active_index;
                        self.active_index = self.sessions.len() - 1;
                        lane.swap_session_projection(&mut self.sessions[previous].parked_lane);
                        lane.swap_session_projection(
                            &mut self.sessions[self.active_index].parked_lane,
                        );
                    }
                    notices.push(format!("opened terminal session {session_id}"));
                }
                Err(error) => {
                    self.remove_pending_terminal_tab(&pending);
                    if was_active {
                        self.active_pending_id = None;
                        lane.swap_session_projection(
                            &mut self.sessions[self.active_index].parked_lane,
                        );
                    }
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

fn new_session_slot(session: TerminalSession, label: String) -> Result<TerminalSessionSlot> {
    let core = TerminalCoreSessionAdapter::new(
        session.session_id.clone(),
        session.context_id.clone(),
        80,
        24,
    )?;
    Ok(TerminalSessionSlot {
        session,
        core,
        label,
        label_is_explicit: false,
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
        pending_restart: false,
        remove_when_closed: false,
        hidden_after_close: false,
        exact_exit_status: None,
        unread_output: false,
        seen_bell_count: 0,
    })
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
        let mut lane = TerminalLaneState::default();
        let (entered_sender, entered_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();

        registry
            .begin_spawn_and_activate_using(&context, &mut lane, move |context, wake| {
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

        registry.sync_lane_tabs(&mut lane);
        assert_eq!(lane.tabs.len(), 2);
        assert_eq!(lane.tabs[1].label, "shell 2");
        assert_eq!(lane.tabs[1].status, "starting");
        assert!(lane.tabs[1].active);
        assert!(!lane.tabs[0].active);
        assert_eq!(lane.active_tab_id.as_deref(), Some("pending-shell-2"));
        assert_eq!(lane.tab_layouts.len(), 2);
        assert_eq!(lane.tab_layouts[1].root.session_ids(), ["pending-shell-2"]);
        assert!(!registry.active_attached());
        assert!(
            !crate::runtime_terminal_input::write_attached_terminal_bytes(&registry, b"blocked")
                .unwrap()
        );
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
        registry.sync_lane_tabs(&mut lane);
        assert_eq!(
            lane.tabs.iter().find(|tab| tab.active).unwrap().label,
            "shell 2"
        );
        assert_eq!(
            lane.tab_layouts
                .iter()
                .flat_map(|tab| tab.root.session_ids())
                .collect::<Vec<_>>(),
            registry
                .sessions
                .iter()
                .map(|slot| slot.session.session_id())
                .collect::<Vec<_>>()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn split_spawn_stays_in_one_tab_and_focuses_the_completed_leaf() {
        let root =
            std::env::temp_dir().join(format!("datum-terminal-async-split-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let context = TerminalLaunchContext::for_project_root(&root);
        let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
        let source_id = registry.active().session_id().to_string();
        let mut lane = TerminalLaneState::default();

        let pending_id = registry
            .begin_split_and_activate(
                &context,
                &mut lane,
                datum_gui_protocol::TerminalSplitDirection::SideBySide,
            )
            .unwrap();
        registry.sync_lane_tabs(&mut lane);
        assert_eq!(lane.tabs.len(), 1);
        assert_eq!(lane.tab_layouts.len(), 1);
        assert_eq!(
            lane.tab_layouts[0].root.session_ids(),
            [source_id.as_str(), pending_id.as_str()]
        );
        assert_eq!(
            lane.tab_layouts[0].focused_session_id, source_id,
            "the existing pane remains focused until its peer is ready"
        );

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !registry.pending_spawns.is_empty() && std::time::Instant::now() < deadline {
            registry.complete_pending_spawns(&mut lane);
            std::thread::yield_now();
        }
        assert!(registry.pending_spawns.is_empty());
        registry.sync_lane_tabs(&mut lane);
        assert_eq!(registry.sessions.len(), 2);
        assert_eq!(lane.tabs.len(), 1);
        assert_eq!(lane.tab_layouts[0].root.session_ids().len(), 2);
        assert_eq!(
            lane.tab_layouts[0].focused_session_id,
            registry.active().session_id()
        );
        assert_eq!(lane.tabs[0].session_id, registry.active().session_id());
        registry
            .activate_with_lane(&source_id, &mut lane)
            .expect("focus original split leaf");
        registry.sync_lane_tabs(&mut lane);
        assert_eq!(lane.tab_layouts[0].focused_session_id, source_id);
        assert_eq!(lane.tabs[0].session_id, source_id);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn failed_split_spawn_removes_its_leaf_without_removing_the_tab() {
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-failed-split-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let context = TerminalLaunchContext::for_project_root(&root);
        let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
        let source_id = registry.active().session_id().to_string();
        let mut lane = TerminalLaneState::default();

        registry
            .begin_spawn_using(
                &context,
                &mut lane,
                PendingTerminalPlacement::Split {
                    source_session_id: source_id.clone(),
                    direction: datum_gui_protocol::TerminalSplitDirection::Stacked,
                },
                |_context, _wake| Err(anyhow::anyhow!("injected split spawn failure")),
            )
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while !registry.pending_spawns.is_empty() && std::time::Instant::now() < deadline {
            registry.complete_pending_spawns(&mut lane);
            std::thread::yield_now();
        }
        registry.sync_lane_tabs(&mut lane);

        assert!(registry.pending_spawns.is_empty());
        assert_eq!(registry.sessions.len(), 1);
        assert_eq!(lane.tabs.len(), 1);
        assert_eq!(lane.tab_layouts.len(), 1);
        assert_eq!(lane.tab_layouts[0].root.session_ids(), [source_id]);
        let _ = fs::remove_dir_all(root);
    }
}
