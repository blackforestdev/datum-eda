//! Reviewed membership delta and exact current-source replay validation.
use super::*;
use crate::substrate::{Operation, Proposal};

fn recorded(
    proposal: &Proposal,
) -> Result<(InterfaceBindingRequest, Uuid, SheetInstance), EngineError> {
    let mut change = None;
    for operation in &proposal.batch.operations {
        match operation {
            Operation::SetSchematicSheetInstance {
                schematic_id,
                instance_id,
                previous_instance,
                instance,
            } if change.is_none() => {
                change = Some((*schematic_id, *instance_id, previous_instance, instance));
            }
            Operation::GuardObjectRevision { .. }
            | Operation::CreateElectricalIdentity { .. }
            | Operation::SetElectricalIdentity { .. } => {}
            _ => {
                return Err(invalid(
                    "Interface binding contains unrelated or duplicate operations",
                ));
            }
        }
    }
    let (schematic, id, previous, next) =
        change.ok_or_else(|| invalid("Interface membership change is missing"))?;
    let before: SheetInstance = serde_json::from_value(previous.clone())?;
    let after: SheetInstance = serde_json::from_value(next.clone())?;
    let added: Vec<_> = after
        .ports
        .iter()
        .filter(|p| !before.ports.contains(p))
        .copied()
        .collect();
    let removed: Vec<_> = before
        .ports
        .iter()
        .filter(|p| !after.ports.contains(p))
        .copied()
        .collect();
    let (port, bind) = match (added.as_slice(), removed.as_slice()) {
        ([port], []) => (*port, true),
        ([], [port]) => (*port, false),
        _ => {
            return Err(invalid(
                "Review must change exactly one interface membership",
            ));
        }
    };
    let mut expected_ports = before.ports.clone();
    if bind {
        expected_ports.push(port);
    } else {
        expected_ports.retain(|p| *p != port);
    }
    let mut expected = previous.clone();
    expected["ports"] = json!(expected_ports);
    if before.uuid != id || after.uuid != id || expected != *next || before.parent_sheet.is_none() {
        return Err(invalid(
            "Interface review changes more than its selected membership",
        ));
    }
    Ok((
        InterfaceBindingRequest {
            proposal_id: proposal.proposal_id,
            instance: id,
            instance_path: vec![],
            parent_port: port,
            bind,
        },
        schematic,
        before,
    ))
}
pub(in crate::api::native_write) fn validate_recorded(
    proposal: &Proposal,
) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}
pub(in crate::api::native_write) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<InterfaceBindingSummary, EngineError> {
    let (mut request, schematic, before) = recorded(proposal)?;
    let source = source(model)?;
    if source.uuid != schematic {
        return Err(invalid("Interface schematic source changed"));
    }
    let child = source
        .sheet_definitions
        .get(&before.definition)
        .ok_or_else(|| invalid("Reviewed child definition is missing"))?
        .root_sheet;
    let paths = sheet_occurrences(&source)?;
    request.instance_path = paths
        .get(&child)
        .filter(|paths| paths.len() == 1)
        .ok_or_else(|| invalid("Reviewed child occurrence is no longer unique"))?[0]
        .clone();
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
            || build(model, &request)?.batch.operations != proposal.batch.operations
        {
            return Err(invalid("Design changed; prepare interface binding again"));
        }
        return preview(model, &request);
    }
    // Terminal responses describe the reviewed operation even after its membership changed.
    let mut candidate = source.clone();
    candidate
        .sheet_instances
        .insert(before.uuid, before.clone());
    let parent = before.parent_sheet.unwrap();
    let mut parent_path = request.instance_path.clone();
    if parent_path.pop() != Some(before.uuid)
        || paths.get(&parent) != Some(&vec![parent_path.clone()])
    {
        return Err(invalid("Reviewed parent occurrence changed"));
    }
    let port = source
        .sheets
        .get(&parent)
        .and_then(|sheet| sheet.ports.get(&request.parent_port))
        .ok_or_else(|| invalid("Reviewed parent port is missing"))?;
    let selected = candidate.sheet_instances.get_mut(&before.uuid).unwrap();
    if !selected.ports.contains(&request.parent_port) {
        selected.ports.push(request.parent_port);
    }
    let label = sheet_interface_bindings(&candidate, before.uuid)?[&request.parent_port];
    Ok(InterfaceBindingSummary {
        instance: before.uuid,
        definition: before.definition,
        parent,
        parent_path,
        child,
        instance_path: request.instance_path,
        parent_port: request.parent_port,
        child_label: label,
        name: port.name.clone(),
        bind: request.bind,
    })
}
