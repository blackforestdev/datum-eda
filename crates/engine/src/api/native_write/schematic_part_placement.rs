//! Explicit initial Part/gate selection over the existing placement composition.
use super::{
    PoolSymbolComponentBinding, PoolSymbolPartBinding, materialized_pool_object, object_revision,
};
use crate::{
    error::EngineError,
    pool::{Entity, Part, Symbol, Unit},
    substrate::{DesignModel, ObjectRevision, Operation},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Captured source intent, never an inferred preferred Part or gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartGateSelection {
    pub part: Uuid,
    pub entity: Uuid,
    pub gate: Uuid,
    pub unit: Uuid,
    pub symbol: Uuid,
    pub part_revision: u64,
    pub entity_revision: u64,
    pub unit_revision: u64,
    pub symbol_revision: u64,
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}
impl PartGateSelection {
    pub fn capture(
        model: &DesignModel,
        part: Uuid,
        gate: Uuid,
        symbol: Uuid,
    ) -> Result<Self, EngineError> {
        let selected: Part =
            serde_json::from_value(materialized_pool_object(model, part, "parts")?)?;
        let raw = materialized_pool_object(model, selected.entity, "entities")?;
        let entity: Entity = serde_json::from_value(raw.clone())?;
        let matches: Vec<_> = raw
            .get("gates")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| invalid("Selected Entity gates are unavailable"))?
            .iter()
            .filter(|(key, _)| Uuid::parse_str(key).ok() == Some(gate))
            .collect();
        if matches.len() != 1 {
            return Err(invalid("Choose one unambiguous gate of the selected Part"));
        }
        let chosen: crate::pool::Gate = serde_json::from_value(matches[0].1.clone())?;
        let body: Symbol =
            serde_json::from_value(materialized_pool_object(model, symbol, "symbols")?)?;
        let unit: Unit =
            serde_json::from_value(materialized_pool_object(model, chosen.unit, "units")?)?;
        if selected.uuid != part
            || entity.uuid != selected.entity
            || chosen.uuid != gate
            || chosen.symbol != symbol
            || body.uuid != symbol
            || body.unit != chosen.unit
            || unit.uuid != chosen.unit
        {
            return Err(invalid(
                "Selected Part, Entity, gate, Unit and Symbol are incompatible",
            ));
        }
        Ok(Self {
            part,
            entity: entity.uuid,
            gate,
            unit: unit.uuid,
            symbol,
            part_revision: object_revision(model, part, "selected Part")?,
            entity_revision: object_revision(model, entity.uuid, "selected Entity")?,
            unit_revision: object_revision(model, unit.uuid, "selected Unit")?,
            symbol_revision: object_revision(model, symbol, "selected Symbol")?,
        })
    }
    pub fn resolve(
        &self,
        model: &DesignModel,
        symbol: Uuid,
    ) -> Result<PoolSymbolComponentBinding, EngineError> {
        if self.symbol != symbol || Self::capture(model, self.part, self.gate, symbol)? != *self {
            return Err(invalid("Explicit Part/gate sources changed; choose again"));
        }
        Ok(PoolSymbolComponentBinding {
            symbol_id: self.symbol,
            symbol_revision: self.symbol_revision,
            unit_id: self.unit,
            unit_revision: self.unit_revision,
            entity_id: self.entity,
            entity_revision: self.entity_revision,
            gate_id: self.gate,
            part: Some(PoolSymbolPartBinding {
                part_id: self.part,
                part_revision: self.part_revision,
            }),
        })
    }
    pub(crate) fn guard(&self, prepared: &mut crate::api::native_write::context::PreparedWrite) {
        for (id, revision) in [
            (self.part, self.part_revision),
            (self.entity, self.entity_revision),
            (self.unit, self.unit_revision),
            (self.symbol, self.symbol_revision),
        ] {
            if !prepared.batch.operations.iter().any(
                |op| matches!(op, Operation::GuardObjectRevision {object_id,..} if *object_id==id),
            ) {
                prepared.batch.operations.insert(
                    0,
                    Operation::GuardObjectRevision {
                        object_id: id,
                        expected_object_revision: ObjectRevision(revision),
                    },
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "schematic_part_placement_tests.rs"]
mod tests;
