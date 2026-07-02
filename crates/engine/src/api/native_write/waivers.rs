//! Check-waiver and check-deviation builders, migrated from
//! `crates/cli/src/command_project_waivers.rs` (the CLI file is now a thin
//! caller over this module).
//!
//! Both writes record a fingerprint-targeted disposition on the schematic
//! root: a [`CheckWaiver`] (`Operation::CreateSchematicWaiver`) or an
//! accepted [`CheckDeviation`] (`Operation::CreateSchematicDeviation`).
//! Disposition ids are deterministic v5 derivations (see [`super::ids`])
//! seeded by the model revision, the finding fingerprint, and the rationale;
//! the batch id is likewise deterministic (namespaced by the project id,
//! seeded by the disposition id) so re-authoring the same disposition at the
//! same revision produces a byte-identical batch.

use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::schematic::{
    CheckDeviation, CheckDomain, CheckWaiver, DeviationApprovalStatus, WaiverTarget,
};
use crate::substrate::{CommitReport, DesignModel, ObjectId, Operation};

use super::context::{PreparedWrite, WriteProvenance, build_batch, commit_prepared};
use super::ids::derive_object_id;

/// Request to waive a check finding by fingerprint on a schematic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSchematicWaiverRequest {
    /// Schematic root the waiver is recorded on.
    pub schematic_id: ObjectId,
    /// Check domain of the waived finding.
    pub domain: CheckDomain,
    /// Fingerprint of the finding being waived.
    pub fingerprint: String,
    /// Human rationale recorded on the waiver.
    pub rationale: String,
    /// Optional author identity recorded on the waiver.
    pub created_by: Option<String>,
}

/// Request to accept a check finding as a deviation by fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSchematicDeviationRequest {
    /// Schematic root the deviation is recorded on.
    pub schematic_id: ObjectId,
    /// Check domain of the accepted finding.
    pub domain: CheckDomain,
    /// Fingerprint of the finding being accepted.
    pub fingerprint: String,
    /// Human rationale recorded on the deviation.
    pub rationale: String,
    /// Optional accepting identity recorded on the deviation.
    pub accepted_by: Option<String>,
}

/// A built (uncommitted) waiver write plus its derived waiver id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSchematicWaiver {
    pub waiver_id: Uuid,
    pub write: PreparedWrite,
}

/// A built (uncommitted) deviation write plus its derived deviation id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSchematicDeviation {
    pub deviation_id: Uuid,
    pub write: PreparedWrite,
}

/// Derive the deterministic waiver id for `request` at the model's current
/// revision.
///
/// Seed layout (persistence-visible, must never drift):
/// `datum-eda:schematic-waiver:<model_revision>:<fingerprint>:<rationale>`
/// namespaced by the project id.
fn derive_schematic_waiver_id(
    model: &DesignModel,
    fingerprint: &str,
    rationale: &str,
) -> Uuid {
    derive_object_id(
        &model.project.project_id,
        "schematic-waiver",
        &[
            model.model_revision.0.clone(),
            fingerprint.to_string(),
            rationale.to_string(),
        ],
    )
}

/// Derive the deterministic deviation id for `request` at the model's current
/// revision.
///
/// Seed layout (persistence-visible, must never drift):
/// `datum-eda:schematic-deviation:<model_revision>:<fingerprint>:<rationale>`
/// namespaced by the project id.
fn derive_schematic_deviation_id(
    model: &DesignModel,
    fingerprint: &str,
    rationale: &str,
) -> Uuid {
    derive_object_id(
        &model.project.project_id,
        "schematic-deviation",
        &[
            model.model_revision.0.clone(),
            fingerprint.to_string(),
            rationale.to_string(),
        ],
    )
}

/// Deterministic batch id for a disposition write: namespaced by the project
/// id, seeded by the derived disposition id.
fn disposition_batch_id(model: &DesignModel, disposition_id: Uuid) -> Uuid {
    Uuid::new_v5(&model.project.project_id, disposition_id.as_bytes())
}

/// Build (do not commit) the batch recording a fingerprint waiver.
pub fn build_create_schematic_waiver(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &CreateSchematicWaiverRequest,
) -> Result<PreparedSchematicWaiver, EngineError> {
    let waiver_id = derive_schematic_waiver_id(model, &request.fingerprint, &request.rationale);
    let waiver = CheckWaiver {
        uuid: waiver_id,
        domain: request.domain.clone(),
        target: WaiverTarget::Fingerprint(request.fingerprint.clone()),
        rationale: request.rationale.clone(),
        created_by: request.created_by.clone(),
    };
    let waiver_payload = serde_json::to_value(&waiver)?;
    let mut batch = build_batch(
        model,
        provenance,
        vec![Operation::CreateSchematicWaiver {
            schematic_id: request.schematic_id,
            waiver_id,
            waiver: waiver_payload,
        }],
    )?;
    batch.batch_id = disposition_batch_id(model, waiver_id);
    Ok(PreparedSchematicWaiver {
        waiver_id,
        write: PreparedWrite {
            batch,
            primary_object_id: Some(waiver_id),
        },
    })
}

/// Build (do not commit) the batch recording an accepted fingerprint
/// deviation.
pub fn build_create_schematic_deviation(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &CreateSchematicDeviationRequest,
) -> Result<PreparedSchematicDeviation, EngineError> {
    let deviation_id =
        derive_schematic_deviation_id(model, &request.fingerprint, &request.rationale);
    let deviation = CheckDeviation {
        uuid: deviation_id,
        domain: request.domain.clone(),
        target: WaiverTarget::Fingerprint(request.fingerprint.clone()),
        rationale: request.rationale.clone(),
        accepted_by: request.accepted_by.clone(),
        approval_status: DeviationApprovalStatus::Accepted,
    };
    let deviation_payload = serde_json::to_value(&deviation)?;
    let mut batch = build_batch(
        model,
        provenance,
        vec![Operation::CreateSchematicDeviation {
            schematic_id: request.schematic_id,
            deviation_id,
            deviation: deviation_payload,
        }],
    )?;
    batch.batch_id = disposition_batch_id(model, deviation_id);
    Ok(PreparedSchematicDeviation {
        deviation_id,
        write: PreparedWrite {
            batch,
            primary_object_id: Some(deviation_id),
        },
    })
}

/// Build and immediately commit a fingerprint waiver through the one
/// journaled commit path. Returns the derived waiver id and the commit
/// report.
pub fn create_schematic_waiver_and_commit(
    model: &mut DesignModel,
    project_root: &Path,
    provenance: WriteProvenance,
    request: &CreateSchematicWaiverRequest,
) -> Result<(Uuid, CommitReport), EngineError> {
    let prepared = build_create_schematic_waiver(model, provenance, request)?;
    let waiver_id = prepared.waiver_id;
    let report = commit_prepared(model, project_root, prepared.write)?;
    Ok((waiver_id, report))
}

/// Build and immediately commit an accepted fingerprint deviation through the
/// one journaled commit path. Returns the derived deviation id and the commit
/// report.
pub fn create_schematic_deviation_and_commit(
    model: &mut DesignModel,
    project_root: &Path,
    provenance: WriteProvenance,
    request: &CreateSchematicDeviationRequest,
) -> Result<(Uuid, CommitReport), EngineError> {
    let prepared = build_create_schematic_deviation(model, provenance, request)?;
    let deviation_id = prepared.deviation_id;
    let report = commit_prepared(model, project_root, prepared.write)?;
    Ok((deviation_id, report))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::test_support::{temp_project_root, write_minimal_project};
    use super::*;
    use crate::substrate::{CommitSource, ProjectResolver};

    fn resolved_minimal_project(name: &str) -> (PathBuf, DesignModel, Uuid) {
        let root = temp_project_root(name);
        let project_id = Uuid::new_v4();
        let board_id = Uuid::new_v4();
        write_minimal_project(&root, project_id, board_id);
        // The shared minimal fixture predates deviations; the schematic-root
        // shard writer requires the `deviations` array, so add it here.
        let schematic_shard_path = root.join("schematic/schematic.json");
        let mut schematic_shard: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&schematic_shard_path).expect("schematic shard should read"),
        )
        .expect("schematic shard should parse");
        schematic_shard["deviations"] = serde_json::json!([]);
        std::fs::write(
            &schematic_shard_path,
            serde_json::to_string_pretty(&schematic_shard).expect("shard should serialize"),
        )
        .expect("schematic shard should write");
        let schematic_id = Uuid::new_v5(&project_id, b"schematic");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should resolve");
        (root, model, schematic_id)
    }

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn waiver_request(schematic_id: Uuid) -> CreateSchematicWaiverRequest {
        CreateSchematicWaiverRequest {
            schematic_id,
            domain: CheckDomain::ERC,
            fingerprint: "erc:unconnected:abc123".to_string(),
            rationale: "reviewed and intentionally open".to_string(),
            created_by: Some("reviewer".to_string()),
        }
    }

    fn deviation_request(schematic_id: Uuid) -> CreateSchematicDeviationRequest {
        CreateSchematicDeviationRequest {
            schematic_id,
            domain: CheckDomain::DRC,
            fingerprint: "drc:clearance:def456".to_string(),
            rationale: "accepted for prototype spin".to_string(),
            accepted_by: Some("approver".to_string()),
        }
    }

    /// The exact derivations the CLI performed before this migration
    /// (`crates/cli/src/command_project_waivers.rs`), reproduced verbatim as
    /// the parity oracle.
    fn cli_waiver_id(model: &DesignModel, fingerprint: &str, rationale: &str) -> Uuid {
        Uuid::new_v5(
            &model.project.project_id,
            format!(
                "datum-eda:schematic-waiver:{}:{}:{}",
                model.model_revision.0, fingerprint, rationale
            )
            .as_bytes(),
        )
    }

    fn cli_deviation_id(model: &DesignModel, fingerprint: &str, rationale: &str) -> Uuid {
        Uuid::new_v5(
            &model.project.project_id,
            format!(
                "datum-eda:schematic-deviation:{}:{}:{}",
                model.model_revision.0, fingerprint, rationale
            )
            .as_bytes(),
        )
    }

    #[test]
    fn build_waiver_matches_cli_authoring_exactly() {
        let (_root, model, schematic_id) = resolved_minimal_project("waiver_build");
        let request = waiver_request(schematic_id);

        let prepared = build_create_schematic_waiver(
            &model,
            test_provenance("waive check finding erc:unconnected:abc123"),
            &request,
        )
        .expect("waiver build should succeed");

        let expected_waiver_id = cli_waiver_id(&model, &request.fingerprint, &request.rationale);
        assert_eq!(prepared.waiver_id, expected_waiver_id);
        assert_eq!(prepared.write.primary_object_id, Some(expected_waiver_id));

        let batch = &prepared.write.batch;
        assert_eq!(
            batch.batch_id,
            Uuid::new_v5(&model.project.project_id, expected_waiver_id.as_bytes()),
            "batch id must stay the CLI's deterministic v5 derivation"
        );
        assert_eq!(
            batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            batch.provenance.reason,
            "waive check finding erc:unconnected:abc123"
        );

        // Creation-only write: exactly one operation, no revision guards.
        assert_eq!(batch.operations.len(), 1);
        let Operation::CreateSchematicWaiver {
            schematic_id: op_schematic_id,
            waiver_id,
            waiver,
        } = &batch.operations[0]
        else {
            panic!("expected CreateSchematicWaiver, got {:?}", batch.operations[0]);
        };
        assert_eq!(*op_schematic_id, schematic_id);
        assert_eq!(*waiver_id, expected_waiver_id);
        let payload: CheckWaiver =
            serde_json::from_value(waiver.clone()).expect("waiver payload should round-trip");
        assert_eq!(
            payload,
            CheckWaiver {
                uuid: expected_waiver_id,
                domain: CheckDomain::ERC,
                target: WaiverTarget::Fingerprint(request.fingerprint.clone()),
                rationale: request.rationale.clone(),
                created_by: request.created_by.clone(),
            }
        );
    }

    #[test]
    fn build_deviation_matches_cli_authoring_exactly() {
        let (_root, model, schematic_id) = resolved_minimal_project("deviation_build");
        let request = deviation_request(schematic_id);

        let prepared = build_create_schematic_deviation(
            &model,
            test_provenance("accept check finding deviation drc:clearance:def456"),
            &request,
        )
        .expect("deviation build should succeed");

        let expected_deviation_id =
            cli_deviation_id(&model, &request.fingerprint, &request.rationale);
        assert_eq!(prepared.deviation_id, expected_deviation_id);
        assert_eq!(
            prepared.write.primary_object_id,
            Some(expected_deviation_id)
        );

        let batch = &prepared.write.batch;
        assert_eq!(
            batch.batch_id,
            Uuid::new_v5(&model.project.project_id, expected_deviation_id.as_bytes()),
        );
        assert_eq!(
            batch.expected_model_revision,
            Some(model.model_revision.clone())
        );

        assert_eq!(batch.operations.len(), 1);
        let Operation::CreateSchematicDeviation {
            schematic_id: op_schematic_id,
            deviation_id,
            deviation,
        } = &batch.operations[0]
        else {
            panic!(
                "expected CreateSchematicDeviation, got {:?}",
                batch.operations[0]
            );
        };
        assert_eq!(*op_schematic_id, schematic_id);
        assert_eq!(*deviation_id, expected_deviation_id);
        let payload: CheckDeviation =
            serde_json::from_value(deviation.clone()).expect("deviation payload should round-trip");
        assert_eq!(
            payload,
            CheckDeviation {
                uuid: expected_deviation_id,
                domain: CheckDomain::DRC,
                target: WaiverTarget::Fingerprint(request.fingerprint.clone()),
                rationale: request.rationale.clone(),
                accepted_by: request.accepted_by.clone(),
                approval_status: DeviationApprovalStatus::Accepted,
            }
        );
    }

    #[test]
    fn waiver_and_commit_lands_through_journaled_path() {
        let (root, mut model, schematic_id) = resolved_minimal_project("waiver_commit");
        let request = waiver_request(schematic_id);
        let before = model.model_revision.clone();

        let (waiver_id, report) = create_schematic_waiver_and_commit(
            &mut model,
            &root,
            test_provenance("waive check finding erc:unconnected:abc123"),
            &request,
        )
        .expect("waiver commit should succeed");

        assert_eq!(report.transaction.before_model_revision, before);
        assert_eq!(report.transaction.after_model_revision, model.model_revision);
        assert_ne!(model.model_revision, before);
        assert!(
            model.objects.contains_key(&waiver_id),
            "committed waiver should be a resolved domain object"
        );
        assert!(
            crate::substrate::transaction_journal_path(&root).exists(),
            "journaled commit should append the transaction journal"
        );
    }

    #[test]
    fn deviation_and_commit_lands_through_journaled_path() {
        let (root, mut model, schematic_id) = resolved_minimal_project("deviation_commit");
        let request = deviation_request(schematic_id);
        let before = model.model_revision.clone();

        let (deviation_id, report) = create_schematic_deviation_and_commit(
            &mut model,
            &root,
            test_provenance("accept check finding deviation drc:clearance:def456"),
            &request,
        )
        .expect("deviation commit should succeed");

        assert_eq!(report.transaction.before_model_revision, before);
        assert_ne!(model.model_revision, before);
        assert!(
            model.objects.contains_key(&deviation_id),
            "committed deviation should be a resolved domain object"
        );
    }

    #[test]
    fn disposition_ids_are_revision_and_input_sensitive() {
        let (_root, model, schematic_id) = resolved_minimal_project("waiver_id_sensitivity");
        let request = waiver_request(schematic_id);

        let a = build_create_schematic_waiver(&model, test_provenance("r"), &request)
            .expect("build should succeed");
        let b = build_create_schematic_waiver(&model, test_provenance("r"), &request)
            .expect("build should succeed");
        assert_eq!(
            a.waiver_id, b.waiver_id,
            "same model revision + inputs must derive the same waiver id"
        );
        assert_eq!(a.write.batch.batch_id, b.write.batch.batch_id);

        let mut different = request.clone();
        different.rationale = "another rationale".to_string();
        let c = build_create_schematic_waiver(&model, test_provenance("r"), &different)
            .expect("build should succeed");
        assert_ne!(a.waiver_id, c.waiver_id);
    }
}
