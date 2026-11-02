//! Library browsing returns to the exact schematic occurrence that requested it.
use super::*;
use datum_gui_protocol::{
    PaneId, library_browser::LibraryControl, schematic_documents::SchematicDocumentKey,
};

pub(crate) struct PlacementOrigin {
    pane: PaneId,
    document: SchematicDocumentKey,
    zoom: Option<PaneId>,
    temporary_library: Option<PaneId>,
}

fn origin_is_current(
    state: &datum_gui_protocol::ReviewWorkspaceState,
    origin: &PlacementOrigin,
) -> bool {
    let (pane, document) = (&origin.pane, &origin.document);
    state.ui.layout.content_for(*pane) == Some(PaneContent::Schematic)
        && state.schematic_documents.panes.get(pane) == Some(document)
        && state.schematic_documents.project == Some(document.project)
}

fn restore_origin_layout(
    layout: &mut datum_gui_protocol::WorkspaceLayout,
    origin: &PlacementOrigin,
) {
    if let Some(temporary) = origin.temporary_library
        && layout.content_for(temporary) == Some(PaneContent::Library)
    {
        layout.focused = temporary;
        layout.close_focused();
    }
    layout.focused = origin.pane;
    layout.zoomed = origin
        .zoom
        .filter(|pane| layout.content_for(*pane).is_some());
}

impl Runtime {
    /// Placement reuses the saved Project definition, even if its original
    /// external source changed; the source read never silently updates it.
    pub(crate) fn preview_existing_project_symbol(&mut self) {
        let state = self.workspace();
        let Some(a) = state
            .library
            .acquisition
            .as_ref()
            .filter(|a| a.placement_pane.is_some())
        else {
            return;
        };
        let Some(preview) = &a.preview else { return };
        let Some(symbol_id) = preview
            .symbol
            .get("uuid")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse::<eda_engine::substrate::ObjectId>().ok())
        else {
            return;
        };
        let Some(symbol) = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "symbols" && e.id == symbol_id)
        else {
            return;
        };
        let Some(unit_id) = symbol
            .raw
            .get("unit")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse::<eda_engine::substrate::ObjectId>().ok())
        else {
            return;
        };
        let Some(unit) = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "units" && e.id == unit_id)
        else {
            return;
        };
        let (symbol, unit) = (symbol.raw.clone(), unit.raw.clone());
        let preview = self
            .session
            .workspace_mut()
            .library
            .acquisition
            .as_mut()
            .unwrap()
            .preview
            .as_mut()
            .unwrap();
        preview.symbol = symbol;
        preview.unit = unit;
        preview.notices.insert(0, "Already in this Project: preview and placement use the saved Project definition. Library updates are not applied.".into());
    }

    pub(crate) fn browse_schematic_libraries(&mut self) -> bool {
        if self.native_edit.pending()
            || self.workspace().library.draft.is_some()
            || self.workspace().library.acquisition.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish the current library edit first",
            );
            return true;
        }
        let pane = self.workspace().ui.layout.focused;
        let Some(document) = self
            .workspace()
            .schematic_documents
            .panes
            .get(&pane)
            .cloned()
        else {
            return true;
        };
        if !self.guard_native_authoring_view(Some(pane)) {
            return true;
        }
        let zoom = self.workspace().ui.layout.zoomed;
        self.activate_library(LibraryControl::BrowseBundled);
        let Some(acquisition) = &mut self.session.workspace_mut().library.acquisition else {
            return true;
        };
        acquisition.placement_pane = Some(pane);
        acquisition.placement_document = Some(document);
        acquisition.placement_zoom = zoom;
        self.cancel_native_schematic();
        let layout = &self.workspace().ui.layout;
        let existing = layout
            .leaves()
            .into_iter()
            .find(|id| layout.content_for(*id) == Some(PaneContent::Library));
        let mut temporary = None;
        self.swap_pane_focus(|layout| {
            let library = existing.unwrap_or_else(|| {
                let id = layout.open_beside_root(
                    PaneContent::Library,
                    datum_gui_protocol::SplitOrientation::Vertical,
                    0.5,
                    true,
                );
                temporary = Some(id);
                id
            });
            layout.focused = library;
            layout.zoomed = Some(library);
        });
        if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
            a.placement_temporary_pane = temporary;
        }
        true
    }

    pub(crate) fn acquisition_placement_origin(&self) -> Option<PlacementOrigin> {
        let acquisition = self.workspace().library.acquisition.as_ref()?;
        Some(PlacementOrigin {
            pane: acquisition.placement_pane?,
            document: acquisition.placement_document.clone()?,
            zoom: acquisition.placement_zoom,
            temporary_library: acquisition.placement_temporary_pane,
        })
    }

    pub(crate) fn acquire_symbol_for_placement(&mut self) -> bool {
        let result = (|| {
            let origin = self
                .acquisition_placement_origin()
                .ok_or("Schematic origin unavailable")?;
            if !origin_is_current(self.workspace(), &origin) {
                return Err(
                    "The original schematic view changed; cancel and browse from that sheet again",
                );
            }
            let acquisition = self.workspace().library.acquisition.as_ref().unwrap();
            if acquisition.loading || acquisition.request().is_err() {
                return Err("Preview a supported symbol before using it");
            }
            let id = acquisition
                .preview
                .as_ref()
                .and_then(|p| p.symbol.get("uuid"))
                .and_then(serde_json::Value::as_str)
                .and_then(|s| s.parse().ok())
                .ok_or("Selected symbol identity unavailable")?;
            Ok((origin, id))
        })();
        let (origin, id) = match result {
            Ok(value) => value,
            Err(reason) => {
                if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                    a.refusal = Some(reason.into());
                }
                self.invalidate_frame();
                return true;
            }
        };
        if self
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .any(|c| c.id == id)
        {
            self.session.workspace_mut().library.acquisition = None;
            self.library_acquisition_work.cancel();
            self.resume_library_placement(origin, Some(id));
            self.log_console_echo(
                ConsoleFeedbackSource::Tool,
                "Using the existing Project symbol; external library changes are not applied",
            );
            true
        } else {
            self.queue_native_edit(NativeEditAction::LibraryAcquire)
        }
    }

    pub(crate) fn browse_project_symbols_for_placement(&mut self) -> bool {
        let Some(origin) = self.acquisition_placement_origin() else {
            return true;
        };
        if !origin_is_current(self.workspace(), &origin) {
            if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                a.refusal = Some(
                    "The original schematic view changed; cancel and browse from that sheet again"
                        .into(),
                );
            }
            self.invalidate_frame();
            return true;
        }
        self.session.workspace_mut().library.acquisition = None;
        self.library_acquisition_work.cancel();
        self.swap_pane_focus(|layout| restore_origin_layout(layout, &origin));
        self.prepare_native_schematic(false);
        true
    }

    pub(crate) fn resume_library_placement(
        &mut self,
        origin: PlacementOrigin,
        symbol: Option<eda_engine::substrate::ObjectId>,
    ) {
        if !origin_is_current(self.workspace(), &origin) {
            self.log_console_refusal(ConsoleFeedbackSource::Tool,
                "The original schematic view changed; select a sheet and place the Project symbol there");
            return;
        }
        self.swap_pane_focus(|layout| restore_origin_layout(layout, &origin));
        let Some(id) = symbol else {
            self.cancel_native_schematic();
            return;
        };
        if !self.prepare_native_schematic(false) {
            return;
        }
        {
            let editor = &mut self.session.workspace_mut().schematic_authoring;
            editor.chosen = None;
            if let Some(index) = editor.choices.iter().position(|c| c.id == id) {
                editor.select(index);
                editor.choice_offset = (index / 4) * 4;
            } else {
                editor.refusal = Some("The selected symbol is unavailable in this Project".into());
            }
        }
        self.start_symbol_preview();
        self.invalidate_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_return_rejects_rebound_occurrence_closed_pane_and_other_project() {
        let mut state = datum_gui_protocol::ReviewWorkspaceState::unopened();
        let pane = state.ui.layout.focused;
        state.ui.layout.set_focused_content(PaneContent::Schematic);
        let key = SchematicDocumentKey {
            project: eda_engine::substrate::ObjectId::new_v4(),
            schematic: eda_engine::substrate::ObjectId::new_v4(),
            source_sheet: eda_engine::substrate::ObjectId::new_v4(),
            instance_path: vec![eda_engine::substrate::ObjectId::new_v4()],
        };
        state.schematic_documents.project = Some(key.project);
        state.schematic_documents.panes.insert(pane, key.clone());
        let origin = PlacementOrigin {
            pane,
            document: key.clone(),
            zoom: None,
            temporary_library: None,
        };
        assert!(origin_is_current(&state, &origin));
        state
            .schematic_documents
            .panes
            .get_mut(&pane)
            .unwrap()
            .instance_path
            .clear();
        assert!(!origin_is_current(&state, &origin));
        state.schematic_documents.panes.insert(pane, key.clone());
        state.schematic_documents.project = Some(eda_engine::substrate::ObjectId::new_v4());
        assert!(!origin_is_current(&state, &origin));
        state.schematic_documents.project = Some(key.project);
        state.ui.layout.set_focused_content(PaneContent::Board);
        assert!(!origin_is_current(&state, &origin));
        state.schematic_documents.panes.remove(&pane);
        assert!(!origin_is_current(&state, &origin));
    }

    #[test]
    fn library_return_restores_zoom_and_removes_only_its_own_temporary_pane() {
        let mut layout = datum_gui_protocol::WorkspaceLayout::single();
        let pane = layout.focused;
        layout.set_focused_content(PaneContent::Schematic);
        let unrelated = layout.open_beside_root(
            PaneContent::Board,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            false,
        );
        let before = layout.leaves();
        let temporary = layout.open_beside_root(
            PaneContent::Library,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            true,
        );
        let mut origin = PlacementOrigin {
            pane,
            document: SchematicDocumentKey {
                project: eda_engine::substrate::ObjectId::new_v4(),
                schematic: eda_engine::substrate::ObjectId::new_v4(),
                source_sheet: eda_engine::substrate::ObjectId::new_v4(),
                instance_path: vec![],
            },
            zoom: Some(pane),
            temporary_library: Some(temporary),
        };
        restore_origin_layout(&mut layout, &origin);
        assert_eq!(layout.leaves(), before);
        assert_eq!(layout.content_for(unrelated), Some(PaneContent::Board));
        assert_eq!(layout.focused, pane);
        assert_eq!(layout.zoomed, Some(pane));
        let reused = layout.open_beside_root(
            PaneContent::Library,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            true,
        );
        origin.temporary_library = None;
        restore_origin_layout(&mut layout, &origin);
        assert_eq!(layout.content_for(reused), Some(PaneContent::Library));
        origin.temporary_library = Some(reused);
        layout.focused = reused;
        layout.set_focused_content(PaneContent::Board);
        restore_origin_layout(&mut layout, &origin);
        assert_eq!(layout.content_for(reused), Some(PaneContent::Board));
    }
}
