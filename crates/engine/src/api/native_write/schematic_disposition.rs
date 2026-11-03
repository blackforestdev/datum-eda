//! Qualified ERC dispositions configure the existing manual Proposal lifecycle.
use super::{
    context::{PreparedWrite, WriteProvenance},
    waivers::*,
};
use crate::{
    error::EngineError,
    schematic::{CheckDeviation, CheckDomain, CheckWaiver, WaiverTarget},
    substrate::{CheckFinding, DesignModel, ModelRevision, Operation, Proposal, SourceShardKind},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionKind {
    Waiver,
    Deviation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispositionSummary {
    pub disposition: DispositionKind,
    pub disposition_id: Uuid,
    pub check_run_id: Uuid,
    pub finding_id: Uuid,
    pub fingerprint: String,
    pub rule_id: String,
    pub message: String,
    pub rationale: String,
}

pub(super) fn build(
    model: &DesignModel,
    provenance: WriteProvenance,
    run: Uuid,
    fingerprint: &str,
    disposition: DispositionKind,
    rationale: &str,
) -> Result<PreparedWrite, EngineError> {
    let finding = finding(model, run, fingerprint, &model.model_revision)?;
    admit_active(finding, rationale)?;
    let root = root(model)?;
    admit_absent(&root, fingerprint)?;
    let schematic_id: Uuid = serde_json::from_value(root["uuid"].clone())?;
    Ok(match disposition {
        DispositionKind::Waiver => {
            build_create_schematic_waiver(
                model,
                provenance,
                &CreateSchematicWaiverRequest {
                    schematic_id,
                    domain: CheckDomain::ERC,
                    fingerprint: fingerprint.into(),
                    rationale: rationale.into(),
                    created_by: Some("datum-gui".into()),
                },
            )?
            .write
        }
        DispositionKind::Deviation => {
            build_create_schematic_deviation(
                model,
                provenance,
                &CreateSchematicDeviationRequest {
                    schematic_id,
                    domain: CheckDomain::ERC,
                    fingerprint: fingerprint.into(),
                    rationale: rationale.into(),
                    accepted_by: Some("datum-gui".into()),
                },
            )?
            .write
        }
    })
}

/// Recognize exact allowed payloads, never a broad allow-list for arbitrary writes.
/// Currentness is required before live preview/apply; terminal recovery and reject
/// still require the original qualified evidence links but may describe stale work.
pub(super) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<DispositionSummary, EngineError> {
    let [run] = proposal.checks_run.as_slice() else {
        return Err(invalid("Disposition requires one linked CheckRun"));
    };
    let [fingerprint] = proposal.finding_fingerprints.as_slice() else {
        return Err(invalid(
            "Disposition requires one linked finding fingerprint",
        ));
    };
    let finding = finding(model, *run, fingerprint, &proposal.prepared_against)?;
    let mut payload = None;
    for operation in &proposal.batch.operations {
        let next = match operation {
            Operation::GuardObjectRevision { .. } => continue,
            Operation::CreateSchematicWaiver {
                schematic_id,
                waiver_id,
                waiver,
            } => {
                let value: CheckWaiver = serde_json::from_value(waiver.clone())?;
                if value.uuid != *waiver_id
                    || value.domain != CheckDomain::ERC
                    || value.target != WaiverTarget::Fingerprint(fingerprint.clone())
                    || value.rationale != proposal.rationale
                    || value.created_by.as_deref() != Some("datum-gui")
                {
                    return Err(invalid(
                        "Waiver proposal payload and finding evidence disagree",
                    ));
                }
                (DispositionKind::Waiver, *waiver_id, *schematic_id)
            }
            Operation::CreateSchematicDeviation {
                schematic_id,
                deviation_id,
                deviation,
            } => {
                let value: CheckDeviation = serde_json::from_value(deviation.clone())?;
                if value.uuid != *deviation_id
                    || value.domain != CheckDomain::ERC
                    || value.target != WaiverTarget::Fingerprint(fingerprint.clone())
                    || value.rationale != proposal.rationale
                    || value.accepted_by.as_deref() != Some("datum-gui")
                {
                    return Err(invalid(
                        "Deviation proposal payload and finding evidence disagree",
                    ));
                }
                (DispositionKind::Deviation, *deviation_id, *schematic_id)
            }
            _ => {
                return Err(invalid(
                    "Disposition proposal contains an unsupported operation",
                ));
            }
        };
        if payload.replace(next).is_some() {
            return Err(invalid(
                "Disposition requires exactly one authored disposition",
            ));
        }
    }
    let (disposition, disposition_id, schematic_id) =
        payload.ok_or_else(|| invalid("Disposition proposal has no disposition"))?;
    let root = root(model)?;
    if root.get("uuid") != Some(&serde_json::json!(schematic_id)) {
        return Err(invalid("Disposition belongs to another schematic"));
    }
    if current {
        if model.model_revision != proposal.prepared_against
            || proposal.batch.expected_model_revision.as_ref() != Some(&proposal.prepared_against)
        {
            return Err(invalid(
                "Design changed; rerun ERC and prepare the disposition again",
            ));
        }
        admit_active(finding, &proposal.rationale)?;
        admit_absent(&root, fingerprint)?;
    }
    Ok(DispositionSummary {
        disposition,
        disposition_id,
        check_run_id: *run,
        finding_id: finding.finding_id,
        fingerprint: fingerprint.clone(),
        rule_id: finding.rule_id.clone(),
        message: finding.message.clone(),
        rationale: proposal.rationale.clone(),
    })
}
pub(super) fn is_disposition(proposal: &Proposal) -> bool {
    proposal.batch.operations.iter().any(|o| {
        matches!(
            o,
            Operation::CreateSchematicWaiver { .. } | Operation::CreateSchematicDeviation { .. }
        )
    })
}
fn finding<'a>(
    model: &'a DesignModel,
    run: Uuid,
    fingerprint: &str,
    revision: &ModelRevision,
) -> Result<&'a CheckFinding, EngineError> {
    let check = model
        .check_runs
        .get(&run)
        .ok_or_else(|| invalid("Selected CheckRun is unavailable; rerun ERC"))?;
    if check.check_run_id != run
        || check.project_id != model.project.project_id
        || check.model_revision != *revision
    {
        return Err(invalid(
            "Selected CheckRun project or source revision is stale",
        ));
    }
    let mut found = check
        .findings
        .iter()
        .filter(|f| f.fingerprint == fingerprint);
    let finding = found
        .next()
        .ok_or_else(|| invalid("Finding fingerprint is not in the selected CheckRun"))?;
    if fingerprint.is_empty() || found.next().is_some() {
        return Err(invalid("Finding fingerprint is missing or ambiguous"));
    }
    if finding.domain != "erc" {
        return Err(invalid("Only ERC findings support this disposition action"));
    }
    Ok(finding)
}
fn admit_active(finding: &CheckFinding, rationale: &str) -> Result<(), EngineError> {
    if rationale.trim().is_empty() {
        return Err(invalid("Enter a nonempty disposition rationale"));
    }
    if finding.status != "active"
        || !finding.waiver_refs.is_empty()
        || !finding.deviation_refs.is_empty()
    {
        return Err(invalid(
            "Finding already has a disposition or is not active",
        ));
    }
    Ok(())
}
fn root(model: &DesignModel) -> Result<Value, EngineError> {
    model.materialized_source_shard_value(SourceShardKind::SchematicRoot)
}
fn admit_absent(root: &Value, fingerprint: &str) -> Result<(), EngineError> {
    for key in ["waivers", "deviations"] {
        let Some(values) = root.get(key) else {
            continue;
        };
        for value in values
            .as_array()
            .ok_or_else(|| invalid("Schematic dispositions are invalid"))?
        {
            let (domain, target) = if key == "waivers" {
                let v: CheckWaiver = serde_json::from_value(value.clone())?;
                (v.domain, v.target)
            } else {
                let v: CheckDeviation = serde_json::from_value(value.clone())?;
                (v.domain, v.target)
            };
            if domain == CheckDomain::ERC && target == WaiverTarget::Fingerprint(fingerprint.into())
            {
                return Err(invalid(
                    "Finding already has a waiver or accepted deviation",
                ));
            }
        }
    }
    Ok(())
}
fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

#[cfg(test)]
#[path = "schematic_disposition_tests.rs"]
mod tests;
