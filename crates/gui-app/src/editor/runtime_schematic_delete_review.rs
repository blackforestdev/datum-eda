//! Manual batch deletion uses the existing Proposal worker and inline review.
use super::*;
use datum_gui_protocol::schematic_authoring::delete_review::DeleteReview;
use eda_engine::api::native_write::schematic_delete_proposal::NativeProposalResponse;

impl Runtime {
    pub(super) fn begin_schematic_delete_review(&mut self) -> bool {
        let result = (|| {
            self.selected_schematic_idle()?;
            DeleteReview::capture(self.workspace())
        })();
        match result {
            Ok(review) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review = Some(review);
                self.queue_native_edit(NativeEditAction::Proposal)
            }
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                true
            }
        }
    }
    pub(in crate::runtime_schematic) fn handle_delete_review_key(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        match review_decision(&event.logical_key, self.modifiers) {
            Some(accept) => self.confirm_delete_review(accept),
            None => true,
        }
    }
    fn confirm_delete_review(&mut self, accept: bool) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let Some(mut review) = self.workspace().schematic_authoring.delete_review.clone() else {
            return false;
        };
        if !accept && (review.closes_locally() || self.native_edit.blocked) {
            // Accepted is not rejectable. Closing is local and never retries a mutation.
            if review.source_current(self.workspace()) {
                let mut scene = review.scene.clone();
                scene.source_revision = review.revision.clone();
                self.session.workspace_mut().schematic_scene = Some(scene);
                self.invalidate_scene();
            }
            self.session
                .workspace_mut()
                .schematic_authoring
                .delete_review = None;
            self.invalidate_frame();
            return true;
        }
        if self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        let result = if accept && !review.current(self.workspace()) {
            Err("Project or source evidence changed; dismiss this Proposal and select again".into())
        } else {
            if !accept {
                // Reject is a metadata operation, and remains possible after a design change.
                review.revision = self.workspace().scene.source_revision.clone();
                review.tip = self
                    .workspace()
                    .supervision
                    .journal
                    .accepted_transaction_tip
                    .as_ref()
                    .and_then(|t| t.parse().ok());
            }
            review.stage(accept)
        };
        if let Err(reason) = result {
            review.refusal = Some(reason);
            self.session
                .workspace_mut()
                .schematic_authoring
                .delete_review = Some(review);
            self.invalidate_frame();
            return true;
        }
        self.session
            .workspace_mut()
            .schematic_authoring
            .delete_review = Some(review);
        self.queue_native_edit(NativeEditAction::Proposal)
    }
    pub(crate) fn restore_delete_review(
        &mut self,
        review: Option<DeleteReview>,
        response: Option<NativeProposalResponse>,
        result: &Result<String, String>,
    ) {
        let Some(mut review) = review else {
            return;
        };
        review.pending = None;
        if let Some(response) = response {
            if let Err(reason) = review.observe(response) {
                review.refusal = Some(reason);
            }
        }
        if let Err(reason) = result {
            review.refusal = Some(reason.clone());
        }
        if !review.finished() {
            if review.ready() && review.current(self.workspace()) {
                self.session.workspace_mut().schematic_scene = Some(review.preview_scene());
                self.invalidate_scene();
            }
            self.session
                .workspace_mut()
                .schematic_authoring
                .delete_review = Some(review);
        }
    }
}

fn review_decision(
    key: &winit::keyboard::Key,
    modifiers: winit::keyboard::ModifiersState,
) -> Option<bool> {
    use winit::keyboard::{Key, NamedKey};
    match key {
        Key::Named(NamedKey::Tab) if modifiers.is_empty() => Some(true),
        Key::Named(NamedKey::Escape) => Some(false),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    #[test]
    fn batch_delete_review_requires_explicit_tab_and_escape_never_accepts() {
        assert_eq!(
            review_decision(&Key::Named(NamedKey::Tab), ModifiersState::empty()),
            Some(true)
        );
        for modifiers in [
            ModifiersState::CONTROL,
            ModifiersState::ALT,
            ModifiersState::SHIFT,
            ModifiersState::SUPER,
        ] {
            assert_eq!(review_decision(&Key::Named(NamedKey::Tab), modifiers), None);
            assert_eq!(
                review_decision(&Key::Named(NamedKey::Escape), modifiers),
                Some(false)
            );
        }
        for key in [
            Key::Named(NamedKey::Enter),
            Key::Named(NamedKey::Delete),
            Key::Character("y".into()),
        ] {
            assert_eq!(review_decision(&key, ModifiersState::empty()), None);
        }
    }
}
