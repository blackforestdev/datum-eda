//! Existing label form configured for one source-qualified in-place edit.
use super::*;
use datum_gui_protocol::schematic_authoring::{LabelDraft, selected::LabelEditDraft};

pub(crate) enum SelectedSchematicEditSnapshot {
    Symbol(SymbolEditDraft, String, String),
    Label(
        LabelEditDraft,
        LabelDraft,
        Option<datum_gui_protocol::BoardReviewSceneV1>,
    ),
}
impl Runtime {
    pub(crate) fn selected_schematic_edit_basis(
        &self,
    ) -> Option<(
        &datum_gui_protocol::AuthoringViewBasis,
        Option<eda_engine::substrate::ObjectId>,
    )> {
        let e = &self.workspace().schematic_authoring;
        if let Some(draft) = &e.label_edit {
            return Some((&draft.basis, draft.tip));
        }
        e.editing.as_ref().map(|draft| (&draft.basis, draft.tip))
    }
    pub(crate) fn selected_schematic_edit_snapshot(&self) -> Option<SelectedSchematicEditSnapshot> {
        let e = &self.workspace().schematic_authoring;
        if let Some(draft) = &e.label_edit {
            return Some(SelectedSchematicEditSnapshot::Label(
                draft.clone(),
                e.label.clone()?,
                e.base.clone(),
            ));
        }
        self.selected_symbol_edit_snapshot()
            .map(|(draft, reference, value)| {
                SelectedSchematicEditSnapshot::Symbol(draft, reference, value)
            })
    }
    pub(crate) fn restore_selected_schematic_edit(
        &mut self,
        snapshot: SelectedSchematicEditSnapshot,
        reason: &str,
    ) {
        match snapshot {
            SelectedSchematicEditSnapshot::Symbol(draft, reference, value) => {
                self.restore_selected_symbol_edit((draft, reference, value), reason)
            }
            SelectedSchematicEditSnapshot::Label(draft, label, prior_base) => {
                let basis = draft.basis.clone();
                let base = if self.native_edit.blocked {
                    prior_base
                } else {
                    self.workspace().schematic_scene.clone()
                };
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.label_edit = Some(draft);
                e.label = Some(label);
                e.base = base;
                e.picker = true;
                e.preview_dirty = false;
                e.pending_request = None;
                e.refusal = Some(reason.into());
                self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
            }
        }
    }
    pub(crate) fn begin_selected_label_edit(&mut self) -> bool {
        let prepare = (|| {
            self.selected_schematic_idle()?;
            selected::label_admission(self.workspace())?;
            let (basis, target) = selected::target(self.workspace())?;
            let original = self
                .workspace()
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|source| source.sheets.get(&basis.source))
                .and_then(|sheet| sheet.labels.get(&target.id))
                .cloned()
                .ok_or("Selected Label source unavailable")?;
            let base = datum_gui_protocol::camera_scene_for_pane(self.workspace(), basis.pane)
                .cloned()
                .ok_or("Selected Label scene unavailable")?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok());
            Ok::<_, String>((
                LabelEditDraft {
                    target,
                    basis,
                    tip,
                    original,
                },
                base,
            ))
        })();
        match prepare {
            Ok((draft, base)) => {
                self.cancel_native_schematic();
                let basis = draft.basis.clone();
                let mut label = LabelDraft::new(draft.original.kind.clone());
                label.name = draft.original.name.clone();
                label.editor =
                    datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&label.name);
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.position = Some(PointNm {
                    x: draft.original.position.x,
                    y: draft.original.position.y,
                });
                e.label = Some(label);
                e.label_edit = Some(draft);
                e.base = Some(base);
                e.picker = true;
                e.preview_dirty = true;
                e.refusal = None;
                self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
                self.invalidate_frame();
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        true
    }
    pub(crate) fn apply_selected_label_fields(&mut self) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let request = (|| {
            let e = &self.workspace().schematic_authoring;
            let draft = e
                .label_edit
                .as_ref()
                .ok_or("No selected Label edit is open")?;
            let label = e.label.as_ref().ok_or("Label fields are unavailable")?;
            let (basis, target) = selected::target(self.workspace())?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok());
            if basis != draft.basis || target != draft.target || tip != draft.tip {
                return Err(
                    "Selection or Project history changed; cancel and reopen the edit".into(),
                );
            }
            let current = e
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&basis.source))
                .and_then(|s| s.labels.get(&target.id));
            if current != Some(&draft.original) {
                return Err("Label source changed; cancel and reopen the edit".into());
            }
            draft.request(&label.name, label.kind.clone())
        })();
        match request {
            Ok(None) => {
                self.cancel_native_schematic();
                self.invalidate_frame();
            }
            Ok(Some(request)) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .pending_request = Some(request);
                return self.queue_native_edit(NativeEditAction::Schematic);
            }
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
                self.invalidate_frame();
            }
        }
        true
    }
}
