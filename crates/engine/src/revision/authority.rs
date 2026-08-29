use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{AlgorithmQualifiedDigest, canonical::canonical_bytes, canonical::digest_bytes};

/// Canonical REV-I02 authority schema version.
///
/// Typed identities are deliberately not UUID-compatible substitutes:
///
/// ```compile_fail
/// use eda_engine::revision::{ConfigurationItemId, ConfigurationRefId};
/// use uuid::Uuid;
/// let item = ConfigurationItemId(Uuid::nil());
/// let _: ConfigurationRefId = item;
/// ```
pub const AUTHORITY_SCHEMA_VERSION: u64 = 1;

macro_rules! authority_families {
    ($(($variant:ident, $id:ident, $tag:literal)),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
            #[serde(transparent)]
            pub struct $id(pub Uuid);
        )+

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum AuthorityRecordKind { $($variant),+ }

        impl AuthorityRecordKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn wire_tag(self) -> &'static str {
                match self { $(Self::$variant => $tag),+ }
            }

        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(tag = "kind", content = "id", rename_all = "snake_case")]
        pub enum AuthorityRef { $($variant($id)),+ }

        impl AuthorityRef {
            pub const fn kind(self) -> AuthorityRecordKind {
                match self { $(Self::$variant(_) => AuthorityRecordKind::$variant),+ }
            }

            pub const fn uuid(self) -> Uuid {
                match self { $(Self::$variant(id) => id.0),+ }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
        pub enum AuthorityRecord { $($variant(AuthorityRecordBody<$id>)),+ }

        impl AuthorityRecord {
            pub const fn kind(&self) -> AuthorityRecordKind {
                match self { $(Self::$variant(_) => AuthorityRecordKind::$variant),+ }
            }

            pub const fn record_ref(&self) -> AuthorityRef {
                match self { $(Self::$variant(body) => AuthorityRef::$variant(body.id)),+ }
            }

            pub const fn project_id(&self) -> Uuid {
                match self { $(Self::$variant(body) => body.project_id),+ }
            }

            pub const fn schema_version(&self) -> u64 {
                match self { $(Self::$variant(body) => body.schema_version),+ }
            }

            pub fn references(&self) -> &[AuthorityRef] {
                match self { $(Self::$variant(body) => &body.references),+ }
            }

            pub fn canonical_digest(&self) -> Result<AlgorithmQualifiedDigest, EngineError> {
                Ok(digest_bytes(&canonical_bytes(self)?))
            }
        }
    };
}

authority_families!(
    (ConfigurationItem, ConfigurationItemId, "configuration_item"),
    (ConfigurationRef, ConfigurationRefId, "configuration_ref"),
    (
        ConfigurationBaseline,
        ConfigurationBaselineId,
        "configuration_baseline"
    ),
    (
        EngineeringRevision,
        EngineeringRevisionId,
        "engineering_revision"
    ),
    (
        RevisionReservation,
        RevisionReservationId,
        "revision_reservation"
    ),
    (BuildIdentity, BuildIdentityId, "build_identity"),
    (EngineeringChange, EngineeringChangeId, "engineering_change"),
    (
        ApprovalAttestation,
        ApprovalAttestationId,
        "approval_attestation"
    ),
    (Effectivity, EffectivityId, "effectivity"),
    (ReleaseCandidate, ReleaseCandidateId, "release_candidate"),
    (Release, ReleaseId, "release"),
    (
        SupersessionEstablished,
        SupersessionEstablishedId,
        "supersession_established"
    ),
    (
        AuthorizationWithdrawn,
        AuthorizationWithdrawnId,
        "authorization_withdrawn"
    ),
    (
        ObsolescenceDeclared,
        ObsolescenceDeclaredId,
        "obsolescence_declared"
    ),
    (
        ControlledDocument,
        ControlledDocumentId,
        "controlled_document"
    ),
    (DocumentIssue, DocumentIssueId, "document_issue"),
    (ReleasePackage, ReleasePackageId, "release_package"),
    (Transmittal, TransmittalId, "transmittal"),
    (StandardsProfile, StandardsProfileId, "standards_profile"),
    (
        RequirementDisposition,
        RequirementDispositionId,
        "requirement_disposition"
    ),
    (AuditEvaluation, AuditEvaluationId, "audit_evaluation"),
    (RetentionPolicy, RetentionPolicyId, "retention_policy"),
    (
        LegalOrPolicyHold,
        LegalOrPolicyHoldId,
        "legal_or_policy_hold"
    ),
    (RecordDisposition, RecordDispositionId, "record_disposition"),
    (
        DependencySnapshot,
        DependencySnapshotId,
        "dependency_snapshot"
    ),
    (SemanticDelta, SemanticDeltaId, "semantic_delta"),
    (ImpactEvaluation, ImpactEvaluationId, "impact_evaluation"),
    (
        EvidenceInputContext,
        EvidenceInputContextId,
        "evidence_input_context"
    ),
    (EvidenceFreshness, EvidenceFreshnessId, "evidence_freshness"),
    (
        LibraryUptakeCandidate,
        LibraryUptakeCandidateId,
        "library_uptake_candidate"
    ),
    (
        BaselineComparison,
        BaselineComparisonId,
        "baseline_comparison"
    ),
    (RegenerationPlan, RegenerationPlanId, "regeneration_plan"),
    (
        ReproductionManifest,
        ReproductionManifestId,
        "reproduction_manifest"
    ),
    (
        ReproductionAttempt,
        ReproductionAttemptId,
        "reproduction_attempt"
    ),
    (
        ExternalMappingReceipt,
        ExternalMappingReceiptId,
        "external_mapping_receipt"
    ),
    (
        AdapterDivergenceObservation,
        AdapterDivergenceObservationId,
        "adapter_divergence_observation"
    ),
    (
        ReleaseMirrorRequest,
        ReleaseMirrorRequestId,
        "release_mirror_request"
    ),
    (
        ReleaseMirrorResult,
        ReleaseMirrorResultId,
        "release_mirror_result"
    ),
    (
        AuthorityExchangeEnvelope,
        AuthorityExchangeEnvelopeId,
        "authority_exchange_envelope"
    ),
    (
        AuthorityExchangeReceipt,
        AuthorityExchangeReceiptId,
        "authority_exchange_receipt"
    ),
    (
        ExternalChangeCandidate,
        ExternalChangeCandidateId,
        "external_change_candidate"
    ),
    (
        CredentialOrTrustEvent,
        CredentialOrTrustEventId,
        "credential_or_trust_event"
    ),
    (
        TrustedTimestampEvidence,
        TrustedTimestampEvidenceId,
        "trusted_timestamp_evidence"
    ),
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityRecordBody<I> {
    pub schema_version: u64,
    pub id: I,
    pub project_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_locator: Option<String>,
    pub references: Vec<AuthorityRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityEventKind {
    RecordAppended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityEvent {
    pub schema_version: u64,
    pub event_id: Uuid,
    pub project_id: Uuid,
    pub sequence: u64,
    pub kind: AuthorityEventKind,
    pub record: AuthorityRef,
    pub record_digest: AlgorithmQualifiedDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_event_digest: Option<AlgorithmQualifiedDigest>,
    pub event_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityEventMaterial {
    schema_version: u64,
    event_id: Uuid,
    project_id: Uuid,
    sequence: u64,
    kind: AuthorityEventKind,
    record: AuthorityRef,
    record_digest: AlgorithmQualifiedDigest,
    previous_event_digest: Option<AlgorithmQualifiedDigest>,
}

impl AuthorityEvent {
    fn material(&self) -> AuthorityEventMaterial {
        AuthorityEventMaterial {
            schema_version: self.schema_version,
            event_id: self.event_id,
            project_id: self.project_id,
            sequence: self.sequence,
            kind: self.kind,
            record: self.record,
            record_digest: self.record_digest.clone(),
            previous_event_digest: self.previous_event_digest.clone(),
        }
    }

    pub fn verify_digest(&self) -> Result<bool, EngineError> {
        Ok(digest_bytes(&canonical_bytes(&self.material())?) == self.event_digest)
    }

    pub fn structural_append(
        project_id: Uuid,
        sequence: u64,
        record: &AuthorityRecord,
        previous_event_digest: Option<AlgorithmQualifiedDigest>,
    ) -> Self {
        let material = AuthorityEventMaterial {
            schema_version: AUTHORITY_SCHEMA_VERSION,
            event_id: Uuid::new_v4(),
            project_id,
            sequence,
            kind: AuthorityEventKind::RecordAppended,
            record: record.record_ref(),
            record_digest: record
                .canonical_digest()
                .expect("fixture record canonicalizes"),
            previous_event_digest,
        };
        let event_digest =
            digest_bytes(&canonical_bytes(&material).expect("fixture event canonicalizes"));
        Self {
            schema_version: material.schema_version,
            event_id: material.event_id,
            project_id: material.project_id,
            sequence: material.sequence,
            kind: material.kind,
            record: material.record,
            record_digest: material.record_digest,
            previous_event_digest: material.previous_event_digest,
            event_digest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpaqueAuthorityEnvelope {
    pub schema_version: u64,
    pub kind: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoritySnapshot {
    pub schema_version: u64,
    pub project_id: Uuid,
    pub records: Vec<AuthorityRecord>,
    pub events: Vec<AuthorityEvent>,
    pub opaque_records: Vec<OpaqueAuthorityEnvelope>,
    pub opaque_events: Vec<OpaqueAuthorityEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityDiagnostic {
    pub code: String,
    pub message: String,
}

impl AuthoritySnapshot {
    pub fn canonicalized(&self) -> Self {
        let mut snapshot = self.clone();
        snapshot.records.sort_by_key(AuthorityRecord::record_ref);
        for record in &mut snapshot.records {
            match record {
                AuthorityRecord::ConfigurationItem(body) => body.references.sort(),
                AuthorityRecord::ConfigurationRef(body) => body.references.sort(),
                AuthorityRecord::ConfigurationBaseline(body) => body.references.sort(),
                AuthorityRecord::EngineeringRevision(body) => body.references.sort(),
                AuthorityRecord::RevisionReservation(body) => body.references.sort(),
                AuthorityRecord::BuildIdentity(body) => body.references.sort(),
                AuthorityRecord::EngineeringChange(body) => body.references.sort(),
                AuthorityRecord::ApprovalAttestation(body) => body.references.sort(),
                AuthorityRecord::Effectivity(body) => body.references.sort(),
                AuthorityRecord::ReleaseCandidate(body) => body.references.sort(),
                AuthorityRecord::Release(body) => body.references.sort(),
                AuthorityRecord::SupersessionEstablished(body) => body.references.sort(),
                AuthorityRecord::AuthorizationWithdrawn(body) => body.references.sort(),
                AuthorityRecord::ObsolescenceDeclared(body) => body.references.sort(),
                AuthorityRecord::ControlledDocument(body) => body.references.sort(),
                AuthorityRecord::DocumentIssue(body) => body.references.sort(),
                AuthorityRecord::ReleasePackage(body) => body.references.sort(),
                AuthorityRecord::Transmittal(body) => body.references.sort(),
                AuthorityRecord::StandardsProfile(body) => body.references.sort(),
                AuthorityRecord::RequirementDisposition(body) => body.references.sort(),
                AuthorityRecord::AuditEvaluation(body) => body.references.sort(),
                AuthorityRecord::RetentionPolicy(body) => body.references.sort(),
                AuthorityRecord::LegalOrPolicyHold(body) => body.references.sort(),
                AuthorityRecord::RecordDisposition(body) => body.references.sort(),
                AuthorityRecord::DependencySnapshot(body) => body.references.sort(),
                AuthorityRecord::SemanticDelta(body) => body.references.sort(),
                AuthorityRecord::ImpactEvaluation(body) => body.references.sort(),
                AuthorityRecord::EvidenceInputContext(body) => body.references.sort(),
                AuthorityRecord::EvidenceFreshness(body) => body.references.sort(),
                AuthorityRecord::LibraryUptakeCandidate(body) => body.references.sort(),
                AuthorityRecord::BaselineComparison(body) => body.references.sort(),
                AuthorityRecord::RegenerationPlan(body) => body.references.sort(),
                AuthorityRecord::ReproductionManifest(body) => body.references.sort(),
                AuthorityRecord::ReproductionAttempt(body) => body.references.sort(),
                AuthorityRecord::ExternalMappingReceipt(body) => body.references.sort(),
                AuthorityRecord::AdapterDivergenceObservation(body) => body.references.sort(),
                AuthorityRecord::ReleaseMirrorRequest(body) => body.references.sort(),
                AuthorityRecord::ReleaseMirrorResult(body) => body.references.sort(),
                AuthorityRecord::AuthorityExchangeEnvelope(body) => body.references.sort(),
                AuthorityRecord::AuthorityExchangeReceipt(body) => body.references.sort(),
                AuthorityRecord::ExternalChangeCandidate(body) => body.references.sort(),
                AuthorityRecord::CredentialOrTrustEvent(body) => body.references.sort(),
                AuthorityRecord::TrustedTimestampEvidence(body) => body.references.sort(),
            }
        }
        snapshot.events.sort_by_key(|event| event.sequence);
        snapshot.opaque_records.sort_by(|left, right| {
            (&left.schema_version, &left.kind, &left.bytes).cmp(&(
                &right.schema_version,
                &right.kind,
                &right.bytes,
            ))
        });
        snapshot.opaque_events.sort_by(|left, right| {
            (&left.schema_version, &left.kind, &left.bytes).cmp(&(
                &right.schema_version,
                &right.kind,
                &right.bytes,
            ))
        });
        snapshot
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, EngineError> {
        canonical_bytes(&self.canonicalized())
    }

    pub fn validate(&self) -> Vec<AuthorityDiagnostic> {
        let mut diagnostics = Vec::new();
        if self.schema_version != AUTHORITY_SCHEMA_VERSION {
            diagnostics.push(diag(
                "authority_unsupported_schema",
                "unsupported authority snapshot schema",
            ));
            return diagnostics;
        }
        let mut records = BTreeMap::new();
        for record in &self.records {
            if record.schema_version() != AUTHORITY_SCHEMA_VERSION {
                diagnostics.push(diag(
                    "authority_unsupported_record_schema",
                    "unsupported authority record schema",
                ));
            }
            if record.project_id() != self.project_id {
                diagnostics.push(diag(
                    "authority_project_mismatch",
                    "authority record belongs to another Project",
                ));
            }
            let key = record.record_ref();
            match records.get(&key) {
                Some(existing) if *existing != record.canonical_digest().ok() => {
                    diagnostics.push(diag(
                        "authority_immutable_body_conflict",
                        "one typed identity has more than one immutable body",
                    ))
                }
                Some(_) => diagnostics.push(diag(
                    "authority_duplicate_identity",
                    "duplicate authority identity",
                )),
                None => {
                    records.insert(key, record.canonical_digest().ok());
                }
            }
        }
        for record in &self.records {
            for reference in record.references() {
                if *reference == record.record_ref() {
                    diagnostics.push(diag(
                        "authority_reference_cycle",
                        "an immutable authority record cannot reference itself",
                    ));
                }
                if !records.contains_key(reference) {
                    diagnostics.push(diag(
                        "authority_reference_missing_or_wrong_kind",
                        "exact typed reference does not resolve",
                    ));
                }
            }
        }
        validate_events(self, &records, &mut diagnostics);
        diagnostics
    }

    pub fn record(&self, reference: AuthorityRef) -> Option<&AuthorityRecord> {
        self.records
            .iter()
            .find(|record| record.record_ref() == reference)
    }

    pub fn records_of_kind(&self, kind: AuthorityRecordKind) -> Vec<&AuthorityRecord> {
        self.records
            .iter()
            .filter(|record| record.kind() == kind)
            .collect()
    }

    pub fn as_of(&self, sequence: u64) -> Vec<&AuthorityRecord> {
        let refs: BTreeSet<_> = self
            .events
            .iter()
            .filter(|event| event.sequence <= sequence)
            .map(|event| event.record)
            .collect();
        self.records
            .iter()
            .filter(|record| refs.contains(&record.record_ref()))
            .collect()
    }
}

fn validate_events(
    snapshot: &AuthoritySnapshot,
    records: &BTreeMap<AuthorityRef, Option<AlgorithmQualifiedDigest>>,
    diagnostics: &mut Vec<AuthorityDiagnostic>,
) {
    let mut previous = None;
    for (index, event) in snapshot.events.iter().enumerate() {
        if event.schema_version != AUTHORITY_SCHEMA_VERSION {
            diagnostics.push(diag(
                "authority_unsupported_event_schema",
                "unsupported authority event schema",
            ));
        }
        if event.project_id != snapshot.project_id || event.sequence != index as u64 {
            diagnostics.push(diag(
                "authority_event_chain_inconsistent",
                "authority event project or sequence is inconsistent",
            ));
        }
        if event.previous_event_digest != previous || !event.verify_digest().unwrap_or(false) {
            diagnostics.push(diag(
                "authority_event_chain_inconsistent",
                "authority event ancestry or digest is inconsistent",
            ));
        }
        if records.get(&event.record).and_then(Clone::clone).as_ref() != Some(&event.record_digest)
        {
            diagnostics.push(diag(
                "authority_event_record_mismatch",
                "authority event does not name the exact immutable record body",
            ));
        }
        previous = Some(event.event_digest.clone());
    }
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
