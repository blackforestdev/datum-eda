//! Complete-sheet annotation configures the shared immutable Proposal review.
use super::delete_review::DeleteReview;
use crate::{AuthoringViewBasis, BoardReviewSceneV1, PaneContent, ReviewWorkspaceState};
use eda_engine::api::native_write::{
    schematic_annotation, schematic_delete_proposal::AnnotationSummary,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationEvidence {
    pub summary: AnnotationSummary,
    pub preview: BoardReviewSceneV1,
}
impl AnnotationEvidence {
    /// Readiness uses captured facts only; canonical preview is read on invocation.
    pub fn admit(state: &ReviewWorkspaceState) -> Result<(), &'static str> {
        if !crate::gui_menu_model::native_project_edit_available(state) {
            return Err("Open a persistent native Project");
        }
        let editor = &state.schematic_authoring;
        if editor.delete_review.is_some()
            || editor.active()
            || editor.picker
            || editor.creation.is_some()
            || state.library.draft.is_some()
            || state.library.acquisition.is_some()
            || matches!(
                state.check_producer.phase,
                crate::CheckProducerPhase::Starting | crate::CheckProducerPhase::Running
            )
        {
            return Err("Finish or cancel the current operation first");
        }
        let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)
            .map_err(|_| "Focus a current supported schematic sheet")?;
        if basis.content != PaneContent::Schematic {
            return Err("Focus a current supported schematic sheet");
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current source unavailable")?;
        let sheet = editor
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .ok_or("Current source sheet unavailable")?;
        let path = &basis
            .document
            .as_ref()
            .ok_or("Qualified sheet unavailable")?
            .instance_path;
        for symbol in sheet.symbols.values() {
            if editor.move_paths.get(&symbol.uuid) != Some(path) {
                return Err("Annotation requires one supported occurrence of every Symbol");
            }
            let owners: Vec<_> = model
                .component_instances
                .values()
                .filter(|ci| ci.placed_symbol_refs.contains(&symbol.uuid))
                .collect();
            if owners.len() > 1
                || owners.first().is_some_and(|ci| {
                    ci.authority != eda_engine::substrate::ComponentInstanceAuthority::Authored
                        || ci.placed_symbol_refs.is_empty()
                        || ci
                            .placed_symbol_refs
                            .iter()
                            .any(|id| !sheet.symbols.contains_key(id))
                        || !ci.placed_package_refs.is_empty()
                })
            {
                return Err(
                    "Annotation requires complete same-sheet schematic-only bindings; nothing is omitted",
                );
            }
        }
        Ok(())
    }
    pub(super) fn capture(
        state: &ReviewWorkspaceState,
        review: &DeleteReview,
    ) -> Result<Self, String> {
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current source unavailable")?;
        if model.project.project_id != review.basis.project
            || model.model_revision.0 != review.basis.revision
        {
            return Err("Annotation source model no longer matches the frozen review basis".into());
        }
        let path = &review
            .basis
            .document
            .as_ref()
            .ok_or("Qualified sheet unavailable")?
            .instance_path;
        let summary = schematic_annotation::preview(model, review.basis.source, path)
            .map_err(|e| e.to_string())?;
        let root = &state
            .backing
            .as_ref()
            .ok_or("Native project source unavailable")?
            .request
            .project_root;
        Self::from_summary(state, review, summary, root)
    }
    pub(crate) fn from_summary(
        state: &ReviewWorkspaceState,
        review: &DeleteReview,
        summary: AnnotationSummary,
        root: &std::path::Path,
    ) -> Result<Self, String> {
        if summary.sheet != review.basis.source
            || review
                .basis
                .document
                .as_ref()
                .is_none_or(|d| d.instance_path != summary.instance_path)
            || summary.symbol_count != review.source.symbols.len()
        {
            return Err("Annotation sheet or complete Symbol census changed".into());
        }
        let mut source = state
            .schematic_authoring
            .source
            .clone()
            .ok_or("Source unavailable")?;
        if source.sheets.get(&summary.sheet) != Some(&review.source) {
            return Err("Annotation source changed".into());
        }
        let mut changed = BTreeSet::new();
        for change in &summary.changes {
            let symbol = source
                .sheets
                .get_mut(&summary.sheet)
                .unwrap()
                .symbols
                .get_mut(&change.symbol)
                .ok_or("Annotation contains another sheet's Symbol")?;
            if !changed.insert(change.symbol)
                || symbol.reference != change.previous_reference
                || change.previous_reference == change.reference
            {
                return Err("Annotation contains changed or duplicate reference evidence".into());
            }
            // Display-only old/new text at the existing owned reference anchor.
            // The source Sheet and canonical requested references remain unchanged.
            symbol.reference = format!("{} -> {}", change.previous_reference, change.reference);
        }
        let preview = if summary.changes.is_empty() {
            review.scene.clone()
        } else {
            state
                .schematic_authoring
                .project_source(&source, summary.sheet, &review.scene, root)
                .map_err(|e| e.to_string())?
        };
        for change in &summary.changes {
            let name = format!("{} -> {}", change.previous_reference, change.reference);
            if !preview.board_texts.iter().any(|text| {
                text.source_object_uuid.as_deref() == Some(&change.symbol.to_string())
                    && text.text == name
                    && preview.board_text_geometries.iter().any(|geometry| {
                        geometry.object_id == text.object_id && !geometry.glyphs.is_empty()
                    })
            }) {
                return Err("Every reference change must have a visible scene preview".into());
            }
        }
        Ok(Self { summary, preview })
    }
}
