//! Library bindings for the shared native single-line controller, not a text owner.
use super::editor::{Snapshot, Target};
use super::*;
use datum_gui_protocol::library_browser::LibraryControl as C;

pub(super) fn target(runtime: &Runtime) -> Option<Target> {
    let state = runtime.workspace();
    if state.ui.active_menu.is_some()
        || state.ui.new_project.open
        || state.ui.project_open.open
        || (state.ui.layout.focused_content() != PaneContent::Library
            && !runtime.component_picker_open())
        || runtime.application_focus()
            != datum_gui_protocol::ApplicationFocus::Editor(state.ui.layout.focused)
    {
        return None;
    }
    if state.library.acquisition.is_some() {
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
    let draft = runtime.workspace().library.draft.as_ref()?;
    let field = draft.fields.get(index)?;
    (!field.selector).then_some(Target::LibraryField(draft.id, index, field.key))
}

pub(super) fn snapshot(runtime: &Runtime, target: Target) -> Option<Snapshot> {
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
                        }
                    }
                }
            }
        }
        Target::LibraryField(id, index, key) => {
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
