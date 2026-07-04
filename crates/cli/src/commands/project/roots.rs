use std::collections::BTreeMap;
use std::path::Path;

use crate::default_native_project_stackup_layers;
use anyhow::{Context, Result};
use eda_engine::api::native_write::genesis::{
    GenesisRootIds, GenesisSpec, bootstrap_native_project, default_genesis_stackup_layers,
};
use eda_engine::api::native_write::project::{
    build_create_project_rule, build_delete_project_rule, build_set_project_name,
    build_set_project_rule, build_set_project_rules,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::ProjectResolver;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;

fn cli_write_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

pub(crate) fn render_symbol_display_mode(mode: &SymbolDisplayMode) -> String {
    match mode {
        SymbolDisplayMode::LibraryDefault => "LibraryDefault",
        SymbolDisplayMode::ShowHiddenPins => "ShowHiddenPins",
        SymbolDisplayMode::HideOptionalPins => "HideOptionalPins",
    }
    .to_string()
}

pub(crate) fn render_hidden_power_behavior(mode: &HiddenPowerBehavior) -> String {
    match mode {
        HiddenPowerBehavior::SourceDefinedImplicit => "SourceDefinedImplicit",
        HiddenPowerBehavior::ExplicitPowerObject => "ExplicitPowerObject",
        HiddenPowerBehavior::PreservedAsImportedMetadata => "PreservedAsImportedMetadata",
    }
    .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectManifest {
    pub(crate) schema_version: u32,
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) pools: Vec<NativeProjectPoolRef>,
    pub(crate) schematic: String,
    pub(crate) board: String,
    pub(crate) rules: String,
    #[serde(default)]
    pub(crate) forward_annotation_review: BTreeMap<String, NativeForwardAnnotationReviewRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeProjectPoolRef {
    pub(crate) path: String,
    pub(crate) priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeForwardAnnotationReviewRecord {
    pub(crate) action_id: String,
    pub(crate) decision: String,
    pub(crate) proposal_action: String,
    pub(crate) reference: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeSchematicRoot {
    pub(crate) schema_version: u32,
    pub(crate) uuid: Uuid,
    pub(crate) sheets: BTreeMap<String, String>,
    pub(crate) definitions: BTreeMap<String, String>,
    pub(crate) instances: Vec<NativeSchematicInstance>,
    pub(crate) variants: BTreeMap<String, NativeVariant>,
    #[serde(default)]
    pub(crate) waivers: Vec<serde_json::Value>,
    #[serde(default)]
    pub(crate) deviations: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeSchematicInstance {
    pub(crate) uuid: Uuid,
    pub(crate) definition: Uuid,
    pub(crate) parent_sheet: Option<Uuid>,
    pub(crate) position: NativePoint,
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) ports: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeSheetDefinitionRoot {
    pub(crate) schema_version: u32,
    pub(crate) uuid: Uuid,
    pub(crate) root_sheet: Uuid,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeVariant {
    pub(crate) name: String,
    pub(crate) fitted_components: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeRulesRoot {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) uuid: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) object_revision: Option<u64>,
    pub(crate) rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeSheetRoot {
    pub(crate) schema_version: u32,
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) frame: Option<SheetFrame>,
    pub(crate) symbols: BTreeMap<String, PlacedSymbol>,
    pub(crate) wires: BTreeMap<String, SchematicWire>,
    pub(crate) junctions: BTreeMap<String, Junction>,
    pub(crate) labels: BTreeMap<String, NetLabel>,
    pub(crate) buses: BTreeMap<String, Bus>,
    pub(crate) bus_entries: BTreeMap<String, BusEntry>,
    pub(crate) ports: BTreeMap<String, HierarchicalPort>,
    pub(crate) noconnects: BTreeMap<String, NoConnectMarker>,
    pub(crate) texts: BTreeMap<String, SchematicText>,
    pub(crate) drawings: BTreeMap<String, SchematicPrimitive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExistingProjectIds {
    pub(crate) project_uuid: Uuid,
    pub(crate) schematic_uuid: Uuid,
    pub(crate) board_uuid: Uuid,
    pub(crate) rules_uuid: Option<Uuid>,
}

pub(crate) fn create_native_project(
    root: &Path,
    name_override: Option<String>,
) -> Result<NativeProjectCreateReportView> {
    let root = root.to_path_buf();
    ensure_project_root(&root)?;

    let default_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("project root must have a terminal directory name"))?;
    let project_name = name_override.unwrap_or(default_name);

    let existing_ids = load_existing_ids(&root)?.map(|ids| GenesisRootIds {
        project: ids.project_uuid,
        schematic: ids.schematic_uuid,
        board: ids.board_uuid,
        rules: ids.rules_uuid,
    });

    // Drift tripwire: the CLI's board-layout default stackup and the engine's
    // genesis stackup must stay byte-identical (also keeps the CLI helper in
    // use for its remaining board-layout callers).
    debug_assert_eq!(
        default_native_project_stackup_layers(),
        default_genesis_stackup_layers(),
        "CLI default stackup drifted from engine genesis stackup"
    );

    let report = bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name,
            existing_ids,
        },
    )?;

    Ok(NativeProjectCreateReportView {
        project_root: root.display().to_string(),
        project_name: report.project_name,
        project_uuid: report.project_uuid.to_string(),
        schematic_uuid: report.schematic_uuid.to_string(),
        board_uuid: report.board_uuid.to_string(),
        files_written: report
            .files_written
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
    })
}

pub(crate) fn set_native_project_name(
    root: &Path,
    name: String,
) -> Result<NativeProjectNameMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared =
        build_set_project_name(&model, cli_write_provenance("set project name")?, &name)?;
    commit_prepared(&mut model, root, prepared).context("failed to commit set project name")?;
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectNameMutationReportView {
        action: "set_project_name".to_string(),
        project_root: root.display().to_string(),
        project_uuid: model.project.project_id.to_string(),
        name: model.project.name,
    })
}

pub(crate) fn set_native_project_rules(
    root: &Path,
    rules_file: &Path,
) -> Result<NativeProjectRulesMutationReportView> {
    let replacement_text = std::fs::read_to_string(rules_file)
        .with_context(|| format!("failed to read {}", rules_file.display()))?;
    let replacement: NativeRulesRoot = serde_json::from_str(&replacement_text)
        .with_context(|| format!("failed to parse {}", rules_file.display()))?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_set_project_rules(
        &model,
        cli_write_provenance("set project rules")?,
        replacement.rules,
    )?;
    commit_prepared(&mut model, root, prepared).context("failed to commit set project rules")?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectRulesMutationReportView {
        action: "set_project_rules".to_string(),
        project_root: root.display().to_string(),
        rule_uuid: None,
        rules_object_revision: project.rules.object_revision,
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn create_native_project_rule(
    root: &Path,
    rule_file: &Path,
) -> Result<NativeProjectRulesMutationReportView> {
    let rule_text = std::fs::read_to_string(rule_file)
        .with_context(|| format!("failed to read {}", rule_file.display()))?;
    let rule: serde_json::Value = serde_json::from_str(&rule_text)
        .with_context(|| format!("failed to parse {}", rule_file.display()))?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let (prepared, rule_id) =
        build_create_project_rule(&model, cli_write_provenance("create project rule")?, rule)?;
    commit_prepared(&mut model, root, prepared).context("failed to commit create project rule")?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectRulesMutationReportView {
        action: "create_project_rule".to_string(),
        project_root: root.display().to_string(),
        rule_uuid: Some(rule_id.to_string()),
        rules_object_revision: project.rules.object_revision,
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn set_native_project_rule(
    root: &Path,
    rule_file: &Path,
) -> Result<NativeProjectRulesMutationReportView> {
    let rule_text = std::fs::read_to_string(rule_file)
        .with_context(|| format!("failed to read {}", rule_file.display()))?;
    let rule: serde_json::Value = serde_json::from_str(&rule_text)
        .with_context(|| format!("failed to parse {}", rule_file.display()))?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let (prepared, rule_id) =
        build_set_project_rule(&model, cli_write_provenance("set project rule")?, rule)?;
    commit_prepared(&mut model, root, prepared).context("failed to commit set project rule")?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectRulesMutationReportView {
        action: "set_project_rule".to_string(),
        project_root: root.display().to_string(),
        rule_uuid: Some(rule_id.to_string()),
        rules_object_revision: project.rules.object_revision,
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn delete_native_project_rule(
    root: &Path,
    rule_id: Uuid,
) -> Result<NativeProjectRulesMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let (prepared, _deleted_rule) = build_delete_project_rule(
        &model,
        cli_write_provenance("delete project rule")?,
        rule_id,
    )?;
    commit_prepared(&mut model, root, prepared).context("failed to commit delete project rule")?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectRulesMutationReportView {
        action: "delete_project_rule".to_string(),
        project_root: root.display().to_string(),
        rule_uuid: Some(rule_id.to_string()),
        rules_object_revision: project.rules.object_revision,
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn query_native_project_rules(root: &Path) -> Result<NativeProjectRulesView> {
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectRulesView {
        domain: "native_project",
        count: project.rules.rules.len(),
        rules: project.rules.rules,
    })
}
