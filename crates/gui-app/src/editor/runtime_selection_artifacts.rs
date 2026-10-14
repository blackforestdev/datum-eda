//! Explicit artifact entry and producer lifetime over the same shared selection.
use super::*;
use datum_gui_protocol::{
    SelectionSubject, SelectionTarget,
    selection_artifacts::{
        WorkspaceSelectionAuthority, build_workspace_selection_view, is_artifact,
    },
};
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionMode, acquire_artifact},
    selection_lifetime::reconcile_selection,
};

impl Runtime {
    pub(super) fn dispatch_native_artifact_command(
        &mut self,
        command: &SessionCommand,
    ) -> Option<bool> {
        let subject = match command {
            SessionCommand::SelectReviewAction(id) => SelectionSubject::Review(id.clone()),
            SessionCommand::SelectCheckFinding(id) => SelectionSubject::Diagnostic(id.clone()),
            SessionCommand::SelectPreviousReviewAction | SessionCommand::SelectNextReviewAction => {
                let state = self.workspace();
                let index = state
                    .review
                    .proposal_actions
                    .iter()
                    .position(|a| a.action_id == state.active_review_target_id)?;
                let next = if matches!(command, SessionCommand::SelectPreviousReviewAction) {
                    index.checked_sub(1)?
                } else {
                    index.checked_add(1)?
                };
                SelectionSubject::Review(state.review.proposal_actions.get(next)?.action_id.clone())
            }
            _ => return None,
        };
        self.acquire_native_artifact(subject)
    }
    pub(super) fn acquire_native_artifact(&mut self, subject: SelectionSubject) -> Option<bool> {
        match self.ensure_selection_authority() {
            Ok(false) => return None,
            Ok(true) => {}
            Err(error) => {
                self.selection_notice(error.to_string());
                return Some(true);
            }
        }
        if self.modifiers.shift_key() && self.modifiers.control_key() {
            return Some(true);
        }
        self.cancel_native_selection_gesture();
        let native = self.native_selection.as_ref().unwrap();
        let state = self.workspace();
        let prior = match &state.selection {
            SelectionTarget::Native(v) => v.subject.clone(),
            _ => SelectionSubject::None,
        };
        let mode = if self.modifiers.control_key() {
            AcquisitionMode::Remove
        } else if self.modifiers.shift_key() {
            AcquisitionMode::Add
        } else {
            AcquisitionMode::Replace
        };
        let authority = WorkspaceSelectionAuthority {
            native: &native.authority,
            model: &native.model,
            workspace: state,
        };
        let outcome = match acquire_artifact(&prior, subject, mode, &authority) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.selection_notice(format!("Artifact acquisition refused: {error:?}"));
                return Some(true);
            }
        };
        let mut view = match build_workspace_selection_view(
            &native.model,
            &native.authority,
            outcome.subject,
            state,
        ) {
            Ok(view) => view,
            Err(error) => {
                self.selection_notice(format!("Artifact projection refused: {error:?}"));
                return Some(true);
            }
        };
        if let Some(refusal) = outcome.refusal {
            view.notice = Some(refusal.explanation());
        }
        let summary = view.summary();
        if let SelectionSubject::Proposal(id) | SelectionSubject::Review(id) = &view.subject {
            self.session.workspace_mut().active_review_target_id = id.clone();
        }
        self.session.workspace_mut().selection = if matches!(view.subject, SelectionSubject::None) {
            SelectionTarget::None
        } else {
            SelectionTarget::Native(Box::new(view))
        };
        self.native_selection.as_mut().unwrap().reset_click_depth();
        self.refresh_selection_accessibility();
        self.log_console_echo(ConsoleFeedbackSource::Selection, summary);
        self.invalidate_frame();
        Some(true)
    }
    pub(super) fn reconcile_native_artifact_selection(&mut self) {
        let (Some(native), SelectionTarget::Native(prior)) =
            (&self.native_selection, &self.workspace().selection)
        else {
            return;
        };
        if !is_artifact(&prior.subject) {
            return;
        }
        let authority = WorkspaceSelectionAuthority {
            native: &native.authority,
            model: &native.model,
            workspace: self.workspace(),
        };
        let outcome = match reconcile_selection(&prior.subject, prior.project, &authority) {
            Ok(outcome) => outcome,
            Err(error) => {
                if let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection {
                    view.stale = true;
                }
                self.selection_notice(format!("Artifact lifetime unavailable: {error:?}"));
                return;
            }
        };
        if outcome.dissolved {
            self.session.workspace_mut().selection = SelectionTarget::None;
            self.refresh_selection_accessibility();
            self.log_console_echo(
                ConsoleFeedbackSource::Selection,
                "Producing artifact invalidated its selected identity; selection cleared",
            );
            self.invalidate_frame();
            return;
        }
        match build_workspace_selection_view(
            &native.model,
            &native.authority,
            outcome.subject,
            self.workspace(),
        ) {
            Ok(mut view) => {
                if datum_gui_protocol::selection_artifacts::artifact_producer_changed(prior, &view)
                {
                    self.session.workspace_mut().selection = SelectionTarget::None;
                    self.refresh_selection_accessibility();
                    self.log_console_echo(
                        ConsoleFeedbackSource::Selection,
                        "Selected action producer changed; selection cleared",
                    );
                    self.invalidate_frame();
                    return;
                }
                view.notice = prior.notice.clone();
                self.session.workspace_mut().selection = SelectionTarget::Native(Box::new(view));
                self.refresh_selection_accessibility();
            }
            Err(error) => {
                self.selection_notice(format!("Artifact projection unavailable: {error:?}"))
            }
        }
    }
}
