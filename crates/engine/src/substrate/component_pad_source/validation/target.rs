//! Mixed library/physical batches require exact producer recomputation. This
//! replaces accepted-only source checks with qualified target checks, not a skip.
use super::*;
use crate::api::native_write::{
    WriteProvenance, board_components::pad_materialization::plan_geometry,
};

pub(super) fn validate(
    model: &DesignModel,
    root: &Path,
    batch: &OperationBatch,
) -> Result<(), EngineError> {
    let target = crate::substrate::library_asset_journal_ops::planned_physical_sources(
        root,
        model,
        batch,
        crate::substrate::library_asset_resolver::server_policy()?,
    )?;
    let provenance = WriteProvenance::new(
        batch.provenance.actor.clone(),
        batch.provenance.source.clone(),
        batch.provenance.reason.clone(),
    );
    let plan = plan_geometry(model, provenance, &target)?;
    if plan.intent.placements.is_empty() {
        return Err(invalid(
            "physical batch has no qualified affected library users",
        ));
    }
    let users: BTreeSet<_> = plan
        .intent
        .placements
        .iter()
        .map(|placement| placement.component_instance)
        .collect();
    let mut actual = Vec::new();
    for op in &batch.operations {
        match op {
            Operation::SetBoardPackagePackage { .. } | Operation::SetElectricalIdentity { .. } => {
                actual.push(op.clone())
            }
            Operation::SetComponentInstance {
                component_instance_id,
                component_instance,
                ..
            } => {
                if users.contains(component_instance_id) {
                    actual.push(op.clone());
                } else if target.component(*component_instance_id) != Some(component_instance) {
                    return Err(invalid(
                        "geometry batch changes an unrelated component beyond exact library pins",
                    ));
                }
            }
            _ => {}
        }
    }
    let encode = |operations: &[Operation]| -> Result<Vec<String>, EngineError> {
        let mut values = operations
            .iter()
            .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
            .map(crate::ir::serialization::to_json_deterministic)
            .collect::<Result<Vec<_>, _>>()?;
        values.sort();
        Ok(values)
    };
    if encode(&actual)? != encode(&plan.operations)? {
        return Err(invalid(
            "physical library batch differs from the complete same-pad producer plan",
        ));
    }
    crate::substrate::library_asset_journal_ops::qualify_physical_plan(
        model,
        &batch
            .operations
            .iter()
            .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}
