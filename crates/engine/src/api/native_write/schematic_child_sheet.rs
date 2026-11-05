//! Blank child creation composes existing source, definition and instance writers.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    schematic_sheets,
};
use crate::{
    connectivity::occurrence_partition::sheet_occurrences,
    error::EngineError,
    ir::geometry::Point,
    schematic::{Schematic, SheetDefinition, SheetInstance},
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot, Operation, Proposal},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
#[path = "schematic_child_sheet_receipt.rs"]
mod receipt;
pub(super) use receipt::{check_destinations, summary, validate_recorded};
pub(super) const REASON: &str = "Create blank child schematic sheet";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildSheetRequest {
    pub proposal_id: Uuid,
    pub parent: Uuid,
    pub instance_path: Vec<Uuid>,
    pub sheet: Uuid,
    pub definition: Uuid,
    pub instance: Uuid,
    pub name: String,
    pub position: Point,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildSheetSummary {
    pub parent: Uuid,
    pub parent_name: String,
    pub instance_path: Vec<Uuid>,
    pub sheet: Uuid,
    pub definition: Uuid,
    pub instance: Uuid,
    pub name: String,
    pub position: Point,
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
pub fn parent_occurrence(
    model: &DesignModel,
    parent: Uuid,
    path: &[Uuid],
) -> Result<String, EngineError> {
    let source = source(model)?;
    let paths = sheet_occurrences(&source)?;
    if paths
        .get(&parent)
        .is_none_or(|p| p.len() != 1 || p[0] != path)
    {
        return Err(invalid(
            "Child creation requires one exact, unrepeated parent occurrence",
        ));
    }
    Ok(source.sheets[&parent].name.clone())
}
fn sheet_payload(request: &ChildSheetRequest) -> Value {
    json!({"schema_version":1,"uuid":request.sheet,"name":request.name,"frame":null,
        "symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},
        "bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}})
}
fn definition(request: &ChildSheetRequest) -> SheetDefinition {
    SheetDefinition {
        uuid: request.definition,
        root_sheet: request.sheet,
        name: request.name.clone(),
    }
}
fn definition_payload(request: &ChildSheetRequest) -> Value {
    json!({"schema_version":1,"uuid":request.definition,"root_sheet":request.sheet,"name":request.name})
}
fn instance(request: &ChildSheetRequest) -> SheetInstance {
    SheetInstance {
        uuid: request.instance,
        definition: request.definition,
        parent_sheet: Some(request.parent),
        position: request.position,
        name: request.name.clone(),
        ports: vec![],
    }
}
pub fn preview(
    model: &DesignModel,
    request: &ChildSheetRequest,
) -> Result<ChildSheetSummary, EngineError> {
    preview_source(model, request).map(|(summary, _)| summary)
}
/// Immutable canonical material for the shared placement ghost; never persists.
pub fn preview_source(
    model: &DesignModel,
    request: &ChildSheetRequest,
) -> Result<(ChildSheetSummary, Schematic), EngineError> {
    let parent_name = parent_occurrence(model, request.parent, &request.instance_path)?;
    if request.name.trim().is_empty()
        || request.name != request.name.trim()
        || request.name.chars().any(char::is_control)
    {
        return Err(invalid("Enter a nonempty, single-line child sheet name"));
    }
    let ids = [request.sheet, request.definition, request.instance];
    let mut source = source(model)?;
    if ids.iter().any(|id| {
        id.is_nil()
            || model.objects.contains_key(id)
            || source.sheets.contains_key(id)
            || source.sheet_definitions.contains_key(id)
            || source.sheet_instances.contains_key(id)
    }) || ids.iter().collect::<std::collections::BTreeSet<_>>().len() != 3
    {
        return Err(invalid(
            "Child source, definition and instance require distinct fresh identities",
        ));
    }
    source.sheets.insert(
        request.sheet,
        serde_json::from_value(sheet_payload(request))?,
    );
    source
        .sheet_definitions
        .insert(request.definition, definition(request));
    source
        .sheet_instances
        .insert(request.instance, instance(request));
    let occurrences = sheet_occurrences(&source)?;
    let mut path = request.instance_path.clone();
    path.push(request.instance);
    if occurrences.get(&request.sheet) != Some(&vec![path]) {
        return Err(invalid("New child has no unique complete occurrence"));
    }
    Ok((
        ChildSheetSummary {
            parent: request.parent,
            parent_name,
            instance_path: request.instance_path.clone(),
            sheet: request.sheet,
            definition: request.definition,
            instance: request.instance,
            name: request.name.clone(),
            position: request.position,
        },
        source,
    ))
}
pub(super) fn build(
    model: &DesignModel,
    request: &ChildSheetRequest,
) -> Result<PreparedWrite, EngineError> {
    preview(model, request)?;
    let source = source(model)?;
    let provenance = WriteProvenance::new("datum-gui", CommitSource::Manual, REASON);
    let writes = [
        schematic_sheets::build_create_schematic_sheet(
            model,
            provenance.clone(),
            source.uuid,
            request.sheet,
            &format!("sheets/{}.json", request.sheet),
            sheet_payload(request),
        )?,
        schematic_sheets::build_create_schematic_definition(
            model,
            provenance.clone(),
            source.uuid,
            request.definition,
            &format!("definitions/{}.json", request.definition),
            definition_payload(request),
        )?,
        schematic_sheets::build_create_schematic_sheet_instance(
            model,
            provenance.clone(),
            source.uuid,
            request.instance,
            serde_json::to_value(instance(request))?,
        )?,
    ];
    let operations = writes
        .into_iter()
        .flat_map(|w| w.batch.operations)
        .filter(|o| !matches!(o, Operation::GuardObjectRevision { .. }));
    let mut write = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .primary_object(request.instance)
        .finish()?;
    write.batch.batch_id = request.proposal_id;
    write.batch = crate::substrate::compose_electrical_transaction(
        model,
        write.batch,
        crate::substrate::TransactionKind::Normal,
    )?;
    Ok(write)
}
pub(super) fn is_child_sheet(proposal: &Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}
#[cfg(test)]
#[path = "schematic_child_sheet_tests.rs"]
mod tests;

pub(super) fn check_new_paths(
    root: &std::path::Path,
    request: &ChildSheetRequest,
) -> Result<(), EngineError> {
    for relative in [
        format!("sheets/{}.json", request.sheet),
        format!("definitions/{}.json", request.definition),
    ] {
        let path = root.join("schematic").join(relative);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                return Err(invalid(
                    "Child source or definition destination already exists; cancel and place again",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(invalid(format!(
                    "Child destination cannot be checked: {error}"
                )));
            }
        }
    }
    Ok(())
}
