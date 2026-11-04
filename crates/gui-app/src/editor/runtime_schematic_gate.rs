//! Fixed component Gate selection configures existing Symbol placement and review.
use super::*;
use datum_gui_protocol::schematic_authoring::gate_placement::GatePlacementDraft;
impl Runtime {
    pub(crate) fn begin_schematic_gate_placement(&mut self) -> bool {
        let result = (|| {
            self.selected_schematic_idle()?;
            GatePlacementDraft::capture(self.workspace())
        })();
        match result {
            Ok(draft) => {
                if !self.prepare_native_schematic(false) {
                    return true;
                }
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.gate_draft = Some(draft);
                e.chosen = None;
                e.choice_offset = 0;
                e.reference.clear();
                e.value.clear();
                e.reference_editor =
                    datum_gui_protocol::single_line_editor::SingleLineEditor::at_end("");
                e.value_editor =
                    datum_gui_protocol::single_line_editor::SingleLineEditor::at_end("");
                e.refusal = None;
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn choose_schematic_gate(&mut self, index: usize) -> bool {
        let Some(mut draft) = self.workspace().schematic_authoring.gate_draft.clone() else {
            return false;
        };
        match draft.choose(self.workspace(), index) {
            Ok(()) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.gate_draft = Some(draft);
                e.chosen = Some(index);
                e.refusal = None;
            }
            Err(reason) => self.session.workspace_mut().schematic_authoring.refusal = Some(reason),
        }
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn review_schematic_gate(
        &mut self,
        at: datum_gui_protocol::PointNm,
    ) -> bool {
        let Some(draft) = self.workspace().schematic_authoring.gate_draft.clone() else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        match draft.stage(self.workspace(), at) {
            Ok(review) => {
                self.cancel_native_schematic();
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review = Some(review);
                return self.queue_native_edit(NativeEditAction::Proposal);
            }
            Err(reason) => self.session.workspace_mut().schematic_authoring.refusal = Some(reason),
        }
        self.invalidate_frame();
        true
    }
}
