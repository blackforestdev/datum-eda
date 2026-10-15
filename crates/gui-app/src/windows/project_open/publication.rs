use super::*;

impl Runtime {
    pub(crate) fn poll_project_open(&mut self) -> bool {
        let result = match self
            .project_open
            .pending
            .as_ref()
            .map(|receiver| receiver.try_recv())
        {
            Some(Ok(result)) => result,
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                Err("Open Project worker disconnected".into())
            }
            _ => return false,
        };
        self.project_open.pending = None;
        self.session.workspace_mut().ui.project_open.busy = false;
        let result = result.and_then(|answer| {
            self.publish_project_open(answer)
                .map_err(|e| format!("{e:#}"))
        });
        if let Err(reason) = result {
            let reason = self.project_open.refusal(reason);
            self.log_console_echo(ConsoleFeedbackSource::Viewport, reason.clone());
            self.session.workspace_mut().ui.project_open.refusal = Some(reason);
        }
        self.invalidate_frame();
        true
    }

    fn publish_project_open(&mut self, answer: Answer) -> Result<()> {
        if self.workspace().project_unopened {
            anyhow::ensure!(
                self.workspace().ui.project_open.open
                    && answer.ticket == self.project_open.generation
                    && answer
                        .origin
                        .matches_unopened(self.project_open.session_generation),
                "Open Project origin changed; retry Open"
            );
            anyhow::ensure!(
                !self.native_edit.pending(),
                "Current Project edit became busy; retry Open"
            );
        } else {
            let root = &self
                .workspace()
                .backing
                .as_ref()
                .context("Current Project backing unavailable")?
                .request
                .project_root;
            validate_publication_origin(
                &answer.origin,
                root,
                self.project_open.session_generation,
                answer.ticket,
                self.project_open.generation,
                self.workspace().ui.project_open.open,
                self.native_edit.pending(),
            )?;
        }
        self.terminal_sessions.project_terminal_ready_to_publish()?;
        if let Some(backing) = &self.workspace().backing {
            let root = backing.request.project_root.clone();
            let origin_workspace = self.workspace().clone();
            self.terminal_sessions
                .remember_project_workspace(&root, &origin_workspace);
        }
        let opened_root = answer.prepared.root.clone();
        let mut workspace = answer.prepared.workspace;
        // Global preferences are application-owned; all Project consumers and
        // project preferences come from the newly prepared source.
        workspace.ui.global_preferences = self.workspace().ui.global_preferences.clone();
        self.terminal_sessions.publish_project_terminal(
            answer.terminal,
            &mut self.session.workspace_mut().ui.terminal,
        )?;
        workspace.ui.terminal = self.workspace().ui.terminal.clone();
        self.cancel_native_selection_gesture();
        self.selection_run.invalidate_project();
        self.native_edit.reset_project();
        self.native_selection = None;
        self.selection_read_cache = Default::default();
        self.selection_gesture = None;
        self.selection_evaluation = None;
        self.selection_tick = None;
        self.selection_release_suppressed = false;
        self.pan_gesture = Default::default();
        self.divider_drag = None;
        self.dock_drag_active = false;
        self.terminal_tab_drag = None;
        self.terminal_tab_drag_release_suppressed = false;
        self.terminal_split_drag = None;
        self.terminal_text_selection_drag = None;
        self.terminal_mouse_button = None;
        self.last_cursor_pos = None;
        self.presented_hits.clear();
        self.presented_console_layout = None;
        self.menu_keys = Default::default();
        self.project_preferences =
            project_preferences_runtime::ProjectPreferencesCoordinator::new();
        self.project_preferences_raise_requested = false;
        self.terminal_production_refresh_pending = false;
        self.terminal_workspace_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.pending_terminal_clipboard_write = None;
        self.camera = CameraState::fit_to_bounds(&workspace.scene.bounds);
        self.pane_cameras.reset(
            workspace.ui.layout.focused,
            workspace.ui.layout.focused_content(),
            self.camera,
        );
        workspace.ui.focus = ApplicationFocus::Editor(workspace.ui.layout.focused);
        self.session = LiveDesignSession::new(workspace);
        self.terminal_launch_context = Some(answer.context);
        self.project_open.created_root = None;
        self.project_open.session_generation = self.project_open.session_generation.wrapping_add(1);
        self.render_sources = Default::default();
        self.sync_terminal_tabs();
        self.resize_terminal_to_dock();
        let message = format!(
            "Opened Project {} at {}",
            self.workspace().scene.project_name,
            opened_root.display()
        );
        append_gui_diagnostic_line(format!(
            "{message}; project_id={}; session_generation={}",
            self.workspace().scene.project_uuid,
            self.project_open.session_generation
        ));
        self.log_console_echo(ConsoleFeedbackSource::Viewport, message);
        Ok(())
    }
}

// Publication qualification is shared by the ordinary worker answer consumer
// and focused copied-root/late-answer controls. No prepared answer can retarget
// its origin or overwrite a canonical write that became busy meanwhile.
fn validate_publication_origin(
    origin: &ProjectOpenOrigin,
    root: &std::path::Path,
    session_generation: u64,
    ticket: u64,
    current_ticket: u64,
    open: bool,
    write_pending: bool,
) -> Result<()> {
    anyhow::ensure!(
        open && ticket == current_ticket && origin.matches(root, session_generation),
        "Open Project origin changed; retry Open"
    );
    anyhow::ensure!(
        !write_pending,
        "Current Project edit became busy; retry Open"
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
