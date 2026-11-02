//! Existing label form configured for one source-qualified in-place edit.
use super::*;
use datum_gui_protocol::schematic_authoring::{
    FieldsEditDraft, LabelDraft,
    selected::{LabelEditDraft, NamedPointEditDraft, PortEditDraft, TextEditDraft},
};

pub(crate) enum SelectedSchematicEditSnapshot {
    Symbol(FieldsEditDraft, String, String),
    NamedPoint(
        NamedPointEditDraft,
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
        if let Some(draft) = &e.named_edit {
            return Some((draft.basis(), draft.tip()));
        }
        e.editing.as_ref().map(|draft| (draft.basis(), draft.tip()))
    }
    pub(crate) fn selected_schematic_edit_snapshot(&self) -> Option<SelectedSchematicEditSnapshot> {
        let e = &self.workspace().schematic_authoring;
        if let Some(draft) = &e.named_edit {
            return Some(SelectedSchematicEditSnapshot::NamedPoint(
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
            SelectedSchematicEditSnapshot::NamedPoint(draft, label, prior_base) => {
                let basis = draft.basis().clone();
                let base = if self.native_edit.blocked {
                    prior_base
                } else {
                    self.workspace().schematic_scene.clone()
                };
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.named_edit = Some(draft);
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
        self.begin_selected_named_point_edit(datum_gui_protocol::AuthoredSelectionClass::Label)
    }
    pub(crate) fn begin_selected_port_edit(&mut self) -> bool {
        self.begin_selected_named_point_edit(datum_gui_protocol::AuthoredSelectionClass::Port)
    }
    pub(crate) fn begin_selected_text_edit(&mut self) -> bool {
        self.begin_selected_named_point_edit(
            datum_gui_protocol::AuthoredSelectionClass::SchematicText,
        )
    }
    fn begin_selected_named_point_edit(
        &mut self,
        class: datum_gui_protocol::AuthoredSelectionClass,
    ) -> bool {
        let prepare = (|| {
            self.selected_schematic_idle()?;
            if class == datum_gui_protocol::AuthoredSelectionClass::SchematicText {
                selected::text_admission(self.workspace())?;
            } else if class == datum_gui_protocol::AuthoredSelectionClass::Port {
                selected::port_admission(self.workspace())?;
            } else {
                selected::label_admission(self.workspace())?;
            }
            let (basis, target) = selected::target(self.workspace())?;
            let sheet = self
                .workspace()
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|source| source.sheets.get(&basis.source))
                .ok_or("Selected schematic source unavailable")?;
            let base = datum_gui_protocol::camera_scene_for_pane(self.workspace(), basis.pane)
                .cloned()
                .ok_or("Selected schematic scene unavailable")?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok());
            let draft = if class == datum_gui_protocol::AuthoredSelectionClass::SchematicText {
                NamedPointEditDraft::Text(TextEditDraft {
                    original: sheet
                        .texts
                        .get(&target.id)
                        .cloned()
                        .ok_or("Selected Text source unavailable")?,
                    target,
                    basis,
                    tip,
                })
            } else if class == datum_gui_protocol::AuthoredSelectionClass::Port {
                NamedPointEditDraft::Port(PortEditDraft {
                    original: sheet
                        .ports
                        .get(&target.id)
                        .cloned()
                        .ok_or("Selected Port source unavailable")?,
                    target,
                    basis,
                    tip,
                })
            } else {
                NamedPointEditDraft::Label(LabelEditDraft {
                    original: sheet
                        .labels
                        .get(&target.id)
                        .cloned()
                        .ok_or("Selected Label source unavailable")?,
                    target,
                    basis,
                    tip,
                })
            };
            Ok::<_, String>((draft, base))
        })();
        match prepare {
            Ok((draft, base)) => {
                self.cancel_native_schematic();
                let basis = draft.basis().clone();
                let label = draft.form();
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.position = Some(draft.position());
                e.label = Some(label);
                e.named_edit = Some(draft);
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
                .named_edit
                .as_ref()
                .ok_or("No selected named-point edit is open")?;
            let label = e
                .label
                .as_ref()
                .ok_or("Name and choice fields are unavailable")?;
            let (basis, target) = selected::target(self.workspace())?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok());
            if &basis != draft.basis() || &target != draft.target() || tip != draft.tip() {
                return Err(
                    "Selection or Project history changed; cancel and reopen the edit".into(),
                );
            }
            let current = e.source.as_ref().and_then(|s| s.sheets.get(&basis.source));
            if current.is_none_or(|sheet| !draft.source_matches(sheet)) {
                return Err("Object source changed; cancel and reopen the edit".into());
            }
            draft.request(label)
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
