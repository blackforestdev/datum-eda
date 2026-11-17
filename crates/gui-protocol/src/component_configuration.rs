//! One disposable engineering choice for placement and Component Properties.
//! Selecting or cancelling a preview never publishes library or instance edits.
use crate::library_browser::acquisition::{LibraryAcquisition, LibraryAcquisitionPreview};
use crate::single_line_editor::SingleLineEditor;
use eda_engine::api::native_write::{
    library_acquisition::LibraryAcquisitionRequest,
    library_gui::component_configuration::ComponentConfigurationRequest,
    schematic_library::part_placement::CompletePartSelection,
};
use eda_engine::pool::{Footprint, Symbol, Unit};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigurationControl {
    Reference,
    Value,
    ChooseFootprint,
    AcceptFootprint,
    MappingPad(usize),
    MappingPin(usize),
    ConfirmMapping,
    Apply,
    Cancel,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigurationPhase {
    Editing,
    SavingDefinitions,
    PreparingInstance,
    ApplyingInstance(Uuid),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigurationTarget {
    pub root: std::path::PathBuf,
    pub basis: crate::AuthoringViewBasis,
    pub symbol: crate::AuthoredSelectionIdentity,
    pub sheet_preimage: serde_json::Value,
    pub original_reference: String,
    pub original_value: String,
}

impl ConfigurationTarget {
    pub fn capture(state: &crate::ReviewWorkspaceState) -> Result<Self, String> {
        let (basis, symbol) = crate::schematic_authoring::selected::symbol_fields_target(state)?;
        let sheet = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .ok_or("Selected schematic source unavailable")?;
        let placed = sheet
            .symbols
            .get(&symbol.id)
            .ok_or("Selected Symbol unavailable")?;
        Ok(Self {
            root: state
                .backing
                .as_ref()
                .ok_or("Open a persistent native Project")?
                .request
                .project_root
                .clone(),
            original_reference: placed.reference.clone(),
            original_value: placed.value.clone(),
            sheet_preimage: serde_json::to_value(sheet).map_err(|e| e.to_string())?,
            basis,
            symbol,
        })
    }

    pub fn verify(&self, state: &crate::ReviewWorkspaceState) -> Result<(), String> {
        if state
            .backing
            .as_ref()
            .is_none_or(|b| b.request.project_root != self.root)
            || state.ui.layout.focused != self.basis.pane
            || state.ui.layout.content_for(self.basis.pane) != Some(crate::PaneContent::Schematic)
            || state.schematic_documents.panes.get(&self.basis.pane) != self.basis.document.as_ref()
            || state
                .library
                .basis
                .as_ref()
                .is_none_or(|m| m.project.project_id != self.basis.project)
        {
            return Err(
                "Component Properties context changed; close and select the current component"
                    .into(),
            );
        }
        if !matches!(&state.selection,crate::SelectionTarget::Native(view) if !view.stale && view.project==self.basis.project && matches!(&view.subject,crate::SelectionSubject::Object(id) if id==&self.symbol) && !view.locked.contains(&self.symbol))
        {
            return Err(
                "The selected component changed; close and reopen Component Properties".into(),
            );
        }
        let sheet = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&self.basis.source))
            .ok_or("Selected source sheet unavailable")?;
        if serde_json::to_value(sheet).map_err(|e| e.to_string())? != self.sheet_preimage {
            return Err("The schematic changed; reopen Component Properties".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentConfiguration {
    pub return_to_transfer: bool,
    pub target: Option<ConfigurationTarget>,
    pub placement_root: Option<std::path::PathBuf>,
    pub picker: Option<LibraryAcquisition>,
    pub pending_proposal:
        Option<eda_engine::api::native_write::schematic_delete_proposal::NativeProposalAction>,
    pub session: Uuid,
    pub part_id: Uuid,
    pub reference: String,
    pub reference_editor: SingleLineEditor,
    pub value: String,
    pub value_editor: SingleLineEditor,
    pub symbol_preview: LibraryAcquisitionPreview,
    pub symbol_format: Option<String>,
    pub footprint_preview: Option<LibraryAcquisitionPreview>,
    pub footprint_chooser: Option<LibraryAcquisition>,
    pub return_scroll_offset: usize,
    pub footprint_last_result: Option<(std::path::PathBuf, String, std::time::Instant)>,
    pub footprint_accept_session: Option<Uuid>,
    pub mappings: BTreeMap<Uuid, Uuid>,
    pub mapping_confirmed: bool,
    pub mapping_source: Option<CompletePartSelection>,
    pub selected_pad: Option<Uuid>,
    pub current_component: Option<CompletePartSelection>,
    pub footprint_change_refusal: Option<String>,
    pub phase: ConfigurationPhase,
    pub focus: ConfigurationControl,
    pub refusal: Option<String>,
}

impl ComponentConfiguration {
    pub fn capture_properties(state: &crate::ReviewWorkspaceState) -> Result<Self, String> {
        let target = ConfigurationTarget::capture(state)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current library source unavailable")?;
        let sheet = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&target.basis.source))
            .ok_or("Schematic source unavailable")?;
        let placed = sheet
            .symbols
            .get(&target.symbol.id)
            .ok_or("Component source unavailable")?;
        let symbol_id: Uuid = placed
            .lib_id
            .as_deref()
            .ok_or("Component Symbol unavailable")?
            .parse()
            .map_err(|_| "Component Symbol identity invalid")?;
        let symbol =
            captured_object(&state.library, symbol_id, "symbols").map_err(|e| e.to_string())?;
        let typed: Symbol = serde_json::from_value(symbol.clone()).map_err(|e| e.to_string())?;
        let unit =
            captured_object(&state.library, typed.unit, "units").map_err(|e| e.to_string())?;
        let mut configuration = Self::begin(
            LibraryAcquisitionPreview {
                name: typed.name,
                source_digest: String::new(),
                source_path: Default::default(),
                symbol,
                unit,
                footprint: None,
                native_part: None,
                notices: Vec::new(),
            },
            None,
            placed.reference.clone(),
            placed.value.clone(),
        )?;
        if let (Some(part), Some(gate)) = (placed.part, placed.gate) {
            match CompletePartSelection::capture(model, part, gate, symbol_id) {
                Ok(component) => configuration.use_component(&state.library, component)?,
                Err(reason) => {
                    configuration.refusal =
                        Some(format!("Current physical binding is unavailable: {reason}"))
                }
            }
        }
        if model.component_instances.values().any(|c| {
            c.placed_symbol_refs.contains(&placed.uuid) && !c.placed_package_refs.is_empty()
        }) {
            configuration.footprint_change_refusal = Some("Footprint Change is available before initial PCB transfer. This component already has a PCB counterpart; Reference and Value remain editable.".into());
        }
        configuration.target = Some(target);
        configuration.focus = ConfigurationControl::Reference;
        Ok(configuration)
    }

    pub fn begin(
        preview: LibraryAcquisitionPreview,
        source_format: Option<String>,
        reference: String,
        value: String,
    ) -> Result<Self, String> {
        let symbol: Symbol =
            serde_json::from_value(preview.symbol.clone()).map_err(|e| e.to_string())?;
        let unit: Unit = serde_json::from_value(preview.unit.clone()).map_err(|e| e.to_string())?;
        if symbol.unit != unit.uuid {
            return Err("Selected Symbol and Unit do not agree".into());
        }
        Ok(Self {
            return_to_transfer: false,
            target: None,
            placement_root: None,
            picker: None,
            pending_proposal: None,
            session: Uuid::new_v4(),
            part_id: Uuid::new_v4(),
            reference_editor: SingleLineEditor::at_end(&reference),
            reference,
            value_editor: SingleLineEditor::at_end(&value),
            value,
            symbol_preview: preview,
            symbol_format: source_format,
            footprint_preview: None,
            footprint_chooser: None,
            return_scroll_offset: 0,
            footprint_last_result: None,
            footprint_accept_session: None,
            mappings: BTreeMap::new(),
            mapping_confirmed: false,
            mapping_source: None,
            selected_pad: None,
            current_component: None,
            footprint_change_refusal: None,
            phase: ConfigurationPhase::Editing,
            focus: ConfigurationControl::Value,
            refusal: None,
        })
    }

    pub fn pending(&self) -> bool {
        self.phase != ConfigurationPhase::Editing
    }

    pub fn verify_sources(
        &self,
        library: &crate::library_browser::LibraryBrowser,
    ) -> Result<(), String> {
        let symbol = self.symbol()?;
        let present = library
            .basis
            .as_ref()
            .is_some_and(|m| m.objects.contains_key(&symbol.uuid));
        if present || self.symbol_format.is_none() {
            if captured_object(library, symbol.uuid, "symbols")? != self.symbol_preview.symbol
                || captured_object(library, symbol.unit, "units")? != self.symbol_preview.unit
            {
                return Err("The selected Symbol or Unit changed; choose it again".into());
            }
        }
        if let Some(preview) = &self.footprint_preview
            && preview.source_path.as_os_str().is_empty()
        {
            let footprint = self.footprint()?;
            if captured_object(library, footprint.uuid, "footprints")?
                != preview.footprint.as_ref().unwrap().footprint
            {
                return Err("The selected Project footprint changed; choose it again".into());
            }
        }
        if let Some(source) = &self.mapping_source {
            source
                .verify(
                    library
                        .basis
                        .as_ref()
                        .ok_or("Current library unavailable")?,
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// An unchanged complete Part keeps its full gate-qualified authority. It
    /// must not be reduced to the single Unit used by a bare-Symbol draft.
    pub fn placement_component(&self) -> Option<&CompletePartSelection> {
        self.current_component.as_ref().filter(|component| {
            self.target.is_none()
                && self.mapping_confirmed
                && self.mapping_source.as_ref() == Some(*component)
                && self.value.trim() == component.value
                && self
                    .footprint()
                    .is_ok_and(|f| f.uuid == component.footprint)
        })
    }

    pub fn can_apply(&self) -> bool {
        if self.pending() || self.footprint_chooser.is_some() {
            return false;
        }
        if let Some(target) = &self.target {
            let same_physical = self.current_component.as_ref().is_some_and(|c| {
                self.mapping_source.as_ref() == Some(c)
                    && self.footprint().is_ok_and(|f| f.uuid == c.footprint)
            });
            if same_physical || self.footprint_change_refusal.is_some() {
                return !self.reference.trim().is_empty()
                    && !self.reference.chars().any(char::is_control)
                    && !self.value.chars().any(char::is_control)
                    && (self.reference != target.original_reference
                        || self.value != target.original_value);
            }
        }
        self.placement_component().is_some() || self.request().is_ok()
    }

    pub fn controls(&self) -> Vec<ConfigurationControl> {
        use ConfigurationControl as C;
        if self.pending() {
            return Vec::new();
        }
        let mut controls = Vec::new();
        if self.target.is_some() {
            controls.push(C::Reference);
        }
        controls.push(C::Value);
        if self.footprint_change_refusal.is_none() {
            controls.push(C::ChooseFootprint);
        }
        if let Ok(footprint) = self.footprint()
            && !self.mapping_confirmed
            && self.footprint_change_refusal.is_none()
        {
            controls.extend((0..footprint.pads.len()).map(C::MappingPad));
            if self.selected_pad.is_some()
                && let Ok(unit) = self.unit()
            {
                controls.extend((0..unit.pins.len()).map(C::MappingPin));
            }
            controls.push(C::ConfirmMapping);
        }
        if self.can_apply() {
            controls.push(C::Apply);
        }
        controls.push(C::Cancel);
        controls
    }

    pub fn begin_footprint_choice(
        &mut self,
        revision: String,
        tip: Option<Uuid>,
    ) -> Result<(), String> {
        if self.pending() {
            return Err("Wait for the current component action".into());
        }
        if let Some(reason) = &self.footprint_change_refusal {
            return Err(reason.clone());
        }
        let mut chooser = LibraryAcquisition::begin(revision, tip);
        chooser.use_footprint_source();
        self.footprint_chooser = Some(chooser);
        self.footprint_last_result = None;
        self.footprint_accept_session = None;
        self.refusal = None;
        Ok(())
    }

    /// Accept into the parent form only. No acquisition request is submitted here.
    pub fn accept_footprint(&mut self) -> Result<(), String> {
        if self.pending() {
            return Err("Wait for the current component action".into());
        }
        let chooser = self
            .footprint_chooser
            .as_ref()
            .ok_or("Open the footprint chooser")?;
        chooser.request()?;
        let preview = chooser
            .preview
            .as_ref()
            .ok_or("Preview the chosen footprint")?;
        if preview.footprint.is_none() {
            return Err("Choose an actual physical footprint".into());
        }
        if self.footprint_preview.as_ref() != Some(preview) {
            self.mappings.clear();
            self.mapping_confirmed = false;
            self.mapping_source = None;
            self.selected_pad = None;
        }
        self.footprint_preview = Some(preview.clone());
        self.footprint_chooser = None;
        self.focus = ConfigurationControl::ConfirmMapping;
        Ok(())
    }

    pub fn map_pin(&mut self, pin: Uuid) -> Result<(), String> {
        if self.pending() {
            return Err("Wait for the current component action".into());
        }
        let pad = self
            .selected_pad
            .ok_or("Select the physical pad to connect")?;
        let unit = self.unit()?;
        let footprint = self.footprint()?;
        if !unit.pins.contains_key(&pin) || !footprint.pads.contains_key(&pad) {
            return Err("Choose an actual pin and pad from these definitions".into());
        }
        self.mappings.insert(pad, pin);
        self.mapping_confirmed = false;
        self.mapping_source = None;
        Ok(())
    }

    /// Reuse only an unambiguous existing correspondence for these exact source
    /// identities. A different value or supplier Part is not selected here.
    pub fn reuse_mapping(
        &mut self,
        library: &crate::library_browser::LibraryBrowser,
    ) -> Result<(), String> {
        let model = library
            .basis
            .as_ref()
            .ok_or("Current library unavailable")?;
        let choices=eda_engine::api::native_write::library_gui::component_configuration::qualified_mapping_choices(model,self.symbol()?.uuid,self.footprint()?.uuid);
        let mut selected = None;
        for choice in choices {
            let raw = captured_object(library, choice.pin_pad_map, "pin_pad_maps")?;
            let map: eda_engine::pool::PinPadMap =
                serde_json::from_value(raw).map_err(|e| e.to_string())?;
            let mapping: BTreeMap<_, _> = map
                .mappings
                .into_iter()
                .filter(|(_, p)| p.gate == choice.part.gate)
                .map(|(pad, p)| (pad, p.pin))
                .collect();
            if let Some((previous, _)) = &selected
                && previous != &mapping
            {
                return Ok(());
            }
            selected = Some((mapping, choice));
        }
        if let Some((mapping, choice)) = selected {
            self.mappings = mapping;
            self.mapping_source = Some(choice);
            self.mapping_confirmed = true;
        }
        Ok(())
    }

    pub fn unit(&self) -> Result<Unit, String> {
        serde_json::from_value(self.symbol_preview.unit.clone()).map_err(|e| e.to_string())
    }

    /// Source pin numbers disambiguate hidden/repeated names such as "~".
    pub fn pin_label(&self, pin: Uuid) -> String {
        let numbers: std::collections::BTreeSet<_> = self
            .symbol()
            .ok()
            .into_iter()
            .flat_map(|s| s.pin_anchors)
            .filter(|a| a.pin == pin)
            .filter_map(|a| a.number)
            .filter(|n| !n.is_empty())
            .collect();
        if !numbers.is_empty() {
            return numbers.into_iter().collect::<Vec<_>>().join(" / ");
        }
        self.unit()
            .ok()
            .and_then(|u| u.pins.get(&pin).map(|p| p.name.clone()))
            .filter(|n| !n.is_empty() && n != "~")
            .unwrap_or_else(|| pin.to_string())
    }

    /// Capture saved definitions by their qualified identities, including all
    /// actual padstacks used by the footprint preview. This does not import them.
    pub fn use_component(
        &mut self,
        library: &crate::library_browser::LibraryBrowser,
        component: CompletePartSelection,
    ) -> Result<(), String> {
        let model = library
            .basis
            .as_ref()
            .ok_or("Current library source unavailable")?;
        component.verify(model).map_err(|e| e.to_string())?;
        if component.part.symbol != self.symbol()?.uuid {
            return Err("Component uses a different Symbol".into());
        }
        let footprint = captured_object(library, component.footprint, "footprints")
            .map_err(|e| e.to_string())?;
        let typed: Footprint =
            serde_json::from_value(footprint.clone()).map_err(|e| e.to_string())?;
        let package =
            captured_object(library, component.package, "packages").map_err(|e| e.to_string())?;
        let padstacks = typed
            .pads
            .values()
            .map(|p| captured_object(library, p.padstack, "padstacks").map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let map = captured_object(library, component.pin_pad_map, "pin_pad_maps")
            .map_err(|e| e.to_string())?;
        let map: eda_engine::pool::PinPadMap =
            serde_json::from_value(map).map_err(|e| e.to_string())?;
        self.mappings = map
            .mappings
            .into_iter()
            .filter(|(_, p)| p.gate == component.part.gate)
            .map(|(pad, p)| (pad, p.pin))
            .collect();
        self.footprint_preview = Some(LibraryAcquisitionPreview {
            name: typed.name,
            source_digest: String::new(),
            source_path: Default::default(),
            unit: serde_json::Value::Null,
            symbol: serde_json::Value::Null,
            footprint: Some(
                crate::library_browser::acquisition::FootprintAcquisitionPreview {
                    package,
                    footprint,
                    padstacks,
                },
            ),
            native_part: None,
            notices: Vec::new(),
        });
        self.mapping_source = Some(component.clone());
        self.mapping_confirmed = true;
        self.current_component = Some(component);
        Ok(())
    }
    pub fn symbol(&self) -> Result<Symbol, String> {
        serde_json::from_value(self.symbol_preview.symbol.clone()).map_err(|e| e.to_string())
    }
    pub fn footprint(&self) -> Result<Footprint, String> {
        let preview = self
            .footprint_preview
            .as_ref()
            .and_then(|p| p.footprint.as_ref())
            .ok_or("Choose a footprint")?;
        serde_json::from_value(preview.footprint.clone()).map_err(|e| e.to_string())
    }

    fn source(preview: &LibraryAcquisitionPreview, format: &str) -> LibraryAcquisitionRequest {
        LibraryAcquisitionRequest {
            source_format: format.into(),
            source_path: preview.source_path.clone(),
            symbol_name: preview.selection_key(),
            expected_source_hash: preview.source_digest.clone(),
        }
    }

    /// The engine independently re-reads sources and validates this correspondence.
    pub fn request(&self) -> Result<ComponentConfigurationRequest, String> {
        let symbol = self.symbol()?;
        let footprint = self.footprint()?;
        let request = ComponentConfigurationRequest {
            symbol: symbol.uuid,
            footprint: footprint.uuid,
            value: self.value.trim().into(),
            mappings: self.mappings.clone(),
            mapping_confirmed: self.mapping_confirmed,
            symbol_source: self
                .symbol_format
                .as_ref()
                .map(|format| Self::source(&self.symbol_preview, format)),
            footprint_source: self
                .footprint_preview
                .as_ref()
                .filter(|p| !p.source_path.as_os_str().is_empty())
                .map(|p| Self::source(p, "kicad_footprint")),
            mapping_source: self.mapping_source.clone(),
        };
        request.validate_value().map_err(|e| e.to_string())?;
        request
            .validate_mapping(&self.unit()?, &symbol, &footprint)
            .map_err(|e| e.to_string())?;
        Ok(request)
    }
}

/// Read the existing captured library projection, qualified against its model.
fn captured_object(
    library: &crate::library_browser::LibraryBrowser,
    id: Uuid,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let model = library
        .basis
        .as_ref()
        .ok_or("Current library source unavailable")?;
    let object = model
        .objects
        .get(&id)
        .filter(|o| o.domain == "pool" && o.kind == kind)
        .ok_or("Library source identity unavailable")?;
    let mut entries = library
        .entries
        .iter()
        .filter(|e| e.id == id && e.kind == kind);
    let entry = entries
        .next()
        .filter(|e| e.warning.is_none() && e.revision == Some(object.object_revision.0))
        .ok_or("Library projection unavailable or stale")?;
    if entries.next().is_some() {
        return Err("Library source identity ambiguous".into());
    }
    Ok(entry.raw.clone())
}
