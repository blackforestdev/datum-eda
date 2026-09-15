use serde::{Deserialize, Serialize};

use crate::substrate::{ObjectId, ObjectRevision};

use super::{
    AlgorithmQualifiedDigest, AuthorityRecordKind, AuthorityRef, ConfigurationBaselineId,
    DependencySnapshotId, EngineeringChangeId, EvidenceInputContextId, ImpactEvaluationId,
    SemanticDeltaId,
};

pub const REV_I05_RECORD_FAMILIES: &[AuthorityRecordKind] = &[
    AuthorityRecordKind::ConfigurationBaseline,
    AuthorityRecordKind::DependencySnapshot,
    AuthorityRecordKind::SemanticDelta,
    AuthorityRecordKind::ImpactEvaluation,
    AuthorityRecordKind::EvidenceInputContext,
    AuthorityRecordKind::EvidenceFreshness,
    AuthorityRecordKind::LibraryUptakeCandidate,
    AuthorityRecordKind::BaselineComparison,
    AuthorityRecordKind::RegenerationPlan,
];

pub const IMPACT_RESULTS: &[&str] = &["affected", "unaffected", "impact_unknown"];
pub const BASELINE_COMPARISON_RESULTS: &[&str] =
    &["added", "removed", "modified", "retargeted", "unchanged"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyNodeKind {
    Design,
    Library,
    Rule,
    Check,
    PublishSource,
    Manufacturing,
    GeneratedArtifact,
    ControlledDocument,
    Package,
    Policy,
}

impl DependencyNodeKind {
    pub const ALL: &'static [Self] = &[
        Self::Design,
        Self::Library,
        Self::Rule,
        Self::Check,
        Self::PublishSource,
        Self::Manufacturing,
        Self::GeneratedArtifact,
        Self::ControlledDocument,
        Self::Package,
        Self::Policy,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyEdgeKind {
    UsesDesign,
    UsesLibrary,
    UsesRule,
    UsesTemplate,
    BindsIdentity,
    ProjectsInto,
    Checks,
    Generates,
    Packages,
    Documents,
    GovernedBy,
    DerivedFrom,
}

impl DependencyEdgeKind {
    pub const ALL: &'static [Self] = &[
        Self::UsesDesign,
        Self::UsesLibrary,
        Self::UsesRule,
        Self::UsesTemplate,
        Self::BindsIdentity,
        Self::ProjectsInto,
        Self::Checks,
        Self::Generates,
        Self::Packages,
        Self::Documents,
        Self::GovernedBy,
        Self::DerivedFrom,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ConfigurationTarget {
    Working {
        model_revision: String,
        accepted_transaction_tip: String,
    },
    Baseline {
        baseline_id: ConfigurationBaselineId,
        baseline_digest: AlgorithmQualifiedDigest,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyNode {
    pub authority_ref: AuthorityRef,
    pub exact_technical_revision: String,
    pub semantic_digest: AlgorithmQualifiedDigest,
    pub node_kind: DependencyNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyEdge {
    pub edge_id: String,
    pub source: AuthorityRef,
    pub target: AuthorityRef,
    pub edge_kind: DependencyEdgeKind,
    pub sensitivity: Vec<String>,
    pub origin: String,
    pub evaluator_id: String,
    pub evaluator_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencySnapshotData {
    pub configuration: ConfigurationTarget,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub evaluator_registry_revision: String,
    pub graph_complete: bool,
    pub unresolved_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticDeltaData {
    pub from_configuration: ConfigurationTarget,
    pub to_configuration: ConfigurationTarget,
    pub subject: AuthorityRef,
    pub changed_observations: Vec<String>,
    pub administrative_observations: Vec<String>,
    pub evaluator_id: String,
    pub evaluator_revision: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactResult {
    Affected,
    Unaffected,
    ImpactUnknown,
}

impl ImpactResult {
    pub const ALL: &'static [Self] = &[Self::Affected, Self::Unaffected, Self::ImpactUnknown];

    pub const fn wire_tag(self) -> &'static str {
        IMPACT_RESULTS[self as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectImpact {
    pub subject: AuthorityRef,
    pub result: ImpactResult,
    pub changed_observations: Vec<String>,
    pub witness_paths: Vec<Vec<String>>,
    pub reason_code: String,
    pub required_actions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_disposition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactEvaluationData {
    pub governing_change: EngineeringChangeId,
    pub from_configuration: ConfigurationTarget,
    pub to_configuration: ConfigurationTarget,
    pub dependency_snapshot: DependencySnapshotId,
    pub evaluator_registry_revision: String,
    pub subjects: Vec<SubjectImpact>,
    pub graph_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DigestQualifiedInput {
    pub input_ref: AuthorityRef,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceInputContextData {
    pub configuration: ConfigurationTarget,
    pub dependency_snapshot: DependencySnapshotId,
    pub inputs: Vec<DigestQualifiedInput>,
    pub policy_refs: Vec<AuthorityRef>,
    pub producer_ref: AuthorityRef,
    pub producer_revision: String,
    pub invocation_digest: AlgorithmQualifiedDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_digest: Option<AlgorithmQualifiedDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFreshnessData {
    pub input_context: EvidenceInputContextId,
    pub target_configuration: ConfigurationTarget,
    pub current: bool,
    pub differing_inputs: Vec<AuthorityRef>,
    pub unresolved_inputs: Vec<AuthorityRef>,
    pub unsupported_inputs: Vec<AuthorityRef>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryRevisionRef {
    pub object_id: ObjectId,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryUptakeCandidateData {
    pub component_instance_id: ObjectId,
    pub binding_id: ObjectId,
    pub pinned_library_ref: LibraryRevisionRef,
    pub proposed_library_ref: LibraryRevisionRef,
    pub pinned_source_resolves: bool,
    pub proposed_source_resolves: bool,
    pub semantic_delta: SemanticDeltaId,
    pub predicted_impact: ImpactEvaluationId,
    pub required_checks: Vec<String>,
    pub required_regeneration: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_change: Option<EngineeringChangeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineMember {
    pub stable_authority_ref: AuthorityRef,
    pub exact_technical_revision: String,
    pub semantic_digest: AlgorithmQualifiedDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<AuthorityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub role: String,
    pub inclusion_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationBaselineData {
    pub human_number: Option<String>,
    pub baseline_type: String,
    pub scope: Vec<AuthorityRef>,
    pub members: Vec<BaselineMember>,
    pub source_model_revision: String,
    pub accepted_transaction_tip: String,
    pub governing_changes: Vec<EngineeringChangeId>,
    pub departures: Vec<AuthorityRef>,
    pub profile_refs: Vec<AuthorityRef>,
    pub establishment_attestations: Vec<AuthorityRef>,
    pub established_by: AuthorityRef,
    pub established_at: u64,
    pub predecessor_baselines: Vec<ConfigurationBaselineId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineComparisonResult {
    Added,
    Removed,
    Modified,
    Retargeted,
    Unchanged,
}

impl BaselineComparisonResult {
    pub const ALL: &'static [Self] = &[
        Self::Added,
        Self::Removed,
        Self::Modified,
        Self::Retargeted,
        Self::Unchanged,
    ];

    pub const fn wire_tag(self) -> &'static str {
        BASELINE_COMPARISON_RESULTS[self as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberComparison {
    pub stable_authority_ref: AuthorityRef,
    pub result: BaselineComparisonResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_revision_delta: Option<(String, String)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_delta: Option<SemanticDeltaId>,
    pub related_changes: Vec<EngineeringChangeId>,
    pub impact_dispositions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineComparisonData {
    pub left_baseline: ConfigurationBaselineId,
    pub right_baseline: ConfigurationBaselineId,
    pub aligned_members: Vec<MemberComparison>,
    pub dependency_delta: Vec<String>,
    pub evidence_delta: Vec<String>,
    pub governing_change_coverage: Vec<EngineeringChangeId>,
    pub unresolved_differences: Vec<AuthorityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegenerationRequest {
    pub output_contract: String,
    pub predecessor_evidence: Option<AuthorityRef>,
    pub reason_paths: Vec<Vec<String>>,
    pub prerequisites: Vec<String>,
    pub producer_ref: AuthorityRef,
    pub producer_revision: String,
    pub expected_input_context: EvidenceInputContextId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegenerationPlanData {
    pub target_configuration: ConfigurationTarget,
    pub basis_impact_evaluation: ImpactEvaluationId,
    pub steps: Vec<RegenerationRequest>,
    pub unchanged_evidence_reused: Vec<AuthorityRef>,
    pub blockers: Vec<String>,
}
