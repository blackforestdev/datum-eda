//! One atomic publication path for immediate and worker-prepared selection.
use super::*;
impl Runtime {
    pub(crate) fn publish_native_acquisition_prepared(
        &mut self,
        candidates: BTreeSet<datum_gui_protocol::AuthoredSelectionIdentity>,
        mode: AcquisitionMode,
        entry: AcquisitionEntry,
        semantic: Option<SelectionSubject>,
        prepared: Option<datum_gui_protocol::selection_view::SelectionView>,
    ) -> bool {
        crate::append_gui_verbose_diagnostic_line(|| "selection publish begin");
        let Some(native) = &self.native_selection else {
            return false;
        };
        let prior = match &self.workspace().selection {
            SelectionTarget::Native(v) => v.subject.clone(),
            _ => SelectionSubject::None,
        };
        let outcome = match acquire_authored(
            &prior,
            candidates,
            mode,
            entry,
            &WorkspaceSelectionAuthority {
                native: &native.authority,
                model: &native.model,
                workspace: self.workspace(),
            },
        ) {
            Ok(o) => o,
            Err(e) => {
                self.selection_notice(format!("Selection refused atomically: {e:?}"));
                return true;
            }
        };
        let announce = outcome.refusal.is_some()
            || outcome.changed
            || semantic.as_ref().is_some_and(|s| *s != prior);
        let subject = semantic.unwrap_or(outcome.subject);
        let state = self.workspace();
        let projection = match prepared {
            Some(view)
                if view.project == native.model.project.project_id
                    && view.revision == native.model.model_revision
                    && view.subject == subject =>
            {
                Ok(view)
            }
            Some(_) => {
                self.selection_notice("Prepared selection no longer matches its authority".into());
                return true;
            }
            None => {
                build_workspace_selection_view(&native.model, &native.authority, subject, state)
            }
        };
        let mut view = match projection {
            Ok(v) => v,
            Err(e) => {
                self.selection_notice(format!("Selection projection refused: {e:?}"));
                return true;
            }
        };
        crate::append_gui_verbose_diagnostic_line(|| "selection view ready");
        if let SelectionTarget::Native(previous) = &state.selection
            && view.subject == previous.subject
        {
            view.scope = previous.scope;
        }
        if let Some(refusal) = outcome.refusal {
            view.notice = Some(refusal.explanation());
        }
        let summary = view.summary();
        self.session.workspace_mut().selection = if matches!(view.subject, SelectionSubject::None) {
            SelectionTarget::None
        } else {
            SelectionTarget::Native(Box::new(view))
        };
        self.native_selection.as_mut().unwrap().lineage = None;
        self.refresh_selection_accessibility();
        crate::append_gui_verbose_diagnostic_line(|| "selection accessibility ready");
        if announce {
            self.log_console_echo(ConsoleFeedbackSource::Selection, summary);
        }
        self.invalidate_frame();
        true
    }
}
