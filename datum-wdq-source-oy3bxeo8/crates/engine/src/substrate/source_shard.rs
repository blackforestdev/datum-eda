use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Component, Path};

use uuid::Uuid;

use super::{
    DomainObject, EngineError, ImportKey, ImportMapEntry, ObjectId, ResolveDiagnostic,
    SourceShardDirtyState, SourceShardKind, SourceShardRef, SourceShardTaxon, collect_uuid_objects,
    domain_for_shard_kind, sha256_hex, source_shard_authority_for_kind,
};

pub(super) fn read_source_shard(
    project_root: &Path,
    kind: SourceShardKind,
    relative_path: &str,
    value: Option<&serde_json::Value>,
) -> Result<SourceShardRef, EngineError> {
    validate_source_shard_ownership_path(&kind, relative_path)?;
    let path = project_root.join(relative_path);
    let bytes = std::fs::read(&path)?;
    let parsed;
    let value = match value {
        Some(value) => value,
        None => {
            parsed = serde_json::from_slice::<serde_json::Value>(&bytes)?;
            &parsed
        }
    };
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    validate_source_shard_schema_version(&kind, relative_path, schema_version)?;
    Ok(SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        authority: source_shard_authority_for_kind(&kind),
        taxon: source_shard_taxon_for_path(&kind, relative_path),
        kind,
        path,
        relative_path: relative_path.to_string(),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    })
}

pub(super) fn collect_referenced_shards(
    project_root: &Path,
    value: &serde_json::Value,
    _parent_shard: &SourceShardRef,
    shards: &mut Vec<SourceShardRef>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<(), EngineError> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    for (key, kind) in [
        ("sheets", SourceShardKind::SchematicSheet),
        ("definitions", SourceShardKind::SchematicDefinition),
    ] {
        let Some(map) = object.get(key).and_then(serde_json::Value::as_object) else {
            continue;
        };
        for relative in map.values().filter_map(serde_json::Value::as_str) {
            let relative_path = format!("schematic/{relative}");
            validate_source_shard_ownership_path(&kind, &relative_path)?;
            let path = project_root.join(&relative_path);
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error)
                    if kind == SourceShardKind::SchematicSheet
                        && error.kind() == ErrorKind::NotFound =>
                {
                    diagnostics.push(ResolveDiagnostic {
                        code: "missing_referenced_schematic_sheet".to_string(),
                        message: error.to_string(),
                        path: Some(path),
                    });
                    continue;
                }
                Err(error)
                    if matches!(
                        kind,
                        SourceShardKind::SchematicSheet | SourceShardKind::SchematicDefinition
                    ) =>
                {
                    diagnostics.push(ResolveDiagnostic {
                        code: "unreadable_referenced_schematic_shard".to_string(),
                        message: error.to_string(),
                        path: Some(path),
                    });
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            let value = serde_json::from_slice::<serde_json::Value>(&bytes)?;
            let schema_version = value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64);
            let kind = kind.clone();
            validate_source_shard_schema_version(&kind, &relative_path, schema_version)?;
            shards.push(SourceShardRef {
                shard_id: Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("datum-eda:source-shard:{relative_path}").as_bytes(),
                ),
                authority: source_shard_authority_for_kind(&kind),
                taxon: source_shard_taxon_for_path(&kind, &relative_path),
                dirty_state: SourceShardDirtyState::Clean,
                kind,
                path,
                relative_path,
                schema_version,
                content_hash: sha256_hex(&bytes),
            });
            let shard = shards.last().expect("referenced shard was just pushed");
            collect_uuid_objects(
                &value,
                shard,
                domain_for_shard_kind(&shard.kind),
                objects,
                import_map,
            );
        }
    }
    Ok(())
}

pub(super) fn source_shard_taxon_for_path(
    kind: &SourceShardKind,
    relative_path: &str,
) -> Option<SourceShardTaxon> {
    match kind {
        SourceShardKind::Relationship
            if relative_path.starts_with(".datum/relationships/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::Relationship)
        }
        SourceShardKind::ComponentInstance
            if relative_path.starts_with(".datum/component_instances/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ComponentInstance)
        }
        SourceShardKind::VariantOverlay
            if relative_path.starts_with(".datum/variants/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::VariantOverlay)
        }
        SourceShardKind::Pool => match relative_path.split('/').nth(1) {
            Some("units") => Some(SourceShardTaxon::PoolUnit),
            Some("symbols") => Some(SourceShardTaxon::PoolSymbol),
            Some("entities") => Some(SourceShardTaxon::PoolEntity),
            Some("parts") => Some(SourceShardTaxon::PoolPart),
            Some("packages") => Some(SourceShardTaxon::PoolPackage),
            Some("footprints") => Some(SourceShardTaxon::PoolFootprint),
            Some("padstacks") => Some(SourceShardTaxon::PoolPadstack),
            Some("pin_pad_maps") => Some(SourceShardTaxon::PoolPinPadMap),
            _ => None,
        },
        SourceShardKind::ManufacturingPlan
            if relative_path.starts_with(".datum/manufacturing_plans/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ManufacturingPlan)
        }
        SourceShardKind::PanelProjection
            if relative_path.starts_with(".datum/panel_projections/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::PanelProjection)
        }
        SourceShardKind::OutputJob
            if relative_path.starts_with(".datum/output_jobs/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::OutputJob)
        }
        SourceShardKind::ImportMap
            if relative_path.starts_with(".datum/import_map/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ImportMap)
        }
        SourceShardKind::ProposalMetadata
            if relative_path.starts_with(".datum/proposals/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ProposalMetadata)
        }
        SourceShardKind::ForwardAnnotationReview
            if relative_path == ".datum/forward_annotation_review/review.json" =>
        {
            Some(SourceShardTaxon::ForwardAnnotationReview)
        }
        SourceShardKind::OutputJobRun
            if relative_path.starts_with(".datum/output_job_runs/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::OutputJobRun)
        }
        SourceShardKind::ArtifactRun
            if relative_path.starts_with(".datum/artifact_runs/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ArtifactRun)
        }
        SourceShardKind::CheckRun
            if relative_path.starts_with(".datum/check_runs/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::CheckRun)
        }
        SourceShardKind::ZoneFill
            if relative_path.starts_with(".datum/zone_fills/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ZoneFill)
        }
        SourceShardKind::ArtifactMetadata
            if relative_path.starts_with(".datum/artifacts/")
                && relative_path.ends_with(".json") =>
        {
            Some(SourceShardTaxon::ArtifactMetadata)
        }
        _ => None,
    }
}

pub(super) fn validate_source_shard_ownership_path(
    kind: &SourceShardKind,
    relative_path: &str,
) -> Result<(), EngineError> {
    if relative_path.is_empty() || Path::new(relative_path).is_absolute() {
        return Err(source_shard_ownership_error(kind, relative_path));
    }
    for component in Path::new(relative_path).components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(source_shard_ownership_error(kind, relative_path));
        }
    }
    let valid = match kind {
        SourceShardKind::ProjectManifest => relative_path == "project.json",
        SourceShardKind::SchematicRoot => relative_path == "schematic/schematic.json",
        SourceShardKind::SchematicSheet | SourceShardKind::SchematicDefinition => {
            relative_path.starts_with("schematic/") && relative_path.ends_with(".json")
        }
        SourceShardKind::BoardRoot => relative_path == "board/board.json",
        SourceShardKind::RulesRoot => relative_path == "rules/rules.json",
        SourceShardKind::Pool => {
            relative_path.starts_with("pool/")
                && relative_path.ends_with(".json")
                && pool_source_shard_subdir(relative_path).is_some()
        }
        SourceShardKind::Relationship => {
            relative_path.starts_with(".datum/relationships/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ComponentInstance => {
            relative_path.starts_with(".datum/component_instances/")
                && relative_path.ends_with(".json")
        }
        SourceShardKind::VariantOverlay => {
            relative_path.starts_with(".datum/variants/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ImportMap => {
            relative_path.starts_with(".datum/import_map/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ManufacturingPlan => {
            relative_path.starts_with(".datum/manufacturing_plans/")
                && relative_path.ends_with(".json")
        }
        SourceShardKind::PanelProjection => {
            relative_path.starts_with(".datum/panel_projections/")
                && relative_path.ends_with(".json")
        }
        SourceShardKind::OutputJob => {
            relative_path.starts_with(".datum/output_jobs/") && relative_path.ends_with(".json")
        }
        SourceShardKind::OutputJobRun => {
            relative_path.starts_with(".datum/output_job_runs/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ArtifactRun => {
            relative_path.starts_with(".datum/artifact_runs/") && relative_path.ends_with(".json")
        }
        SourceShardKind::CheckRun => {
            relative_path.starts_with(".datum/check_runs/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ZoneFill => {
            relative_path.starts_with(".datum/zone_fills/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ArtifactMetadata => {
            relative_path.starts_with(".datum/artifacts/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ProposalMetadata => {
            relative_path.starts_with(".datum/proposals/") && relative_path.ends_with(".json")
        }
        SourceShardKind::ForwardAnnotationReview => {
            relative_path == ".datum/forward_annotation_review/review.json"
        }
    };
    if valid {
        Ok(())
    } else {
        Err(source_shard_ownership_error(kind, relative_path))
    }
}

pub(super) fn dirty_state_for_materialized_shard(
    project_root: &Path,
    relative_path: &str,
    materialized_content_hash: &str,
) -> SourceShardDirtyState {
    match std::fs::read(project_root.join(relative_path)) {
        Ok(bytes) if sha256_hex(&bytes) == materialized_content_hash => {
            SourceShardDirtyState::Clean
        }
        Ok(_) => SourceShardDirtyState::Dirty,
        Err(error) if error.kind() == ErrorKind::NotFound => SourceShardDirtyState::Missing,
        Err(_) => SourceShardDirtyState::Unknown,
    }
}

fn pool_source_shard_subdir(relative_path: &str) -> Option<&str> {
    match relative_path.split('/').nth(1) {
        Some(
            subdir @ ("units" | "symbols" | "entities" | "parts" | "packages" | "footprints"
            | "padstacks" | "pin_pad_maps"),
        ) => Some(subdir),
        _ => None,
    }
}

fn source_shard_ownership_error(kind: &SourceShardKind, relative_path: &str) -> EngineError {
    EngineError::Validation(format!(
        "source shard ownership mismatch: {kind:?} cannot own `{relative_path}`"
    ))
}

pub(super) fn validate_source_shard_schema_version(
    kind: &SourceShardKind,
    relative_path: &str,
    schema_version: Option<u64>,
) -> Result<(), EngineError> {
    const SUPPORTED_SCHEMA_VERSION: u64 = 1;
    if let Some(version) = schema_version
        && version > SUPPORTED_SCHEMA_VERSION
    {
        return Err(EngineError::Validation(format!(
            "unsupported {kind:?} schema_version {version} in {relative_path}; supported <= {SUPPORTED_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}
