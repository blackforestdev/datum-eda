use super::*;

impl Runtime {
    pub(crate) fn begin_project_open(&mut self) -> bool {
        // Bind this action to the current Location/candidate, never an older scan.
        self.refresh_folder_browser();
        let request = (|| -> Result<(ProjectOpenOrigin, ProjectTerminalReservation)> {
            anyhow::ensure!(
                !self.folder_host_actions_pending(),
                "Wait for the submitted folder action to finish before opening a Project"
            );
            anyhow::ensure!(
                !self.native_edit.pending() && !self.check_work.pending(),
                "Wait for the current Project edit before opening another Project"
            );
            anyhow::ensure!(
                self.project_open.pending.is_none(),
                "Open Project is already preparing"
            );
            anyhow::ensure!(
                self.workspace().ui.project_open.open_enabled(),
                "Choose a native Project directory"
            );
            let origin = if self.workspace().project_unopened {
                ProjectOpenOrigin::capture_unopened(self.project_open.session_generation)
            } else {
                let root = &self
                    .workspace()
                    .backing
                    .as_ref()
                    .context("Current Project backing unavailable")?
                    .request
                    .project_root;
                ProjectOpenOrigin::capture(root, self.project_open.session_generation)?
            };
            Ok((origin, self.terminal_sessions.reserve_project_terminal()?))
        })();
        let (origin, reservation) = match request {
            Ok(request) => request,
            Err(error) => {
                let reason = self.project_open.refusal(format!("{error:#}"));
                self.log_console_echo(ConsoleFeedbackSource::Viewport, reason.clone());
                self.session.workspace_mut().ui.project_open.refusal = Some(reason);
                self.invalidate_frame();
                return true;
            }
        };
        let directory = self.workspace().ui.project_open.directory.clone();
        let include_review = self.workspace_include_review;
        let profile = self.terminal_profiles.selected().clone();
        self.project_open.generation = self.project_open.generation.wrapping_add(1);
        let ticket = self.project_open.generation;
        let wake = self.project_open.wake.clone();
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("datum-project-open".into())
            .spawn(move || {
                let result = (|| -> Result<Answer> {
                    let prepared = super::prepare_project_directory(&directory, include_review)?;
                    let mut context =
                        crate::terminal_session::terminal_launch_context_from_state_with_cursor(
                            &prepared.root,
                            &prepared.workspace,
                            None,
                        );
                    context.terminal_profile = profile;
                    let terminal = reservation.prepare(context.clone())?;
                    // The PTY can take time to initialize. Revalidate the exact
                    // resolved source and history again after all staging work.
                    let admission = prepared.admit()?;
                    Ok(Answer {
                        origin,
                        ticket,
                        prepared,
                        terminal,
                        context,
                        _admission: admission,
                    })
                })()
                .map_err(|error| format!("{error:#}"));
                let _ = sender.send(result);
                let _ = wake.send_event(());
            });
        match worker {
            Ok(_) => {
                self.project_open.pending = Some(receiver);
                let dialog = &mut self.session.workspace_mut().ui.project_open;
                dialog.busy = true;
                dialog.refusal = None;
            }
            Err(error) => {
                let reason = self
                    .project_open
                    .refusal(format!("Open Project worker unavailable: {error}"));
                self.log_console_echo(ConsoleFeedbackSource::Viewport, reason.clone());
                self.session.workspace_mut().ui.project_open.refusal = Some(reason);
            }
        }
        self.invalidate_frame();
        true
    }
}
