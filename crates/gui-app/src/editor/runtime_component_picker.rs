//! Existing native acquisition and placement, presented on the captured sheet.
use super::*;
use datum_gui_protocol::library_browser::LibraryControl as C;
use datum_gui_protocol::library_browser::acquisition::{
    AcquisitionSource, LibraryAcquisition, LibrarySourceSymbol, catalog::SourceAvailability,
};

impl Runtime {
    pub(crate) fn handle_component_picker_navigation(&mut self, key: &Key) -> bool {
        if !matches!(
            key,
            Key::Named(
                NamedKey::Tab
                    | NamedKey::Enter
                    | NamedKey::ArrowDown
                    | NamedKey::ArrowUp
                    | NamedKey::PageDown
                    | NamedKey::PageUp
            )
        ) || !self.component_picker_open()
        {
            return false;
        }
        let library = &self.workspace().library;
        let a = library.acquisition.as_ref().unwrap();
        let results = a.results();
        let selected = results.iter().position(|index| {
            let source = &a.sources[*index];
            a.selected.as_deref() == Some(source.name.as_str())
                && a.selected_source.as_ref() == Some(&source.path)
        });
        let focus = library.focus.clone();
        if matches!(key, Key::Named(NamedKey::Enter)) {
            if self.modifiers.alt_key() || self.modifiers.shift_key() {
                return false;
            }
            let Some(C::AcquireResult(index)) = focus else {
                return false;
            };
            let eligible = results.contains(&index)
                && a.sources.get(index).is_some_and(|source| {
                    !source.is_unavailable()
                        && a.selected.as_deref() == Some(source.name.as_str())
                        && a.selected_source.as_ref() == Some(&source.path)
                })
                && a.picker_can_place(library);
            return if eligible {
                self.activate_library(C::AcquireImport)
            } else {
                true
            };
        }
        let row_height =
            datum_gui_protocol::library_browser::acquisition::COMPONENT_PICKER_ROW_HEIGHT;
        let viewport = self.presented_hits.regions().iter().find_map(|hit| {
            if let HitTarget::Library(C::Scroll { height, offset, .. }) = &hit.target {
                Some((*height as f32, *offset as f32))
            } else {
                None
            }
        });
        if matches!(key, Key::Named(NamedKey::Tab)) {
            let mut controls = vec![C::AcquireSearch];
            if let Some(index) = selected.or_else(|| (!results.is_empty()).then_some(0)) {
                controls.push(C::AcquireResult(results[index]));
            }
            if a.unavailable_count() > 0 {
                controls.push(C::AcquireShowUnavailable);
            }
            if a.picker_can_place(library) {
                controls.push(C::AcquireImport);
            }
            controls.push(C::Cancel);
            let current = focus
                .as_ref()
                .and_then(|c| controls.iter().position(|v| v == c));
            let next = current.map_or(0, |i| {
                (i + if self.modifiers.shift_key() {
                    controls.len() - 1
                } else {
                    1
                }) % controls.len()
            });
            let control = controls[next].clone();
            if let C::AcquireResult(index) = control {
                self.select_component_picker_result(index);
            } else {
                let library = &mut self.session.workspace_mut().library;
                library.focus = Some(control);
                library.reveal_focus = true;
            }
            self.invalidate_frame();
            return true;
        }
        let from_search = focus == Some(C::AcquireSearch);
        let in_results = matches!(focus, Some(C::AcquireResult(_)));
        let direction = match key {
            Key::Named(NamedKey::ArrowDown) if from_search || in_results => 1isize,
            Key::Named(NamedKey::ArrowUp) if in_results => -1,
            Key::Named(NamedKey::PageDown) if from_search || in_results => {
                (viewport.map_or(row_height, |v| v.0) / row_height)
                    .floor()
                    .max(1.) as isize
            }
            Key::Named(NamedKey::PageUp) if in_results => {
                -((viewport.map_or(row_height, |v| v.0) / row_height)
                    .floor()
                    .max(1.) as isize)
            }
            _ => return false,
        };
        if !results.is_empty() {
            let position = if from_search {
                selected.unwrap_or(0)
            } else {
                (selected.unwrap_or(0) as isize + direction).clamp(0, results.len() as isize - 1)
                    as usize
            };
            self.select_component_picker_result(results[position]);
        }
        self.invalidate_frame();
        true
    }

    fn select_component_picker_result(&mut self, index: usize) {
        let library = &self.workspace().library;
        let a = library.acquisition.as_ref().unwrap();
        let results = a.results();
        let Some(position) = results.iter().position(|i| *i == index) else {
            return;
        };
        let source = &a.sources[index];
        let already_selected = a.selected.as_deref() == Some(source.name.as_str())
            && a.selected_source.as_ref() == Some(&source.path);
        let row_height =
            datum_gui_protocol::library_browser::acquisition::COMPONENT_PICKER_ROW_HEIGHT;
        let viewport = self.presented_hits.regions().iter().find_map(|hit| {
            if let HitTarget::Library(C::Scroll { height, offset, .. }) = &hit.target {
                Some((*height as f32, *offset as f32))
            } else {
                None
            }
        });
        if let Some((height, offset)) = viewport {
            let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
            scroll.layout(
                datum_gui_viewport::ScreenRectPx {
                    x: 0.,
                    y: 0.,
                    width: 1.,
                    height,
                },
                results.len() as f32 * row_height,
            );
            scroll.set_offset(offset);
            scroll.reveal(
                position as f32 * row_height,
                (position + 1) as f32 * row_height,
            );
            self.session.workspace_mut().library.scroll_offset = scroll.offset() as usize;
        }
        if already_selected {
            let library = &mut self.session.workspace_mut().library;
            library.focus = Some(C::AcquireResult(index));
            library.reveal_focus = true;
        } else {
            self.activate_library(C::AcquireResult(index));
        }
    }

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
