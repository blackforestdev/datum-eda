//! A compatible Part/Gate configures the existing choice form and Proposal worker.
use super::*;
use datum_gui_protocol::schematic_authoring::{LabelDraft, delete_review::DeleteReview};
impl Runtime {
    pub(crate) fn begin_schematic_assignment(&mut self) -> bool {
        let result = (|| {
            self.selected_schematic_idle()?;
            DeleteReview::capture_assignment(self.workspace())
        })();
        match result {
            Ok(review) => {
                if !self.prepare_native_schematic(false) {
                    return true;
                }
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.chosen = None;
                e.label = Some(LabelDraft::new(eda_engine::schematic::LabelKind::Local));
                e.assignment_draft = Some(review);
                e.refusal = None;
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn cycle_assignment_choice(&mut self, gate: bool) -> bool {
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(assignment) = e
            .assignment_draft
            .as_mut()
            .and_then(|r| r.assignment.as_mut())
        else {
            return false;
        };
        assignment.cycle(gate);
        if let Some(label) = &mut e.label {
            label.focus = usize::from(gate);
        }
        e.refusal = None;
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn preview_schematic_assignment(&mut self) -> bool {
        let Some(mut review) = self
            .workspace()
            .schematic_authoring
            .assignment_draft
            .clone()
        else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        if let Err(reason) = review.stage_assignment(self.workspace()) {
            self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
            self.invalidate_frame();
            return true;
        }
        self.cancel_native_schematic();
        self.session
            .workspace_mut()
            .schematic_authoring
            .delete_review = Some(review);
        self.queue_native_edit(NativeEditAction::Proposal)
    }
}
