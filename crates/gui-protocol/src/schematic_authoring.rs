//! Consumer state for native schematic authoring; no persistence authority.
use crate::{BoardReviewSceneV1, PointNm};
use eda_engine::api::native_write::schematic_library::{
    materialize_pool_symbol_pins, resolve_pool_symbol_component_binding,
};
use eda_engine::pool::Symbol;
use eda_engine::substrate::{DesignModel, ElectricalSelectionSnapshot};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LibrarySymbolChoice {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub symbol: Symbol,
    pub pins: Vec<eda_engine::schematic::SymbolPin>,
    pub binding_status: String,
    pub refusal: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchematicEditRequest {
    PlaceSymbol {
        sheet: Uuid,
        library_symbol: Uuid,
        reference: String,
        value: String,
        x_nm: i64,
        y_nm: i64,
    },
    DrawWire {
        sheet: Uuid,
        points: Vec<eda_engine::ir::geometry::Point>,
    },
}
impl SchematicEditRequest {
    pub fn verb_params(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::PlaceSymbol {
                sheet,
                library_symbol,
                reference,
                value,
                x_nm,
                y_nm,
            } => (
                "datum.schematic.place_symbol",
                serde_json::json!({"sheet":sheet,"library_symbol":library_symbol,"reference":reference,"value":value,"x_nm":x_nm,"y_nm":y_nm}),
            ),
            Self::DrawWire { sheet, points } => (
                "datum.schematic.draw_wire",
                serde_json::json!({"sheet":sheet,"points":points}),
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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SchematicAuthoring {
    pub move_paths: std::collections::BTreeMap<Uuid, Vec<Uuid>>,
    pub move_bases: std::collections::BTreeMap<
        Uuid,
        Result<eda_engine::api::native_write::schematic_symbol_move::SymbolMoveBasis, String>,
    >,
    pub source: Option<eda_engine::schematic::Schematic>,
    pub sheet: Option<Uuid>,
    pub choices: Vec<LibrarySymbolChoice>,
    pub chosen: Option<usize>,
    pub choice_offset: usize,
    pub picker: bool,
    pub field: PlacementField,
    pub reference: String,
    pub value: String,
    pub position: Option<PointNm>,
    pub preview_dirty: bool,
    pub wire: bool,
    pub vertices: Vec<PointNm>,
    pub last_wire_click: Option<(PointNm, std::time::Instant)>,
    pub base: Option<BoardReviewSceneV1>,
    pub pending_request: Option<SchematicEditRequest>,
    pub refusal: Option<String>,
    pub taken_references: Vec<String>,
}
impl SchematicAuthoring {
    pub fn from_model(model: &DesignModel) -> anyhow::Result<Self> {
        let basis = ElectricalSelectionSnapshot::capture(model)
            .map_err(|e| anyhow::anyhow!("Schematic source unavailable: {e:?}"))?;
        let mut state = Self::default();
        state.source = basis.schematic_source().cloned();
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
                let available = binding.binding.as_ref().is_some_and(|b| b.part.is_some());
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
                    refusal: (!available).then(|| binding.diagnostics.join("; ")),
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
        self.refusal = None;
        true
    }
}

#[path = "schematic_authoring_preview.rs"]
mod preview;
