//! Instance membership configures the shared choice form and Proposal worker.
use super::*;
use datum_gui_protocol::schematic_authoring::{
    LabelDraft, interface_binding::InterfaceBindingDraft,
};
impl Runtime {
    pub(crate) fn begin_schematic_interface_binding(&mut self) -> bool {
        match InterfaceBindingDraft::capture(self.workspace()) {
            Ok(draft) => {
                if !self.prepare_native_circuit(false) {
                    return true;
                }
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.chosen = None;
                e.label = Some(LabelDraft::new(
                    eda_engine::schematic::LabelKind::Hierarchical,
                ));
                e.interface_draft = Some(draft);
                e.refusal = None;
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn cycle_interface_choice(&mut self) -> bool {
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(draft) = &mut e.interface_draft else {
            return false;
        };
        draft.cycle();
        e.refusal = None;
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn preview_schematic_interface_binding(&mut self) -> bool {
        let Some(draft) = self.workspace().schematic_authoring.interface_draft.clone() else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        match draft.stage(self.workspace()) {
            Ok(review) => {
                self.cancel_native_schematic();
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review = Some(review);
                self.queue_native_edit(NativeEditAction::Proposal)
            }
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
                self.invalidate_frame();
                true
            }
        }
    }
}
