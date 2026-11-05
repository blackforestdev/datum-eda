//! Configure the shared name form, snapped placement and explicit Proposal review.
use super::*;
use datum_gui_protocol::schematic_authoring::child_sheet::ChildSheetDraft;
impl Runtime {
    pub(crate) fn begin_child_sheet(&mut self) -> bool {
        if !self.prepare_native_schematic_in(false, true) {
            return true;
        }
        match ChildSheetDraft::capture(self.workspace()) {
            Ok(draft) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.creation = Some(NewSchematicDraft {
                    name: String::new(),
                    editor: Default::default(),
                    focus: 0,
                    revision: draft.review.revision.clone(),
                    tip: draft.review.tip,
                    refusal: None,
                });
                e.child_draft = Some(draft);
                e.picker = false;
                e.chosen = None;
            }
            Err(reason) => {
                self.cancel_native_schematic();
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
            }
        }
        self.invalidate_frame();
        true
    }
    pub(crate) fn choose_child_sheet_name(&mut self) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let Some(mut child) = self.workspace().schematic_authoring.child_draft.clone() else {
            return false;
        };
        let Some(form) = &self.workspace().schematic_authoring.creation else {
            return false;
        };
        match child.name(self.workspace(), form.name.trim().into()) {
            Ok(()) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.child_draft = Some(child);
                e.creation = None;
                e.picker = false;
                e.position = None;
                e.preview_dirty = false;
                e.refusal = None;
            }
            Err(reason) => {
                if let Some(form) = &mut self.session.workspace_mut().schematic_authoring.creation {
                    form.refusal = Some(reason);
                }
            }
        }
        self.invalidate_frame();
        true
    }
    pub(crate) fn review_child_sheet(&mut self, at: datum_gui_protocol::PointNm) -> bool {
        let Some(child) = self.workspace().schematic_authoring.child_draft.clone() else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            return true;
        }
        match child.stage(self.workspace(), at) {
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
    pub(crate) fn child_sheet_key(&mut self, event: &KeyEvent) -> bool {
        match event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.cancel_native_schematic();
                self.invalidate_frame();
                true
            }
            Key::Named(NamedKey::Enter) => {
                if let Some(at) = self.workspace().schematic_authoring.position {
                    return self.review_child_sheet(at);
                }
                true
            }
            // Sheet geometry has no rotation/mirror authoring in this increment.
            Key::Named(NamedKey::Tab) => true,
            Key::Character(ref key)
                if ["r", "x", "y"].iter().any(|s| key.eq_ignore_ascii_case(s)) =>
            {
                true
            }
            _ => false,
        }
    }
}
