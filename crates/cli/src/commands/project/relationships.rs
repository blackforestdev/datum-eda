use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{
    ComponentInstanceAuthority, DerivedRelationshipStatus, DerivedVariantPopulation,
    ProjectResolver, Relationship, ResolveDiagnostic, VariantOverlay,
};
use serde::Serialize;
use uuid::Uuid;

use super::{LoadedNativeProject, load_native_project_with_resolved_board_and_model};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRelationshipsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) relationship_count: usize,
    pub(crate) relationships: BTreeMap<Uuid, Relationship>,
    pub(crate) statuses: BTreeMap<Uuid, DerivedRelationshipStatus>,
    pub(crate) component_instance_diagnostics:
        NativeProjectComponentInstanceRelationshipDiagnostics,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectComponentInstanceRelationshipDiagnostics {
    pub(crate) schematic_symbol_count: usize,
    pub(crate) authored_component_instance_count: usize,
    pub(crate) board_package_count: usize,
    pub(crate) unplaced_component_instance_count: usize,
    pub(crate) unmatched_symbol_count: usize,
    pub(crate) unmatched_package_count: usize,
    pub(crate) stale_or_missing_ref_count: usize,
    pub(crate) ambiguous_join_count: usize,
    pub(crate) unplaced_component_instances: Vec<Uuid>,
    pub(crate) unmatched_symbols: Vec<ResolveDiagnostic>,
    pub(crate) unmatched_packages: Vec<ResolveDiagnostic>,
    pub(crate) stale_or_missing_refs: Vec<ResolveDiagnostic>,
    pub(crate) ambiguous_joins: Vec<ResolveDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectVariantsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) variant_count: usize,
    pub(crate) variants: BTreeMap<Uuid, VariantOverlay>,
    pub(crate) populations: BTreeMap<Uuid, BTreeMap<Uuid, DerivedVariantPopulation>>,
}

pub(crate) fn query_native_project_relationships(
    root: &Path,
) -> Result<NativeProjectRelationshipsView> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let component_instance_diagnostics =
        component_instance_relationship_diagnostics(&project, &model)?;
    Ok(NativeProjectRelationshipsView {
        contract: "relationships_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        relationship_count: model.relationships.len(),
        relationships: model.relationships,
        statuses: model.relationship_statuses,
        component_instance_diagnostics,
    })
}

pub(crate) fn query_native_project_variants(root: &Path) -> Result<NativeProjectVariantsView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectVariantsView {
        contract: "variants_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        variant_count: model.variants.len(),
        variants: model.variants,
        populations: model.variant_populations,
    })
}

fn component_instance_relationship_diagnostics(
    project: &LoadedNativeProject,
    model: &eda_engine::substrate::DesignModel,
) -> Result<NativeProjectComponentInstanceRelationshipDiagnostics> {
    let schematic_symbol_count = materialized_schematic_symbol_count(project, model)?;
    let board_package_count = project.board.packages.len();
    let authored_component_instances = model
        .component_instances
        .values()
        .filter(|instance| instance.authority == ComponentInstanceAuthority::Authored)
        .collect::<Vec<_>>();
    let unplaced_component_instances = authored_component_instances
        .iter()
        .filter(|instance| instance.placed_package_refs.is_empty())
        .map(|instance| instance.id)
        .collect::<Vec<_>>();
    let unmatched_symbols = component_instance_diagnostics_by_code(
        &model.diagnostics,
        &["component_instance_unmatched_symbol"],
    );
    let unmatched_packages = component_instance_diagnostics_by_code(
        &model.diagnostics,
        &["component_instance_unmatched_package"],
    );
    let stale_or_missing_refs = component_instance_diagnostics_by_code(
        &model.diagnostics,
        &[
            "component_instance_unresolved_ref",
            "component_instance_unresolved_part_ref",
            "component_instance_invalid_part_ref",
            "component_instance_invalid_symbol_roles",
            "component_instance_invalid_package_roles",
        ],
    );
    let ambiguous_joins = component_instance_diagnostics_by_code(
        &model.diagnostics,
        &["component_instance_ambiguous_join"],
    );
    Ok(NativeProjectComponentInstanceRelationshipDiagnostics {
        schematic_symbol_count,
        authored_component_instance_count: authored_component_instances.len(),
        board_package_count,
        unplaced_component_instance_count: unplaced_component_instances.len(),
        unmatched_symbol_count: unmatched_symbols.len(),
        unmatched_package_count: unmatched_packages.len(),
        stale_or_missing_ref_count: stale_or_missing_refs.len(),
        ambiguous_join_count: ambiguous_joins.len(),
        unplaced_component_instances,
        unmatched_symbols,
        unmatched_packages,
        stale_or_missing_refs,
        ambiguous_joins,
    })
}

fn materialized_schematic_symbol_count(
    project: &LoadedNativeProject,
    model: &eda_engine::substrate::DesignModel,
) -> Result<usize> {
    project
        .schematic
        .sheets
        .values()
        .map(|relative_path| {
            let path = project.root.join("schematic").join(relative_path);
            let sheet_value = model
                .materialized_source_shard_value_by_relative_path(&format!(
                    "schematic/{relative_path}"
                ))
                .with_context(|| format!("failed to materialize {}", path.display()))?;
            Ok(sheet_value
                .get("symbols")
                .and_then(serde_json::Value::as_object)
                .map_or(0, serde_json::Map::len))
        })
        .sum()
}

fn component_instance_diagnostics_by_code(
    diagnostics: &[ResolveDiagnostic],
    codes: &[&str],
) -> Vec<ResolveDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| codes.contains(&diagnostic.code.as_str()))
        .cloned()
        .collect()
}
