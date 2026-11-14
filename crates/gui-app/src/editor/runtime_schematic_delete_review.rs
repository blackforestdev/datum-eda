//! Source-qualified schematic actions share one Proposal worker and inline review.
use super::*;
use datum_gui_protocol::schematic_authoring::delete_review::DeleteReview;
use eda_engine::api::native_write::schematic_delete_proposal::NativeProposalResponse;

impl Runtime {
    pub(crate) fn begin_schematic_annotation(&mut self) -> bool {
        self.begin_annotation_scope(false)
    }
    pub(crate) fn begin_schematic_project_annotation(&mut self) -> bool {
        self.begin_annotation_scope(true)
    }
    fn begin_annotation_scope(&mut self, project: bool) -> bool {
        let result = (|| {
            self.selected_schematic_idle()?;
            if project {
                DeleteReview::capture_project_annotation(self.workspace())
            } else {
                DeleteReview::capture_annotation(self.workspace())
            }
        })();
        match result {
            Ok(Some(review)) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review = Some(review);
                if project {
                    self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                        self.workspace().ui.layout.focused,
                    ));
                }
                self.queue_native_edit(NativeEditAction::Proposal)
            }
            Ok(None) => {
                self.log_console_echo(
                    ConsoleFeedbackSource::Tool,
                    if project {
                        "This project is already annotated"
                    } else {
                        "This sheet is already annotated"
                    },
                );
                self.invalidate_frame();
                true
            }
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                self.invalidate_frame();
                true
            }
        }
    }

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
            if event.state == ElementState::Pressed
                && event.logical_key
                    == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
            {
                return self.confirm_delete_review(false);
            }
            return true;
        }
        if self.handle_field_table_key(event) {
            return true;
        }
        self.handle_schematic_review_input(&event.logical_key, event.state)
    }
    /// Window dispatch keeps field-table priority before this logical-key route.
    pub(crate) fn handle_schematic_review_input(
        &mut self,
        key: &winit::keyboard::Key,
        state: ElementState,
    ) -> bool {
        if state != ElementState::Pressed
            || self.workspace().schematic_authoring.delete_review.is_none()
        {
            return false;
        }
        match review_decision(key, self.modifiers) {
            Some(accept) => self.confirm_delete_review(accept),
            None => true,
        }
    }
    pub(super) fn confirm_delete_review(&mut self, accept: bool) -> bool {
        if self.native_edit.pending() {
            if !accept
                && self
                    .workspace()
                    .schematic_authoring
                    .delete_review
                    .as_ref()
                    .is_some_and(|review| review.board_field_revision.is_some())
            {
                let editor = &mut self.session.workspace_mut().schematic_authoring;
                editor.delete_review = None;
                editor.circuit_basis = None;
                editor.circuit_preview = None;
                self.log_console_echo(
                    ConsoleFeedbackSource::Tool,
                    "Review dismissed; an in-flight operation will still finish",
                );
                self.invalidate_scene();
                self.invalidate_frame();
            }
            return true;
        }
        let Some(mut review) = self.workspace().schematic_authoring.delete_review.clone() else {
            return false;
        };
        if accept && let Some(binding) = &review.interface_binding {
            let layout = self
                .current_layout()
                .viewport_panes(&self.workspace().ui.layout);
            let fits = layout
                .panes
                .iter()
                .find(|pane| pane.id == review.basis.pane)
                .is_some_and(|pane| {
                    datum_gui_protocol::schematic_authoring::interface_binding::review_layout(
                        binding,
                        review.refusal.as_deref(),
                        pane.rect.scene.width,
                        pane.rect.scene.height,
                    )
                    .fits
                });
            if !fits {
                self.log_console_refusal(
                    ConsoleFeedbackSource::Tool,
                    "Enlarge the parent pane to review every interface identity before accepting",
                );
                return true;
            }
        }
        if accept && let Some(fields) = &review.board_field_revision {
            let panes = self
                .current_layout()
                .viewport_panes(&self.workspace().ui.layout);
            let fits = panes
                .panes
                .iter()
                .find(|pane| pane.id == review.basis.pane)
                .is_some_and(|pane| {
                    let body = pane.rect.body();
                    let scene = pane.rect.scene;
                    let clearance = if pane.id == self.workspace().ui.layout.focused {
                        datum_gui_protocol::schematic_authoring::review_layout::console_geometry(
                            body.width,
                            body.height,
                            self.scale_factor,
                            self.workspace().ui.console.visible_latest().is_some(),
                            self.workspace().ui.console.history_expanded(),
                        )
                        .map_or(0., |geometry| {
                            (scene.y + scene.height
                                - (body.y + body.height - geometry.occupied_height))
                                .max(0.)
                        })
                    } else {
                        0.
                    };
                    datum_gui_render::Renderer::board_field_review_layout(
                        fields,
                        review.refusal.as_deref(),
                        scene.width,
                        scene.height,
                        clearance,
                    )
                    .is_ok_and(|layout| layout.fits)
                });
            if !fits {
                self.log_console_refusal(ConsoleFeedbackSource::Tool,
                    "Enlarge the pane to review every old and new Reference and Value before accepting");
                return true;
            }
        }
        if !accept && (review.closes_locally() || self.native_edit.blocked) {
            // Accepted is not rejectable. Closing is local and never retries a mutation.
            if review.basis.document.is_some() {
                // Discard local preview state even after a source change; the
                // committed document cache supplies the current pane's scene.
                let editor = &mut self.session.workspace_mut().schematic_authoring;
                editor.circuit_basis = None;
                editor.circuit_preview = None;
                self.invalidate_scene();
            } else if review.source_current(self.workspace()) {
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
                if review.basis.document.is_some() {
                    let mut basis = review.basis.clone();
                    basis.revision = review.revision.clone();
                    let editor = &mut self.session.workspace_mut().schematic_authoring;
                    editor.circuit_basis = Some(basis);
                    editor.circuit_preview = Some(review.preview_scene());
                } else {
                    self.session.workspace_mut().schematic_scene = Some(review.preview_scene());
                }
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
