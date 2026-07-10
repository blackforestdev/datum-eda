use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use super::artifact::{
    insert_manufacturing_plan_objects, insert_output_job_objects, insert_panel_projection_objects,
    read_artifact_metadata_shards, read_manufacturing_plan_shards, read_output_job_run_shards,
    read_output_job_shards, read_panel_projection_shards,
};
use super::artifact_run::read_artifact_run_shards;
use super::check_run::read_check_run_shards;
use super::component_instance::{collect_component_instances, read_component_instance_shards};
use super::component_instance_journal_ops::apply_component_instance_journal_to_map;
use super::forward_annotation_review_journal_ops::FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH;
use super::generated_evidence_journal_ops::{
    apply_artifact_metadata_journal_to_map, apply_artifact_run_journal_to_map,
    apply_check_run_journal_to_map, apply_output_job_run_journal_to_map,
};
use super::import_map::read_import_map_shards;
use super::import_map_journal_ops::apply_import_map_journal_to_map;
use super::journal::{read_journal_cursor, read_transaction_journal};
use super::production_journal_ops::apply_production_journal_to_maps;
use super::proposal::read_proposal_shards;
use super::proposal_journal_ops::apply_proposal_journal_to_map;
use super::relationship::read_relationship_shards;
use super::relationship_journal_ops::apply_relationship_journal_to_maps;
use super::replay::{replay_import_map_shards, validate_and_replay_journal};
use super::replay_proposal::replay_proposal_shards;
use super::replay_schematic::add_missing_journal_schematic_sheet_shards;
use super::run_evidence_validation::validate_run_evidence_links;
use super::source_shard::{collect_referenced_shards, read_source_shard};
use super::variant::{
    propagate_variant_population_to_component_instances, read_variant_overlay_shards,
};
use super::zone_fill::read_zone_fill_shards;
use super::zone_fill_journal_ops::apply_zone_fill_journal_to_map;
use super::{
    DesignModel, EngineError, ProjectManifestSummary, ProjectResolver, ResolveDiagnostic,
    SourceShardKind, collect_uuid_objects, compute_model_revision, domain_for_shard_kind,
    read_json_value, sort_source_shards,
};

#[derive(Debug, Deserialize)]
struct NativeProjectManifestShape {
    schema_version: Option<u64>,
    uuid: Uuid,
    name: String,
    #[serde(default)]
    pools: Vec<NativePoolRefShape>,
    schematic: String,
    board: String,
    rules: String,
}

#[derive(Debug, Deserialize)]
struct NativePoolRefShape {
    path: String,
}

impl ProjectResolver {
    pub fn new(project_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn resolve(&self) -> Result<DesignModel, EngineError> {
        let manifest_path = self.project_root.join("project.json");
        let manifest_value = read_json_value(&manifest_path)?;
        let manifest: NativeProjectManifestShape = serde_json::from_value(manifest_value.clone())?;

        let mut diagnostics = Vec::new();
        let mut shards = Vec::new();
        let mut objects = BTreeMap::new();
        let mut import_map = BTreeMap::new();

        let manifest_shard = read_source_shard(
            &self.project_root,
            SourceShardKind::ProjectManifest,
            "project.json",
            Some(&manifest_value),
        )?;
        collect_uuid_objects(
            &manifest_value,
            &manifest_shard,
            "project",
            &mut objects,
            &mut import_map,
        );
        shards.push(manifest_shard);
        let (journal_records, journal_diagnostics) = read_transaction_journal(&self.project_root);
        diagnostics.extend(journal_diagnostics);

        let shard_specs = [
            (SourceShardKind::SchematicRoot, manifest.schematic.as_str()),
            (SourceShardKind::BoardRoot, manifest.board.as_str()),
            (SourceShardKind::RulesRoot, manifest.rules.as_str()),
        ];

        for (kind, relative_path) in shard_specs {
            match read_source_shard(&self.project_root, kind, relative_path, None) {
                Ok(shard) => {
                    let value = read_json_value(&shard.path)?;
                    collect_uuid_objects(
                        &value,
                        &shard,
                        domain_for_shard_kind(&shard.kind),
                        &mut objects,
                        &mut import_map,
                    );
                    collect_referenced_shards(
                        &self.project_root,
                        &value,
                        &shard,
                        &mut shards,
                        &mut objects,
                        &mut import_map,
                        &mut diagnostics,
                    )?;
                    shards.push(shard);
                }
                Err(error) => {
                    if is_unsupported_schema_version_error(&error) {
                        return Err(error);
                    }
                    diagnostics.push(ResolveDiagnostic {
                        code: "missing_required_shard".to_string(),
                        message: error.to_string(),
                        path: Some(self.project_root.join(relative_path)),
                    });
                }
            }
        }

        for pool_ref in &manifest.pools {
            read_pool_ref_shards(
                &self.project_root,
                &pool_ref.path,
                &mut shards,
                &mut objects,
                &mut import_map,
                &mut diagnostics,
            )?;
        }

        let (manufacturing_plan_shards, manufacturing_plans, manufacturing_plan_diagnostics) =
            read_manufacturing_plan_shards(&self.project_root);
        insert_manufacturing_plan_objects(
            &manufacturing_plan_shards,
            &manufacturing_plans,
            &mut objects,
        );
        shards.extend(manufacturing_plan_shards);
        diagnostics.extend(manufacturing_plan_diagnostics);
        let (panel_projection_shards, panel_projections, panel_projection_diagnostics) =
            read_panel_projection_shards(&self.project_root);
        insert_panel_projection_objects(&panel_projection_shards, &panel_projections, &mut objects);
        shards.extend(panel_projection_shards);
        diagnostics.extend(panel_projection_diagnostics);
        let (output_job_shards, output_jobs, output_job_diagnostics) =
            read_output_job_shards(&self.project_root);
        insert_output_job_objects(&output_job_shards, &output_jobs, &mut objects);
        shards.extend(output_job_shards);
        diagnostics.extend(output_job_diagnostics);
        let (output_job_run_shards, output_job_runs, output_job_run_diagnostics) =
            read_output_job_run_shards(&self.project_root);
        shards.extend(output_job_run_shards);
        diagnostics.extend(output_job_run_diagnostics);
        let (artifact_run_shards, artifact_runs, artifact_run_diagnostics) =
            read_artifact_run_shards(&self.project_root);
        shards.extend(artifact_run_shards);
        diagnostics.extend(artifact_run_diagnostics);
        let (check_run_shards, check_runs, check_run_diagnostics) =
            read_check_run_shards(&self.project_root);
        shards.extend(check_run_shards);
        diagnostics.extend(check_run_diagnostics);
        let (zone_fill_shards, persisted_zone_fills, zone_fill_diagnostics) =
            read_zone_fill_shards(&self.project_root);
        shards.extend(zone_fill_shards);
        diagnostics.extend(zone_fill_diagnostics);
        let (artifact_shards, artifact_metadata, artifact_diagnostics) =
            read_artifact_metadata_shards(&self.project_root);
        shards.extend(artifact_shards);
        diagnostics.extend(artifact_diagnostics);
        let (proposal_shards, proposals, proposal_diagnostics) =
            read_proposal_shards(&self.project_root);
        shards.extend(proposal_shards);
        diagnostics.extend(proposal_diagnostics);
        if self
            .project_root
            .join(FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH)
            .exists()
        {
            match read_source_shard(
                &self.project_root,
                SourceShardKind::ForwardAnnotationReview,
                FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH,
                None,
            ) {
                Ok(shard) => shards.push(shard),
                Err(error) => diagnostics.push(ResolveDiagnostic {
                    code: "invalid_forward_annotation_review".to_string(),
                    message: error.to_string(),
                    path: Some(
                        self.project_root
                            .join(FORWARD_ANNOTATION_REVIEW_RELATIVE_PATH),
                    ),
                }),
            }
        }
        let (import_map_shards, import_map_diagnostics) =
            read_import_map_shards(&self.project_root, &objects, &mut import_map);
        shards.extend(import_map_shards);
        diagnostics.extend(import_map_diagnostics);
        let (
            component_instance_shards,
            persisted_component_instances,
            component_instance_diagnostics,
        ) = read_component_instance_shards(&self.project_root, &mut objects);
        shards.extend(component_instance_shards);
        diagnostics.extend(component_instance_diagnostics);
        let (relationship_shards, relationships, relationship_statuses, relationship_diagnostics) =
            read_relationship_shards(&self.project_root, &mut objects);
        shards.extend(relationship_shards);
        diagnostics.extend(relationship_diagnostics);
        let (variant_shards, variants, variant_populations, variant_diagnostics) =
            read_variant_overlay_shards(&self.project_root, &mut objects);
        shards.extend(variant_shards);
        diagnostics.extend(variant_diagnostics);

        shards.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.relative_path.cmp(&b.relative_path))
        });

        let journal = validate_and_replay_journal(
            &self.project_root,
            &manifest.uuid,
            &mut shards,
            &mut objects,
            &journal_records,
            &mut diagnostics,
        )?;
        add_missing_journal_schematic_sheet_shards(&self.project_root, &mut shards, &journal)?;
        replay_import_map_shards(&self.project_root, &mut shards, &journal)?;
        replay_proposal_shards(&self.project_root, &mut shards, &journal)?;
        sort_source_shards(&mut shards);
        let mut manufacturing_plans = manufacturing_plans;
        let mut panel_projections = panel_projections;
        let mut output_jobs = output_jobs;
        apply_production_journal_to_maps(
            &journal,
            &mut manufacturing_plans,
            &mut panel_projections,
            &mut output_jobs,
        )?;
        let mut relationships = relationships;
        let mut relationship_statuses = relationship_statuses;
        let mut variants = variants;
        let mut variant_populations = variant_populations;
        apply_relationship_journal_to_maps(
            &journal,
            &objects,
            &mut relationships,
            &mut relationship_statuses,
            &mut variants,
            &mut variant_populations,
        )?;
        let (journal_cursor, cursor_diagnostics) =
            read_journal_cursor(&self.project_root, journal.len());
        diagnostics.extend(cursor_diagnostics);
        let mut persisted_component_instances = persisted_component_instances;
        apply_component_instance_journal_to_map(&journal, &mut persisted_component_instances)?;
        apply_import_map_journal_to_map(&journal, &mut import_map)?;
        let mut persisted_zone_fills = persisted_zone_fills;
        apply_zone_fill_journal_to_map(&journal, &mut persisted_zone_fills)?;
        let mut output_job_runs = output_job_runs;
        apply_output_job_run_journal_to_map(&journal, &mut output_job_runs)?;
        let mut artifact_runs = artifact_runs;
        apply_artifact_run_journal_to_map(&journal, &mut artifact_runs)?;
        let mut check_runs = check_runs;
        apply_check_run_journal_to_map(&journal, &mut check_runs)?;
        let mut artifact_metadata = artifact_metadata;
        apply_artifact_metadata_journal_to_map(&journal, &mut artifact_metadata)?;
        let mut proposals = proposals;
        apply_proposal_journal_to_map(&journal, &mut proposals)?;
        let component_instances = collect_component_instances(
            &shards,
            &journal,
            &objects,
            persisted_component_instances,
            &mut diagnostics,
        )?;
        propagate_variant_population_to_component_instances(
            &mut variant_populations,
            &component_instances,
            &mut diagnostics,
        );
        let computed_model_revision = compute_model_revision(&manifest.uuid, &shards, &objects);
        let model_revision = journal
            .last()
            .map(|transaction| transaction.after_model_revision.clone())
            .unwrap_or(computed_model_revision);

        let mut model = DesignModel {
            project: ProjectManifestSummary {
                project_id: manifest.uuid,
                name: manifest.name,
                schema_version: manifest.schema_version,
            },
            model_revision,
            source_shards: shards,
            objects,
            component_instances,
            relationships,
            relationship_statuses,
            variants,
            variant_populations,
            import_map,
            zone_fills: BTreeMap::new(),
            manufacturing_plans,
            panel_projections,
            output_jobs,
            output_job_runs,
            artifact_runs,
            check_runs,
            artifact_metadata,
            proposals,
            journal,
            journal_cursor,
            diagnostics,
        };
        model.zone_fills = super::derive_model_zone_fills(&model, persisted_zone_fills)?;
        validate_run_evidence_links(&mut model);
        Ok(model)
    }
}

fn is_unsupported_schema_version_error(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::Validation(message)
            if message.contains("unsupported") && message.contains("schema_version")
    )
}

fn read_pool_ref_shards(
    project_root: &Path,
    pool_path: &str,
    shards: &mut Vec<super::SourceShardRef>,
    objects: &mut BTreeMap<super::ObjectId, super::DomainObject>,
    import_map: &mut BTreeMap<super::ImportKey, super::ImportMapEntry>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<(), EngineError> {
    let path = project_root.join(pool_path);
    if path.is_dir() {
        for subdir in [
            "units",
            "symbols",
            "entities",
            "parts",
            "packages",
            "footprints",
            "padstacks",
            "pin_pad_maps",
        ] {
            read_pool_directory_shards(
                project_root,
                &path.join(subdir),
                pool_path,
                subdir,
                shards,
                objects,
                import_map,
                diagnostics,
            )?;
        }
        return Ok(());
    }
    match read_source_shard(project_root, SourceShardKind::Pool, pool_path, None) {
        Ok(shard) => {
            let value = read_json_value(&shard.path)?;
            collect_uuid_objects(&value, &shard, "pool", objects, import_map);
            shards.push(shard);
        }
        Err(error) => diagnostics.push(ResolveDiagnostic {
            code: "missing_pool_shard".to_string(),
            message: error.to_string(),
            path: Some(path),
        }),
    }
    Ok(())
}

// Substrate helper threads many record/shard fields.
#[allow(clippy::too_many_arguments)]
fn read_pool_directory_shards(
    project_root: &Path,
    directory: &Path,
    pool_path: &str,
    subdir: &str,
    shards: &mut Vec<super::SourceShardRef>,
    objects: &mut BTreeMap<super::ObjectId, super::DomainObject>,
    import_map: &mut BTreeMap<super::ImportKey, super::ImportMapEntry>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<(), EngineError> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<PathBuf>>();
    paths.sort();
    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!("{pool_path}/{subdir}/{filename}");
        match read_source_shard(project_root, SourceShardKind::Pool, &relative_path, None) {
            Ok(shard) => {
                let value = match read_json_value(&shard.path) {
                    Ok(value) => value,
                    Err(error) => {
                        diagnostics.push(ResolveDiagnostic {
                            code: "unreadable_pool_shard".to_string(),
                            message: error.to_string(),
                            path: Some(path),
                        });
                        continue;
                    }
                };
                collect_uuid_objects(&value, &shard, "pool", objects, import_map);
                set_pool_root_object_kind(&value, subdir, objects);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(ResolveDiagnostic {
                code: "missing_pool_shard".to_string(),
                message: error.to_string(),
                path: Some(path),
            }),
        }
    }
    Ok(())
}

fn set_pool_root_object_kind(
    value: &serde_json::Value,
    subdir: &str,
    objects: &mut BTreeMap<super::ObjectId, super::DomainObject>,
) {
    let Some(object_id) = value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        return;
    };
    if let Some(object) = objects.get_mut(&object_id) {
        object.kind = subdir.to_string();
    }
}
