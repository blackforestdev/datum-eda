//! Existing native acquisition and placement, presented on the captured sheet.
use super::*;
use datum_gui_protocol::library_browser::LibraryControl as C;
use datum_gui_protocol::library_browser::acquisition::{
    AcquisitionSource, LibraryAcquisition, LibrarySourceSymbol, catalog::SourceAvailability,
};

impl Runtime {
    pub(crate) fn component_picker_input_size(&self, search: bool) -> f32 {
        if search && self.component_picker_open() {
            16.
        } else {
            11.5
        }
    }

    pub(crate) fn component_picker_open(&self) -> bool {
        self.workspace()
            .library
            .acquisition
            .as_ref()
            .is_some_and(|a| {
                a.component_picker
                    && a.placement_pane == Some(self.workspace().ui.layout.focused)
                    && self.workspace().ui.layout.focused_content() == PaneContent::Schematic
            })
    }

    pub(crate) fn open_component_picker(&mut self) {
        let state = self.workspace();
        let pane = state.ui.layout.focused;
        let Some(document) = state.schematic_documents.panes.get(&pane).cloned() else {
            return;
        };
        let mut acquisition = LibraryAcquisition::begin(
            state.scene.source_revision.clone(),
            state
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok()),
        );
        acquisition.source = AcquisitionSource::BundledSymbols;
        acquisition.path.clear();
        acquisition.component_picker = true;
        acquisition.placement_pane = Some(pane);
        acquisition.placement_document = Some(document);
        acquisition.placement_zoom = state.ui.layout.zoomed;
        if let Some(backing) = &state.backing {
            let mut parts = std::collections::BTreeMap::new();
            for choice in &state.schematic_authoring.components.choices {
                if let Some(component) = &choice.component {
                    let metadata = state
                        .library
                        .entries
                        .iter()
                        .find(|entry| entry.kind == "parts" && entry.id == component.part.part)
                        .map(|entry| {
                            format!(
                                "{} {}",
                                entry
                                    .raw
                                    .get("description")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or(""),
                                entry
                                    .raw
                                    .get("tags")
                                    .map(ToString::to_string)
                                    .unwrap_or_default()
                            )
                        })
                        .unwrap_or_default();
                    parts
                        .entry(component.part.part)
                        .or_insert_with(|| LibrarySourceSymbol {
                            path: backing.request.project_root.clone(),
                            name: component.part.part.to_string(),
                            label: choice.name.clone(),
                            description: format!(
                                "{} {} {} {}",
                                choice.name, choice.value, component.package_name, metadata
                            ),
                            availability: SourceAvailability::Supported,
                        });
                }
            }
            acquisition.project_components = parts.into_values().collect();
        }
        acquisition.loading = true;
        self.library_acquisition_work.start(&acquisition, None);
        let browser = &mut self.session.workspace_mut().library;
        browser.focus = Some(C::AcquireSearch);
        browser.scroll_offset = 0;
        browser.reveal_focus = true;
        browser.acquisition = Some(acquisition);
    }

    pub(crate) fn confirm_component_picker(&mut self) -> bool {
        let result = (|| {
            let origin = self
                .acquisition_placement_origin()
                .ok_or("Captured schematic unavailable")?;
            if !origin_is_current(self.workspace(), &origin) {
                return Err(
                    "The original schematic view changed; cancel and open Place Component again",
                );
            }
            let a = self.workspace().library.acquisition.as_ref().unwrap();
            a.selected_native_part()
                .ok_or("Preview a complete component before placing")?;
            if !a
                .preview
                .as_ref()
                .and_then(|p| p.native_part.as_ref())
                .is_some_and(|part| part.in_project(&self.workspace().library))
            {
                return Err("The project component changed; cancel and search again");
            }
            a.sources
                .iter()
                .find(|s| {
                    Some(s.name.as_str()) == a.selected.as_deref()
                        && Some(&s.path) == a.selected_source.as_ref()
                })
                .cloned()
                .ok_or("Selected source identity unavailable; refresh the source")
        })();
        match result {
            Ok(source) => {
                let a = self
                    .session
                    .workspace_mut()
                    .library
                    .acquisition
                    .as_mut()
                    .unwrap();
                a.placement_confirmation = Some(a.session);
                a.refusal = None;
                a.loading = true;
                // Re-read the pinned closure before handing off to the existing ghost.
                self.library_acquisition_work.start(a, Some(source));
            }
            Err(reason) => {
                self.session
                    .workspace_mut()
                    .library
                    .acquisition
                    .as_mut()
                    .unwrap()
                    .refusal = Some(reason.into())
            }
        }
        self.invalidate_frame();
        true
    }

    pub(crate) fn finish_component_picker_placement(&mut self) {
        let Some(origin) = self.acquisition_placement_origin() else {
            return;
        };
        let Some(id) = self
            .workspace()
            .library
            .acquisition
            .as_ref()
            .and_then(|a| a.selected_native_part())
        else {
            return;
        };
        if !origin_is_current(self.workspace(), &origin) {
            self.session
                .workspace_mut()
                .library
                .acquisition
                .as_mut()
                .unwrap()
                .refusal = Some(
                "The original schematic view changed; cancel and open Place Component again".into(),
            );
            return;
        }
        self.session.workspace_mut().library.acquisition = None;
        self.library_acquisition_work.cancel();
        self.resume_library_placement(origin, Some(id));
        // The source preview identifies the first gate; further gates retain
        // the existing explicit Place another Gate workflow.
        if self.workspace().schematic_authoring.picker
            && self.workspace().schematic_authoring.chosen.is_some()
        {
            self.start_symbol_preview();
        }
    }

    pub(crate) fn restore_component_picker_after_adoption(
        &mut self,
        mut acquisition: LibraryAcquisition,
    ) {
        let Some(origin) = self.acquisition_placement_origin_for(&acquisition) else {
            return;
        };
        if !origin_is_current(self.workspace(), &origin) {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, "Library adopted; the original schematic changed. Open Place Component on the intended sheet.");
            return;
        }
        self.swap_pane_focus(|layout| restore_origin_layout(layout, &origin));
        if !self.prepare_native_circuit(false) {
            return;
        }
        self.session
            .workspace_mut()
            .schematic_authoring
            .begin_component_placement();
        acquisition.revision = self.workspace().scene.source_revision.clone();
        acquisition.tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .as_ref()
            .and_then(|tip| tip.parse().ok());
        acquisition.placement_confirmation = None;
        acquisition.refusal = None;
        self.session.workspace_mut().library.acquisition = Some(acquisition);
        self.session.workspace_mut().library.focus = Some(C::AcquireSearch);
        self.invalidate_frame();
    }
}
