use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    AlgorithmQualifiedDigest, ApprovalAttestationId, AuthorityRecordKind, AuthorityRef,
    ConfigurationBaselineData, ConfigurationBaselineId, ConfigurationItemId, ControlledDocumentId,
    DocumentIssueId, EffectivityId, EngineeringChangeId, EngineeringRevisionId,
    ProjectRevisionPolicyId, ReleaseCandidateId, ReleaseId, ReleasePackageId, RevisionSchemeId,
};

pub const REV_I06_RECORD_FAMILIES: &[AuthorityRecordKind] = &[
    AuthorityRecordKind::EngineeringRevision,
    AuthorityRecordKind::ReleaseCandidate,
    AuthorityRecordKind::Release,
    AuthorityRecordKind::SupersessionEstablished,
    AuthorityRecordKind::AuthorizationWithdrawn,
    AuthorityRecordKind::ObsolescenceDeclared,
    AuthorityRecordKind::ControlledDocument,
    AuthorityRecordKind::DocumentIssue,
    AuthorityRecordKind::ReleasePackage,
    AuthorityRecordKind::Transmittal,
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactAuthorityRevision {
    pub authority_ref: AuthorityRef,
    pub exact_technical_revision: String,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactOutput {
    pub artifact_ref: AuthorityRef,
    pub byte_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringRevisionData {
    pub configuration_item: ConfigurationItemId,
    pub revision_namespace: String,
    pub revision_scheme: RevisionSchemeId,
    pub scheme_version: u64,
    pub revision_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suitability_or_status: Option<String>,
    pub baseline: ConfigurationBaselineId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_revision: Option<EngineeringRevisionId>,
    pub governing_changes: Vec<EngineeringChangeId>,
    pub allocated_by_policy: ProjectRevisionPolicyId,
    pub issued_by_release: ReleaseId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateEvaluationContext {
    pub model_revision: String,
    pub accepted_transaction_tip: String,
    pub policy_digest: AlgorithmQualifiedDigest,
    pub source_revisions: Vec<ExactAuthorityRevision>,
    pub evidence_digests: Vec<AlgorithmQualifiedDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseReadinessFinding {
    pub code: String,
    pub subject: AuthorityRef,
    pub blocking: bool,
    pub cleared: bool,
    pub witness: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedRevisionAllocation {
    pub configuration_item: ConfigurationItemId,
    pub engineering_revision_id: EngineeringRevisionId,
    pub revision_namespace: String,
    pub revision_scheme: RevisionSchemeId,
    pub scheme_version: u64,
    pub revision_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suitability_or_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_revision: Option<EngineeringRevisionId>,
    pub governing_changes: Vec<EngineeringChangeId>,
    pub changed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reuse_existing_revision: Option<EngineeringRevisionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedDocumentIssue {
    pub document_issue_id: DocumentIssueId,
    pub controlled_document: ControlledDocumentId,
    pub engineering_revision: EngineeringRevisionId,
    pub source_revisions: Vec<ExactAuthorityRevision>,
    pub render_context_digest: AlgorithmQualifiedDigest,
    pub outputs: Vec<ExactOutput>,
    pub approvals: Vec<ApprovalAttestationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedReleasePackage {
    pub release_package_id: ReleasePackageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_number: Option<String>,
    pub name: String,
    pub purpose: String,
    pub ordered_member_refs: Vec<AuthorityRef>,
    pub exact_artifacts: Vec<ExactOutput>,
    pub package_metadata_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseCandidateData {
    pub candidate_key: Uuid,
    pub candidate_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<ReleaseCandidateId>,
    pub proposed_baseline: ConfigurationBaselineData,
    pub proposed_revision_allocations: Vec<ProposedRevisionAllocation>,
    pub governing_changes: Vec<EngineeringChangeId>,
    pub departures: Vec<AuthorityRef>,
    pub effectivity: Vec<EffectivityId>,
    pub evidence_refs: Vec<AuthorityRef>,
    pub proposed_document_issues: Vec<PreparedDocumentIssue>,
    pub proposed_packages: Vec<PreparedReleasePackage>,
    pub approval_policy: ProjectRevisionPolicyId,
    pub attestations: Vec<ApprovalAttestationId>,
    pub standards_evaluations: Vec<AuthorityRef>,
    pub readiness_findings: Vec<ReleaseReadinessFinding>,
    pub last_evaluated_context: CandidateEvaluationContext,
    pub evaluation_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_number: Option<String>,
    pub released_at: i64,
    pub release_authority_attestation: ApprovalAttestationId,
    pub source_candidate: ReleaseCandidateId,
    pub baseline: ConfigurationBaselineId,
    pub engineering_revisions: Vec<EngineeringRevisionId>,
    pub document_issues: Vec<DocumentIssueId>,
    pub release_packages: Vec<ReleasePackageId>,
    pub governing_changes: Vec<EngineeringChangeId>,
    pub departures: Vec<AuthorityRef>,
    pub effectivity: Vec<EffectivityId>,
    pub qualifying_evidence: Vec<AuthorityRef>,
    pub audit_evaluations: Vec<AuthorityRef>,
    pub policy_evaluation_digest: AlgorithmQualifiedDigest,
    pub engine_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupersessionEstablishedData {
    pub prior_revision: EngineeringRevisionId,
    pub successor_revision: EngineeringRevisionId,
    pub authority_attestation: ApprovalAttestationId,
    pub rationale: String,
    pub effectivity: Vec<EffectivityId>,
    pub established_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationWithdrawnData {
    pub subject: AuthorityRef,
    pub authority_attestation: ApprovalAttestationId,
    pub rationale: String,
    pub effectivity: Vec<EffectivityId>,
    pub withdrawn_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObsolescenceDeclaredData {
    pub subject: AuthorityRef,
    pub authority_attestation: ApprovalAttestationId,
    pub rationale: String,
    pub effectivity: Vec<EffectivityId>,
    pub declared_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledDocumentData {
    pub document_number: String,
    pub name: String,
    pub document_kind: String,
    pub source_ref: AuthorityRef,
    pub revision_scheme: RevisionSchemeId,
    pub owning_configuration_item: ConfigurationItemId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_block_definition_ref: Option<AuthorityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentIssueData {
    pub controlled_document: ControlledDocumentId,
    pub engineering_revision: EngineeringRevisionId,
    pub baseline: ConfigurationBaselineId,
    pub source_revisions: Vec<ExactAuthorityRevision>,
    pub render_context_digest: AlgorithmQualifiedDigest,
    pub outputs: Vec<ExactOutput>,
    pub approvals: Vec<ApprovalAttestationId>,
    pub issued_by_release: ReleaseId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePackageData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_number: Option<String>,
    pub name: String,
    pub purpose: String,
    pub release: ReleaseId,
    pub ordered_member_refs: Vec<AuthorityRef>,
    pub exact_artifacts: Vec<ExactOutput>,
    pub package_metadata_digest: AlgorithmQualifiedDigest,
    pub manifest_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransmittalData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmittal_number: Option<String>,
    pub release_package: ReleasePackageId,
    pub exact_package_digest: AlgorithmQualifiedDigest,
    pub sender: String,
    pub recipients: Vec<String>,
    pub delivery_policy_ref: AuthorityRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification_egress_context: Option<String>,
    pub sent_at: i64,
    pub delivery_evidence: Vec<ExactOutput>,
    pub receipt_evidence: Vec<ExactOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBlockRevisionProjection {
    pub controlled_document: ControlledDocumentId,
    pub document_number: String,
    pub configuration_item: ConfigurationItemId,
    pub revision_label: String,
    pub suitability_or_status: Option<String>,
    pub baseline: ConfigurationBaselineId,
    pub release: ReleaseId,
}
