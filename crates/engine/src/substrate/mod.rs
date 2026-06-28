use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::EngineError;

mod artifact;
mod artifact_run;
mod artifact_validation;
mod board_journal_ops;
mod board_json_maps;
mod board_list_journal_ops;
mod board_package_json;
mod board_package_move;
mod board_root_journal_ops;
mod check_run;
mod commit;
mod component_instance;
mod component_instance_journal_ops;
mod forward_annotation_review_journal_ops;
mod generated_evidence;
mod generated_evidence_journal_ops;
mod import_map;
mod import_map_journal_ops;
mod journal;
mod journal_io;
mod journal_operation_hooks;
mod operation;
mod operation_application;
mod operation_application_batch;
mod operation_application_board_payloads;
mod operation_application_component_instance;
mod operation_application_dispatch;
mod operation_application_object_revision;
mod operation_application_objects;
mod operation_application_production;
mod operation_application_relationship;
mod operation_application_schematic;
mod operation_application_schematic_definition;
mod operation_application_schematic_instance;
mod operation_application_schematic_waiver;
mod pool_journal_ops;
mod production_journal_ops;
mod project_manifest_journal_ops;
mod project_resolver;
mod proposal;
mod proposal_journal_ops;
mod proposal_policy;
mod proposal_validation;
mod relationship;
mod relationship_journal_ops;
mod replay;
mod replay_forward_annotation;
mod replay_generated_evidence;
mod replay_objects;
mod replay_pool;
mod replay_proposal;
mod replay_schematic;
mod rules_journal_ops;
mod run_evidence_validation;
mod schematic_definition_journal_ops;
mod schematic_root_journal_ops;
mod schematic_sheet_journal_ops;
mod schematic_sheet_maps;
mod source_shard;
mod source_shard_ref_builders;
mod transaction_links;
mod undo_redo;
mod variant;
mod zone_fill;
mod zone_fill_geometry;
mod zone_fill_journal_ops;

pub use artifact::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactProductionProjection, ArtifactValidationState, ManufacturingPlan,
    OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJob, OutputJobLogEntry, OutputJobLogLevel, OutputJobRun,
    OutputJobRunLauncher, OutputJobRunProvenance, OutputJobRunStatus,
    PRODUCTION_RECORD_SCHEMA_VERSION, PanelBoardInstance, PanelProjection,
};
pub use artifact_run::{ARTIFACT_RUN_SCHEMA_VERSION, ArtifactRun};
pub use check_run::{
    CHECK_RUN_SCHEMA_VERSION, CHECK_RUN_STANDARDS_BASIS_REGISTRY, CheckFinding, CheckRun,
    CheckRunCoverageEntry, CheckRunProfileBasis, PROCESS_APERTURE_STANDARDS_BASIS_ID,
    StandardsBasis, StandardsBasisRegistryEntry, ZONE_FILL_HONESTY_STANDARDS_BASIS_ID,
    standards_basis_for_id, standards_basis_id_for_check_code, standards_basis_registry_entry,
};
pub use component_instance::{COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION, ComponentInstanceShard};
pub use import_map::{
    IMPORT_MAP_SHARD_SCHEMA_VERSION, ImportIdentityAllocation, ImportMapEntryStatus,
    ImportMapShard, allocate_import_identity,
};
pub use journal::transaction_journal_path;
use journal::{
    canonical_json_hash, materialized_shard_value, replay_journal_shard_value, sort_source_shards,
    stage_operation_shard_writes, update_staged_source_hashes,
};
pub use operation::Operation;
use operation_application::apply_operation;
pub use proposal::*;
pub use relationship::{RELATIONSHIP_SHARD_SCHEMA_VERSION, RelationshipShard};
pub use rules_journal_ops::validate_native_project_rule_payload;
pub use variant::{VARIANT_OVERLAY_SHARD_SCHEMA_VERSION, VariantOverlayShard};
use zone_fill::derive_model_zone_fills;
pub use zone_fill::{
    ZONE_FILL_SCHEMA_VERSION, ZoneFill, ZoneFillCopperContext, ZoneFillState,
    compute_bounded_zone_fill, zone_fill_copper_projection_zones,
};

pub type ObjectId = Uuid;
pub type ComponentInstanceId = Uuid;
pub type ImportKey = String;
pub const JOURNAL_RELATIVE_PATH: &str = ".datum/journal/transactions.jsonl";
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectRevision(pub u64);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModelRevision(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceShardKind {
    ProjectManifest,
    SchematicRoot,
    BoardRoot,
    RulesRoot,
    SchematicSheet,
    SchematicDefinition,
    Pool,
    Relationship,
    ComponentInstance,
    VariantOverlay,
    ImportMap,
    ManufacturingPlan,
    PanelProjection,
    OutputJob,
    OutputJobRun,
    ArtifactRun,
    CheckRun,
    ZoneFill,
    ArtifactMetadata,
    ProposalMetadata,
    ForwardAnnotationReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceShardAuthority {
    AuthoredDesign,
    ImportedDesign,
    SidecarMetadata,
    GeneratedEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceShardDirtyState {
    Clean,
    Dirty,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceShardTaxon {
    ComponentInstance,
    Relationship,
    VariantOverlay,
    PoolUnit,
    PoolSymbol,
    PoolEntity,
    PoolPart,
    PoolPackage,
    PoolFootprint,
    PoolPadstack,
    PoolPinPadMap,
    ManufacturingPlan,
    PanelProjection,
    OutputJob,
    ImportMap,
    ProposalMetadata,
    ForwardAnnotationReview,
    OutputJobRun,
    ArtifactRun,
    CheckRun,
    ZoneFill,
    ArtifactMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceShardRef {
    pub shard_id: Uuid,
    pub kind: SourceShardKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taxon: Option<SourceShardTaxon>,
    pub path: PathBuf,
    pub relative_path: String,
    pub authority: SourceShardAuthority,
    pub dirty_state: SourceShardDirtyState,
    pub schema_version: Option<u64>,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainObject {
    pub object_id: ObjectId,
    pub object_revision: ObjectRevision,
    pub source_shard_id: Uuid,
    pub domain: String,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentInstanceAuthority {
    Authored,
    CompatibilityDerived,
}

impl Default for ComponentInstanceAuthority {
    fn default() -> Self {
        Self::Authored
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub id: ComponentInstanceId,
    pub object_revision: ObjectRevision,
    #[serde(default)]
    pub authority: ComponentInstanceAuthority,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_ref: Option<ObjectId>,
    pub placed_symbol_refs: Vec<ObjectId>,
    pub placed_package_refs: Vec<ObjectId>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub placed_symbol_roles: BTreeMap<ObjectId, ComponentInstanceRoleMetadata>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub placed_package_roles: BTreeMap<ObjectId, ComponentInstanceRoleMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInstanceRoleMetadata {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionedRef {
    pub object_id: ObjectId,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    ImplementedBy,
    BoardOnly,
    SchematicOnly,
    ReverseEngineered,
    Pending,
    Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivedRelationshipStatus {
    Implemented,
    PendingImplementation,
    UnresolvedMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivedVariantPopulation {
    Applicable,
    NotApplicableForVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FittedState {
    Fitted,
    Unfitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AuthoredIntentRecord {
    LayoutDeviation {
        rationale: String,
        accepted_by: String,
    },
    AcceptedDeviation {
        rationale: String,
        accepted_by: String,
    },
    Waiver {
        waiver_id: ObjectId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub id: ObjectId,
    pub kind: RelationshipKind,
    pub from: Vec<RevisionedRef>,
    pub to: Vec<RevisionedRef>,
    #[serde(default)]
    pub authored_intent: Vec<AuthoredIntentRecord>,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<RelationshipKind>,
    #[serde(default)]
    pub authored_intent: Vec<AuthoredIntentRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantOverlay {
    pub id: ObjectId,
    pub name: String,
    pub base_model_revision: ModelRevision,
    pub variant_revision: ObjectRevision,
    #[serde(default)]
    pub fitted: BTreeMap<ObjectId, FittedState>,
    #[serde(default)]
    pub relationship_overrides: BTreeMap<ObjectId, RelationshipOverride>,
    #[serde(default)]
    pub property_overrides: BTreeMap<ObjectId, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMapEntry {
    pub import_key: ImportKey,
    pub object_id: ObjectId,
    pub source_shard_id: Uuid,
    #[serde(default)]
    pub status: ImportMapEntryStatus,
    #[serde(default)]
    pub source_tool: String,
    #[serde(default)]
    pub source_path: String,
    #[serde(default)]
    pub source_object_ref: String,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationBatch {
    pub batch_id: Uuid,
    pub expected_model_revision: Option<ModelRevision>,
    pub provenance: CommitProvenance,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitProvenance {
    pub actor: String,
    pub source: CommitSource,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitSource {
    Manual,
    Cli,
    Test,
    Tool,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CommitDiff {
    pub created: Vec<ObjectId>,
    pub modified: Vec<ObjectId>,
    pub deleted: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub transaction_id: Uuid,
    pub batch_id: Uuid,
    #[serde(default)]
    pub transaction_kind: TransactionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_of: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redo_of: Option<Uuid>,
    pub before_model_revision: ModelRevision,
    pub after_model_revision: ModelRevision,
    pub provenance: CommitProvenance,
    pub diff: CommitDiff,
    pub operations: Vec<Operation>,
    #[serde(default)]
    pub inverse_operations: Vec<Operation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    #[default]
    Normal,
    Undo,
    Redo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitReport {
    pub transaction: TransactionRecord,
    pub journal_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCursor {
    pub applied_transaction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveDiagnostic {
    pub code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifestSummary {
    pub project_id: Uuid,
    pub name: String,
    pub schema_version: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignModel {
    pub project: ProjectManifestSummary,
    pub model_revision: ModelRevision,
    pub source_shards: Vec<SourceShardRef>,
    pub objects: BTreeMap<ObjectId, DomainObject>,
    pub component_instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
    pub relationships: BTreeMap<ObjectId, Relationship>,
    pub relationship_statuses: BTreeMap<ObjectId, DerivedRelationshipStatus>,
    pub variants: BTreeMap<ObjectId, VariantOverlay>,
    pub variant_populations: BTreeMap<ObjectId, BTreeMap<ObjectId, DerivedVariantPopulation>>,
    pub import_map: BTreeMap<ImportKey, ImportMapEntry>,
    pub zone_fills: BTreeMap<ObjectId, ZoneFill>,
    pub manufacturing_plans: BTreeMap<ObjectId, ManufacturingPlan>,
    pub panel_projections: BTreeMap<ObjectId, PanelProjection>,
    pub output_jobs: BTreeMap<ObjectId, OutputJob>,
    pub output_job_runs: BTreeMap<Uuid, OutputJobRun>,
    pub artifact_runs: BTreeMap<Uuid, ArtifactRun>,
    pub check_runs: BTreeMap<Uuid, CheckRun>,
    pub artifact_metadata: BTreeMap<Uuid, ArtifactMetadata>,
    pub proposals: BTreeMap<Uuid, Proposal>,
    pub journal: Vec<TransactionRecord>,
    pub journal_cursor: JournalCursor,
    pub diagnostics: Vec<ResolveDiagnostic>,
}
#[derive(Debug, Clone)]
pub struct ProjectResolver {
    project_root: PathBuf,
}
impl DesignModel {
    pub fn materialized_source_shard_value(
        &self,
        kind: SourceShardKind,
    ) -> Result<serde_json::Value, EngineError> {
        let shard = self
            .source_shards
            .iter()
            .find(|shard| shard.kind == kind)
            .ok_or_else(|| {
                EngineError::Validation(format!("model missing {kind:?} source shard"))
            })?;
        materialized_shard_value(self, shard)
    }

    pub fn materialized_source_shard_value_by_relative_path(
        &self,
        relative_path: &str,
    ) -> Result<serde_json::Value, EngineError> {
        let shard = self
            .source_shards
            .iter()
            .find(|shard| shard.relative_path == relative_path)
            .ok_or_else(|| {
                EngineError::Validation(format!("model missing {relative_path} source shard"))
            })?;
        materialized_shard_value(self, shard)
    }
}

pub(super) fn read_json_value(path: &Path) -> Result<serde_json::Value, EngineError> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub(super) fn collect_uuid_objects(
    value: &serde_json::Value,
    shard: &SourceShardRef,
    domain: &str,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
) {
    collect_uuid_objects_at(value, shard, domain, "$", objects, import_map);
}

fn collect_uuid_objects_at(
    value: &serde_json::Value,
    shard: &SourceShardRef,
    domain: &str,
    pointer: &str,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(object_id) = map
                .get("uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
            {
                objects.entry(object_id).or_insert_with(|| DomainObject {
                    object_id,
                    object_revision: object_revision_for(value),
                    source_shard_id: shard.shard_id,
                    domain: domain.to_string(),
                    kind: kind_from_pointer(pointer),
                });

                if let Some(import_key) = map.get("import_key").and_then(serde_json::Value::as_str)
                {
                    import_map.insert(
                        import_key.to_string(),
                        ImportMapEntry {
                            import_key: import_key.to_string(),
                            object_id,
                            source_shard_id: shard.shard_id,
                            status: ImportMapEntryStatus::Active,
                            source_tool: String::new(),
                            source_path: shard.relative_path.clone(),
                            source_object_ref: pointer.to_string(),
                            source_hash: shard.content_hash.clone(),
                        },
                    );
                }
            }

            for (key, nested) in map {
                collect_uuid_objects_at(
                    nested,
                    shard,
                    domain,
                    &format!("{pointer}/{key}"),
                    objects,
                    import_map,
                );
            }
        }
        serde_json::Value::Array(values) => {
            for (index, nested) in values.iter().enumerate() {
                collect_uuid_objects_at(
                    nested,
                    shard,
                    domain,
                    &format!("{pointer}/{index}"),
                    objects,
                    import_map,
                );
            }
        }
        _ => {}
    }
}

fn object_revision_for(value: &serde_json::Value) -> ObjectRevision {
    value
        .get("object_revision")
        .and_then(serde_json::Value::as_u64)
        .map(ObjectRevision)
        .unwrap_or(ObjectRevision(0))
}

fn kind_from_pointer(pointer: &str) -> String {
    pointer
        .rsplit('/')
        .find(|segment| !segment.is_empty() && segment.parse::<usize>().is_err())
        .unwrap_or("object")
        .to_string()
}

pub(super) fn domain_for_shard_kind(kind: &SourceShardKind) -> &'static str {
    match kind {
        SourceShardKind::ProjectManifest => "project",
        SourceShardKind::SchematicRoot
        | SourceShardKind::SchematicSheet
        | SourceShardKind::SchematicDefinition => "schematic",
        SourceShardKind::BoardRoot => "board",
        SourceShardKind::RulesRoot => "rules",
        SourceShardKind::Pool => "pool",
        SourceShardKind::Relationship => "relationship",
        SourceShardKind::ComponentInstance => "component_instance",
        SourceShardKind::VariantOverlay => "variant",
        SourceShardKind::ImportMap => "import",
        SourceShardKind::ManufacturingPlan => "manufacturing",
        SourceShardKind::PanelProjection => "manufacturing",
        SourceShardKind::OutputJob | SourceShardKind::OutputJobRun => "output",
        SourceShardKind::ArtifactRun => "artifact",
        SourceShardKind::CheckRun => "check",
        SourceShardKind::ZoneFill => "zone_fill",
        SourceShardKind::ArtifactMetadata => "artifact",
        SourceShardKind::ProposalMetadata => "proposal",
        SourceShardKind::ForwardAnnotationReview => "forward_annotation_review",
    }
}

pub(super) fn source_shard_authority_for_kind(kind: &SourceShardKind) -> SourceShardAuthority {
    match kind {
        SourceShardKind::ProjectManifest
        | SourceShardKind::SchematicRoot
        | SourceShardKind::SchematicSheet
        | SourceShardKind::SchematicDefinition
        | SourceShardKind::BoardRoot
        | SourceShardKind::RulesRoot
        | SourceShardKind::Pool
        | SourceShardKind::Relationship
        | SourceShardKind::ComponentInstance
        | SourceShardKind::VariantOverlay
        | SourceShardKind::ManufacturingPlan
        | SourceShardKind::PanelProjection
        | SourceShardKind::OutputJob => SourceShardAuthority::AuthoredDesign,
        SourceShardKind::ImportMap
        | SourceShardKind::ProposalMetadata
        | SourceShardKind::ForwardAnnotationReview => SourceShardAuthority::SidecarMetadata,
        SourceShardKind::OutputJobRun
        | SourceShardKind::ArtifactRun
        | SourceShardKind::CheckRun
        | SourceShardKind::ZoneFill
        | SourceShardKind::ArtifactMetadata => SourceShardAuthority::GeneratedEvidence,
    }
}

fn compute_model_revision(
    project_id: &Uuid,
    shards: &[SourceShardRef],
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> ModelRevision {
    let mut hasher = Sha256::new();
    hasher.update(project_id.as_bytes());
    for shard in shards {
        if matches!(
            shard.kind,
            SourceShardKind::ArtifactMetadata
                | SourceShardKind::ArtifactRun
                | SourceShardKind::OutputJobRun
                | SourceShardKind::CheckRun
                | SourceShardKind::ZoneFill
                | SourceShardKind::ImportMap
                | SourceShardKind::ProposalMetadata
                | SourceShardKind::ForwardAnnotationReview
        ) {
            continue;
        }
        hasher.update(shard.relative_path.as_bytes());
        hasher.update(shard.content_hash.as_bytes());
    }
    for (object_id, object) in objects {
        hasher.update(object_id.as_bytes());
        hasher.update(object.object_revision.0.to_be_bytes());
        hasher.update(object.source_shard_id.as_bytes());
    }
    ModelRevision(sha256_digest_hex(hasher.finalize().as_slice()))
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    sha256_digest_hex(hasher.finalize().as_slice())
}

fn sha256_digest_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests;
