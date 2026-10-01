//! Explicit adoption of proved terminal lineage in canonical relationship source.
//! Existing placed UUIDs and representation bytes are retained.
use super::*;
use crate::substrate::{NetRelationshipIntent, RevisionedRef};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PlacedPinAdoption {
    pub sheet_id: Uuid,
    pub symbol_id: Uuid,
    pub placed_pin_id: Uuid,
    pub library_pin_id: Uuid,
}

/// Adopt explicit placed-terminal → library-pin correspondence. The pinned
/// PinPadMap cell, gate, library unit and ComponentInstance placement are checked
/// by the final canonical validator; no name matching or startup repair occurs.
pub fn build_adopt_pin_correspondence(
    model: &DesignModel,
    provenance: WriteProvenance,
    mut relationship: ElectricalIdentityRecord,
    adoptions: &[PlacedPinAdoption],
) -> Result<PreparedWrite, EngineError> {
    let invalid = |reason: &str| EngineError::Validation(format!("pin adoption refused: {reason}"));
    let ElectricalIdentity::NetRelationship {
        intent: NetRelationshipIntent::Implemented,
        evidence,
        ..
    } = &mut relationship.identity
    else {
        return Err(invalid("requires explicit Implemented correspondence"));
    };
    if adoptions.is_empty()
        || evidence.is_empty()
        || evidence.iter().any(|e| e.library_pad.is_none())
    {
        return Err(invalid("missing adoption/cell evidence"));
    }
    let mut pins = BTreeSet::new();
    for adoption in adoptions {
        if !pins.insert(adoption.placed_pin_id) {
            return Err(invalid("duplicate pin adoption"));
        }
        let object = model
            .objects
            .get(&adoption.symbol_id)
            .ok_or_else(|| invalid("missing symbol"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .ok_or_else(|| invalid("missing sheet basis"))?;
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if value.get("uuid").and_then(serde_json::Value::as_str)
            != Some(adoption.sheet_id.to_string().as_str())
        {
            return Err(invalid("symbol/sheet mismatch"));
        }
        let symbol: crate::schematic::PlacedSymbol = serde_json::from_value(
            value
                .get("symbols")
                .and_then(|v| v.get(adoption.symbol_id.to_string()))
                .ok_or_else(|| invalid("missing symbol payload"))?
                .clone(),
        )?;
        let matches: Vec<_> = symbol
            .pins
            .iter()
            .filter(|p| p.uuid == adoption.placed_pin_id)
            .collect();
        let [pin] = &matches[..] else {
            return Err(invalid("missing/ambiguous placed pin"));
        };
        if pin
            .library_pin
            .is_some_and(|id| id != adoption.library_pin_id)
        {
            return Err(invalid("existing provenance contradicts adoption"));
        }
        let library = model
            .objects
            .get(&adoption.library_pin_id)
            .filter(|o| o.domain == "pool")
            .ok_or_else(|| invalid("missing library pin"))?;
        let mut found = false;
        for item in evidence
            .iter_mut()
            .filter(|e| e.schematic_terminal.source_id == adoption.placed_pin_id)
        {
            if !model
                .component_instances
                .get(&item.component_instance.object_id)
                .is_some_and(|ci| ci.placed_symbol_refs.contains(&adoption.symbol_id))
            {
                return Err(invalid("foreign ComponentInstance"));
            }
            if item
                .library_pin
                .as_ref()
                .is_some_and(|r| r.object_id != adoption.library_pin_id)
            {
                return Err(invalid("conflicting relationship provenance"));
            }
            item.library_pin = Some(RevisionedRef {
                object_id: adoption.library_pin_id,
                object_revision: library.object_revision,
            });
            found = true;
        }
        if !found {
            return Err(invalid("adoption outside explicit evidence"));
        }
    }
    if model.electrical_identities.contains_key(&relationship.id) {
        build_set_electrical_identity(model, provenance, relationship)
    } else {
        build_create_electrical_identity(model, provenance, relationship)
    }
}
