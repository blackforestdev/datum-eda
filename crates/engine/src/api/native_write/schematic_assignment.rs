//! Explicit same-geometry Part/Gate assignment through the shared Proposal lifecycle.
use super::{
    context::{PreparedWrite, WriteProvenance},
    schematic_library::materialized_pool_object,
};
use crate::{
    error::EngineError,
    schematic::PlacedSymbol,
    substrate::{DesignModel, Operation, Proposal},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
#[path = "schematic_assignment_component.rs"]
mod component;
pub(super) use component::{delete_bindings, repin_symbols};
#[path = "schematic_assignment_options.rs"]
mod options_impl;
pub use options_impl::options;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentChoice {
    pub part: Uuid,
    pub part_name: String,
    pub entity: Uuid,
    pub entity_name: String,
    pub gate: Uuid,
    pub gate_name: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AssignmentOptions {
    pub sheet: Uuid,
    pub original: PlacedSymbol,
    pub component_instance: Option<Uuid>,
    pub choices: Vec<AssignmentChoice>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentSummary {
    pub symbol: Uuid,
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub component_instance: Uuid,
    pub reference: String,
    pub previous_part: Option<Uuid>,
    pub choice: AssignmentChoice,
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
fn id(value: &Value, field: &str) -> Result<Uuid, EngineError> {
    serde_json::from_value(value[field].clone()).map_err(Into::into)
}
pub(super) fn build(
    model: &DesignModel,
    provenance: WriteProvenance,
    symbol: Uuid,
    path: &[Uuid],
    part: Uuid,
    gate: Uuid,
) -> Result<PreparedWrite, EngineError> {
    let options = options(model, symbol, path)?;
    let choice = options
        .choices
        .iter()
        .find(|c| c.part == part && c.gate == gate)
        .ok_or_else(|| invalid("Choose a compatible Part and Gate for this Symbol"))?;
    if options.component_instance.is_some()
        && options.original.part == Some(part)
        && options.original.entity == Some(choice.entity)
        && options.original.gate == Some(gate)
    {
        return Err(invalid("Part assignment is unchanged"));
    }
    let previous = super::schematic_symbol_payload::source_symbol_payload(
        model,
        options.sheet,
        &options.original,
    )?;
    let mut value = previous.clone();
    value["part"] = json!(part);
    value["entity"] = json!(choice.entity);
    value["gate"] = json!(gate);
    let ci = component::operation(model, &options, choice)?;
    let operations = vec![
        Operation::SetSchematicSymbol {
            sheet_id: options.sheet,
            symbol_id: symbol,
            symbol: value,
        },
        ci,
    ];
    Ok(PreparedWrite {
        batch: super::context::build_batch(model, provenance, operations)?,
        primary_object_id: Some(symbol),
    })
}
pub(super) fn is_assignment(proposal: &Proposal) -> bool {
    proposal
        .batch
        .operations
        .iter()
        .any(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
        && proposal.batch.operations.iter().any(|op| {
            matches!(
                op,
                Operation::CreateComponentInstance { .. } | Operation::SetComponentInstance { .. }
            )
        })
}
struct RecordedAssignment {
    sheet: Uuid,
    symbol: Uuid,
    component_instance: Uuid,
    next: Value,
    part: Uuid,
    entity: Uuid,
    gate: Uuid,
}
/// Recorded payload validation does not require live source to permit dismissal.
pub(super) fn validate_recorded(proposal: &Proposal) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}
fn recorded(proposal: &Proposal) -> Result<RecordedAssignment, EngineError> {
    let mut set = None;
    let mut ci = None;
    for op in &proposal.batch.operations {
        match op {
            Operation::GuardObjectRevision { .. } => {}
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } if set.is_none() => set = Some((*sheet_id, *symbol_id, symbol)),
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                component_instance,
                ..
            } if ci.is_none() => ci = Some((*component_instance_id, component_instance)),
            _ => {
                return Err(invalid(
                    "Assignment proposal contains unrelated or duplicate operations",
                ));
            }
        }
    }
    let (sheet, symbol, next) = set.ok_or_else(|| invalid("Assignment Symbol is unavailable"))?;
    let (component_instance, payload) =
        ci.ok_or_else(|| invalid("Assignment component is unavailable"))?;
    let new: PlacedSymbol = serde_json::from_value(next.clone())?;
    let part = new
        .part
        .ok_or_else(|| invalid("Assignment Part is unavailable"))?;
    let entity = new
        .entity
        .ok_or_else(|| invalid("Assignment Entity is unavailable"))?;
    let gate = new
        .gate
        .ok_or_else(|| invalid("Assignment Gate is unavailable"))?;
    if new.uuid != symbol
        || payload["uuid"] != json!(component_instance)
        || payload["part_ref"]["object_id"] != json!(part)
        || payload["placed_symbol_refs"]
            .as_array()
            .is_none_or(|r| r.len() != 1 || r[0]["object_id"] != json!(symbol))
        || payload["placed_package_refs"]
            .as_array()
            .is_none_or(|r| !r.is_empty())
    {
        return Err(invalid(
            "Assignment proposal changes unrelated source or binding identity",
        ));
    }
    Ok(RecordedAssignment {
        sheet,
        symbol,
        component_instance,
        next: next.clone(),
        part,
        entity,
        gate,
    })
}
/// Live compatible summaries qualify preview/apply; terminal rejection may omit
/// this optional display detail after the authored source has disappeared.
pub(super) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<AssignmentSummary, EngineError> {
    let RecordedAssignment {
        sheet,
        symbol,
        component_instance,
        next,
        part,
        entity,
        gate,
    } = recorded(proposal)?;

    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Assignment source unavailable: {e:?}")))?;
    let sources = snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Assignment occurrence unavailable: {e:?}")))?;
    let paths: Vec<_> = sources
        .iter()
        .filter(|s| s.occurrence.class == "symbols" && s.occurrence.source_id == symbol)
        .map(|s| s.occurrence.instance_path.clone())
        .collect();
    let [path] = paths.as_slice() else {
        return Err(invalid(
            "Assignment requires one matching Symbol occurrence",
        ));
    };
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid("Design changed; prepare Part assignment again"));
        }
        let rebuilt = build(
            model,
            WriteProvenance::new(
                "datum-gui",
                crate::substrate::CommitSource::Manual,
                "Assign schematic Part",
            ),
            symbol,
            path,
            part,
            gate,
        )?;
        if rebuilt.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Assignment proposal differs from current compatible binding",
            ));
        }
    }
    // Canonical SetSymbol carries only its new payload. The applied journal's
    // immutable inverse owns the preimage; unapplied drafts use revision-guarded source.
    let previous = if let Some(transaction) = proposal.applied_transaction_id {
        let tx = model
            .journal
            .iter()
            .find(|tx| tx.transaction_id == transaction)
            .ok_or_else(|| invalid("Assignment transaction is unavailable"))?;
        if !tx.operations.iter().any(|op| {
            matches!(op,
            Operation::SetSchematicSymbol { sheet_id, symbol_id, symbol: payload }
            if *sheet_id == sheet && *symbol_id == symbol && *payload == next)
        }) {
            return Err(invalid(
                "Assignment transaction does not match its proposal",
            ));
        }
        tx.inverse_operations
            .iter()
            .find_map(|op| match op {
                Operation::SetSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol: payload,
                } if *sheet_id == sheet && *symbol_id == symbol => Some(payload.clone()),
                _ => None,
            })
            .ok_or_else(|| invalid("Assignment preimage is unavailable"))?
    } else {
        if proposal.prepared_against != model.model_revision {
            return Err(invalid("Assignment source revision is unavailable"));
        }
        let source = super::schematic_symbol_payload::source_sheet_payload(model, sheet, symbol)?;
        source["symbols"][symbol.to_string()].clone()
    };
    let old: PlacedSymbol = serde_json::from_value(previous.clone())?;
    let mut expected = previous;
    expected["part"] = json!(part);
    expected["entity"] = json!(entity);
    expected["gate"] = json!(gate);
    if old.uuid != symbol || expected != next {
        return Err(invalid("Assignment changes unrelated Symbol source"));
    }
    let part_value = materialized_pool_object(model, part, "parts")?;
    let entity_value = materialized_pool_object(model, entity, "entities")?;
    Ok(AssignmentSummary {
        symbol,
        sheet,
        instance_path: path.clone(),
        component_instance,
        reference: old.reference,
        previous_part: old.part,
        choice: AssignmentChoice {
            part,
            part_name: part_value["mpn"].as_str().unwrap_or("").into(),
            entity,
            entity_name: entity_value["name"].as_str().unwrap_or("").into(),
            gate,
            gate_name: entity_value["gates"][gate.to_string()]["name"]
                .as_str()
                .unwrap_or("")
                .into(),
        },
    })
}
#[cfg(test)]
#[path = "schematic_assignment_tests.rs"]
pub(crate) mod tests;
