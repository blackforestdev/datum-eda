//! One read-only public service for revision-authority discovery and queries.
//!
//! The service owns the public inventory used by the engine, CLI, daemon, and
//! MCP verb catalog. It deliberately exposes no generic authority writer:
//! mutations remain the typed operations implemented by REV-I03 through
//! REV-I08, and Design mutation remains fenced through
//! `prepare_revision_design_commit`.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{error::EngineError, substrate::ProjectResolver};

use super::{AuthorityResolution, AuthoritySnapshot, RevisionAuthorityStore, transaction_tip};

pub const REVISION_PUBLIC_OPERATIONS: &[&str] = &[
    "accept_external_change_candidate",
    "add_or_update_affected_item",
    "adopt_library_revision",
    "adopt_project_revision_policy",
    "assign_scoped_role",
    "authorize_change",
    "begin_change_implementation",
    "cancel_change",
    "capture_dependency_snapshot",
    "close_change",
    "compare_baselines",
    "consume_project_revision_seed_snapshot",
    "create_engineering_change",
    "create_external_change_candidate",
    "create_regeneration_plan",
    "create_release_candidate",
    "create_reproduction_manifest",
    "defer_change",
    "define_controlled_document",
    "define_effectivity",
    "delegate_scoped_role",
    "evaluate_semantic_delta",
    "execute_regeneration_step",
    "expire_revision_reservation",
    "link_implementation_transaction",
    "merge_successor_work",
    "prepare_document_issue",
    "prepare_release_package",
    "reassign_successor_transactions",
    "record_adapter_divergence_observation",
    "record_approval_attestation",
    "record_credential_or_trust_event",
    "record_deviation_departure",
    "record_external_mapping_receipt",
    "record_impact_evaluation",
    "record_implementation_verification",
    "record_legacy_revision_fact_mapping",
    "record_mirror_result",
    "record_reproduction_attempt",
    "record_transmittal",
    "record_trusted_timestamp_evidence",
    "record_waiver_departure",
    "refresh_release_candidate_evaluation",
    "register_actor_identity",
    "register_revision_scheme",
    "reject_change",
    "reject_or_quarantine_external_change_candidate",
    "release_configuration",
    "release_revision_reservation",
    "request_change_rework",
    "request_release_mirror",
    "reserve_revision_label",
    "revoke_approval_attestation",
    "revoke_role_delegation",
    "revoke_scoped_role",
    "set_change_effectivity",
    "set_change_rationale_and_classification",
    "split_successor_work",
    "submit_change_for_impact_review",
    "supersede_approval_attestation",
    "supersede_effectivity",
    "supersede_project_revision_policy",
    "supersede_revision_scheme",
    "verify_authority_exchange",
    "withdraw_release_standing",
];

pub const REVISION_PUBLIC_QUERIES: &[&str] = &[
    "adapter_status",
    "approval_required",
    "approval_valid",
    "as_of_authority",
    "assess_legacy_revision_fact",
    "audit_evaluate",
    "audit_show",
    "baseline_compare",
    "baseline_members",
    "baseline_show",
    "change_affected_items",
    "change_evidence",
    "change_show",
    "configuration_current",
    "configuration_why_member",
    "document_issues",
    "effectivity_resolve",
    "evaluate_approval_policy",
    "evaluate_capability",
    "evaluate_change_transition",
    "evaluate_design_mutation_authority",
    "evaluate_reservation_availability",
    "evidence_freshness",
    "exchange_prerequisites",
    "exchange_show",
    "exchange_verify",
    "impact_explain",
    "impact_path",
    "library_uptake_preview",
    "package_members",
    "query_attestation_standing",
    "query_change_transaction_links",
    "query_effective_role_assignments",
    "query_legacy_revision_fact_mapping",
    "query_open_successor_work",
    "query_project_seed_receipt",
    "records_retention_status",
    "regeneration_plan",
    "release_compare",
    "release_readiness",
    "release_reproduce",
    "release_show",
    "release_verify",
    "reproduction_explain",
    "resolve_engineering_change",
    "resolve_project_revision_policy",
    "resolve_revision_reservation",
    "resolve_revision_scheme",
    "revision_history",
    "revision_resolve_label",
    "status_summary",
    "transmittal_show",
    "trust_path",
];

pub const REVISION_PUBLIC_REFUSALS: &[&str] = &[
    "adapter_unavailable",
    "authority_integrity_unavailable",
    "capability_denied",
    "change_not_authorized",
    "change_transition_invalid",
    "context_stale",
    "departure_invalid_or_expired",
    "dependency_graph_incomplete",
    "destination_mismatch",
    "exchange_prerequisite_missing",
    "exchange_replay",
    "external_semantic_conflict",
    "external_signature_invalid",
    "external_signature_unauthorized",
    "external_state_unrepresentable",
    "missing_byte_reproducibility",
    "missing_pinned_library",
    "missing_regeneration_prerequisite",
    "output_mismatch",
    "permission_denied",
    "producer_environment_mismatch",
    "project_mismatch",
    "required_mapping_missing",
    "required_mirror_missing",
    "revision_reservation_conflict",
    "stale_release_candidate",
    "trusted_time_unavailable",
    "unassessed_library_update",
    "unknown_query",
    "unsupported_algorithm",
];

pub const REVISION_PROPOSAL_TWINS: &[&str] = &[
    "add_or_update_affected_item",
    "adopt_library_revision",
    "adopt_project_revision_policy",
    "capture_dependency_snapshot",
    "compare_baselines",
    "create_engineering_change",
    "create_external_change_candidate",
    "create_regeneration_plan",
    "create_reproduction_manifest",
    "define_controlled_document",
    "define_effectivity",
    "evaluate_semantic_delta",
    "record_adapter_divergence_observation",
    "record_deviation_departure",
    "record_external_mapping_receipt",
    "record_impact_evaluation",
    "record_legacy_revision_fact_mapping",
    "record_reproduction_attempt",
    "record_waiver_departure",
    "register_actor_identity",
    "register_revision_scheme",
    "set_change_effectivity",
    "set_change_rationale_and_classification",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionPublicCatalog {
    pub schema_version: u64,
    pub operations: Vec<String>,
    pub queries: Vec<String>,
    pub refusals: Vec<String>,
    pub proposal_twins: Vec<String>,
    pub authority_record_families: Vec<String>,
}

pub fn revision_public_catalog() -> RevisionPublicCatalog {
    RevisionPublicCatalog {
        schema_version: 1,
        operations: strings(REVISION_PUBLIC_OPERATIONS),
        queries: strings(REVISION_PUBLIC_QUERIES),
        refusals: strings(REVISION_PUBLIC_REFUSALS),
        proposal_twins: strings(REVISION_PROPOSAL_TWINS),
        authority_record_families: super::AuthorityRecordKind::ALL
            .iter()
            .map(|kind| kind.wire_tag().to_string())
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionQueryRequest {
    pub project_id: Uuid,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of_sequence: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_model_revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionRefusalPayload {
    pub code: String,
    pub affected_identities: Vec<String>,
    pub controlling_policy_or_requirement: String,
    pub expected_facts: Vec<String>,
    pub current_facts: Vec<String>,
    pub witness: Vec<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionQueryResponse {
    pub schema_version: u64,
    pub project_id: Uuid,
    pub model_revision: String,
    pub transaction_tip: Option<Uuid>,
    pub query: String,
    pub as_of_sequence: Option<u64>,
    pub result: Option<Value>,
    pub refusal: Option<RevisionRefusalPayload>,
}

pub fn query_revision_authority(
    project_root: &Path,
    request: &RevisionQueryRequest,
) -> Result<RevisionQueryResponse, EngineError> {
    let model = ProjectResolver::new(project_root).resolve()?;
    let model_revision = model.model_revision.0.clone();
    let tip = transaction_tip(&model.journal);
    if model.project.project_id != request.project_id {
        return Ok(refused(
            request,
            model_revision,
            tip,
            "project_mismatch",
            vec![request.project_id.to_string()],
            vec![model.project.project_id.to_string()],
            "select the Project named by the request",
        ));
    }
    if let Some(expected) = &request.expected_model_revision
        && expected != &model_revision
    {
        return Ok(refused(
            request,
            model_revision,
            tip,
            "context_stale",
            vec![expected.clone()],
            vec![model.model_revision.0],
            "refresh Project context and resubmit",
        ));
    }
    if !REVISION_PUBLIC_QUERIES.contains(&request.query.as_str()) {
        return Ok(refused(
            request,
            model_revision,
            tip,
            "unknown_query",
            strings(REVISION_PUBLIC_QUERIES),
            vec![request.query.clone()],
            "choose a query from revision.catalog",
        ));
    }

    let resolution =
        RevisionAuthorityStore::new(project_root).resolve_authority(request.project_id);
    let result = match resolution {
        AuthorityResolution::Unconfigured => json!({
            "authority_state": "unmanaged",
            "records": [],
            "events": [],
        }),
        AuthorityResolution::ReadOnlyDiagnostic { diagnostics, .. } => {
            return Ok(RevisionQueryResponse {
                schema_version: 1,
                project_id: request.project_id,
                model_revision,
                transaction_tip: tip,
                query: request.query.clone(),
                as_of_sequence: request.as_of_sequence,
                result: None,
                refusal: Some(RevisionRefusalPayload {
                    code: "authority_integrity_unavailable".into(),
                    affected_identities: vec![request.project_id.to_string()],
                    controlling_policy_or_requirement: "PM-037 immutable authority integrity"
                        .into(),
                    expected_facts: vec!["verified canonical authority snapshot".into()],
                    current_facts: diagnostics.into_iter().map(|item| item.code).collect(),
                    witness: vec!["revision authority resolver".into()],
                    remediation: "repair or restore authority before querying it".into(),
                }),
            });
        }
        AuthorityResolution::Resolved { snapshot } => query_snapshot(&snapshot, request),
    };
    Ok(RevisionQueryResponse {
        schema_version: 1,
        project_id: request.project_id,
        model_revision,
        transaction_tip: tip,
        query: request.query.clone(),
        as_of_sequence: request.as_of_sequence,
        result: Some(result),
        refusal: None,
    })
}

pub fn render_revision_query_human(response: &RevisionQueryResponse) -> String {
    if let Some(refusal) = &response.refusal {
        return format!(
            "revision query refused: {}\ncurrent: {}\nremediation: {}",
            refusal.code,
            refusal.current_facts.join(", "),
            refusal.remediation
        );
    }
    let count = response
        .result
        .as_ref()
        .and_then(|value| value.get("records"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!(
        "revision query: {}\nproject: {}\nmodel revision: {}\nrecords: {}",
        response.query, response.project_id, response.model_revision, count
    )
}

fn query_snapshot(snapshot: &AuthoritySnapshot, request: &RevisionQueryRequest) -> Value {
    let records: Vec<_> = match request.as_of_sequence {
        Some(sequence) => snapshot.as_of(sequence).into_iter().cloned().collect(),
        None => snapshot.records.clone(),
    };
    let events: Vec<_> = snapshot
        .events
        .iter()
        .filter(|event| {
            request
                .as_of_sequence
                .is_none_or(|sequence| event.sequence <= sequence)
        })
        .cloned()
        .collect();
    json!({
        "authority_state": "managed",
        "records": records,
        "events": events,
    })
}

fn refused(
    request: &RevisionQueryRequest,
    model_revision: String,
    tip: Option<Uuid>,
    code: &str,
    expected: Vec<String>,
    current: Vec<String>,
    remediation: &str,
) -> RevisionQueryResponse {
    RevisionQueryResponse {
        schema_version: 1,
        project_id: request.project_id,
        model_revision,
        transaction_tip: tip,
        query: request.query.clone(),
        as_of_sequence: request.as_of_sequence,
        result: None,
        refusal: Some(RevisionRefusalPayload {
            code: code.into(),
            affected_identities: vec![request.project_id.to_string()],
            controlling_policy_or_requirement: "PM-037 public revision service".into(),
            expected_facts: expected,
            current_facts: current,
            witness: vec!["engine revision public service".into()],
            remediation: remediation.into(),
        }),
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};

    fn project(label: &str) -> (std::path::PathBuf, Uuid) {
        let root =
            std::env::temp_dir().join(format!("datum_revision_public_{label}_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let report = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Revision Public Fixture".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        (root, report.project_uuid)
    }

    #[test]
    fn public_inventories_are_sorted_unique_and_add_no_authority_family() {
        for inventory in [
            REVISION_PUBLIC_OPERATIONS,
            REVISION_PUBLIC_QUERIES,
            REVISION_PUBLIC_REFUSALS,
            REVISION_PROPOSAL_TWINS,
        ] {
            assert!(
                inventory.windows(2).all(|pair| pair[0] < pair[1]),
                "inventory must be sorted and unique"
            );
        }
        let catalog = revision_public_catalog();
        assert_eq!(
            catalog.authority_record_families.len(),
            super::super::AuthorityRecordKind::ALL.len()
        );
        assert!(
            catalog
                .proposal_twins
                .iter()
                .all(|operation| catalog.operations.contains(operation))
        );
        assert!(
            catalog
                .proposal_twins
                .iter()
                .all(|operation| !operation.contains("approval")
                    && !operation.contains("authorize")
                    && !operation.starts_with("release_"))
        );
    }

    #[test]
    fn unmanaged_query_is_prompt_free_and_stale_refusal_is_deterministic() {
        let (root, project_id) = project("unmanaged");
        let request = RevisionQueryRequest {
            project_id,
            query: "configuration_current".into(),
            as_of_sequence: None,
            expected_model_revision: None,
        };
        let response = query_revision_authority(&root, &request).unwrap();
        assert_eq!(
            response.result.as_ref().unwrap()["authority_state"],
            "unmanaged"
        );
        assert!(response.refusal.is_none());
        let stale = RevisionQueryRequest {
            expected_model_revision: Some("stale".into()),
            ..request
        };
        let left = query_revision_authority(&root, &stale).unwrap();
        let right = query_revision_authority(&root, &stale).unwrap();
        assert_eq!(left.refusal, right.refusal);
        assert_eq!(left.refusal.unwrap().code, "context_stale");
        let _ = std::fs::remove_dir_all(root);
    }
}
