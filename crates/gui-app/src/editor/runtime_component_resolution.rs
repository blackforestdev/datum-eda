//! Retained assignment context delegates to existing Library and Proposal owners.
use super::*;
use datum_gui_protocol::{
    library_authoring::LibraryDraft,
    library_browser::{LibraryControl as C, component_resolution::ResolutionAction as A},
    schematic_authoring::assignment::AssignmentEvidence,
};
#[path = "runtime_component_configuration_pointer.rs"]
mod configuration_pointer;
impl Runtime {
    pub(crate) fn trace_component_editor(&self, label: &str) {
        if let Some(c) = &self.workspace().library.configuration {
            self.trace_click(format!("component editor {label}; focus {:?}; field {:?}; phase {:?}; native_pending {}; value_bytes {}; search_bytes {}; legacy_picker {}; legacy_label {}",
                self.application_focus(), self.workspace().library.focus, c.phase, self.native_edit.pending(), c.value.len(), self.workspace().library.acquisition.as_ref().map_or(0,|a|a.search.len()), self.workspace().schematic_authoring.picker, self.workspace().schematic_authoring.label.is_some()));
        }
    }

    pub(crate) fn handle_component_configuration_key(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> bool {
        use datum_gui_protocol::library_browser::component_resolution::configuration::ConfigurationControl as F;
        let Some(configuration) = self
            .workspace()
            .library
            .configuration
            .as_ref()
            .filter(|c| c.target.is_some() || c.footprint_chooser.is_some())
        else {
            return false;
        };
        self.trace_component_editor(&format!("key {:?}", event.logical_key));
        if configuration.footprint_chooser.is_some() {
            return self.handle_library_key(event);
        }
        if event.state != ElementState::Pressed
            || configuration.pending()
            || self.native_edit.pending()
        {
            return true;
        }
        if self.handle_shared_editor_key(event) {
            return true;
        }
        if event.repeat
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return true;
        }
        let configuration = self.workspace().library.configuration.as_ref().unwrap();
        let controls = configuration.controls();
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.configuration_library_control(&C::Configure(F::Cancel));
            }
            Key::Named(NamedKey::Enter) if !self.modifiers.shift_key() => {
                if controls.contains(&configuration.focus) {
                    self.configuration_library_control(&C::Configure(configuration.focus));
                }
            }
            Key::Named(NamedKey::Tab) if !controls.is_empty() => {
                let index = controls.iter().position(|c| *c == configuration.focus);
                let next = if self.modifiers.shift_key() {
                    index.map_or(controls.len() - 1, |i| {
                        (i + controls.len() - 1) % controls.len()
                    })
                } else {
                    index.map_or(0, |i| (i + 1) % controls.len())
                };
                let control = controls[next];
                let library = &mut self.session.workspace_mut().library;
                library.configuration.as_mut().unwrap().focus = control;
                library.focus = Some(match control {
                    F::Reference => C::Field(0),
                    F::Value => C::Field(1),
                    other => C::Configure(other),
                });
                library.reveal_focus = true;
                self.invalidate_frame();
            }
            _ => {}
        }
        true
    }
    pub(crate) fn library_configuration_input_available(&self) -> bool {
        self.workspace().ui.layout.focused_content() == PaneContent::Library
            || self.component_picker_open()
            || self.workspace().library.configuration.is_some()
    }

    pub(crate) fn component_configuration_field_focused(&self) -> bool {
        self.workspace()
            .library
            .configuration
            .as_ref()
            .is_some_and(|c| {
                c.footprint_chooser.is_none()
                    && matches!(self.workspace().library.focus, Some(C::Field(_)))
            })
    }

    fn restore_component_configuration_focus(&mut self) {
        use datum_gui_protocol::library_browser::component_resolution::configuration::ConfigurationControl as F;
        let library = &mut self.session.workspace_mut().library;
        let Some(configuration) = library.configuration.as_ref() else {
            return;
        };
        library.focus = Some(match configuration.focus {
            F::Reference => C::Field(0),
            F::Value => C::Field(1),
            other => C::Configure(other),
        });
        library.reveal_focus = true;
    }

    fn confirm_footprint_result(&mut self, index: usize) -> bool {
        let library = &mut self.session.workspace_mut().library;
        let (Some(configuration), Some(acquisition)) =
            (&mut library.configuration, &library.acquisition)
        else {
            return false;
        };
        let Some(source) = acquisition.sources.get(index) else {
            return false;
        };
        let now = std::time::Instant::now();
        let repeated =
            configuration
                .footprint_last_result
                .as_ref()
                .is_some_and(|(path, name, at)| {
                    path == &source.path
                        && name == &source.name
                        && now.duration_since(*at) < std::time::Duration::from_millis(500)
                });
        configuration.footprint_last_result = Some((source.path.clone(), source.name.clone(), now));
        if !repeated
            || acquisition.selected.as_ref() != Some(&source.name)
            || acquisition.selected_source.as_ref() != Some(&source.path)
        {
            return false;
        }
        if acquisition.loading {
            // Accept only this read's eventual preview, never a later selection.
            configuration.footprint_accept_session = Some(acquisition.session);
            return false;
        }
        acquisition.request().is_ok()
            && acquisition
                .preview
                .as_ref()
                .is_some_and(|p| p.footprint.is_some())
    }

    pub(crate) fn configuration_library_control(&mut self, control: &C) -> bool {
        use datum_gui_protocol::library_browser::component_resolution::configuration::ConfigurationControl as F;
        let Some(configuration) = self.workspace().library.configuration.as_ref() else {
            return self.resolution_library_return(control);
        };
        let chooser = configuration.footprint_chooser.is_some();
        let pending = configuration.pending();
        let properties = configuration.target.is_some();
        let action = match control {
            C::AcquireResult(index) if chooser => {
                if self.confirm_footprint_result(*index) {
                    F::AcceptFootprint
                } else {
                    return false;
                }
            }
            C::Configure(action) => *action,
            C::Field(0) if !chooser => F::Reference,
            C::Field(1) if !chooser => F::Value,
            C::AcquireImport if chooser => F::AcceptFootprint,
            C::Cancel if chooser || properties => F::Cancel,
            C::Cancel => {
                self.session.workspace_mut().library.configuration = None;
                return false;
            }
            _ => return false,
        };
        if pending || self.native_edit.pending() || self.check_work.pending() {
            return true;
        }
        let result = (|| -> Result<(), String> {
            if action == F::Cancel {
                if chooser {
                    self.library_acquisition_work.cancel();
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    configuration.footprint_chooser = None;
                    let picker = configuration.picker.clone();
                    let offset = configuration.return_scroll_offset;
                    self.session.workspace_mut().library.acquisition = picker;
                    self.session.workspace_mut().library.scroll_offset = offset;
                    self.restore_component_configuration_focus();
                } else {
                    let return_to_transfer = self
                        .workspace()
                        .library
                        .configuration
                        .as_ref()
                        .unwrap()
                        .return_to_transfer;
                    self.session.workspace_mut().library.configuration = None;
                    if return_to_transfer {
                        self.begin_board_transfer();
                    }
                }
                return Ok(());
            }
            if !chooser {
                self.verify_component_configuration_context()?;
            }
            match action {
                F::Reference | F::Value => {
                    self.session.workspace_mut().library.focus =
                        Some(C::Field(usize::from(action == F::Value)));
                    self.session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap()
                        .focus = action;
                }
                F::ChooseFootprint => {
                    let offset = self.workspace().library.scroll_offset;
                    let revision = self.workspace().scene.source_revision.clone();
                    let tip = self
                        .workspace()
                        .supervision
                        .journal
                        .accepted_transaction_tip
                        .as_ref()
                        .and_then(|s| s.parse().ok());
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    configuration.begin_footprint_choice(revision, tip)?;
                    configuration.focus = F::ChooseFootprint;
                    configuration.return_scroll_offset = offset;
                    let acquisition = configuration.footprint_chooser.clone();
                    self.session.workspace_mut().library.acquisition = acquisition;
                    self.session.workspace_mut().library.scroll_offset = 0;
                    self.session.workspace_mut().library.focus = Some(C::AcquireSearch);
                    self.activate_library(C::AcquireLoad);
                    self.session.workspace_mut().library.focus = Some(C::AcquireSearch);
                }
                F::AcceptFootprint => {
                    let chosen = self
                        .workspace()
                        .library
                        .acquisition
                        .clone()
                        .ok_or("Choose a footprint")?;
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    configuration.footprint_chooser = Some(chosen);
                    configuration.accept_footprint()?;
                    let picker = configuration.picker.clone();
                    let offset = configuration.return_scroll_offset;
                    self.session.workspace_mut().library.acquisition = picker;
                    self.session.workspace_mut().library.scroll_offset = offset;
                    self.library_acquisition_work.cancel();
                    let mut configuration = self
                        .workspace()
                        .library
                        .configuration
                        .as_ref()
                        .unwrap()
                        .clone();
                    configuration.reuse_mapping(&self.workspace().library)?;
                    configuration.focus = F::ChooseFootprint;
                    self.session.workspace_mut().library.configuration = Some(configuration);
                    self.restore_component_configuration_focus();
                }
                F::MappingPad(index) => {
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    let mut pads: Vec<_> =
                        configuration.footprint()?.pads.keys().copied().collect();
                    pads.sort();
                    configuration.selected_pad =
                        Some(*pads.get(index).ok_or("Select an actual pad")?);
                    configuration.focus = action;
                }
                F::MappingPin(index) => {
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    let mut pins: Vec<_> = configuration.unit()?.pins.keys().copied().collect();
                    pins.sort();
                    configuration.map_pin(*pins.get(index).ok_or("Select an actual pin")?)?;
                    configuration.focus = action;
                }
                F::ConfirmMapping => {
                    let configuration = self
                        .session
                        .workspace_mut()
                        .library
                        .configuration
                        .as_mut()
                        .unwrap();
                    configuration.mapping_confirmed = true;
                    if let Err(reason) = configuration.request() {
                        configuration.mapping_confirmed = false;
                        return Err(reason);
                    }
                    configuration.refusal = None;
                    configuration.focus = action;
                }
                F::Apply => self.apply_component_configuration()?,
                F::Cancel => unreachable!(),
            }
            Ok(())
        })();
        if let Err(reason) = result
            && let Some(configuration) = &mut self.session.workspace_mut().library.configuration
        {
            configuration.refusal = Some(reason);
        }
        self.invalidate_frame();
        true
    }

    pub(crate) fn begin_component_properties(&mut self) -> bool {
        let result = self.selected_schematic_idle().and_then(|()| {
            datum_gui_protocol::library_browser::component_resolution::configuration::ComponentConfiguration::capture_properties(self.workspace())
        });
        match result {
            Ok(configuration) => {
                self.session.workspace_mut().library.configuration = Some(configuration);
                self.session.workspace_mut().library.scroll_offset = 0;
                self.session.workspace_mut().library.focus = Some(C::Field(0));
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    pub(crate) fn component_resolution_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        let Some(context) = self
            .workspace()
            .library
            .resolution
            .as_ref()
            .filter(|r| !r.in_library)
        else {
            return false;
        };
        if self.native_edit.pending()
            || self.check_work.pending()
            || event.repeat
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return true;
        }
        let actions = context.actions(&self.workspace().library);
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                return self.handle_component_resolution(A::Cancel);
            }
            Key::Named(NamedKey::Enter) if !self.modifiers.shift_key() => {
                if let Some(action) = context.focus.filter(|a| actions.contains(a)) {
                    return self.handle_component_resolution(action);
                }
            }
            Key::Named(NamedKey::Tab) => {
                let n = actions.len();
                let index = context
                    .focus
                    .and_then(|f| actions.iter().position(|a| *a == f));
                let next = if self.modifiers.shift_key() {
                    index.map_or(n - 1, |i| (i + n - 1) % n)
                } else {
                    index.map_or(0, |i| (i + 1) % n)
                };
                self.session
                    .workspace_mut()
                    .library
                    .resolution
                    .as_mut()
                    .unwrap()
                    .focus = Some(actions[next]);
                self.invalidate_frame();
            }
            _ => {}
        }
        true
    }
    pub(crate) fn begin_component_resolution(
        &mut self,
        target: datum_gui_protocol::AuthoredSelectionIdentity,
        transfer: bool,
    ) -> Result<(), String> {
        self.selected_schematic_idle()?;
        AssignmentEvidence::admit_target(self.workspace(), &target)?;
        if !self
            .ensure_selection_authority()
            .map_err(|e| e.to_string())?
        {
            return Err("Current component selection authority unavailable".into());
        }
        self.publish_native_acquisition(
            [target].into_iter().collect(),
            datum_gui_viewport::selection_acquisition::AcquisitionMode::Replace,
            datum_gui_viewport::selection_acquisition::AcquisitionEntry::Individual,
            None,
        );
        let mut configuration = datum_gui_protocol::library_browser::component_resolution::configuration::ComponentConfiguration::capture_properties(self.workspace())?;
        configuration.return_to_transfer = transfer;
        self.session.workspace_mut().library.configuration = Some(configuration);
        self.session.workspace_mut().library.focus = Some(C::Field(0));
        self.session.workspace_mut().ui.board_transfer = None;
        Ok(())
    }
    pub(crate) fn resolution_library_return(&mut self, control: &C) -> bool {
        let b = &self.workspace().library;
        if *control != C::Cancel
            || b.resolution.is_none()
            || b.draft.is_some()
            || b.acquisition.is_some()
        {
            return false;
        }
        self.handle_component_resolution(A::Return)
    }
    pub(crate) fn handle_component_resolution(&mut self, action: A) -> bool {
        let Some(mut context) = self.workspace().library.resolution.clone() else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() {
            return true;
        }
        if !matches!(action, A::Return | A::Cancel)
            && !context.actions(&self.workspace().library).contains(&action)
        {
            return true;
        }
        if action != A::Return {
            context.focus = Some(action);
        }
        let result = (|| -> Result<(), String> {
            if self.workspace().library.draft.is_some()
                || self.workspace().library.acquisition.is_some()
            {
                return Err("Finish or cancel the Library draft or source preview first".into());
            }
            if let Err(reason) = context.verify(self.workspace()) {
                if action == A::Cancel {
                    self.session.workspace_mut().library.resolution = None;
                    return Ok(());
                }
                return Err(reason);
            }
            self.session.workspace_mut().ui.layout.focused = context.basis.pane;
            self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                context.basis.pane,
            ));
            if action != A::Cancel {
                AssignmentEvidence::admit_target(self.workspace(), &context.evidence.target)?;
            }
            match action {
                A::Cancel => {
                    self.session.workspace_mut().library.resolution = None;
                    if context.transfer {
                        self.begin_board_transfer();
                    }
                }
                A::Return => {
                    context.in_library = false;
                    context.evidence = AssignmentEvidence::capture_resolution(
                        self.workspace(),
                        &context.evidence.target,
                    )?;
                    context.refusal = None;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                }
                A::Assign => {
                    let review = datum_gui_protocol::schematic_authoring::delete_review::DeleteReview::capture_assignment_target(self.workspace(), &context.evidence.target)?;
                    if !self.prepare_native_circuit(false) {
                        return Ok(());
                    }
                    self.session.workspace_mut().library.resolution = None;
                    let editor = &mut self.session.workspace_mut().schematic_authoring;
                    editor.chosen = None;
                    editor.label = Some(datum_gui_protocol::schematic_authoring::LabelDraft::new(
                        eda_engine::schematic::LabelKind::Local,
                    ));
                    editor.assignment_draft = Some(review);
                    editor.refusal = None;
                }
                A::Footprint => {
                    context.in_library = true;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                    self.open_library();
                    let pane = self.workspace().ui.layout.focused;
                    self.session
                        .workspace_mut()
                        .library
                        .resolution
                        .as_mut()
                        .unwrap()
                        .library_pane = Some(pane);
                    self.session.workspace_mut().library.kind = 5;
                    self.activate_library(C::Acquire);
                }
                A::Entity | A::Part | A::Map => {
                    let kind = match action {
                        A::Entity => "entities",
                        A::Part => "parts",
                        _ => "pin_pad_maps",
                    };
                    let model = self
                        .workspace()
                        .library
                        .basis
                        .as_ref()
                        .ok_or("Library source unavailable")?;
                    let mut draft = LibraryDraft::begin(
                        &self.workspace().library,
                        None,
                        kind,
                        model.model_revision.0.clone(),
                        eda_engine::revision::transaction_tip(&model.journal),
                    )?;
                    if action == A::Entity {
                        let symbol: eda_engine::substrate::ObjectId = context
                            .evidence
                            .options
                            .original
                            .lib_id
                            .as_deref()
                            .ok_or("Saved Symbol missing")?
                            .parse()
                            .map_err(|_| "Saved Symbol invalid")?;
                        let raw = self
                            .workspace()
                            .library
                            .entries
                            .iter()
                            .find(|e| e.id == symbol && e.kind == "symbols" && e.warning.is_none())
                            .ok_or("Saved Symbol unavailable")?;
                        let unit = raw
                            .raw
                            .get("unit")
                            .and_then(serde_json::Value::as_str)
                            .and_then(|s| s.parse().ok())
                            .ok_or("Saved Symbol Unit unavailable")?;
                        draft.retain_entity_symbol(symbol, unit)?;
                    }
                    context.authored.push((kind.into(), draft.id));
                    context.in_library = true;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                    self.open_library();
                    let pane = self.workspace().ui.layout.focused;
                    self.session
                        .workspace_mut()
                        .library
                        .resolution
                        .as_mut()
                        .unwrap()
                        .library_pane = Some(pane);
                    let b = &mut self.session.workspace_mut().library;
                    b.draft = Some(draft);
                    b.focus = Some(C::Field(0));
                    b.refusal = None;
                }
            }
            Ok(())
        })();
        if let Err(reason) = result {
            if let Some(c) = &mut self.session.workspace_mut().library.resolution {
                c.refusal = Some(reason.clone());
            }
            self.session.workspace_mut().library.refusal = Some(reason);
        }
        self.invalidate_frame();
        true
    }
}
