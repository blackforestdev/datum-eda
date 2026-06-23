use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    footprint_package_import_key, import_footprint_document_with_import_map,
};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ImportMapEntry, ImportMapShard, Operation, OperationBatch,
    ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

use super::{load_native_project, resolve_native_project_pool_path};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectKiCadFootprintImportView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) package_uuid: Uuid,
    pub(crate) package_path: String,
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
    let reused_existing_identity = report
        .metadata
        .get("reused_existing_identity")
        .map(|value| value == "true")
        .unwrap_or(false);
    let project = load_native_project(root)?;
    let pool_root = resolve_native_project_pool_path(root, pool_path);
    let package_relative_path = format!("{pool_path}/packages/{package_uuid}.json");
    let package_path = pool_root
        .join("packages")
        .join(format!("{package_uuid}.json"));
    let import_key = footprint_package_import_key(source);
    let import_map_relative_path = format!(".datum/import_map/kicad-footprint-{package_uuid}.json");
    let import_map_path = root.join(&import_map_relative_path);
    let mut operations = Vec::new();
    if !project
        .manifest
        .pools
        .iter()
        .any(|pool| pool.path == pool_path)
    {
        operations.push(Operation::AddProjectPoolRef {
            path: pool_path.to_string(),
            priority: next_pool_priority(&project.manifest.pools),
        });
    }
    if !reused_existing_identity {
        let package_value = serde_json::to_value(&imported.package)?;
        let package_source_shard_id = source_shard_id_for_relative_path(&package_relative_path);
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
            package: package_value,
        });
        operations.push(Operation::CreateImportMapShard {
            relative_path: import_map_relative_path.clone(),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries: vec![ImportMapEntry {
                    import_key: import_key.clone(),
                    object_id: package_uuid,
                    source_shard_id: package_source_shard_id,
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

    Ok(NativeProjectKiCadFootprintImportView {
        contract: "native_project_kicad_footprint_import_v1",
        project_id: after_write.project.project_id,
        package_uuid,
        package_path: package_path.display().to_string(),
        padstack_count: report.counts.padstacks,
        pool_path: pool_path.to_string(),
        import_key,
        import_map_path: import_map_path.display().to_string(),
        reused_existing_identity,
    })
}

fn validate_project_local_pool_path(pool_path: &str) -> Result<()> {
    let path = PathBuf::from(pool_path);
    if pool_path.trim().is_empty() || path.is_absolute() {
        bail!("project pool path must be a non-empty relative path");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        bail!("project pool path must not contain parent-directory components");
    }
    Ok(())
}

fn next_pool_priority(pools: &[super::NativeProjectPoolRef]) -> u32 {
    pools.iter().map(|pool| pool.priority).max().unwrap_or(0) + 1
}

fn source_shard_id_for_relative_path(relative_path: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    )
}
