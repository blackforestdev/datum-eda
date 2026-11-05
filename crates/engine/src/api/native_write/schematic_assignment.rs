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
use std::collections::BTreeSet;
use uuid::Uuid;
#[path = "schematic_assignment_component.rs"]
pub(super) mod component;
pub(super) use component::{delete_bindings, repin_symbols};
#[path = "schematic_assignment_options.rs"]
pub(super) mod options_impl;
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
    /// Selected Symbol first, followed by the other members of its shared owner.
    pub members: Vec<PlacedSymbol>,
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
    #[serde(default)]
    pub symbols: Vec<Uuid>,
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
    let mut operations = Vec::new();
    for member in &options.members {
        let mut value =
            super::schematic_symbol_payload::source_symbol_payload(model, options.sheet, member)?;
        value["part"] = json!(part);
        // Multi-Gate choices retain the complete existing Entity/Gate mapping.
        if options.members.len() == 1 {
            value["entity"] = json!(choice.entity);
            value["gate"] = json!(gate);
        }
        operations.push(Operation::SetSchematicSymbol {
            sheet_id: options.sheet,
            symbol_id: member.uuid,
            symbol: value,
        });
    }
    operations.push(component::operation(model, &options, choice)?);
    Ok(PreparedWrite {
        batch: super::context::build_batch(model, provenance, operations)?,
        primary_object_id: Some(symbol),
    })
}
pub(super) fn is_assignment(proposal: &Proposal) -> bool {
    if super::schematic_annotation::is_annotation(proposal)
        || super::schematic_field_batch::is_field_batch(proposal)
    {
        return false;
    }
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
    members: Vec<(Uuid, Value)>,
    part: Uuid,
    entity: Uuid,
    gate: Uuid,
}
/// Recorded payload validation does not require live source to permit dismissal.
pub(super) fn validate_recorded(proposal: &Proposal) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}
fn recorded(proposal: &Proposal) -> Result<RecordedAssignment, EngineError> {
    let mut set = Vec::new();
    let mut ci = None;
    let mut creates_owner = false;
    for op in &proposal.batch.operations {
        match op {
            Operation::GuardObjectRevision { .. } => {}
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } => set.push((*sheet_id, *symbol_id, symbol)),
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                component_instance,
                ..
            } if ci.is_none() => {
                creates_owner = matches!(op, Operation::CreateComponentInstance { .. });
                ci = Some((*component_instance_id, component_instance));
            }
            _ => {
                return Err(invalid(
                    "Assignment proposal contains unrelated or duplicate operations",
                ));
            }
        }
    }
    let &(sheet, symbol, next) = set
        .first()
        .ok_or_else(|| invalid("Assignment Symbol is unavailable"))?;
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
    let mut ids = BTreeSet::new();
    let mut gates = BTreeSet::new();
    let mut members = Vec::new();
    for (owner, id, next) in &set {
        let member: PlacedSymbol = serde_json::from_value((*next).clone())?;
        if *owner != sheet
            || member.uuid != *id
            || !ids.insert(*id)
            || member.part != Some(part)
            || member.entity != Some(entity)
            || (set.len() > 1 && member.gate.is_none_or(|gate| !gates.insert(gate)))
        {
            return Err(invalid(
                "Assignment must preserve one complete compatible component",
            ));
        }
        members.push((*id, (*next).clone()));
    }
    let refs = payload["placed_symbol_refs"]
        .as_array()
        .ok_or_else(|| invalid("Assignment component members unavailable"))?;
    let recorded_ids = refs
        .iter()
        .map(|r| serde_json::from_value::<Uuid>(r["object_id"].clone()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if new.uuid != symbol
        || payload["uuid"] != json!(component_instance)
        || payload["part_ref"]["object_id"] != json!(part)
        || refs.len() != ids.len()
        || recorded_ids != ids
        || (creates_owner && set.len() > 1)
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
        members,
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
        members,
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
    let mut selected = None;
    let mut prior_part = None;
    for (id, next) in &members {
        let previous = previous_symbol(model, proposal, sheet, *id, next)?;
        let old: PlacedSymbol = serde_json::from_value(previous.clone())?;
        let mut expected = previous;
        expected["part"] = json!(part);
        if members.len() == 1 {
            expected["entity"] = json!(entity);
            expected["gate"] = json!(gate);
        } else if old.entity != Some(entity)
            || prior_part
                .replace(old.part)
                .is_some_and(|previous| previous != old.part)
        {
            return Err(invalid(
                "Shared assignment must retain one Entity and original Part",
            ));
        }
        if old.uuid != *id || expected != *next {
            return Err(invalid("Assignment changes unrelated Symbol source"));
        }
        if *id == symbol {
            selected = Some(old);
        }
    }
    let old = selected.ok_or_else(|| invalid("Assignment selected member unavailable"))?;
    let part_value = materialized_pool_object(model, part, "parts")?;
    let entity_value = materialized_pool_object(model, entity, "entities")?;
    Ok(AssignmentSummary {
        symbol,
        sheet,
        instance_path: path.clone(),
        component_instance,
        reference: old.reference,
        symbols: members.iter().map(|(id, _)| *id).collect(),
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
fn previous_symbol(
    model: &DesignModel,
    proposal: &Proposal,
    sheet: Uuid,
    symbol: Uuid,
    next: &Value,
) -> Result<Value, EngineError> {
    let previous = if let Some(transaction) = proposal.applied_transaction_id {
        let tx = model
            .journal
            .iter()
            .find(|tx| tx.transaction_id == transaction)
            .ok_or_else(|| invalid("Assignment transaction is unavailable"))?;
        if !tx.operations.iter().any(|op| {
            matches!(op,
            Operation::SetSchematicSymbol { sheet_id, symbol_id, symbol: payload }
            if *sheet_id == sheet && *symbol_id == symbol && payload == next)
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
    Ok(previous)
}
#[cfg(test)]
#[path = "schematic_assignment_tests.rs"]
pub(crate) mod tests;
