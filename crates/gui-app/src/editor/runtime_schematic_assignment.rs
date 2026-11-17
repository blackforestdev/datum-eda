//! A compatible Part/Gate configures the existing choice form and Proposal worker.
use super::*;
#[path = "runtime_component_resolution.rs"]
mod resolution;
impl Runtime {
    pub(crate) fn begin_schematic_assignment(&mut self) -> bool {
        self.begin_component_properties()
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
