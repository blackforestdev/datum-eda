//! Shared guarded Symbol/component receipt grammar and immutable inverse sources.
use super::schematic_symbol_payload::source_sheet_payload;
use crate::{
    error::EngineError,
    schematic::PlacedSymbol,
    substrate::{DesignModel, Operation, Proposal},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
pub(super) struct Recorded<'a> {
    pub(super) sheets: BTreeMap<Uuid, Uuid>,
    pub(super) symbols: BTreeMap<Uuid, &'a Value>,
    pub(super) symbol_count: usize,
    pub(super) bound_metadata: bool,
}
impl Recorded<'_> {
    pub(super) fn single_sheet(&self) -> Result<Uuid, EngineError> {
        let ids: BTreeSet<_> = self.sheets.values().copied().collect();
        if ids.len() != 1 {
            return Err(invalid(
                "This Symbol edit requires exactly one source sheet",
            ));
        }
        Ok(*ids.first().unwrap())
    }
}
/// A receipt identifies the actual unique source occurrence, including empty sheets.
pub(super) fn occurrence_path(model: &DesignModel, sheet: Uuid) -> Result<Vec<Uuid>, EngineError> {
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Symbol edit source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Symbol edit schematic source unavailable"))?;
    let paths = crate::connectivity::occurrence_partition::sheet_occurrences(source)?;
    paths
        .get(&sheet)
        .filter(|paths| paths.len() == 1)
        .map(|paths| paths[0].clone())
        .ok_or_else(|| invalid("Symbol edit requires one current, unrepeated sheet occurrence"))
}
pub(super) fn recorded(proposal: &Proposal) -> Result<Recorded<'_>, EngineError> {
    let bound_metadata = proposal.batch.provenance.reason == super::schematic_field_batch::REASON
        && proposal.batch.operations.iter().any(|op| matches!(op,
            Operation::SetComponentInstance { previous_component_instance, .. }
                if previous_component_instance["placed_package_refs"].as_array().is_some_and(|refs| !refs.is_empty())));
    if bound_metadata {
        DesignModel::validate_component_metadata_proposal(proposal)?;
    }
    let mut sheets = BTreeMap::new();
    let mut symbols = BTreeMap::new();
    let mut guards = BTreeSet::new();
    let mut components = BTreeSet::new();
    let mut owners = BTreeSet::new();
    let mut bound_members = BTreeSet::new();
    let changed: BTreeSet<_> = proposal
        .batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::SetSchematicSymbol { symbol_id, .. } = op {
                Some(*symbol_id)
            } else {
                None
            }
        })
        .collect();
    for op in &proposal.batch.operations {
        match op {
            Operation::GuardObjectRevision { object_id, .. } => {
                if !guards.insert(*object_id) {
                    return Err(invalid("Symbol field edit contains duplicate guards"));
                }
            }
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } => {
                let typed: PlacedSymbol = serde_json::from_value(symbol.clone())?;
                if typed.uuid != *symbol_id
                    || symbols.insert(*symbol_id, symbol).is_some()
                    || sheets.insert(*symbol_id, *sheet_id).is_some()
                {
                    return Err(invalid(
                        "Symbol field edit changes duplicate or unrelated Symbols",
                    ));
                }
            }
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance: old,
                component_instance: next,
            } => {
                let refs = old["placed_symbol_refs"]
                    .as_array()
                    .ok_or_else(|| invalid("Symbol field edit binding references unavailable"))?;
                if refs.is_empty()
                    || (!bound_metadata
                        && old["placed_package_refs"]
                            .as_array()
                            .is_none_or(|v| !v.is_empty()))
                {
                    return Err(invalid("Symbol field edit binding is not schematic-only"));
                }
                let mut expected = old.clone();
                expected["object_revision"] = json!(
                    old["object_revision"]
                        .as_u64()
                        .and_then(|v| v.checked_add(1))
                        .ok_or_else(|| invalid("Symbol field edit component epoch unavailable"))?
                );
                let mut touched = false;
                for (index, member) in refs.iter().enumerate() {
                    let symbol: Uuid = serde_json::from_value(member["object_id"].clone())?;
                    if !bound_members.insert(symbol) {
                        return Err(invalid(
                            "Symbol field edit contains ambiguous component members",
                        ));
                    }
                    if changed.contains(&symbol) {
                        touched = true;
                        owners.insert(symbol);
                        expected["placed_symbol_refs"][index]["object_revision"] = json!(
                            member["object_revision"]
                                .as_u64()
                                .and_then(|v| v.checked_add(1))
                                .ok_or_else(|| invalid(
                                    "Symbol field edit Symbol epoch unavailable"
                                ))?
                        );
                    }
                }
                if old["uuid"] != json!(component_instance_id)
                    || expected != *next
                    || !components.insert(*component_instance_id)
                    || !touched
                {
                    return Err(invalid("Symbol field edit changes unrelated binding data"));
                }
            }
            Operation::SetElectricalIdentity { record, .. } if bound_metadata => {
                if !guards.contains(&record.id) {
                    return Err(invalid(
                        "Bound field edit lacks its Net evidence source guard",
                    ));
                }
            }
            _ => return Err(invalid("Symbol field edit contains unrelated operations")),
        }
    }
    if symbols.is_empty()
        || !owners.iter().all(|id| symbols.contains_key(id))
        || !bound_members.iter().all(|id| guards.contains(id))
        || !symbols
            .keys()
            .chain(&components)
            .all(|id| guards.contains(id))
        || !components.is_disjoint(&symbols.keys().copied().collect())
    {
        return Err(invalid(
            "Symbol field edit family or complete source guards unavailable",
        ));
    }
    let symbol_count = if bound_metadata {
        symbols.len()
    } else {
        guards.len() - components.len()
    };
    Ok(Recorded {
        sheets,
        symbols,
        symbol_count,
        bound_metadata,
    })
}
pub(super) fn old_payloads(
    model: &DesignModel,
    proposal: &Proposal,
    recorded: &Recorded<'_>,
) -> Result<BTreeMap<Uuid, Value>, EngineError> {
    let old_payloads: BTreeMap<_, _> = if let Some(id) = proposal.applied_transaction_id {
        let tx = model
            .journal
            .iter()
            .find(|tx| tx.transaction_id == id)
            .ok_or_else(|| invalid("Symbol field edit transaction unavailable"))?;
        if recorded.bound_metadata {
            model.validate_component_metadata_transaction(proposal, tx)?;
        }
        if !proposal
            .batch
            .operations
            .iter()
            .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
            .all(|op| tx.operations.contains(op))
        {
            return Err(invalid(
                "Symbol field edit transaction does not match its proposal",
            ));
        }
        let mut old = BTreeMap::new();
        for op in &tx.inverse_operations {
            if let Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } = op
            {
                if recorded.sheets.get(symbol_id) != Some(sheet_id)
                    || !recorded.symbols.contains_key(symbol_id)
                    || old.insert(*symbol_id, symbol.clone()).is_some()
                {
                    return Err(invalid("Symbol field edit inverse source does not match"));
                }
            }
        }
        old
    } else {
        if proposal.prepared_against != model.model_revision {
            return Err(invalid("Symbol field edit source revision unavailable"));
        }
        let mut sources = BTreeMap::new();
        for (symbol, sheet) in &recorded.sheets {
            if !sources.contains_key(sheet) {
                sources.insert(*sheet, source_sheet_payload(model, *sheet, *symbol)?);
            }
        }
        recorded
            .symbols
            .keys()
            .map(|id| {
                (
                    *id,
                    sources[&recorded.sheets[id]]["symbols"][id.to_string()].clone(),
                )
            })
            .collect()
    };
    Ok(old_payloads)
}
