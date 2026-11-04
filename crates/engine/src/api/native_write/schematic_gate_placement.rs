//! Explicit placement of one unused Gate into a current schematic-only owner.
use super::{
    schematic_assignment::{component, options_impl},
    schematic_library::{materialize_pool_symbol_pins, materialized_pool_object},
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    pool,
    schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode},
    substrate::{DesignModel, Operation, Proposal},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use uuid::Uuid;
#[path = "schematic_gate_placement_receipt.rs"]
mod receipt;
pub(super) use receipt::{build, summary, validate_recorded};
pub(super) const REASON: &str = "Place another schematic Gate";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatePlacementRequest {
    pub proposal_id: Uuid,
    pub component_instance: Uuid,
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub gate: Uuid,
    pub library_symbol: Uuid,
    pub reference: String,
    pub value: String,
    pub x_nm: i64,
    pub y_nm: i64,
    pub rotation_deg: i32,
    pub mirrored: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatePlacementChoice {
    pub gate: Uuid,
    pub gate_name: String,
    pub unit: Uuid,
    pub library_symbol: Uuid,
    pub symbol_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatePlacementOptions {
    pub component_instance: Uuid,
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub part: Uuid,
    pub part_name: String,
    pub entity: Uuid,
    pub entity_name: String,
    pub choices: Vec<GatePlacementChoice>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatePlacementSummary {
    pub component_instance: Uuid,
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub part: Uuid,
    pub entity: Uuid,
    pub choice: GatePlacementChoice,
    pub symbol: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation_deg: i32,
    pub mirrored: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePlacementPreview {
    pub summary: GatePlacementSummary,
    pub symbol: PlacedSymbol,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePlacementMaterial {
    pub library_symbol: pool::Symbol,
    pub symbol: PlacedSymbol,
}
fn invalid(s: impl Into<String>) -> EngineError {
    EngineError::Validation(s.into())
}
fn id(raw: &Value, key: &str) -> Result<Uuid, EngineError> {
    Ok(serde_json::from_value(raw[key].clone())?)
}
pub(super) fn is_gate_placement(p: &Proposal) -> bool {
    p.batch.provenance.reason == REASON
}

#[path = "schematic_gate_options.rs"]
mod gate_options;
pub use gate_options::options;
use gate_options::{occurrence, owner_symbol, source};
pub fn materialize(
    model: &DesignModel,
    captured: &GatePlacementOptions,
    gate: Uuid,
    proposal: Uuid,
) -> Result<GatePlacementMaterial, EngineError> {
    let fresh = options(
        model,
        owner_symbol(model, captured.component_instance)?,
        &captured.instance_path,
    )?;
    if fresh != *captured {
        return Err(invalid("Gate options changed; select the component again"));
    }
    let choice = fresh
        .choices
        .iter()
        .find(|c| c.gate == gate)
        .ok_or_else(|| invalid("Choose an unused compatible Gate"))?;
    let library_symbol = serde_json::from_value(materialized_pool_object(
        model,
        choice.library_symbol,
        "symbols",
    )?)?;
    let seed = [
        proposal.to_string(),
        fresh.component_instance.to_string(),
        gate.to_string(),
    ];
    let symbol_id =
        super::ids::derive_object_id(&model.project.project_id, "gate-placed-symbol", &seed);
    let mut pins = materialize_pool_symbol_pins(model, Some(&choice.library_symbol.to_string()))?;
    for pin in &mut pins {
        let mut seed = seed.to_vec();
        seed.push(
            pin.library_pin
                .ok_or_else(|| invalid("Native pin identity unavailable"))?
                .to_string(),
        );
        pin.uuid =
            super::ids::derive_object_id(&model.project.project_id, "gate-placed-pin", &seed);
    }
    if model.objects.contains_key(&symbol_id)
        || pins.iter().any(|p| model.objects.contains_key(&p.uuid))
    {
        return Err(invalid("Prepared Gate identities are already occupied"));
    }
    let symbol = PlacedSymbol {
        uuid: symbol_id,
        part: Some(fresh.part),
        entity: Some(fresh.entity),
        gate: Some(gate),
        lib_id: Some(choice.library_symbol.to_string()),
        reference: String::new(),
        value: String::new(),
        fields: vec![],
        pins,
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    Ok(GatePlacementMaterial {
        library_symbol,
        symbol,
    })
}
impl GatePlacementMaterial {
    /// Pointer preview is pure and retains the exact eventual authored identities.
    pub fn placed(
        &self,
        reference: &str,
        value: &str,
        at: Point,
        rotation_deg: i32,
        mirrored: bool,
    ) -> Result<PlacedSymbol, EngineError> {
        if rotation_deg.rem_euclid(90) != 0 {
            return Err(invalid("Gate rotation must be cardinal"));
        }
        let mut symbol = self.symbol.clone();
        for pin in &mut symbol.pins {
            pin.position =
                super::schematic_edit::place_local_point(pin.position, at, rotation_deg, mirrored)?;
        }
        symbol.position = at;
        symbol.rotation = rotation_deg;
        symbol.mirrored = mirrored;
        symbol.reference = reference.into();
        symbol.value = value.into();
        Ok(symbol)
    }
}
pub fn preview(
    model: &DesignModel,
    request: &GatePlacementRequest,
) -> Result<GatePlacementPreview, EngineError> {
    if request.reference.is_empty()
        || request
            .reference
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
        || request.value.chars().any(char::is_control)
    {
        return Err(invalid(
            "Supply a unique nonempty reference without whitespace and a single-line value",
        ));
    }
    let options = options(
        model,
        owner_symbol(model, request.component_instance)?,
        &request.instance_path,
    )?;
    if options.sheet != request.sheet {
        return Err(invalid("Gate placement sheet changed"));
    }
    let choice = options
        .choices
        .iter()
        .find(|c| c.gate == request.gate && c.library_symbol == request.library_symbol)
        .cloned()
        .ok_or_else(|| invalid("Gate body is not a compatible unused choice"))?;
    if source(model)?
        .sheets
        .values()
        .any(|s| s.symbols.values().any(|s| s.reference == request.reference))
    {
        return Err(invalid("Reference is already used"));
    }
    let material = materialize(model, &options, request.gate, request.proposal_id)?;
    let position = Point::new(request.x_nm, request.y_nm);
    let symbol = material.placed(
        &request.reference,
        &request.value,
        position,
        request.rotation_deg,
        request.mirrored,
    )?;
    Ok(GatePlacementPreview {
        summary: GatePlacementSummary {
            component_instance: options.component_instance,
            sheet: options.sheet,
            instance_path: options.instance_path,
            part: options.part,
            entity: options.entity,
            choice,
            symbol: symbol.uuid,
            reference: symbol.reference.clone(),
            value: symbol.value.clone(),
            position,
            rotation_deg: symbol.rotation,
            mirrored: symbol.mirrored,
        },
        symbol,
    })
}

#[cfg(test)]
#[path = "schematic_gate_placement_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "schematic_gate_placement_refusal_tests.rs"]
mod refusal_tests;

#[cfg(test)]
#[path = "schematic_singleton_compatibility_tests.rs"]
mod singleton_compatibility_tests;
