//! Immutable typed design-edit receipt over the existing Proposal gateway.
use super::{
    context::{PreparedWrite, WriteProvenance},
    design_clipboard::{ClipboardClass, DesignClipboardBundle},
    design_clipboard_edit::{self, DesignEditIntent, place_operations, transform_operations},
    registry::NativeWriteContext,
    schematic_edit::require_unique_occurrence,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, SchematicWire, Sheet},
    substrate::{ElectricalSelectionSnapshot, Operation, Proposal, ProposalStatus},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

pub const REASON: &str = "Edit selected design objects";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    sheet: Uuid,
    instance_path: Vec<Uuid>,
    edit: DesignEditIntent,
    before: Sheet,
    cut_source: Option<DesignClipboardBundle>,
    #[serde(default)]
    transform_source: Option<DesignClipboardBundle>,
    electrical: Vec<Operation>,
    guards: Vec<Operation>,
}
pub fn is_design_edit(proposal: &Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}

pub fn prepare(
    context: &NativeWriteContext<'_>,
    mut provenance: WriteProvenance,
    proposal_id: Uuid,
    sheet: Uuid,
    path: &[Uuid],
    edit: &DesignEditIntent,
) -> Result<(PreparedWrite, String), EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Design edit source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Design edit source unavailable"))?;
    require_unique_occurrence(source, sheet, path)?;
    let before = source
        .sheets
        .get(&sheet)
        .ok_or_else(|| invalid("Design edit sheet unavailable"))?
        .clone();
    let cut_source = match edit {
        DesignEditIntent::Cut { targets } => Some(DesignClipboardBundle::capture(
            context.model,
            sheet,
            path,
            targets,
            Point::zero(),
        )?),
        _ => None,
    };
    let transform_source = match edit {
        DesignEditIntent::TransformExisting {
            targets, reference, ..
        } => Some(DesignClipboardBundle::capture(
            context.model,
            sheet,
            path,
            targets,
            *reference,
        )?),
        _ => None,
    };
    provenance.reason = REASON.into();
    if proposal_id.is_nil() {
        return Err(invalid("Design edit proposal identity is empty"));
    }
    let mut prepared = design_clipboard_edit::build_with_id(
        context,
        provenance,
        Some(proposal_id),
        sheet,
        path,
        edit,
    )?;
    prepared.batch = crate::substrate::compose_electrical_transaction(
        context.model,
        prepared.batch,
        crate::substrate::TransactionKind::Normal,
    )?;
    let receipt = Receipt {
        version: 1,
        sheet,
        instance_path: path.to_vec(),
        edit: edit.clone(),
        before,
        cut_source,
        transform_source,
        guards: prepared
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::GuardObjectRevision { .. }))
            .cloned()
            .collect(),
        electrical: prepared
            .batch
            .operations
            .iter()
            .filter(|op| electrical(op))
            .cloned()
            .collect(),
    };
    let rationale = serde_json::to_string(&receipt)?;
    Ok((prepared, rationale))
}
fn recorded(proposal: &Proposal) -> Result<Receipt, EngineError> {
    if !is_design_edit(proposal) {
        return Err(invalid("Design edit proposal family unavailable"));
    }
    let receipt: Receipt = serde_json::from_str(&proposal.rationale)?;
    if receipt.version != 1
        || receipt.sheet.is_nil()
        || receipt.before.uuid != receipt.sheet
        || proposal.batch.batch_id != proposal.proposal_id
    {
        return Err(invalid("Invalid design edit receipt"));
    }
    let expected = match &receipt.edit {
        DesignEditIntent::TransformExisting {
            targets,
            reference,
            placement,
            contact_policy,
        } => {
            if receipt.cut_source.is_some() {
                return Err(invalid("Transform receipt contains Cut authority"));
            }
            let captured = receipt
                .transform_source
                .as_ref()
                .ok_or_else(|| invalid("Transform source receipt missing"))?;
            if captured.source_project != proposal.project_id
                || captured.source_revision != proposal.prepared_against.0
                || captured.instance_path != receipt.instance_path
            {
                return Err(invalid(
                    "Transform receipt differs from its source authority",
                ));
            }
            transform_operations(
                &receipt.before,
                captured,
                targets,
                *reference,
                *placement,
                *contact_policy,
            )?
        }
        DesignEditIntent::PlaceBundle {
            bundle,
            placement,
            edit_seed,
            contact_policy,
        } => {
            if receipt.cut_source.is_some() || receipt.transform_source.is_some() {
                return Err(invalid("Paste receipt contains existing-source authority"));
            }
            place_operations(
                &receipt.before,
                bundle,
                placement,
                edit_seed,
                *contact_policy,
            )?
            .0
        }
        DesignEditIntent::Cut { targets } => {
            if receipt.transform_source.is_some() {
                return Err(invalid("Cut receipt contains transform authority"));
            }
            let bundle = receipt
                .cut_source
                .as_ref()
                .ok_or_else(|| invalid("Cut source receipt missing"))?;
            bundle.validate()?;
            if bundle.source_project != proposal.project_id
                || bundle.source_revision != proposal.prepared_against.0
                || bundle.source_sheet != receipt.sheet
                || bundle.instance_path != receipt.instance_path
                || bundle.reference != Point::zero()
            {
                return Err(invalid("Cut receipt does not match its source authority"));
            }
            let requested = targets
                .iter()
                .map(|t| (t.class, t.id))
                .collect::<std::collections::BTreeSet<_>>();
            let captured = bundle
                .items
                .iter()
                .map(|t| (t.class, t.id))
                .collect::<std::collections::BTreeSet<_>>();
            if requested.len() != targets.len() || requested != captured {
                return Err(invalid("Cut receipt target set changed"));
            }
            let mut operations = Vec::new();
            for item in &bundle.items {
                if item.raw.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
                    return Err(invalid("Cut receipt contains a locked object"));
                }
                match item.class {
                    ClipboardClass::Wire => {
                        let wire: SchematicWire = serde_json::from_value(item.raw.clone())?;
                        if receipt.before.wires.get(&item.id) != Some(&wire) {
                            return Err(invalid("Cut wire differs from its captured source"));
                        }
                        if receipt
                            .before
                            .bus_entries
                            .values()
                            .any(|entry| entry.wire == Some(item.id))
                        {
                            return Err(invalid("Cut wire has a surviving Bus Entry"));
                        }
                        operations.push((
                            item.id,
                            Operation::DeleteSchematicWire {
                                sheet_id: receipt.sheet,
                                wire_id: item.id,
                                wire: item.raw.clone(),
                            },
                        ));
                    }
                    ClipboardClass::Junction => {
                        let junction: Junction = serde_json::from_value(item.raw.clone())?;
                        if receipt.before.junctions.get(&item.id) != Some(&junction) {
                            return Err(invalid("Cut Junction differs from its captured source"));
                        }
                        operations.push((
                            item.id,
                            Operation::DeleteSchematicJunction {
                                sheet_id: receipt.sheet,
                                junction_id: item.id,
                                junction: item.raw.clone(),
                            },
                        ));
                    }
                }
            }
            operations.sort_by_key(|(id, _)| *id);
            operations.into_iter().map(|(_, op)| op).collect()
        }
    };
    let mut known = Vec::new();
    let mut identity = Vec::new();
    let mut guards = Vec::new();
    for op in &proposal.batch.operations {
        if electrical(op) {
            identity.push(op.clone());
            continue;
        }
        match op {
            Operation::GuardObjectRevision { .. } => {
                guards.push(op.clone());
            }
            Operation::CreateSchematicWire { .. }
            | Operation::DeleteSchematicWire { .. }
            | Operation::DeleteSchematicJunction { .. } => known.push(op.clone()),
            Operation::PlaceSchematicMarker {
                marker_kind: crate::substrate::SchematicMarkerKind::Junction,
                ..
            } => known.push(op.clone()),
            _ => {
                return Err(invalid(
                    "Design edit proposal contains unrelated operations",
                ));
            }
        }
    }
    if known != expected
        || guards != receipt.guards
        || receipt
            .guards
            .iter()
            .any(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
        || identity != receipt.electrical
        || receipt.electrical.iter().any(|op| !electrical(op))
    {
        return Err(invalid(
            "Design edit proposal differs from its immutable typed receipt",
        ));
    }
    Ok(receipt)
}
/// Validate exact cached geometry and identity allocations, never regenerate IDs.
pub fn validate_recorded(proposal: &Proposal) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}

pub fn validate_current(
    context: &NativeWriteContext<'_>,
    proposal: &Proposal,
) -> Result<(), EngineError> {
    let receipt = recorded(proposal)?;
    if proposal.project_id != context.model.project.project_id
        || proposal.prepared_against != context.model.model_revision
        || proposal.batch.expected_model_revision.as_ref() != Some(&context.model.model_revision)
    {
        return Err(invalid("Design changed; prepare the design edit again"));
    }
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Design edit source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Design edit source unavailable"))?;
    require_unique_occurrence(source, receipt.sheet, &receipt.instance_path)?;
    if source.sheets.get(&receipt.sheet) != Some(&receipt.before) {
        return Err(invalid("Reviewed design edit source changed"));
    }
    match &receipt.edit {
        DesignEditIntent::TransformExisting {
            targets, reference, ..
        } => {
            let current = DesignClipboardBundle::capture(
                context.model,
                receipt.sheet,
                &receipt.instance_path,
                targets,
                *reference,
            )?;
            if receipt.transform_source.as_ref() != Some(&current) {
                return Err(invalid("Reviewed raw transform source changed"));
            }
            // Recheck full-source dependencies/locks, but retain the exact
            // immutable operations and already recorded identity allocation.
            design_clipboard_edit::build(
                context,
                WriteProvenance::new(
                    "design-edit-admission",
                    crate::substrate::CommitSource::Manual,
                    REASON,
                ),
                receipt.sheet,
                &receipt.instance_path,
                &receipt.edit,
            )?;
        }
        DesignEditIntent::Cut { targets } => {
            let current = DesignClipboardBundle::capture(
                context.model,
                receipt.sheet,
                &receipt.instance_path,
                targets,
                Point::zero(),
            )?;
            if receipt.cut_source.as_ref() != Some(&current) {
                return Err(invalid("Reviewed raw Cut source changed"));
            }
            // Admission only: reuse the complete deletion dependency census,
            // including foreign-sheet Bus Entries. Never replace cached ops or
            // recompose their immutable electrical nomination.
            design_clipboard_edit::build(
                context,
                WriteProvenance::new(
                    "design-edit-admission",
                    crate::substrate::CommitSource::Manual,
                    REASON,
                ),
                receipt.sheet,
                &receipt.instance_path,
                &receipt.edit,
            )?;
        }
        DesignEditIntent::PlaceBundle { .. } => {
            let ids = proposal
                .batch
                .operations
                .iter()
                .filter_map(|op| match op {
                    Operation::CreateSchematicWire { wire_id, .. } => Some(*wire_id),
                    Operation::PlaceSchematicMarker { marker_id, .. } => Some(*marker_id),
                    _ => None,
                })
                .collect();
            validate_created_ids(context, proposal, &ids)?;
        }
    }
    // The canonical Proposal apply path still validates immutable PM053
    // before/final nomination; receipt replay never grants semantic identity.
    Ok(())
}
/// Restore pre-draft freshness after the gateway persists its exact own cache.
/// Only canonical metadata bodies of this same Draft/Accepted record are
/// excluded. Actual design operations, foreign metadata and arbitrary raw
/// references remain in the full recursive identity census.
fn validate_created_ids(
    context: &NativeWriteContext<'_>,
    proposal: &Proposal,
    ids: &BTreeSet<Uuid>,
) -> Result<(), EngineError> {
    if context.model.proposals.get(&proposal.proposal_id) != Some(proposal)
        || !matches!(
            proposal.status,
            ProposalStatus::Draft | ProposalStatus::Accepted
        )
        || proposal.applied_transaction_id.is_some()
    {
        return Err(invalid(
            "Reviewed paste is not the current unapplied Proposal",
        ));
    }
    let own = serde_json::to_value(proposal)?;
    let mut draft = proposal.clone();
    draft.status = ProposalStatus::Draft;
    let mut accepted = proposal.clone();
    accepted.status = ProposalStatus::Accepted;
    let allowed = [
        serde_json::to_value(draft)?,
        serde_json::to_value(accepted)?,
    ];
    let own_id = proposal.proposal_id.to_string();
    let path = format!(".datum/proposals/{own_id}.json");
    let mut model = serde_json::to_value(context.model)?;
    if let Some(records) = model.get_mut("proposals").and_then(Value::as_object_mut) {
        if records.get(&own_id) == Some(&own) {
            records.remove(&own_id);
        }
    }
    if let Some(journal) = model.get_mut("journal").and_then(Value::as_array_mut) {
        for transaction in journal {
            for key in ["operations", "inverse_operations"] {
                if let Some(ops) = transaction.get_mut(key).and_then(Value::as_array_mut) {
                    for op in ops {
                        if matches!(
                            op.get("kind").and_then(Value::as_str),
                            Some(
                                "create_proposal_metadata"
                                    | "set_proposal_metadata"
                                    | "delete_proposal_metadata"
                            )
                        ) && op.get("proposal_id").and_then(Value::as_str)
                            == Some(own_id.as_str())
                            && op.get("relative_path").and_then(Value::as_str)
                                == Some(path.as_str())
                        {
                            for field in ["proposal", "previous_proposal"] {
                                if let Some(body) = op.get_mut(field) {
                                    if allowed.contains(body) {
                                        *body = Value::Null;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut values = vec![model];
    for shard in &context.model.source_shards {
        if shard.relative_path.ends_with(".json") {
            let value = context
                .model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            if shard.relative_path != path || value != own {
                values.push(value);
            }
        }
    }
    for id in ids {
        if id.is_nil()
            || values.iter().any(|value| {
                super::schematic_wire_edit_adapter::contains_id(value, &id.to_string())
            })
        {
            return Err(invalid(
                "Reviewed paste identity already exists or is externally referenced",
            ));
        }
    }
    Ok(())
}

fn electrical(op: &Operation) -> bool {
    matches!(
        op,
        Operation::CreateElectricalIdentity { .. }
            | Operation::SetElectricalIdentity { .. }
            | Operation::DeleteElectricalIdentity { .. }
    )
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
