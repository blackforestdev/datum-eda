use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ComponentInstance, ComponentInstanceId, DerivedVariantPopulation, DomainObject, FittedState,
    ObjectId, ResolveDiagnostic, SourceShardDirtyState, SourceShardKind, SourceShardRef,
    VariantOverlay, read_json_value, sha256_hex, source_shard_authority_for_kind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantOverlayShard {
    pub schema_version: u64,
    pub variants: Vec<VariantOverlay>,
}

pub(super) fn read_variant_overlay_shards(
    project_root: &Path,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ObjectId, VariantOverlay>,
    BTreeMap<ObjectId, BTreeMap<ObjectId, DerivedVariantPopulation>>,
    Vec<ResolveDiagnostic>,
) {
    let variant_dir = project_root.join(".datum/variants");
    let mut shards = Vec::new();
    let mut variants = BTreeMap::new();
    let mut populations = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&variant_dir) else {
        return (shards, variants, populations, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/variants/{filename}");
        let path = project_root.join(&relative_path);
        match read_variant_overlay_shard(path, relative_path) {
            Ok((shard, variant_shard)) => {
                insert_variants(
                    &shard,
                    variant_shard.variants,
                    objects,
                    &mut variants,
                    &mut populations,
                    &mut diagnostics,
                );
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, variants, populations, diagnostics)
}

fn read_variant_overlay_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, VariantOverlayShard), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_variant_overlay_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_variant_overlay_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let shard = SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: SourceShardKind::VariantOverlay,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::VariantOverlay),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let variant_shard = serde_json::from_value::<VariantOverlayShard>(value).map_err(|error| {
        ResolveDiagnostic {
            code: "invalid_variant_overlay_shard".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        }
    })?;
    Ok((shard, variant_shard))
}

fn insert_variants(
    shard: &SourceShardRef,
    input: Vec<VariantOverlay>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    variants: &mut BTreeMap<ObjectId, VariantOverlay>,
    populations: &mut BTreeMap<ObjectId, BTreeMap<ObjectId, DerivedVariantPopulation>>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) {
    for variant in input {
        if variants.contains_key(&variant.id) {
            diagnostics.push(ResolveDiagnostic {
                code: "variant_duplicate_id".to_string(),
                message: format!("duplicate variant id {}", variant.id),
                path: Some(shard.path.clone()),
            });
            continue;
        }
        objects.insert(
            variant.id,
            DomainObject {
                object_id: variant.id,
                object_revision: variant.variant_revision,
                source_shard_id: shard.shard_id,
                domain: "variant".to_string(),
                kind: "variant_overlay".to_string(),
            },
        );
        populations.insert(variant.id, derive_variant_population(&variant));
        variants.insert(variant.id, variant);
    }
}

fn derive_variant_population(
    variant: &VariantOverlay,
) -> BTreeMap<ObjectId, DerivedVariantPopulation> {
    variant
        .fitted
        .iter()
        .map(|(object_id, fitted_state)| {
            let population = match fitted_state {
                FittedState::Fitted => DerivedVariantPopulation::Applicable,
                FittedState::Unfitted => DerivedVariantPopulation::NotApplicableForVariant,
            };
            (*object_id, population)
        })
        .collect()
}

pub(super) fn propagate_variant_population_to_component_instances(
    populations: &mut BTreeMap<ObjectId, BTreeMap<ObjectId, DerivedVariantPopulation>>,
    component_instances: &BTreeMap<ComponentInstanceId, ComponentInstance>,
) {
    for population in populations.values_mut() {
        for (component_instance_id, component_instance) in component_instances {
            let component_population = population.get(component_instance_id).copied();
            if let Some(component_population) = component_population {
                for object_id in component_instance
                    .placed_symbol_refs
                    .iter()
                    .chain(&component_instance.placed_package_refs)
                {
                    let propagated =
                        merge_population(population.get(object_id).copied(), component_population)
                            .expect("component population merge should produce a value");
                    population.insert(*object_id, propagated);
                }
            }
            let propagated = component_instance
                .placed_symbol_refs
                .iter()
                .chain(&component_instance.placed_package_refs)
                .filter_map(|object_id| population.get(object_id).copied())
                .fold(None, merge_population);
            if let Some(propagated) = propagated {
                population.insert(*component_instance_id, propagated);
            }
        }
    }
}

fn merge_population(
    current: Option<DerivedVariantPopulation>,
    next: DerivedVariantPopulation,
) -> Option<DerivedVariantPopulation> {
    match (current, next) {
        (Some(DerivedVariantPopulation::NotApplicableForVariant), _)
        | (_, DerivedVariantPopulation::NotApplicableForVariant) => {
            Some(DerivedVariantPopulation::NotApplicableForVariant)
        }
        _ => Some(DerivedVariantPopulation::Applicable),
    }
}
