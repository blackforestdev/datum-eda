//! Library bindings for the shared native single-line controller, not a text owner.
use super::editor::{Snapshot, Target};
use super::*;
use datum_gui_protocol::library_browser::LibraryControl as C;
use datum_gui_protocol::library_browser::revision_workflow::{
    RevisionControl as R,
    provenance::{ProvenanceAction as P, ProvenanceField},
};

pub(super) fn pointer_target(runtime: &Runtime, control: &C) -> Option<Target> {
    match control {
        C::AcquirePath => Some(Target::LibraryPath),
        C::AcquireSearch => Some(Target::LibrarySearch),
        C::Field(index) => field_target(runtime, *index),
        C::Revision(control) if control.is_text_field() => target(runtime),
        _ => None,
    }
}

pub(super) fn target(runtime: &Runtime) -> Option<Target> {
    let state = runtime.workspace();
    if state.ui.active_menu.is_some()
        || state.ui.new_project.open
        || state.ui.project_open.open
        || (state.ui.layout.focused_content() != PaneContent::Library
            && !runtime.component_picker_open()
            && state.library.configuration.is_none())
        || runtime.application_focus()
            != datum_gui_protocol::ApplicationFocus::Editor(state.ui.layout.focused)
    {
        return None;
    }
    if let Some(C::Revision(
        control @ R::Provenance {
            action: P::Field(field),
            ..
        },
    )) = state.library.focus
    {
        let review = state.library.revision_workflow.as_ref()?;
        review.validate_review_control(control).ok()?;
        if !review.provenance_edit_available(
            &state.library,
            &state.backing.as_ref()?.request.project_root,
        ) {
            return None;
        }
        return Some(Target::LibraryField(
            field.row,
            field.index,
            "library_provenance_field",
        ));
    }
    if let Some(C::Revision(
        datum_gui_protocol::library_browser::revision_workflow::RevisionControl::Purpose(session),
    )) = state.library.focus
    {
        let review = state.library.revision_workflow.as_ref()?;
        if review.session_id() != session {
            return None;
        }
        return Some(Target::LibraryField(
            review.session_id(),
            0,
            "library_revision_purpose",
        ));
    }
    if state.library.acquisition.is_some() && !matches!(state.library.focus, Some(C::Field(_))) {
        return match state.library.focus {
            Some(C::AcquirePath) => Some(Target::LibraryPath),
            Some(C::AcquireSearch) => Some(Target::LibrarySearch),
            _ => None,
        };
    }
    if let Some(C::Field(index)) = state.library.focus {
        field_target(runtime, index)
    } else {
        None
    }
}

pub(super) fn field_target(runtime: &Runtime, index: usize) -> Option<Target> {
    if let Some(configuration) = &runtime.workspace().library.configuration
        && configuration.footprint_chooser.is_none()
    {
        return match index {
            0 => Some(Target::LibraryField(
                configuration.session,
                0,
                "component_reference",
            )),
            1 => Some(Target::LibraryField(
                configuration.session,
                1,
                "component_value",
            )),
            _ => None,
        };
    }
    let draft = runtime.workspace().library.draft.as_ref()?;
    let field = draft.fields.get(index)?;
    (!field.selector).then_some(Target::LibraryField(draft.id, index, field.key))
}

pub(super) fn snapshot(runtime: &Runtime, target: Target) -> Option<Snapshot> {
    runtime.trace_component_editor(&format!("snapshot {target:?}"));
    let browser = &runtime.workspace().library;
    let (text, editor) = match target {
        Target::LibraryPath | Target::LibrarySearch => {
            let a = browser.acquisition.as_ref()?;
            if target == Target::LibraryPath {
                (&a.path, a.path_editor)
            } else {
                (&a.search, a.search_editor)
            }
        }
        Target::LibraryField(id, index, key) => {
            if key == "library_provenance_field" {
                let review = browser.revision_workflow.as_ref()?;
                if !review.provenance_edit_available(
                    browser,
                    &runtime.workspace().backing.as_ref()?.request.project_root,
                ) {
                    return None;
                }
                let field = review.provenance_field(ProvenanceField { row: id, index })?;
                return Some(Snapshot {
                    text: field.text.clone(),
                    editor: field.editor,
                    editable: !review.is_submitted()
                        && !runtime.native_edit.pending()
                        && !runtime.native_edit.blocked
                        && datum_gui_protocol::CheckProducerRequest::workspace_available(
                            runtime.workspace(),
                        ),
                });
            }
            if key == "library_revision_purpose" {
                let review = browser
                    .revision_workflow
                    .as_ref()
                    .filter(|review| review.session_id() == id)?;
                return Some(Snapshot {
                    text: review.purpose().into(),
                    editor: review.purpose_editor(),
                    editable: !review.is_submitted() && !runtime.native_edit.pending(),
                });
            }
            if let Some(configuration) = &browser.configuration
                && configuration.session == id
                && browser.focus == Some(C::Field(index))
            {
                let (text, editor) = match (index, key) {
                    (0, "component_reference") => {
                        (&configuration.reference, configuration.reference_editor)
                    }
                    (1, "component_value") => (&configuration.value, configuration.value_editor),
                    _ => return None,
                };
                return Some(Snapshot {
                    text: text.clone(),
                    editor,
                    editable: !configuration.pending() && !runtime.native_edit.pending(),
                });
            }
            let draft = browser.draft.as_ref()?;
            let field = draft.fields.get(index)?;
            if draft.id != id
                || draft.focused != index
                || browser.focus != Some(C::Field(index))
                || field.key != key
                || field.selector
            {
                return None;
            }
            (&field.text, field.editor)
        }
        _ => return None,
    };
    Some(Snapshot {
        text: text.clone(),
        editor,
        editable: !runtime.native_edit.pending(),
    })
}

pub(super) fn store(runtime: &mut Runtime, target: Target, snapshot: Snapshot, changed: bool) {
    if matches!(
        target,
        Target::LibraryField(_, _, "library_provenance_field")
    ) && (runtime.native_edit.pending()
        || runtime.native_edit.blocked
        || !datum_gui_protocol::CheckProducerRequest::workspace_available(runtime.workspace())
        || !runtime
            .workspace()
            .library
            .revision_workflow
            .as_ref()
            .zip(runtime.workspace().backing.as_ref())
            .is_some_and(|(r, b)| {
                r.provenance_edit_available(&runtime.workspace().library, &b.request.project_root)
            }))
    {
        return;
    }
    let browser = &mut runtime.session.workspace_mut().library;
    match target {
        Target::LibraryPath | Target::LibrarySearch => {
            if let Some(a) = &mut browser.acquisition {
                if target == Target::LibraryPath {
                    a.path = snapshot.text;
                    a.path_editor = snapshot.editor;
                    if changed {
                        a.source_changed();
                    }
                } else {
                    a.search = snapshot.text;
                    a.search_editor = snapshot.editor;
                    if changed {
                        a.search_changed();
                        if a.component_picker {
                            browser.scroll_offset = 0;
                            browser.configuration = None;
                        }
                    }
                }
            }
        }
        Target::LibraryField(id, index, key) => {
            if key == "library_provenance_field" {
                if !matches!(browser.focus, Some(C::Revision(R::Provenance { action: P::Field(field), .. })) if field == (ProvenanceField { row: id, index }))
                {
                    return;
                }
                if let Some(review) = browser.revision_workflow.as_mut() {
                    let target = ProvenanceField { row: id, index };
                    match review.set_provenance_field(target, snapshot.text, snapshot.editor) {
                        Ok(()) => {
                            browser.focus =
                                Some(C::Revision(review.provenance_control(P::Field(target))))
                        }
                        Err(error) => browser.refusal = Some(error),
                    }
                }
                runtime.invalidate_frame();
                return;
            }
            if key == "library_revision_purpose" {
                if let Some(review) = browser
                    .revision_workflow
                    .as_mut()
                    .filter(|review| review.session_id() == id)
                {
                    if changed && let Err(error) = review.set_purpose(snapshot.text) {
                        browser.refusal = Some(error);
                        return;
                    }
                    review.set_purpose_editor(snapshot.editor);
                }
                runtime.invalidate_frame();
                return;
            }
            if let Some(configuration) = &mut browser.configuration
                && configuration.session == id
                && browser.focus == Some(C::Field(index))
            {
                match (index, key) {
                    (0, "component_reference") => {
                        configuration.reference = snapshot.text;
                        configuration.reference_editor = snapshot.editor;
                    }
                    (1, "component_value") => {
                        configuration.value = snapshot.text;
                        configuration.value_editor = snapshot.editor;
                    }
                    _ => return,
                };
                if changed {
                    configuration.refusal = None;
                }
                runtime.invalidate_frame();
                return;
            }
            if let Some(draft) = &mut browser.draft
                && draft.id == id
                && draft.focused == index
                && browser.focus == Some(C::Field(index))
                && draft
                    .fields
                    .get(index)
                    .is_some_and(|field| field.key == key && !field.selector)
            {
                if changed {
                    draft.change(snapshot.text);
                }
                draft.fields[index].editor = snapshot.editor;
            }
        }
        _ => return,
    }
    runtime.invalidate_frame();
}
