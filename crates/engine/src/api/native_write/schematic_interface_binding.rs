//! Explicit instance membership changes through the canonical hierarchy writer.
use super::{
    context::{PreparedWrite, WriteProvenance},
    schematic_sheets,
};
use crate::{
    connectivity::occurrence_partition::{sheet_interface_bindings, sheet_occurrences},
    error::EngineError,
    schematic::{Schematic, SheetInstance},
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

pub(super) const REASON: &str = "Change child schematic interface binding";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceBindingRequest {
    pub proposal_id: Uuid,
    pub instance: Uuid,
    /// Complete selected child occurrence, including the selected instance.
    pub instance_path: Vec<Uuid>,
    pub parent_port: Uuid,
    pub bind: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceBindingSummary {
    pub instance: Uuid,
    pub definition: Uuid,
    pub parent: Uuid,
    pub parent_path: Vec<Uuid>,
    pub child: Uuid,
    pub instance_path: Vec<Uuid>,
    pub parent_port: Uuid,
    pub child_label: Uuid,
    pub name: String,
    pub bind: bool,
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
fn source(model: &DesignModel) -> Result<Schematic, EngineError> {
    ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Hierarchy source unavailable: {e:?}")))?
        .schematic_source()
        .cloned()
        .ok_or_else(|| invalid("Native schematic unavailable"))
}
pub fn preview(
    model: &DesignModel,
    request: &InterfaceBindingRequest,
) -> Result<InterfaceBindingSummary, EngineError> {
    let source = source(model)?;
    let instance = source
        .sheet_instances
        .get(&request.instance)
        .ok_or_else(|| invalid("Selected sheet instance is missing"))?;
    let parent = instance
        .parent_sheet
        .ok_or_else(|| invalid("Selected instance has no parent interface"))?;
    let child = source
        .sheet_definitions
        .get(&instance.definition)
        .ok_or_else(|| invalid("Selected child definition is missing"))?
        .root_sheet;
    let mut parent_path = request.instance_path.clone();
    if parent_path.pop() != Some(instance.uuid) {
        return Err(invalid(
            "Selected child occurrence does not end at this instance",
        ));
    }
    let paths = sheet_occurrences(&source)?;
    if paths.get(&parent) != Some(&vec![parent_path.clone()])
        || paths.get(&child) != Some(&vec![request.instance_path.clone()])
    {
        return Err(invalid(
            "Interface editing requires exact, unrepeated parent and child occurrences",
        ));
    }
    // Validate the entire original membership before deriving one explicit change.
    sheet_interface_bindings(&source, instance.uuid)?;
    if instance.ports.contains(&request.parent_port) == request.bind {
        return Err(invalid("Interface already has the requested binding state"));
    }
    let port = source
        .sheets
        .get(&parent)
        .and_then(|s| s.ports.get(&request.parent_port))
        .ok_or_else(|| invalid("Selected parent port is missing from the actual parent source"))?;
    let name = port.name.clone();
    // The same resolver validates a candidate membership and identifies its exact label.
    let mut candidate = source.clone();
    let selected = candidate.sheet_instances.get_mut(&instance.uuid).unwrap();
    if !selected.ports.contains(&request.parent_port) {
        selected.ports.push(request.parent_port);
    }
    let label = sheet_interface_bindings(&candidate, instance.uuid)?[&request.parent_port];
    Ok(InterfaceBindingSummary {
        instance: instance.uuid,
        definition: instance.definition,
        parent,
        parent_path,
        child,
        instance_path: request.instance_path.clone(),
        parent_port: request.parent_port,
        child_label: label,
        name,
        bind: request.bind,
    })
}
pub(super) fn build(
    model: &DesignModel,
    request: &InterfaceBindingRequest,
) -> Result<PreparedWrite, EngineError> {
    preview(model, request)?;
    let source = source(model)?;
    let raw = super::schematic_symbol_payload::source_sheet_payload(
        model,
        source.uuid,
        request.instance,
    )?;
    let candidates: Vec<_> = raw
        .get("instances")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("Native instance payload is unavailable"))?
        .iter()
        .filter(|instance| instance.get("uuid") == Some(&json!(request.instance)))
        .collect();
    if candidates.len() != 1 {
        return Err(invalid("Native instance payload is missing or ambiguous"));
    }
    let previous = candidates[0].clone();
    let mut typed: SheetInstance = serde_json::from_value(previous.clone())?;
    if source.sheet_instances.get(&request.instance) != Some(&typed) {
        return Err(invalid("Native instance payload does not match its source"));
    }
    if request.bind {
        typed.ports.push(request.parent_port);
    } else {
        typed.ports.retain(|port| *port != request.parent_port);
    }
    let mut next = previous.clone();
    next["ports"] = json!(typed.ports);
    let mut write = schematic_sheets::build_set_schematic_sheet_instance(
        model,
        WriteProvenance::new("datum-gui", CommitSource::Manual, REASON),
        source.uuid,
        request.instance,
        previous,
        next,
    )?;
    write.batch.batch_id = request.proposal_id;
    write.batch = crate::substrate::compose_electrical_transaction(
        model,
        write.batch,
        crate::substrate::TransactionKind::Normal,
    )?;
    Ok(write)
}

#[path = "schematic_interface_binding_receipt.rs"]
mod receipt;
pub(super) use receipt::{summary, validate_recorded};
pub(super) fn is_interface_binding(proposal: &crate::substrate::Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}
#[cfg(test)]
#[path = "schematic_interface_binding_tests.rs"]
mod tests;
