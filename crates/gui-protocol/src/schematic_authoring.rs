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
#[path = "schematic_named_point.rs"]
mod named_point;
#[path = "schematic_selection_edit.rs"]
pub mod selected;
pub use named_point::LabelDraft;
#[path = "schematic_bus.rs"]
mod bus;
pub use bus::{BusEditDraft, bus_fields};
#[path = "schematic_fields.rs"]
mod fields;
pub use fields::FieldsEditDraft;
#[path = "schematic_custom_field.rs"]
mod custom_field;
pub use custom_field::{CustomFieldControl, CustomFieldDraft};
#[path = "schematic_bus_entry.rs"]
mod bus_entry;
pub use bus_entry::BusEntryDraft;

#[path = "schematic_assignment.rs"]
pub mod assignment;
#[path = "schematic_delete_review.rs"]
pub mod delete_review;
#[path = "erc_disposition.rs"]
pub mod disposition;

#[derive(Debug, Clone, PartialEq)]
pub struct LibrarySymbolChoice {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub symbol: Symbol,
    pub pins: Vec<eda_engine::schematic::SymbolPin>,
    pub binding_status: String,
    pub binding_diagnostics: Vec<String>,
    pub refusal: Option<String>,
}
impl LibrarySymbolChoice {
    pub fn binding_label(&self) -> &'static str {
        match self.binding_status.as_str() {
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchematicEditRequest {
    Text {
        sheet: Uuid,
        text: Option<Uuid>,
        instance_path: Vec<Uuid>,
        value: Option<String>,
        x_nm: Option<i64>,
        y_nm: Option<i64>,
        rotation_deg: Option<i32>,
    },
    CustomField {
        symbol: Uuid,
        instance_path: Vec<Uuid>,
        field_id: Option<Uuid>,
        key: Option<String>,
        value: Option<String>,
        clear: bool,
        visible: Option<bool>,
        position: Option<eda_engine::ir::geometry::Point>,
    },
    AddBusEntry {
        sheet: Uuid,
        bus: Uuid,
        instance_path: Vec<Uuid>,
        wire: Uuid,
        at: PointNm,
        wire_end: eda_engine::api::native_write::schematic_bus::WireEnd,
    },
    DeleteBusEntry {
        bus: Uuid,
        bus_entry: Uuid,
        instance_path: Vec<Uuid>,
    },
    CreateBus {
        sheet: Uuid,
        bus: Option<Uuid>,
        instance_path: Vec<Uuid>,
        name: String,
        members: Vec<String>,
        points: Option<Vec<eda_engine::ir::geometry::Point>>,
    },
    PlacePort {
        sheet: Uuid,
        port: Option<Uuid>,
        instance_path: Vec<Uuid>,
        name: String,
        direction: PortDirection,
        x_nm: i64,
        y_nm: i64,
    },
    EditLabel {
        label: Uuid,
        instance_path: Vec<Uuid>,
        name: String,
        kind: LabelKind,
    },
    EditSymbol {
        symbol: Uuid,
        instance_path: Vec<Uuid>,
        reference: String,
        value: String,
    },
    DeleteObject {
        object: Uuid,
        instance_path: Vec<Uuid>,
        kind: selected::DeleteKind,
    },
    CreateSheet {
        sheet: Uuid,
        name: String,
    },
    PlaceSymbol {
        sheet: Uuid,
        library_symbol: Uuid,
        reference: String,
        value: String,
        x_nm: i64,
        y_nm: i64,
        #[serde(default)]
        rotation_deg: i32,
        #[serde(default)]
        mirrored: bool,
    },
    DrawWire {
        sheet: Uuid,
        points: Vec<eda_engine::ir::geometry::Point>,
    },
    PlaceMarker {
        sheet: Uuid,
        kind: MarkerKind,
        x_nm: i64,
        y_nm: i64,
        symbol: Option<Uuid>,
        pin: Option<Uuid>,
    },
    PlaceLabel {
        sheet: Uuid,
        kind: LabelKind,
        name: String,
        x_nm: i64,
        y_nm: i64,
    },
}
impl SchematicEditRequest {
    pub fn verb_params(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Text {
                sheet,
                text,
                instance_path,
                value,
                x_nm,
                y_nm,
                rotation_deg,
            } => {
                let mut params = serde_json::json!({"sheet":sheet,"instance_path":instance_path});
                if let Some(id) = text {
                    params["text"] = serde_json::json!(id);
                }
                if let Some(value) = value {
                    params["value"] = serde_json::json!(value);
                }
                if let Some(x) = x_nm {
                    params["x_nm"] = serde_json::json!(x);
                }
                if let Some(y) = y_nm {
                    params["y_nm"] = serde_json::json!(y);
                }
                if let Some(rotation) = rotation_deg {
                    params["rotation_deg"] = serde_json::json!(rotation);
                }
                ("datum.schematic.place_text", params)
            }
            Self::CustomField {
                symbol,
                instance_path,
                field_id,
                key,
                value,
                clear,
                visible,
                position,
            } => {
                let mut params = serde_json::json!({"symbol":symbol,"instance_path":instance_path,"field":"custom"});
                if let Some(id) = field_id {
                    params["field_id"] = serde_json::json!(id);
                }
                if let Some(key) = key {
                    params["key"] = serde_json::json!(key);
                }
                if let Some(value) = value {
                    params["value"] = serde_json::json!(value);
                }
                if *clear {
                    params["clear"] = serde_json::json!(true);
                }
                if let Some(visible) = visible {
                    params["visible"] = serde_json::json!(visible);
                }
                if let Some(position) = position {
                    params["position"] = serde_json::json!(position);
                }
                ("datum.schematic.set_symbol_field", params)
            }
            Self::AddBusEntry {
                sheet,
                bus,
                instance_path,
                wire,
                at,
                wire_end,
            } => (
                "datum.schematic.create_bus",
                serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":instance_path,"entries":[{"wire":wire,"x_nm":at.x,"y_nm":at.y,"wire_end":wire_end}]}),
            ),
            Self::DeleteBusEntry {
                bus,
                bus_entry,
                instance_path,
            } => (
                "datum.schematic.delete_bus_entry",
                serde_json::json!({"bus":bus,"bus_entry":bus_entry,"instance_path":instance_path}),
            ),
            Self::CreateBus {
                sheet,
                bus,
                instance_path,
                name,
                members,
                points,
            } => {
                let mut params = serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":instance_path,"name":name,"members":members});
                if let Some(points) = points {
                    params["points"] = serde_json::json!(points);
                }
                ("datum.schematic.create_bus", params)
            }
            Self::PlacePort {
                sheet,
                port,
                instance_path,
                name,
                direction,
                x_nm,
                y_nm,
            } => (
                "datum.schematic.place_port",
                serde_json::json!({"sheet":sheet,"port":port,"instance_path":instance_path,"name":name,"direction":match direction {
                    PortDirection::Input => "input", PortDirection::Output => "output",
                    PortDirection::Bidirectional => "bidirectional", PortDirection::Passive => "passive",
                },"x_nm":x_nm,"y_nm":y_nm}),
            ),
            Self::EditLabel {
                label,
                instance_path,
                name,
                kind,
            } => (
                "datum.schematic.rename_label",
                serde_json::json!({"label":label,"instance_path":instance_path,"name":name,"kind": match kind {
                    LabelKind::Local => "local", LabelKind::Global => "global", LabelKind::Hierarchical => "hierarchical", LabelKind::Power => "power",
                }}),
            ),
            Self::EditSymbol {
                symbol,
                instance_path,
                reference,
                value,
            } => (
                "datum.schematic.set_symbol_reference",
                serde_json::json!({"symbol":symbol,"instance_path":instance_path,"reference":reference,"value":value}),
            ),
            Self::DeleteObject {
                object,
                instance_path,
                kind,
            } => {
                let (verb, key) = kind.verb_and_key();
                let mut params = serde_json::json!({"instance_path":instance_path});
                params[key] = serde_json::json!(object);
                (verb, params)
            }
            Self::CreateSheet { sheet, name } => (
                "datum.schematic.create_sheet",
                serde_json::json!({"sheet":sheet,"name":name}),
            ),
            Self::PlaceSymbol {
                sheet,
                library_symbol,
                reference,
                value,
                x_nm,
                y_nm,
                rotation_deg,
                mirrored,
            } => (
                "datum.schematic.place_symbol",
                serde_json::json!({"sheet":sheet,"library_symbol":library_symbol,"reference":reference,"value":value,"x_nm":x_nm,"y_nm":y_nm,"rotation_deg":rotation_deg,"mirrored":mirrored}),
            ),
            Self::DrawWire { sheet, points } => (
                "datum.schematic.draw_wire",
                serde_json::json!({"sheet":sheet,"points":points}),
            ),
            Self::PlaceMarker {
                sheet,
                kind,
                x_nm,
                y_nm,
                symbol,
                pin,
            } => (
                "datum.schematic.place_marker",
                serde_json::json!({"sheet":sheet,"kind":kind.token(),"x_nm":x_nm,"y_nm":y_nm,"symbol":symbol,"pin":pin}),
            ),
            Self::PlaceLabel {
                sheet,
                kind,
                name,
                x_nm,
                y_nm,
            } => (
                "datum.schematic.place_label",
                serde_json::json!({"sheet":sheet,"kind": match kind {
                    LabelKind::Local => "local", LabelKind::Global => "global",
                    LabelKind::Hierarchical => "hierarchical", LabelKind::Power => "power",
                },"name":name,"x_nm":x_nm,"y_nm":y_nm}),
            ),
        }
    }
}
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
    pub delete_review: Option<delete_review::DeleteReview>,
    pub disposition_draft: Option<delete_review::DeleteReview>,
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
    pub choices: Vec<LibrarySymbolChoice>,
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
            || self.chosen.is_none()
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
                let id = symbol.uuid.to_string();
                let binding = resolve_pool_symbol_component_binding(model, Some(&id))?;
                Ok(LibrarySymbolChoice {
                    id: symbol.uuid,
                    name: symbol.name.clone(),
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
        state
            .choices
            .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
        Ok(state)
    }
    pub fn select(&mut self, index: usize) -> bool {
        let Some(choice) = self.choices.get(index) else {
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
        self.value = choice.name.clone();
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
