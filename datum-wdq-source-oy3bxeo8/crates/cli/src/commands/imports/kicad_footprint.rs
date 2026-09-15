use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::imports::{
    build_kicad_footprint_import, kicad_footprint_import_map_relative_path,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    footprint_package_import_key, import_footprint_document_with_import_map,
};
use eda_engine::substrate::{ImportMapEntry, ProjectResolver};
use serde::Serialize;
use uuid::Uuid;

use super::imports::{source_shard_id_for_relative_path, validate_project_local_pool_path};
use crate::cli_commit_source;
use crate::resolve_native_project_pool_path;

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
    let pool_root = resolve_native_project_pool_path(root, pool_path);
    let package_relative_path = format!("{pool_path}/packages/{package_uuid}.json");
    let import_key = footprint_package_import_key(source);
    let import_map_relative_path = kicad_footprint_import_map_relative_path(package_uuid);
    let import_map_entries = if reused_existing_identity {
        Vec::new()
    } else {
        let source_hash = compute_source_hash_file(source)?;
        vec![ImportMapEntry {
            import_key: import_key.clone(),
            object_id: package_uuid,
            source_shard_id: source_shard_id_for_relative_path(&package_relative_path),
            status: eda_engine::substrate::ImportMapEntryStatus::Active,
            source_tool: "kicad".to_string(),
            source_path: source.display().to_string(),
            source_object_ref: import_key.clone(),
            source_hash,
        }]
    };
    let write = build_kicad_footprint_import(
        &before,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            format!("import KiCad footprint {}", source.display()),
        ),
        pool_path,
        &imported,
        reused_existing_identity,
        import_map_entries,
    )?;
    if let Some(prepared) = write.prepared {
        let mut model = before;
        commit_prepared(&mut model, root, prepared)?;
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
