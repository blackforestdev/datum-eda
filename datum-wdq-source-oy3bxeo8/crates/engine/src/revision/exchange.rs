use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    AlgorithmQualifiedDigest, AuthorityDiagnostic, AuthorityRecord, AuthorityRecordKind,
    AuthorityRef, ExactAuthorityRevision, ReleaseId,
    canonical::{canonical_bytes, digest_bytes, validate_digest},
};

pub const REV_I08_RECORD_FAMILIES: &[AuthorityRecordKind] = &[
    AuthorityRecordKind::ExternalMappingReceipt,
    AuthorityRecordKind::AdapterDivergenceObservation,
    AuthorityRecordKind::ReleaseMirrorRequest,
    AuthorityRecordKind::ReleaseMirrorResult,
    AuthorityRecordKind::AuthorityExchangeEnvelope,
    AuthorityRecordKind::AuthorityExchangeReceipt,
    AuthorityRecordKind::ExternalChangeCandidate,
    AuthorityRecordKind::CredentialOrTrustEvent,
    AuthorityRecordKind::TrustedTimestampEvidence,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingDirection {
    Exported,
    Observed,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalObjectIdentity {
    pub algorithm: String,
    pub encoding: String,
    pub object_kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalMappingReceiptData {
    pub direction: MappingDirection,
    pub datum_ref: AuthorityRef,
    pub datum_digest: AlgorithmQualifiedDigest,
    pub external_objects: Vec<ExternalObjectIdentity>,
    pub repository_identity: String,
    pub adapter_version: String,
    pub observed_at: i64,
    pub verification_result: String,
    pub prior_receipt: Option<AuthorityRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterDivergence {
    AdapterAbsent,
    Unmapped,
    InSync,
    DatumAhead,
    ExternalAhead,
    Diverged,
    WorktreeDirty,
    MissingPrerequisite,
    RefMovedOrDeleted,
    ObjectInvalid,
    AdapterUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterDivergenceObservationData {
    pub adapter_identity: String,
    pub datum_ref: Option<AuthorityRef>,
    pub observed_objects: Vec<ExternalObjectIdentity>,
    pub divergence: AdapterDivergence,
    pub observed_at: i64,
    pub prior_observation: Option<AuthorityRef>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseMirrorRequestData {
    pub release: ReleaseId,
    pub release_digest: AlgorithmQualifiedDigest,
    pub adapter_identity: String,
    pub destination: String,
    pub idempotency_key: Uuid,
    pub requested_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum ReleaseMirrorOutcome {
    Succeeded {
        external_objects: Vec<ExternalObjectIdentity>,
    },
    Failed {
        reason_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseMirrorResultData {
    pub request: AuthorityRef,
    pub release: ReleaseId,
    pub release_digest: AlgorithmQualifiedDigest,
    pub attempt: u32,
    pub completed_at: i64,
    pub outcome: ReleaseMirrorOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeCompleteness {
    Full,
    Incremental,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeManifestEntry {
    pub logical_name: String,
    pub digest: AlgorithmQualifiedDigest,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityExchangeEnvelopeData {
    pub exchange_format_version: u64,
    pub exchange_schema_version: u64,
    pub exchange_id: Uuid,
    pub source_project: Uuid,
    pub source_replica: String,
    pub intended_destination: Option<String>,
    pub created_order: u64,
    pub asserted_time: Option<i64>,
    pub completeness: ExchangeCompleteness,
    pub base_configuration_refs: Vec<ExactAuthorityRevision>,
    pub manifest: Vec<ExchangeManifestEntry>,
    pub prerequisite_digests: Vec<AlgorithmQualifiedDigest>,
    pub policy_and_trust_context_refs: Vec<AuthorityRef>,
    pub canonical_payload_digest: AlgorithmQualifiedDigest,
    pub signature_evidence: Vec<AuthorityRef>,
    pub challenge_nonce: Option<String>,
    pub confidentiality_and_egress_metadata: Vec<String>,
    pub optional_transport_payloads: Vec<ExchangeManifestEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeDisposition {
    Quarantined,
    Verified,
    MissingPrerequisite,
    ReplayRefused,
    DestinationRefused,
    Corrupt,
    UnknownSchema,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityExchangeReceiptData {
    pub envelope: AuthorityRef,
    pub exchange_id: Uuid,
    pub source_project: Uuid,
    pub destination_project: Uuid,
    pub media_or_channel: String,
    pub observed_at: i64,
    pub integrity_valid: bool,
    pub signature_valid: Option<bool>,
    pub signer_authorized: Option<bool>,
    pub missing_prerequisites: Vec<AlgorithmQualifiedDigest>,
    pub disposition: ExchangeDisposition,
    pub resulting_transactions: Vec<String>,
    pub retained_payload_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCandidateDisposition {
    Quarantined,
    IntegrityVerified,
    Compatible,
    Resolved,
    Compared,
    Validated,
    Translated,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalChangeCandidateData {
    pub source_kind: String,
    pub source_receipt: AuthorityRef,
    pub claimed_project_id: Uuid,
    pub claimed_base: Option<AuthorityRef>,
    pub received_payload_digest: AlgorithmQualifiedDigest,
    pub resolver_findings: Vec<String>,
    pub migration_findings: Vec<String>,
    pub semantic_delta: Option<AuthorityRef>,
    pub textual_conflicts: Vec<String>,
    pub semantic_conflicts: Vec<String>,
    pub signature_valid: Option<bool>,
    pub signer_authorized: Option<bool>,
    pub proposed_operations: Vec<String>,
    pub affected_authority_refs: Vec<AuthorityRef>,
    pub disposition: ExternalCandidateDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustEventKind {
    CredentialRegistered,
    CredentialRevoked,
    CredentialCompromised,
    TrustRootAdded,
    TrustRootRemoved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialOrTrustEventData {
    pub event_kind: TrustEventKind,
    pub actor_or_root: AuthorityRef,
    pub method_class: String,
    pub asserted_at: i64,
    pub evidence: Vec<AuthorityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedTimestampEvidenceData {
    pub subject: AuthorityRef,
    pub subject_digest: AlgorithmQualifiedDigest,
    pub method_class: String,
    pub asserted_time: i64,
    pub observed_at: i64,
    pub evidence_digest: AlgorithmQualifiedDigest,
}

pub fn verify_exchange(
    envelope_ref: AuthorityRef,
    envelope: &AuthorityExchangeEnvelopeData,
    destination_project: Uuid,
    destination_identity: &str,
    known_prerequisites: &BTreeSet<AlgorithmQualifiedDigest>,
    seen_exchange_ids: &BTreeSet<Uuid>,
    observed_at: i64,
) -> AuthorityExchangeReceiptData {
    let missing_prerequisites = envelope
        .prerequisite_digests
        .iter()
        .filter(|digest| !known_prerequisites.contains(*digest))
        .cloned()
        .collect::<Vec<_>>();
    let digest_valid =
        envelope_digest(envelope).is_ok_and(|digest| digest == envelope.canonical_payload_digest);
    let disposition =
        if envelope.exchange_schema_version != 1 || envelope.exchange_format_version != 1 {
            ExchangeDisposition::UnknownSchema
        } else if seen_exchange_ids.contains(&envelope.exchange_id) {
            ExchangeDisposition::ReplayRefused
        } else if envelope
            .intended_destination
            .as_deref()
            .is_some_and(|v| v != destination_identity)
        {
            ExchangeDisposition::DestinationRefused
        } else if !digest_valid {
            ExchangeDisposition::Corrupt
        } else if !missing_prerequisites.is_empty() {
            ExchangeDisposition::MissingPrerequisite
        } else {
            ExchangeDisposition::Verified
        };
    AuthorityExchangeReceiptData {
        envelope: envelope_ref,
        exchange_id: envelope.exchange_id,
        source_project: envelope.source_project,
        destination_project,
        media_or_channel: "offline_media".into(),
        observed_at,
        integrity_valid: digest_valid,
        signature_valid: None,
        signer_authorized: None,
        missing_prerequisites,
        disposition,
        resulting_transactions: Vec::new(),
        retained_payload_digest: envelope.canonical_payload_digest.clone(),
    }
}

pub fn envelope_digest(
    data: &AuthorityExchangeEnvelopeData,
) -> Result<AlgorithmQualifiedDigest, crate::error::EngineError> {
    let mut material = data.clone();
    material.canonical_payload_digest = AlgorithmQualifiedDigest(String::new());
    Ok(digest_bytes(&canonical_bytes(&material)?))
}

pub fn record_mirror_attempt(
    request_ref: AuthorityRef,
    request: &ReleaseMirrorRequestData,
    attempt: u32,
    completed_at: i64,
    outcome: ReleaseMirrorOutcome,
) -> ReleaseMirrorResultData {
    ReleaseMirrorResultData {
        request: request_ref,
        release: request.release,
        release_digest: request.release_digest.clone(),
        attempt,
        completed_at,
        outcome,
    }
}

pub(crate) fn validate_rev_i08_record(record: &AuthorityRecord) -> Vec<AuthorityDiagnostic> {
    let valid = match record {
        AuthorityRecord::ExternalMappingReceipt(body) => validate_mapping(&body.semantics),
        AuthorityRecord::AdapterDivergenceObservation(body) => {
            !body.semantics.adapter_identity.is_empty()
        }
        AuthorityRecord::ReleaseMirrorRequest(body) => {
            !body.semantics.adapter_identity.is_empty()
                && validate_digest(&body.semantics.release_digest).is_ok()
        }
        AuthorityRecord::ReleaseMirrorResult(body) => {
            body.semantics.attempt > 0 && validate_digest(&body.semantics.release_digest).is_ok()
        }
        AuthorityRecord::AuthorityExchangeEnvelope(body) => validate_envelope(&body.semantics),
        AuthorityRecord::AuthorityExchangeReceipt(body) => {
            validate_digest(&body.semantics.retained_payload_digest).is_ok()
        }
        AuthorityRecord::ExternalChangeCandidate(body) => {
            validate_digest(&body.semantics.received_payload_digest).is_ok()
        }
        AuthorityRecord::CredentialOrTrustEvent(body) => !body.semantics.method_class.is_empty(),
        AuthorityRecord::TrustedTimestampEvidence(body) => {
            validate_digest(&body.semantics.subject_digest).is_ok()
                && validate_digest(&body.semantics.evidence_digest).is_ok()
        }
        _ => return Vec::new(),
    };
    if valid {
        Vec::new()
    } else {
        vec![AuthorityDiagnostic {
            code: "revision_exchange_record_invalid".into(),
            message: "REV-I08 record violates transport-neutral subordinate-adapter authority"
                .into(),
        }]
    }
}

fn validate_mapping(data: &ExternalMappingReceiptData) -> bool {
    !data.repository_identity.is_empty()
        && !data.adapter_version.is_empty()
        && validate_digest(&data.datum_digest).is_ok()
        && !data.external_objects.is_empty()
        && sorted_unique(&data.external_objects)
        && data.external_objects.iter().all(|o| {
            !o.algorithm.is_empty()
                && !o.encoding.is_empty()
                && !o.object_kind.is_empty()
                && !o.value.is_empty()
        })
}

fn validate_envelope(data: &AuthorityExchangeEnvelopeData) -> bool {
    data.exchange_format_version == 1
        && data.exchange_schema_version == 1
        && !data.source_replica.is_empty()
        && !data.manifest.is_empty()
        && sorted_unique(&data.manifest)
        && sorted_unique(&data.prerequisite_digests)
        && sorted_unique(&data.policy_and_trust_context_refs)
        && sorted_unique(&data.signature_evidence)
        && sorted_unique(&data.optional_transport_payloads)
        && data.canonical_payload_digest
            == envelope_digest(data).unwrap_or_else(|_| AlgorithmQualifiedDigest(String::new()))
}

fn sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
