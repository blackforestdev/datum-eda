use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use eda_engine::substrate::{
    DerivedRelationshipStatus, DerivedVariantPopulation, ProjectResolver, Relationship,
    VariantOverlay,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRelationshipsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) relationship_count: usize,
    pub(crate) relationships: BTreeMap<Uuid, Relationship>,
    pub(crate) statuses: BTreeMap<Uuid, DerivedRelationshipStatus>,
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
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectRelationshipsView {
        contract: "relationships_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        relationship_count: model.relationships.len(),
        relationships: model.relationships,
        statuses: model.relationship_statuses,
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
