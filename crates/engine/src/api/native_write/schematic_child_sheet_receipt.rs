//! Reconstruct and revalidate the complete immutable blank-child Proposal.
use super::*;
fn recorded(proposal: &Proposal) -> Result<(ChildSheetRequest, Uuid), EngineError> {
    let (mut sheet, mut def, mut placed) = (None, None, None);
    for operation in &proposal.batch.operations {
        match operation {
            Operation::CreateSchematicSheet {
                schematic_id,
                sheet_id,
                relative_path,
                sheet: payload,
            } if sheet.is_none() => {
                sheet = Some((*schematic_id, *sheet_id, relative_path, payload))
            }
            Operation::CreateSchematicDefinition {
                schematic_id,
                definition_id,
                relative_path,
                definition: payload,
            } if def.is_none() => {
                def = Some((*schematic_id, *definition_id, relative_path, payload))
            }
            Operation::CreateSchematicSheetInstance {
                schematic_id,
                instance_id,
                instance: payload,
            } if placed.is_none() => placed = Some((*schematic_id, *instance_id, payload)),
            Operation::GuardObjectRevision { .. }
            | Operation::CreateElectricalIdentity { .. }
            | Operation::SetElectricalIdentity { .. } => {}
            _ => {
                return Err(invalid(
                    "Child creation contains unrelated or duplicate operations",
                ));
            }
        }
    }
    let (schematic, sheet_id, sheet_path, sheet_raw) =
        sheet.ok_or_else(|| invalid("Child source missing"))?;
    let (def_schematic, definition_id, def_path, def_raw) =
        def.ok_or_else(|| invalid("Child definition missing"))?;
    let (instance_schematic, instance_id, instance_raw) =
        placed.ok_or_else(|| invalid("Child instance missing"))?;
    let placed: SheetInstance = serde_json::from_value(instance_raw.clone())?;
    let request = ChildSheetRequest {
        proposal_id: proposal.proposal_id,
        parent: placed
            .parent_sheet
            .ok_or_else(|| invalid("Child parent missing"))?,
        instance_path: vec![],
        sheet: sheet_id,
        definition: definition_id,
        instance: instance_id,
        name: placed.name.clone(),
        position: placed.position,
    };
    if schematic != def_schematic
        || schematic != instance_schematic
        || *sheet_path != format!("sheets/{sheet_id}.json")
        || *def_path != format!("definitions/{definition_id}.json")
        || *sheet_raw != sheet_payload(&request)
        || *def_raw != definition_payload(&request)
        || *instance_raw != serde_json::to_value(instance(&request))?
    {
        return Err(invalid(
            "Child source, definition and instance do not match the reviewed blank sheet",
        ));
    }
    let ids = [
        request.parent,
        request.sheet,
        request.definition,
        request.instance,
    ];
    if ids.iter().any(Uuid::is_nil)
        || ids.iter().collect::<std::collections::BTreeSet<_>>().len() != 4
        || request.name.trim().is_empty()
        || request.name != request.name.trim()
        || request.name.chars().any(char::is_control)
    {
        return Err(invalid("Child identity or name is invalid"));
    }
    Ok((request, schematic))
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
) -> Result<ChildSheetSummary, EngineError> {
    let (mut request, schematic) = recorded(proposal)?;
    let source = source(model)?;
    if source.uuid != schematic {
        return Err(invalid("Child schematic source changed"));
    }
    let paths = sheet_occurrences(&source)?;
    let paths = paths
        .get(&request.parent)
        .filter(|p| p.len() == 1)
        .ok_or_else(|| invalid("Child parent occurrence changed"))?;
    request.instance_path = paths[0].clone();
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
            || build(model, &request)?.batch.operations != proposal.batch.operations
        {
            return Err(invalid("Design changed; prepare child creation again"));
        }
    }
    Ok(ChildSheetSummary {
        parent: request.parent,
        parent_name: source.sheets[&request.parent].name.clone(),
        instance_path: request.instance_path,
        sheet: request.sheet,
        definition: request.definition,
        instance: request.instance,
        name: request.name,
        position: request.position,
    })
}

pub(in crate::api::native_write) fn check_destinations(
    proposal: &Proposal,
    root: &std::path::Path,
) -> Result<(), EngineError> {
    let (request, _) = recorded(proposal)?;
    check_new_paths(root, &request)
}
