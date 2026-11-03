//! ERC evidence configures the existing rationale form and Proposal controller.
use super::*;
use datum_gui_protocol::schematic_authoring::{LabelDraft, delete_review::DeleteReview};
use eda_engine::api::native_write::schematic_delete_proposal::DispositionKind;

impl Runtime {
    pub(crate) fn begin_erc_disposition(&mut self, kind: DispositionKind) -> bool {
        let result = (|| {
            self.selected_schematic_idle()?;
            DeleteReview::capture_disposition(self.workspace(), kind)
        })();
        match result {
            Ok(review) => {
                if !self.prepare_native_schematic(false) {
                    return true;
                }
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.chosen = None;
                e.label = Some(LabelDraft::new(eda_engine::schematic::LabelKind::Local));
                e.disposition_draft = Some(review);
                e.refusal = None;
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    pub(in crate::runtime_schematic) fn preview_erc_disposition(&mut self) -> bool {
        let Some(mut review) = self
            .workspace()
            .schematic_authoring
            .disposition_draft
            .clone()
        else {
            return false;
        };
        let rationale = self
            .workspace()
            .schematic_authoring
            .label
            .as_ref()
            .map(|l| l.name.clone())
            .unwrap_or_default();
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        if let Err(reason) = review.stage_disposition(self.workspace(), &rationale) {
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
