//! Prepare destination terminals without projecting a pending tab or parking
//! the current lane. Reservations include rejected children until reaped.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct ProjectTerminalReservation {
    reservations: Arc<AtomicUsize>,
    rejected: Option<Arc<Mutex<Vec<RejectedProjectTerminal>>>>,
    wake: TerminalWakeGate,
}

impl Drop for ProjectTerminalReservation {
    fn drop(&mut self) {
        self.reservations.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) struct RejectedProjectTerminal {
    pub(crate) session: TerminalSession,
    pub(crate) failure_reported: bool,
    _reservation: ProjectTerminalReservation,
}

impl ProjectTerminalReservation {
    fn reject(mut self, session: TerminalSession) {
        let _ = session.terminate();
        let queue = self.rejected.take().expect("rejection queue");
        queue
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(RejectedProjectTerminal {
                session,
                failure_reported: false,
                _reservation: self,
            });
    }

    /// Runs only on the preparation worker, including terminal core creation.
    pub(crate) fn prepare(self, context: TerminalLaunchContext) -> Result<PreparedProjectTerminal> {
        let (result, rejected) =
            crate::terminal_process::prepare_terminal_process(&context, self.wake.clone());
        let session = match result {
            Ok(session) => session,
            Err(error) => {
                if let Some(session) = rejected {
                    self.reject(session);
                }
                return Err(error);
            }
        };
        let core = TerminalCoreSessionAdapter::new_with_profile(
            session.session_id.clone(),
            session.context_id.clone(),
            80,
            24,
            &session.terminal_profile,
        )
        .map_err(anyhow::Error::from);
        self.finish_prepare(session, context, core)
    }

    fn finish_prepare(
        self,
        session: TerminalSession,
        context: TerminalLaunchContext,
        core: Result<TerminalCoreSessionAdapter>,
    ) -> Result<PreparedProjectTerminal> {
        let core = match core {
            Ok(core) => core,
            Err(error) => {
                self.reject(session);
                return Err(error);
            }
        };
        let slot = spawn::session_slot(session, String::new(), core);
        Ok(PreparedProjectTerminal {
            slot: Some(slot),
            context,
            reservation: Some(self),
        })
    }
}

pub(crate) struct PreparedProjectTerminal {
    slot: Option<TerminalSessionSlot>,
    context: TerminalLaunchContext,
    reservation: Option<ProjectTerminalReservation>,
}

impl Drop for PreparedProjectTerminal {
    fn drop(&mut self) {
        if let (Some(slot), Some(reservation)) = (self.slot.take(), self.reservation.take()) {
            reservation.reject(slot.session);
        }
    }
}

impl TerminalSessionRegistry {
    // One round-robin rejected child shares the SAME turn counters with live
    // terminals. Its bounded transport chunk leaves live terminals admission;
    // draining neither waits nor empties an arbitrary producer on this thread.
    pub(crate) fn drain_rejected_project_terminal(
        &mut self,
        report: &mut drain::TerminalDrainReport,
        lane: &mut TerminalLaneState,
    ) {
        let mut rejected = self
            .rejected_project_terminals
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if rejected.is_empty() {
            if lane.project_open_cleanup_failure.take().is_some() {
                report.tabs_changed = true;
            }
            return;
        }
        let index = self.next_rejected_drain_index % rejected.len();
        self.next_rejected_drain_index = (index + 1) % rejected.len();
        let child = &mut rejected[index];
        if let Some(event) = child.session.try_recv_control_event() {
            report.events += 1;
            match event {
                TerminalEvent::Exited(status) => {
                    let _ = mark_terminal_session_exit(&child.session, status);
                    let _ = crate::terminal_session_events::record_terminal_exit_event(&child.session, status);
                }
                TerminalEvent::Error(error) => report.notices.push(format!(
                    "Open Project rejected terminal {:?} failure ({:?}, errno {:?}; accepted {}, written {}, remaining {}; {} requests / {} bytes undelivered)",
                    error.stage, error.kind, error.os_code, error.accepted_bytes, error.written_bytes,
                    error.remaining_bytes, error.undelivered_requests, error.total_undelivered_bytes)),
                _ => {}
            }
        }
        if let Some(bytes) = child.session.try_recv_output(drain::APPLY_BATCH_BYTES.min(
            crate::terminal_transport::GUI_DRAIN_BYTE_LIMIT.saturating_sub(report.output_bytes),
        )) {
            report.events += 1;
            report.output_events += 1;
            report.output_bytes += bytes.len();
        }
        if let Some(snapshot) = child.session.shutdown_snapshot()
            && snapshot.phase == crate::terminal_transport::ShutdownPhase::Failed
            && !child.failure_reported
        {
            report.notices.push(format!(
                "Open Project terminal cleanup failed: {}; RETRY termination",
                snapshot
                    .failure
                    .as_deref()
                    .unwrap_or("owned processes remain")
            ));
            child.failure_reported = true;
        }
        rejected.retain(|child| {
            !child.session.shutdown_snapshot().is_some_and(|snapshot| {
                snapshot.phase == crate::terminal_transport::ShutdownPhase::Closed
                    && snapshot.leader_reaped
                    && snapshot.surviving_processes.is_empty()
                    && child.session.presentation_complete()
            })
        });
        let failure = rejected.iter().find_map(|child| {
            child
                .session
                .shutdown_snapshot()
                .filter(|snapshot| {
                    snapshot.phase == crate::terminal_transport::ShutdownPhase::Failed
                })
                .map(|snapshot| {
                    format!(
                        "Open Project terminal cleanup failed: {}",
                        snapshot
                            .failure
                            .as_deref()
                            .unwrap_or("owned processes remain")
                    )
                })
        });
        if lane.project_open_cleanup_failure != failure {
            lane.project_open_cleanup_failure = failure;
            report.tabs_changed = true;
        }
        report.pending = rejected
            .iter()
            .any(|child| child.session.has_pending_event());
    }

    pub(crate) fn project_terminal_ready_to_publish(&self) -> Result<()> {
        if !self.pending_spawns.is_empty()
            || self.active_pending_id.is_some()
            || self.sessions.iter().any(|slot| slot.pending_restart)
        {
            anyhow::bail!("Wait for the pending terminal session before opening a project");
        }
        Ok(())
    }

    pub(crate) fn reserve_project_terminal(&self) -> Result<ProjectTerminalReservation> {
        self.project_terminal_ready_to_publish()?;
        ensure_session_capacity(
            self.sessions.len()
                + self.pending_spawns.len()
                + self.project_open_reservations.load(Ordering::Acquire),
        )?;
        self.project_open_reservations
            .fetch_add(1, Ordering::AcqRel);
        Ok(ProjectTerminalReservation {
            reservations: self.project_open_reservations.clone(),
            rejected: Some(self.rejected_project_terminals.clone()),
            wake: self.terminal_wake.clone(),
        })
    }

    /// No project or tab mutation happens until all fallible work and guards
    /// pass. Ordinary pending spawns may have appeared since reservation.
    pub(crate) fn publish_project_terminal(
        &mut self,
        mut prepared: PreparedProjectTerminal,
        lane: &mut TerminalLaneState,
    ) -> Result<()> {
        self.project_terminal_ready_to_publish()?;
        let mut slot = prepared.slot.take().expect("prepared terminal slot");
        slot.label = self.reserve_session_label();
        let id = slot.session.session_id().to_owned();
        if let Some(current) = self.sessions.get_mut(self.active_index) {
            lane.swap_session_projection(&mut current.parked_lane);
        }
        lane.swap_session_projection(&mut slot.parked_lane);
        self.origin_contexts
            .insert(id.clone(), prepared.context.clone());
        self.sessions.push(slot);
        self.add_standalone_terminal_tab(id);
        self.active_index = self.sessions.len() - 1;
        self.active_pending_id = None;
        self.projection_managed = true;
        self.sync_lane_tabs(lane);
        Ok(())
    }

    pub(crate) fn prune_project_origins(&mut self) {
        let live: std::collections::HashSet<_> = self
            .sessions
            .iter()
            .map(|slot| slot.session.session_id().to_owned())
            .chain(
                self.pending_spawns
                    .iter()
                    .map(|pending| pending.pending_id.clone()),
            )
            .collect();
        self.origin_contexts.retain(|id, _| live.contains(id));
        self.origin_workspaces.retain(|root, _| {
            self.origin_contexts
                .values()
                .any(|context| context.project_root.as_ref() == Some(root))
        });
    }

    pub(crate) fn update_active_origin_context(&mut self, context: TerminalLaunchContext) {
        self.origin_contexts
            .insert(self.active().session_id().to_owned(), context);
    }

    pub(crate) fn restart_origin(
        &self,
        id: &str,
        current: &TerminalLaunchContext,
    ) -> TerminalLaunchContext {
        self.origin_contexts
            .get(id)
            .filter(|origin| origin.project_root != current.project_root)
            .unwrap_or(current)
            .clone()
    }

    pub(crate) fn transfer_restart_origin(
        &mut self,
        old: &str,
        new: &str,
        context: TerminalLaunchContext,
    ) {
        self.origin_contexts.remove(old);
        self.origin_contexts.insert(new.to_owned(), context);
    }

    pub(crate) fn remember_project_workspace(
        &mut self,
        root: &Path,
        workspace: &ReviewWorkspaceState,
    ) {
        self.origin_workspaces
            .insert(root.to_owned(), Arc::new(workspace.clone()));
    }

    pub(crate) fn active_origin_context(&self) -> Option<&TerminalLaunchContext> {
        let slot = self.sessions.get(self.active_index)?;
        self.origin_contexts.get(slot.session.session_id())
    }

    pub(crate) fn origin_workspace(&self, root: &Path) -> Option<&ReviewWorkspaceState> {
        self.origin_workspaces.get(root).map(Arc::as_ref)
    }
}

#[cfg(test)]
#[path = "project_open/tests.rs"]
mod tests;
