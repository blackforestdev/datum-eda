use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    CommitDiff, CommitProvenance, CommitReport, CommitSource, DesignModel, ModelRevision, ObjectId,
    Operation, OperationBatch, ResolveDiagnostic, SourceShardDirtyState, SourceShardKind,
    SourceShardRef, read_json_value, sha256_hex, sort_source_shards,
    source_shard_authority_for_kind, stage_operation_shard_writes, update_staged_source_hashes,
};
use crate::error::EngineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Accepted,
    Deferred,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSource {
    Manual,
    Cli,
    Tool,
    Assistant,
    Check,
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRef {
    pub proposal_id: Uuid,
    pub prepared_against: ModelRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub schema_version: u64,
    pub proposal_id: Uuid,
    pub project_id: Uuid,
    pub prepared_against: ModelRevision,
    pub batch: OperationBatch,
    pub rationale: String,
    pub affected_objects: Vec<ObjectId>,
    pub checks_run: Vec<Uuid>,
    pub finding_fingerprints: Vec<String>,
    pub source: ProposalSource,
    pub status: ProposalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_transaction_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalApplyBlocker {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalApplyValidation {
    pub proposal_id: Uuid,
    pub status: ProposalStatus,
    pub prepared_against: ModelRevision,
    pub current_model_revision: ModelRevision,
    pub prepared_against_current_model: bool,
    pub batch_revision_guard_matches: bool,
    pub can_apply: bool,
    pub blockers: Vec<ProposalApplyBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalPreview {
    pub proposal_id: Uuid,
    pub prepared_against: ModelRevision,
    pub current_model_revision: ModelRevision,
    pub preview_after_model_revision: ModelRevision,
    pub diff: CommitDiff,
    pub affected_objects: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalCreateRequest {
    pub proposal_id: Option<Uuid>,
    pub batch: OperationBatch,
    pub rationale: String,
    pub source: ProposalSource,
    pub checks_run: Vec<Uuid>,
    pub finding_fingerprints: Vec<String>,
}

pub(super) fn read_proposal_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, Proposal>,
    Vec<ResolveDiagnostic>,
) {
    let proposal_dir = project_root.join(".datum/proposals");
    let mut shards = Vec::new();
    let mut proposals = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&proposal_dir) else {
        return (shards, proposals, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/proposals/{filename}");
        let path = project_root.join(&relative_path);
        match read_proposal_shard(path, relative_path) {
            Ok((shard, proposal)) => {
                proposals.insert(proposal.proposal_id, proposal);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, proposals, diagnostics)
}

pub fn commit_proposal_metadata_journaled(
    model: &mut DesignModel,
    project_root: &Path,
    proposal: Proposal,
) -> Result<Proposal, EngineError> {
    if proposal.project_id != model.project.project_id {
        return Err(EngineError::Validation(format!(
            "proposal {} project_id does not match project",
            proposal.proposal_id
        )));
    }
    if proposal.prepared_against != model.model_revision {
        return Err(EngineError::Validation(format!(
            "proposal {} prepared against {}, current {}",
            proposal.proposal_id, proposal.prepared_against.0, model.model_revision.0
        )));
    }
    if proposal.batch.expected_model_revision.as_ref() != Some(&proposal.prepared_against) {
        return Err(EngineError::Validation(format!(
            "proposal {} batch does not carry the prepared model revision guard",
            proposal.proposal_id
        )));
    }
    if model.proposals.contains_key(&proposal.proposal_id) {
        return Err(EngineError::Validation(format!(
            "proposal {} already exists",
            proposal.proposal_id
        )));
    }
    validate_proposal_source_policy(model, &proposal)?;
    let proposal_id = proposal.proposal_id;
    let source = proposal.source;
    commit_proposal_metadata(
        model,
        project_root,
        Operation::CreateProposalMetadata {
            proposal_id,
            relative_path: proposal_relative_path(proposal_id),
            proposal: proposal_value(&proposal)?,
        },
        source,
        format!("create proposal {proposal_id}"),
    )?;
    Ok(model
        .proposals
        .get(&proposal_id)
        .cloned()
        .unwrap_or(proposal))
}

pub fn create_draft_proposal_from_batch(
    model: &mut DesignModel,
    project_root: &Path,
    request: ProposalCreateRequest,
) -> Result<Proposal, EngineError> {
    let mut batch = request.batch;
    let prepared_against = model.model_revision.clone();
    if let Some(expected) = &batch.expected_model_revision {
        if expected != &prepared_against {
            return Err(EngineError::Operation(format!(
                "proposal batch revision guard mismatch: expected {}, current {}",
                expected.0, prepared_against.0
            )));
        }
    } else {
        batch.expected_model_revision = Some(prepared_against.clone());
    }

    let mut preview = model.clone();
    let preview_report = preview.commit(batch.clone())?;
    let mut affected_objects = preview_report.transaction.diff.created;
    affected_objects.extend(preview_report.transaction.diff.modified);
    affected_objects.extend(preview_report.transaction.diff.deleted);
    affected_objects.sort();
    affected_objects.dedup();

    let proposal_id = request.proposal_id.unwrap_or_else(Uuid::new_v4);
    if model.proposals.contains_key(&proposal_id) {
        return Err(EngineError::Validation(format!(
            "proposal {proposal_id} already exists"
        )));
    }
    let proposal = Proposal {
        schema_version: 1,
        proposal_id,
        project_id: model.project.project_id,
        prepared_against,
        batch,
        rationale: request.rationale,
        affected_objects,
        checks_run: request.checks_run,
        finding_fingerprints: request.finding_fingerprints,
        source: request.source,
        status: ProposalStatus::Draft,
        applied_transaction_id: None,
    };
    validate_proposal_source_policy(model, &proposal)?;
    commit_proposal_metadata(
        model,
        project_root,
        Operation::CreateProposalMetadata {
            proposal_id,
            relative_path: proposal_relative_path(proposal_id),
            proposal: proposal_value(&proposal)?,
        },
        request.source,
        format!("create proposal {proposal_id}"),
    )?;
    Ok(proposal)
}

pub fn apply_accepted_proposal(
    model: &mut DesignModel,
    project_root: &Path,
    proposal_id: Uuid,
) -> Result<CommitReport, EngineError> {
    let validation = validate_proposal_apply(model, proposal_id)?;
    if !validation.can_apply {
        return Err(EngineError::Validation(format!(
            "proposal {proposal_id} is not applyable: {}",
            validation
                .blockers
                .iter()
                .map(|blocker| format!("{}: {}", blocker.code, blocker.message))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }
    let proposal = model
        .proposals
        .get(&proposal_id)
        .cloned()
        .ok_or_else(|| EngineError::Validation(format!("proposal {proposal_id} not found")))?;

    let predicted_transaction_id =
        predict_journaled_transaction_id(model, project_root, &proposal.batch)?;
    let mut applied = proposal.clone();
    applied.status = ProposalStatus::Applied;
    applied.applied_transaction_id = Some(predicted_transaction_id);
    let mut batch = proposal.batch.clone();
    batch.operations.push(Operation::SetProposalMetadata {
        proposal_id,
        relative_path: proposal_relative_path(proposal_id),
        previous_proposal: proposal_value(&proposal)?,
        proposal: proposal_value(&applied)?,
    });
    let report = model.commit_journaled(project_root, batch)?;
    if report.transaction.transaction_id != predicted_transaction_id {
        return Err(EngineError::Operation(format!(
            "proposal {proposal_id} predicted transaction id {predicted_transaction_id} but committed {}",
            report.transaction.transaction_id
        )));
    }
    Ok(report)
}

pub fn validate_proposal_apply(
    model: &DesignModel,
    proposal_id: Uuid,
) -> Result<ProposalApplyValidation, EngineError> {
    let proposal = model
        .proposals
        .get(&proposal_id)
        .ok_or_else(|| EngineError::Validation(format!("proposal {proposal_id} not found")))?;
    let prepared_against_current_model = proposal.prepared_against == model.model_revision;
    let batch_revision_guard_matches =
        proposal.batch.expected_model_revision.as_ref() == Some(&proposal.prepared_against);
    let mut blockers = Vec::new();
    if proposal.status != ProposalStatus::Accepted {
        blockers.push(ProposalApplyBlocker {
            code: "missing_acceptance".to_string(),
            message: format!(
                "proposal status is {:?}; expected accepted before apply",
                proposal.status
            ),
        });
    }
    if !prepared_against_current_model {
        blockers.push(ProposalApplyBlocker {
            code: "stale_model_revision".to_string(),
            message: format!(
                "proposal prepared against {}, current {}",
                proposal.prepared_against.0, model.model_revision.0
            ),
        });
    }
    if !batch_revision_guard_matches {
        blockers.push(ProposalApplyBlocker {
            code: "missing_revision_guard".to_string(),
            message: "proposal batch does not carry the prepared model revision guard".to_string(),
        });
    }
    blockers.extend(proposal_source_policy_blockers(model, proposal));
    Ok(ProposalApplyValidation {
        proposal_id,
        status: proposal.status,
        prepared_against: proposal.prepared_against.clone(),
        current_model_revision: model.model_revision.clone(),
        prepared_against_current_model,
        batch_revision_guard_matches,
        can_apply: blockers.is_empty(),
        blockers,
    })
}

pub fn preview_proposal_diff(
    model: &DesignModel,
    proposal_id: Uuid,
) -> Result<ProposalPreview, EngineError> {
    let proposal = model
        .proposals
        .get(&proposal_id)
        .ok_or_else(|| EngineError::Validation(format!("proposal {proposal_id} not found")))?;
    let mut preview = model.clone();
    let report = preview.commit(proposal.batch.clone())?;
    let mut affected_objects = report.transaction.diff.created.clone();
    affected_objects.extend(report.transaction.diff.modified.clone());
    affected_objects.extend(report.transaction.diff.deleted.clone());
    affected_objects.sort();
    affected_objects.dedup();
    Ok(ProposalPreview {
        proposal_id,
        prepared_against: proposal.prepared_against.clone(),
        current_model_revision: model.model_revision.clone(),
        preview_after_model_revision: report.transaction.after_model_revision,
        diff: report.transaction.diff,
        affected_objects,
    })
}

pub fn review_proposal_status(
    model: &mut DesignModel,
    project_root: &Path,
    proposal_id: Uuid,
    status: ProposalStatus,
) -> Result<Proposal, EngineError> {
    if !matches!(
        status,
        ProposalStatus::Accepted | ProposalStatus::Deferred | ProposalStatus::Rejected
    ) {
        return Err(EngineError::Validation(format!(
            "proposal review status must be accepted, deferred, or rejected, got {status:?}"
        )));
    }
    let mut proposal = model
        .proposals
        .get(&proposal_id)
        .cloned()
        .ok_or_else(|| EngineError::Validation(format!("proposal {proposal_id} not found")))?;
    if proposal.status != ProposalStatus::Draft {
        return Err(EngineError::Validation(format!(
            "proposal {proposal_id} has status {:?}; expected draft",
            proposal.status
        )));
    }
    if status == ProposalStatus::Accepted {
        validate_proposal_acceptance_policy(model, &proposal)?;
    }
    let previous = proposal.clone();
    proposal.status = status;
    commit_proposal_metadata(
        model,
        project_root,
        Operation::SetProposalMetadata {
            proposal_id,
            relative_path: proposal_relative_path(proposal_id),
            previous_proposal: proposal_value(&previous)?,
            proposal: proposal_value(&proposal)?,
        },
        proposal.source,
        format!("review proposal {proposal_id} as {status:?}"),
    )?;
    Ok(proposal)
}

fn commit_proposal_metadata(
    model: &mut DesignModel,
    project_root: &Path,
    operation: Operation,
    source: ProposalSource,
    reason: String,
) -> Result<CommitReport, EngineError> {
    model.commit_journaled(
        project_root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: commit_source_for_proposal_source(source),
                reason,
            },
            operations: vec![operation],
        },
    )
}

fn commit_source_for_proposal_source(source: ProposalSource) -> CommitSource {
    match source {
        ProposalSource::Manual => CommitSource::Manual,
        ProposalSource::Cli => CommitSource::Cli,
        ProposalSource::Tool | ProposalSource::Check | ProposalSource::Import => CommitSource::Tool,
        ProposalSource::Assistant => CommitSource::Assistant,
    }
}

fn proposal_relative_path(proposal_id: Uuid) -> String {
    format!(".datum/proposals/{proposal_id}.json")
}

fn proposal_value(proposal: &Proposal) -> Result<serde_json::Value, EngineError> {
    serde_json::to_value(proposal).map_err(EngineError::from)
}

fn predict_journaled_transaction_id(
    model: &DesignModel,
    project_root: &Path,
    batch: &OperationBatch,
) -> Result<Uuid, EngineError> {
    let staged_writes = stage_operation_shard_writes(project_root, model, batch)?;
    let mut committed = model.clone();
    update_staged_source_hashes(&mut committed.source_shards, &staged_writes);
    sort_source_shards(&mut committed.source_shards);
    let report = committed.commit(batch.clone());
    cleanup_stage_dir(project_root, batch.batch_id);
    Ok(report?.transaction.transaction_id)
}

fn cleanup_stage_dir(project_root: &Path, batch_id: Uuid) {
    let stage_dir = project_root.join(".datum/stage").join(batch_id.to_string());
    match std::fs::remove_dir_all(&stage_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {}
    }
}

fn validate_proposal_acceptance_policy(
    model: &DesignModel,
    proposal: &Proposal,
) -> Result<(), EngineError> {
    if proposal.prepared_against != model.model_revision {
        return Err(EngineError::Operation(format!(
            "proposal {} cannot be accepted: prepared against {}, current {}",
            proposal.proposal_id, proposal.prepared_against.0, model.model_revision.0
        )));
    }
    if proposal.batch.expected_model_revision.as_ref() != Some(&proposal.prepared_against) {
        return Err(EngineError::Validation(format!(
            "proposal {} cannot be accepted: batch does not carry the prepared model revision guard",
            proposal.proposal_id
        )));
    }
    validate_proposal_source_policy(model, proposal)?;
    Ok(())
}

fn validate_proposal_source_policy(
    model: &DesignModel,
    proposal: &Proposal,
) -> Result<(), EngineError> {
    let blockers = proposal_source_policy_blockers(model, proposal);
    if blockers.is_empty() {
        return Ok(());
    }
    Err(EngineError::Validation(format!(
        "proposal {} violates source policy: {}",
        proposal.proposal_id,
        blockers
            .iter()
            .map(|blocker| format!("{}: {}", blocker.code, blocker.message))
            .collect::<Vec<_>>()
            .join("; ")
    )))
}

fn proposal_source_policy_blockers(
    model: &DesignModel,
    proposal: &Proposal,
) -> Vec<ProposalApplyBlocker> {
    let mut blockers = Vec::new();
    if proposal.source == ProposalSource::Check {
        if proposal.checks_run.is_empty() {
            blockers.push(ProposalApplyBlocker {
                code: "missing_check_evidence".to_string(),
                message: "check-authored proposals must reference at least one CheckRun"
                    .to_string(),
            });
        }
        if proposal.finding_fingerprints.is_empty() {
            blockers.push(ProposalApplyBlocker {
                code: "missing_finding_fingerprint".to_string(),
                message:
                    "check-authored proposals must reference at least one CheckFinding fingerprint"
                        .to_string(),
            });
        }
        for fingerprint in &proposal.finding_fingerprints {
            if !is_sha256_fingerprint(fingerprint) {
                blockers.push(ProposalApplyBlocker {
                    code: "invalid_finding_fingerprint".to_string(),
                    message: format!(
                        "check-authored proposal fingerprint `{fingerprint}` must be a sha256:<64 lowercase hex> value"
                    ),
                });
            }
        }
        let mut linked_fingerprints = std::collections::BTreeSet::new();
        for check_run_id in &proposal.checks_run {
            let Some(check_run) = model.check_runs.get(check_run_id) else {
                blockers.push(ProposalApplyBlocker {
                    code: "unknown_check_run".to_string(),
                    message: format!(
                        "check-authored proposal references CheckRun {check_run_id}, but it is not present in the resolved model"
                    ),
                });
                continue;
            };
            for finding in &check_run.findings {
                linked_fingerprints.insert(finding.fingerprint.as_str());
            }
        }
        for fingerprint in &proposal.finding_fingerprints {
            if is_sha256_fingerprint(fingerprint)
                && !linked_fingerprints.contains(fingerprint.as_str())
            {
                blockers.push(ProposalApplyBlocker {
                    code: "unlinked_finding_fingerprint".to_string(),
                    message: format!(
                        "check-authored proposal fingerprint `{fingerprint}` is not present in any referenced CheckRun"
                    ),
                });
            }
        }
    }
    blockers
}

fn is_sha256_fingerprint(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn read_proposal_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, Proposal), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_proposal_metadata".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_proposal_metadata".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let shard = SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: SourceShardKind::ProposalMetadata,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ProposalMetadata),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let proposal =
        serde_json::from_value::<Proposal>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_proposal_metadata".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    let expected_filename = format!("{}.json", proposal.proposal_id);
    let actual_filename = shard
        .path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if actual_filename != expected_filename {
        return Err(ResolveDiagnostic {
            code: "proposal_filename_mismatch".to_string(),
            message: format!(
                "proposal metadata filename {actual_filename} does not match embedded proposal id {}",
                proposal.proposal_id
            ),
            path: Some(shard.path.clone()),
        });
    }
    Ok((shard, proposal))
}
