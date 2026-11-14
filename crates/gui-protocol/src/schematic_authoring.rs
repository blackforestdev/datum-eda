//! Consumer state for native schematic authoring; no persistence authority.
use crate::{BoardReviewSceneV1, PointNm};
use eda_engine::api::native_write::schematic_library::{
    materialize_pool_symbol_pins, resolve_pool_symbol_component_binding,
};
use eda_engine::pool::Symbol;
use eda_engine::schematic::{LabelKind, PortDirection};
use eda_engine::substrate::{DesignModel, ElectricalSelectionSnapshot};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[path = "schematic_part_choices.rs"]
mod part_choices;
pub use eda_engine::api::native_write::schematic_library::part_placement::PartGateSelection;
#[path = "schematic_component_placement.rs"]
mod component_placement;
pub use component_placement::ComponentPlacement;
#[path = "schematic_named_point.rs"]
mod named_point;
#[path = "schematic_selection_edit.rs"]
pub mod selected;
pub use named_point::LabelDraft;
#[path = "schematic_bus.rs"]
mod bus;
#[path = "design_clipboard_placement.rs"]
pub mod design_clipboard_placement;
#[path = "schematic_wire_draft.rs"]
mod wire_draft;
pub use bus::{BusEditDraft, bus_fields};
pub use design_clipboard_placement::DesignClipboardPlacement;
#[path = "schematic_fields.rs"]
mod fields;
pub use fields::FieldsEditDraft;
#[path = "schematic_custom_field.rs"]
mod custom_field;
pub use custom_field::{CustomFieldControl, CustomFieldDraft};
#[path = "schematic_bus_entry.rs"]
mod bus_entry;
pub use bus_entry::BusEntryDraft;

#[path = "schematic_child_sheet.rs"]
pub mod child_sheet;
#[path = "schematic_gate_placement.rs"]
pub mod gate_placement;
#[path = "schematic_interface_binding.rs"]
pub mod interface_binding;
#[path = "proposal_review_layout.rs"]
pub mod review_layout;

#[path = "schematic_annotation.rs"]
pub mod annotation;
#[path = "schematic_assignment.rs"]
pub mod assignment;
#[path = "schematic_delete_review.rs"]
pub mod delete_review;
#[path = "erc_disposition.rs"]
pub mod disposition;
#[path = "schematic_field_table.rs"]
pub mod field_table;
#[path = "schematic_project_annotation.rs"]
pub mod project_annotation;

#[derive(Debug, Clone, PartialEq)]
pub struct LibrarySymbolChoice {
    pub id: Uuid,
    pub explicit_part: Option<PartGateSelection>,
    pub component: Option<
        eda_engine::api::native_write::schematic_library::part_placement::CompletePartSelection,
    >,
    pub name: String,
    pub prefix: String,
    pub value: String,
    pub symbol: Symbol,
    pub pins: Vec<eda_engine::schematic::SymbolPin>,
    pub binding_status: String,
    pub binding_diagnostics: Vec<String>,
    pub refusal: Option<String>,
}
impl LibrarySymbolChoice {
    pub fn binding_label(&self) -> &'static str {
        match self.binding_status.as_str() {
            "explicit_part_gate" => "Explicit Part and gate selected",
            "bound_with_part" => "Part binding resolved",
            "bound_without_part" => "No Part binding: no matching Part",
            "ambiguous_part" => "No Part binding: multiple compatible Parts",
            "incompatible_part" => "No Part binding: no compatible Part",
            "unresolved_entity_gate" => "No Part binding: no matching entity/gate",
            "ambiguous_entity_gate" => "No Part binding: multiple entity/gate matches",
            _ => "No Part binding",
        }
    }
}
/// One marker placement command, configured by its electrical meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind {
    Junction,
    NoConnect,
}
impl MarkerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Junction => "Junction",
            Self::NoConnect => "No-Connect",
        }
    }
    pub fn token(self) -> &'static str {
        match self {
            Self::Junction => "junction",
            Self::NoConnect => "noconnect",
        }
    }
}
#[path = "schematic_edit_request.rs"]
mod schematic_edit_request;
pub use schematic_edit_request::SchematicEditRequest;

#[path = "schematic_wire_anchor.rs"]
pub mod schematic_wire_anchor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlacementField {
    #[default]
    Reference,
    Value,
}
#[derive(Debug, Clone, Copy)]
pub enum PlacementTransform {
    Rotate,
    FlipX,
    FlipY,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NewSchematicDraft {
    pub name: String,
    pub editor: crate::single_line_editor::SingleLineEditor,
    /// Name, Cancel, Create; shared keyboard and pointer focus order.
    pub focus: usize,
    pub revision: String,
    pub tip: Option<Uuid>,
    pub refusal: Option<String>,
}
impl PlacementTransform {
    /// Mirror local X before rotation; world-axis flips compose with that pose.
    pub fn apply(self, rotation_deg: i32, mirrored: bool) -> (i32, bool) {
        let rotation = i64::from(rotation_deg);
        let rotation = match self {
            Self::Rotate => rotation + 90,
            Self::FlipX => -rotation,
            Self::FlipY => 180 - rotation,
        }
        .rem_euclid(360) as i32;
        (
            rotation,
            mirrored ^ matches!(self, Self::FlipX | Self::FlipY),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SchematicAuthoring {
    /// Only Symbol placement, Wire and Reference/Value opt into this qualified basis.
    pub circuit_basis: Option<crate::AuthoringViewBasis>,
    pub circuit_preview: Option<BoardReviewSceneV1>,
    pub delete_review: Option<delete_review::DeleteReview>,
    pub disposition_draft: Option<delete_review::DeleteReview>,
    pub gate_draft: Option<gate_placement::GatePlacementDraft>,
    pub child_draft: Option<child_sheet::ChildSheetDraft>,
    pub design_placement: Option<DesignClipboardPlacement>,
    pub interface_draft: Option<interface_binding::InterfaceBindingDraft>,
    pub assignment_draft: Option<delete_review::DeleteReview>,
    pub creation: Option<NewSchematicDraft>,
    pub editing: Option<FieldsEditDraft>,
    pub named_edit: Option<selected::NamedPointEditDraft>,
    pub label: Option<LabelDraft>,
    pub marker: Option<MarkerKind>,
    pub move_paths: std::collections::BTreeMap<Uuid, Vec<Uuid>>,
    pub move_bases: std::collections::BTreeMap<
        Uuid,
        Result<eda_engine::api::native_write::schematic_symbol_move::SymbolMoveBasis, String>,
    >,
    pub text_move_bases: std::collections::BTreeMap<
        Uuid,
        Result<eda_engine::api::native_write::schematic_text::TextMoveBasis, String>,
    >,
    pub label_move_bases: std::collections::BTreeMap<
        Uuid,
        Result<eda_engine::api::native_write::schematic_label_move::LabelMoveBasis, String>,
    >,
    pub source: Option<eda_engine::schematic::Schematic>,
    pub sheet: Option<Uuid>,
    /// Disposable source-qualified projection basis for existing placed IDs.
    /// Legacy unbound sources keep the ordinary current-source display basis.
    pub placed_symbol_definitions: std::collections::BTreeMap<Uuid, Result<Option<Symbol>, String>>,
    pub choices: Vec<LibrarySymbolChoice>,
    pub components: ComponentPlacement,
    pub chosen: Option<usize>,
    pub choice_offset: usize,
    pub picker: bool,
    pub field: PlacementField,
    pub reference: String,
    pub reference_editor: crate::single_line_editor::SingleLineEditor,
    pub value: String,
    pub value_editor: crate::single_line_editor::SingleLineEditor,
    pub position: Option<PointNm>,
    pub rotation_deg: i32,
    pub mirrored: bool,
    pub preview_dirty: bool,
    /// Bus configures the shared polyline gesture and two-field form.
    pub bus: bool,
    pub bus_entry: Option<BusEntryDraft>,
    pub bus_entry_choice: usize,
    pub wire: bool,
    pub vertices: Vec<PointNm>,
    /// Transient posture shared by Wire and Bus preview and anchored vertices.
    pub wire_route: eda_engine::api::native_write::schematic_wire_edit::WireRouteConfig,
    pub last_wire_click: Option<(PointNm, std::time::Instant)>,
    pub base: Option<BoardReviewSceneV1>,
    pub pending_request: Option<SchematicEditRequest>,
    pub refusal: Option<String>,
    pub taken_references: Vec<String>,
}
impl SchematicAuthoring {
    /// The engine reflects local X before rotating. World-X/Y flips therefore
    /// change both pose fields; toggling local mirror alone is wrong after R.
    pub fn transform_placement(&mut self, transform: PlacementTransform) -> bool {
        if !self.picker
            && matches!(transform, PlacementTransform::Rotate)
            && let Some(text) = self
                .label
                .as_mut()
                .filter(|draft| draft.text_rotation.is_some())
        {
            text.cycle_kind();
            self.preview_dirty = true;
            return true;
        }
        if self.picker
            || self.wire
            || self.label.is_some()
            || self.marker.is_some()
            || (self.chosen.is_none() && self.design_placement.is_none())
        {
            return false;
        }
        (self.rotation_deg, self.mirrored) = transform.apply(self.rotation_deg, self.mirrored);
        self.preview_dirty = true;
        true
    }
    pub fn from_model(model: &DesignModel) -> anyhow::Result<Self> {
        let basis = ElectricalSelectionSnapshot::capture(model)
            .map_err(|e| anyhow::anyhow!("Schematic source unavailable: {e:?}"))?;
        let mut current_symbols = std::collections::BTreeMap::new();
        let mut state = Self {
            source: basis.schematic_source().cloned(),
            ..Default::default()
        };
        if let Some(source) = basis.schematic_source() {
            let authored = basis
                .authored_sources()
                .map_err(|e| anyhow::anyhow!("Schematic occurrences unavailable: {e:?}"))?;
            for symbol in source
                .sheets
                .values()
                .flat_map(|sheet| sheet.symbols.values())
            {
                let paths: Vec<_> = authored
                    .iter()
                    .filter(|s| {
                        s.occurrence.class == "symbols" && s.occurrence.source_id == symbol.uuid
                    })
                    .map(|s| s.occurrence.instance_path.clone())
                    .collect();
                let qualified = if paths.len() == 1 {
                    state.move_paths.insert(symbol.uuid, paths[0].clone());
                    eda_engine::api::native_write::schematic_symbol_move::SymbolMoveBasis::capture(
                        &basis,
                        symbol.uuid,
                        &paths[0],
                    )
                    .map_err(|e| e.to_string())
                } else {
                    Err("Move requires one unique resolved Symbol occurrence".into())
                };
                state.move_bases.insert(symbol.uuid, qualified);
            }
            for label in source
                .sheets
                .values()
                .flat_map(|sheet| sheet.labels.values())
            {
                let paths: Vec<_> = authored
                    .iter()
                    .filter(|s| {
                        s.occurrence.class == "labels" && s.occurrence.source_id == label.uuid
                    })
                    .map(|s| s.occurrence.instance_path.clone())
                    .collect();
                let qualified = if paths.len() == 1 {
                    state.move_paths.insert(label.uuid, paths[0].clone());
                    eda_engine::api::native_write::schematic_label_move::LabelMoveBasis::capture(
                        &basis, label.uuid, &paths[0],
                    )
                    .map_err(|e| e.to_string())
                } else {
                    Err("Move requires one unique resolved Label occurrence".into())
                };
                state.label_move_bases.insert(label.uuid, qualified);
            }
            for text in source
                .sheets
                .values()
                .flat_map(|sheet| sheet.texts.values())
            {
                let paths: Vec<_> = authored
                    .iter()
                    .filter(|s| {
                        s.occurrence.class == "texts" && s.occurrence.source_id == text.uuid
                    })
                    .map(|s| s.occurrence.instance_path.clone())
                    .collect();
                let qualified = if paths.len() == 1 {
                    state.move_paths.insert(text.uuid, paths[0].clone());
                    eda_engine::api::native_write::schematic_text::TextMoveBasis::capture(
                        &basis, text.uuid, &paths[0],
                    )
                    .map_err(|e| e.to_string())
                } else {
                    Err("Move requires one unique resolved Text occurrence".into())
                };
                state.text_move_bases.insert(text.uuid, qualified);
            }
            let instantiated: std::collections::BTreeSet<_> = source
                .sheet_instances
                .values()
                .filter_map(|instance| source.sheet_definitions.get(&instance.definition))
                .map(|definition| definition.root_sheet)
                .collect();
            state.sheet = source
                .sheets
                .values()
                .filter(|sheet| !instantiated.contains(&sheet.uuid))
                .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
                .map(|s| s.uuid);
            state.taken_references = source
                .sheets
                .values()
                .flat_map(|sheet| sheet.symbols.values().map(|s| s.reference.clone()))
                .collect();
        }
        for object in model
            .objects
            .values()
            .filter(|o| o.domain == "pool" && o.kind == "symbols")
        {
            let choice = (|| -> anyhow::Result<LibrarySymbolChoice> {
                let shard = model
                    .source_shards
                    .iter()
                    .find(|s| s.shard_id == object.source_shard_id)
                    .ok_or_else(|| anyhow::anyhow!("Library source unavailable"))?;
                let symbol: Symbol = serde_json::from_value(
                    model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?,
                )?;
                current_symbols.insert(symbol.uuid, symbol.clone());
                let id = symbol.uuid.to_string();
                let binding = resolve_pool_symbol_component_binding(model, Some(&id))?;
                Ok(LibrarySymbolChoice {
                    id: symbol.uuid,
                    explicit_part: None,
                    component: None,
                    name: symbol.name.clone(),
                    value: symbol.name.clone(),
                    symbol: symbol.clone(),
                    prefix: symbol
                        .default_refdes_prefix
                        .clone()
                        .unwrap_or_else(|| "U".into()),
                    pins: materialize_pool_symbol_pins(model, Some(&id))?,
                    binding_status: binding.status.into(),
                    binding_diagnostics: binding.diagnostics,
                    // Materialization above is the eligibility boundary. A missing
                    // or ambiguous Part is an explicit non-ComponentInstance placement.
                    refusal: None,
                })
            })();
            match choice {
                Ok(choice) => state.choices.push(choice),
                Err(error) => {
                    state.refusal = Some(format!("A native library entry is unavailable: {error}"))
                }
            }
        }
        if let Some(source) = &state.source {
            for placed in source
                .sheets
                .values()
                .flat_map(|sheet| sheet.symbols.values())
            {
                let definition = if let Some(id) = placed
                    .lib_id
                    .as_deref()
                    .and_then(|id| id.parse::<Uuid>().ok())
                {
                    match model.materialized_placed_symbol_definition(placed.uuid, id) {
                        Ok(Some(raw)) => serde_json::from_value::<Symbol>(raw)
                            .map(Some)
                            .map_err(|e| e.to_string()),
                        // This is the existing legacy display-only fallback, not
                        // a fabricated historical binding or a new revision reader.
                        Ok(None) => current_symbols
                            .get(&id)
                            .cloned()
                            .map(Some)
                            .ok_or_else(|| "Native library source unavailable".to_owned()),
                        Err(error) => Err(error.to_string()),
                    }
                } else {
                    Ok(None)
                };
                state
                    .placed_symbol_definitions
                    .insert(placed.uuid, definition);
            }
        }
        state
            .choices
            .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
        part_choices::append(&mut state, model);
        Ok(state)
    }
    pub fn picker_choices(&self) -> &[LibrarySymbolChoice] {
        self.gate_draft.as_ref().map_or_else(
            || {
                if self.components.active {
                    self.components.choices.as_slice()
                } else {
                    &self.choices
                }
            },
            |draft| &draft.choices,
        )
    }
    pub fn select(&mut self, index: usize) -> bool {
        if self.gate_draft.is_some() {
            return false;
        }
        let Some(choice) = self.picker_choices().get(index).cloned() else {
            return false;
        };
        if let Some(refusal) = &choice.refusal {
            self.refusal = Some(refusal.clone());
            return false;
        }
        self.chosen = Some(index);
        let n = (1..)
            .find(|n| {
                !self
                    .taken_references
                    .contains(&format!("{}{n}", choice.prefix))
            })
            .unwrap();
        self.reference = format!("{}{n}", choice.prefix);
        self.value = choice.value.clone();
        self.reference_editor =
            crate::single_line_editor::SingleLineEditor::at_end(&self.reference);
        self.value_editor = crate::single_line_editor::SingleLineEditor::at_end(&self.value);
        self.refusal = None;
        true
    }
}

#[path = "schematic_authoring_preview.rs"]
mod preview;

#[cfg(test)]
#[path = "schematic_authoring_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_authoring_pose_tests.rs"]
mod pose_tests;

#[cfg(test)]
#[path = "schematic_child_circuit_tests.rs"]
mod child_circuit_tests;
