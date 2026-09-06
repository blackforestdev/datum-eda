//! Versioned Global Preferences product request, response, and refusal schemas.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::repository::{GenerationRef, MutationReceipt};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceSchemaRefV1 {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceRepositoryStatusV1 {
    DefaultsOnly,
    Ready,
    PreservedUnreadable,
    MigrationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceContextV1 {
    pub scope: String,
    pub repository_status: PreferenceRepositoryStatusV1,
    pub generation: Option<GenerationRef>,
    pub active_catalog_digest: String,
    pub active_descriptor_count: u32,
    pub reserved_descriptor_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PreferenceControlViewV1 {
    BooleanSwitch {
        off_label: String,
        on_label: String,
    },
    EnumeratedSingleChoice {
        choices: Vec<PreferenceEnumChoiceV1>,
    },
    IntegerStepper {
        min: i64,
        max: i64,
        step: u64,
        suffix: String,
    },
    IdentityEntry {
        nullable: bool,
        placeholder: String,
    },
    StructuredEditor {
        action_label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceEnumChoiceV1 {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceSectionViewV1 {
    pub id: String,
    pub label: String,
    pub order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceControlDescriptorV1 {
    pub key: String,
    pub label: String,
    pub description: String,
    pub section_id: String,
    pub row_order: u32,
    pub control: PreferenceControlViewV1,
    pub default_value: Option<Value>,
    pub effect_timing: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceValueViewV1 {
    pub key: String,
    pub label: String,
    pub description: String,
    pub section_id: String,
    pub section_label: String,
    pub row_order: u32,
    pub control: PreferenceControlViewV1,
    pub effective_value: Option<Value>,
    pub user_value: Option<Value>,
    pub effect_timing: String,
    pub writable: bool,
    pub provenance_summary: String,
    pub generation: Option<GenerationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceConsideredFactV1 {
    pub id: String,
    pub source: String,
    pub value: Option<Value>,
    pub directive: Option<String>,
    pub disposition: String,
    pub reason: String,
    pub origin: String,
    pub actor: String,
    pub disclosure: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceExplanationV1 {
    pub key: String,
    pub effective_value: Option<Value>,
    pub outcome_reason: String,
    pub considered: Vec<PreferenceConsideredFactV1>,
    pub absent_sources: Vec<String>,
    pub evaluation_stages: Vec<String>,
    pub available_actions: Vec<String>,
    pub writable: bool,
    pub effect_timing: String,
    pub generation: Option<GenerationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceMatchKindV1 {
    Label,
    Description,
    StableKey,
    RegisteredAlias,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceSearchMatchV1 {
    pub value: PreferenceValueViewV1,
    pub match_kind: PreferenceMatchKindV1,
    pub matched_vocabulary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescribeResultV1 {
    pub sections: Vec<PreferenceSectionViewV1>,
    pub controls: Vec<PreferenceControlDescriptorV1>,
    pub active_catalog_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListResultV1 {
    pub values: Vec<PreferenceValueViewV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetResultV1 {
    pub value: PreferenceValueViewV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchResultV1 {
    pub query: String,
    pub matches: Vec<PreferenceSearchMatchV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainResultV1 {
    pub explanation: PreferenceExplanationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectUnitsSourceV1 {
    Global {
        expected_generation: Option<GenerationRef>,
    },
    Factory {
        profile_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectUnitsSeedPreviewV1 {
    pub source: ProjectUnitsSourceV1,
    pub pinned_generation: Option<GenerationRef>,
    pub profile: Value,
    pub receipt_preview: Value,
    pub units_seed_catalog_digest: String,
    pub seed_application_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "generation", rename_all = "snake_case")]
pub enum HeadExpectationV1 {
    Missing,
    Generation(GenerationRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceActorKindV1 {
    HumanGui,
    HumanCli,
    McpAgent,
    ScriptAgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceActorV1 {
    pub kind: PreferenceActorKindV1,
    pub session_id: String,
    pub local_actor_id: String,
    pub invocation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PreferenceMutationRequestV1 {
    SetUser {
        key: String,
        value: Value,
        expected: HeadExpectationV1,
        request_id: Uuid,
        reason: String,
    },
    ResetUser {
        key: String,
        expected: HeadExpectationV1,
        request_id: Uuid,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceMutationResultV1 {
    pub changed: bool,
    pub generation: Option<GenerationRef>,
    pub value: PreferenceValueViewV1,
    pub explanation: PreferenceExplanationV1,
    pub receipt: Option<MutationReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceProposalV1 {
    pub schema: PreferenceSchemaRefV1,
    pub proposal_id: Uuid,
    pub proposal_digest: String,
    pub prepared_against: HeadExpectationV1,
    pub active_catalog_digest: String,
    pub mutation: PreferenceMutationRequestV1,
    pub requesting_actor: PreferenceActorV1,
    pub rationale: String,
    pub creation_session: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PreferenceProposalActionV1 {
    Prepare {
        mutation: PreferenceMutationRequestV1,
        rationale: String,
    },
    Validate {
        proposal: PreferenceProposalV1,
    },
    AcceptAndApply {
        proposal: PreferenceProposalV1,
    },
    Reject {
        proposal: PreferenceProposalV1,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedProposalResultV1 {
    pub proposal: PreferenceProposalV1,
    pub current_value: PreferenceValueViewV1,
    pub explanation: PreferenceExplanationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalValidationResultV1 {
    pub valid: bool,
    pub proposal: PreferenceProposalV1,
    pub current_generation: Option<GenerationRef>,
    pub explanation: PreferenceExplanationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedProposalResultV1 {
    pub proposal_id: Uuid,
    pub mutation_result: PreferenceMutationResultV1,
    pub acceptance_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedProposalResultV1 {
    pub proposal_id: Uuid,
    pub rejected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "result", rename_all = "snake_case")]
pub enum PreferenceProposalResultV1 {
    Prepared(PreparedProposalResultV1),
    Validated(ProposalValidationResultV1),
    Accepted(AcceptedProposalResultV1),
    Rejected(RejectedProposalResultV1),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceErrorCodeV1 {
    InvalidRequest,
    InvalidQuery,
    UnknownSection,
    UnknownPreferenceKey,
    ReservedPreferenceKey,
    InvalidPreferenceValue,
    IneligibleSource,
    GenerationRequired,
    StaleGeneration,
    WriterConflict,
    RepositoryUnreadable,
    MigrationRequired,
    RepositoryIo,
    UnauthorizedActor,
    HumanPresenceRequired,
    ProposalRequired,
    ProposalInvalid,
    ProposalStale,
    MissingAcceptance,
    AcceptanceMismatch,
    AcceptanceExpired,
    AcceptanceConsumed,
    IdempotencyConflict,
    SeedSourceUnavailable,
    SeedIncomplete,
    ProjectTargetExists,
    GenesisPublishFailed,
    UnsupportedSchemaVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceErrorV1 {
    pub code: PreferenceErrorCodeV1,
    pub message: String,
    pub details: BTreeMap<String, Value>,
    pub current_context: PreferenceContextV1,
    pub preserved_draft: Option<Value>,
    pub preserved_proposal: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PreferenceQueryV1 {
    Describe,
    List { section: Option<String> },
    Get { key: String },
    Search { query: String },
    Explain { key: String },
    PreviewProjectUnitsSeed { source: ProjectUnitsSourceV1 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "result", rename_all = "snake_case")]
pub enum PreferenceQueryResultV1 {
    Describe(DescribeResultV1),
    List(ListResultV1),
    Get(GetResultV1),
    Search(SearchResultV1),
    Explain(ExplainResultV1),
    PreviewProjectUnitsSeed(ProjectUnitsSeedPreviewV1),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", content = "request", rename_all = "snake_case")]
pub enum PreferenceProductPayloadV1 {
    Query(PreferenceQueryV1),
    Mutation(PreferenceMutationRequestV1),
    Proposal(PreferenceProposalActionV1),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceProductRequestV1 {
    pub schema: PreferenceSchemaRefV1,
    pub payload: PreferenceProductPayloadV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", content = "result", rename_all = "snake_case")]
pub enum PreferenceProductResultV1 {
    Query(PreferenceQueryResultV1),
    Mutation(PreferenceMutationResultV1),
    Proposal(PreferenceProposalResultV1),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceProductResponseV1 {
    pub ok: bool,
    pub schema: PreferenceSchemaRefV1,
    pub context: PreferenceContextV1,
    pub result: Option<PreferenceProductResultV1>,
    pub error: Option<PreferenceErrorV1>,
}
