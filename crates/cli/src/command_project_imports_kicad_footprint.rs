use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    footprint_package_import_key, import_footprint_document_with_import_map,
};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ImportMapEntry, ImportMapShard, Operation, OperationBatch,
    ProjectResolver, SourceShardKind,
};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_imports::{
    next_pool_priority, source_shard_id_for_relative_path, validate_project_local_pool_path,
};
use super::{NativeProjectManifest, resolve_native_project_pool_path};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectKiCadFootprintImportView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) package_uuid: Uuid,
    pub(crate) footprint_uuid: Uuid,
    pub(crate) package_path: String,
    pub(crate) footprint_path: String,
    pub(crate) padstack_count: usize,
    pub(crate) pool_path: String,
    pub(crate) import_key: String,
    pub(crate) import_map_path: String,
    pub(crate) reused_existing_identity: bool,
}

pub(crate) fn import_native_project_kicad_footprint(
    root: &Path,
    source: &Path,
    pool_path: &str,
) -> Result<NativeProjectKiCadFootprintImportView> {
    validate_project_local_pool_path(pool_path)?;
    let before = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let (imported, report) = import_footprint_document_with_import_map(source, &before.import_map)
        .with_context(|| format!("failed to import KiCad footprint {}", source.display()))?;
    let package_uuid = imported.package.uuid;
    let footprint_uuid = imported.footprint.uuid;
    let reused_existing_identity = report
        .metadata
        .get("reused_existing_identity")
        .map(|value| value == "true")
        .unwrap_or(false);
    let project_manifest: NativeProjectManifest = serde_json::from_value(
        before
            .materialized_source_shard_value(SourceShardKind::ProjectManifest)
            .context("failed to materialize project manifest")?,
    )
    .context("failed to parse resolver-materialized project manifest")?;
    let pool_root = resolve_native_project_pool_path(root, pool_path);
    let package_relative_path = format!("{pool_path}/packages/{package_uuid}.json");
    let footprint_relative_path = format!("{pool_path}/footprints/{footprint_uuid}.json");
    let import_key = footprint_package_import_key(source);
    let import_map_relative_path = format!(".datum/import_map/kicad-footprint-{package_uuid}.json");
    let mut operations = Vec::new();
    if !project_manifest
        .pools
        .iter()
        .any(|pool| pool.path == pool_path)
    {
        operations.push(Operation::AddProjectPoolRef {
            path: pool_path.to_string(),
            priority: next_pool_priority(&project_manifest.pools),
        });
    }
    if !reused_existing_identity {
        let mut footprint_value = serde_json::to_value(&imported.footprint)?;
        if let Some(document) = footprint_value.as_object_mut() {
            document.insert("schema_version".to_string(), serde_json::json!(1));
        }
        let source_hash = compute_source_hash_file(source)?;
        for padstack in imported.padstacks {
            operations.push(Operation::CreatePoolPadstack {
                padstack_id: padstack.uuid,
                relative_path: format!("{pool_path}/padstacks/{}.json", padstack.uuid),
                padstack: serde_json::to_value(padstack)?,
            });
        }
        operations.push(Operation::CreatePoolPackage {
            package_id: package_uuid,
            relative_path: package_relative_path.clone(),
            package: serde_json::to_value(&imported.package)?,
        });
        operations.push(Operation::CreatePoolLibraryObject {
            object_id: footprint_uuid,
            relative_path: footprint_relative_path.clone(),
            object_kind: "footprints".to_string(),
            object: footprint_value,
        });
        operations.push(Operation::CreateImportMapShard {
            relative_path: import_map_relative_path.clone(),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries: vec![ImportMapEntry {
                    import_key: import_key.clone(),
                    object_id: package_uuid,
                    source_shard_id: source_shard_id_for_relative_path(&package_relative_path),
                    status: eda_engine::substrate::ImportMapEntryStatus::Active,
                    source_tool: "kicad".to_string(),
                    source_path: source.display().to_string(),
                    source_object_ref: import_key.clone(),
                    source_hash,
                }],
            })?,
        });
    }
    if !operations.is_empty() {
        let mut model = before;
        model.commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: format!("import KiCad footprint {}", source.display()),
                },
                operations,
            },
        )?;
    }

    let after_write = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve imported pool package {}", root.display()))?;
    after_write.objects.get(&package_uuid).ok_or_else(|| {
        anyhow::anyhow!("imported package {package_uuid} was not resolver-visible")
    })?;
    after_write.objects.get(&footprint_uuid).ok_or_else(|| {
        anyhow::anyhow!("imported footprint {footprint_uuid} was not resolver-visible")
    })?;

    Ok(NativeProjectKiCadFootprintImportView {
        contract: "native_project_kicad_footprint_import_v1",
        project_id: after_write.project.project_id,
        package_uuid,
        footprint_uuid,
        package_path: pool_root
            .join("packages")
            .join(format!("{package_uuid}.json"))
            .display()
            .to_string(),
        footprint_path: pool_root
            .join("footprints")
            .join(format!("{footprint_uuid}.json"))
            .display()
            .to_string(),
        padstack_count: report.counts.padstacks,
        pool_path: pool_path.to_string(),
        import_key,
        import_map_path: root.join(import_map_relative_path).display().to_string(),
        reused_existing_identity,
    })
}
