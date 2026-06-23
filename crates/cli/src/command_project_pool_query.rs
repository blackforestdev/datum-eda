use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{DesignModel, ProjectResolver, SourceShardKind};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{LoadedNativeProject, NativeProjectInspectPoolRefView, load_native_project};

pub(super) fn collect_native_project_pool_ref_views(
    project: &LoadedNativeProject,
) -> Vec<NativeProjectInspectPoolRefView> {
    project
        .manifest
        .pools
        .iter()
        .map(|pool_ref| {
            let resolved_path =
                super::resolve_native_project_pool_path(&project.root, &pool_ref.path);
            NativeProjectInspectPoolRefView {
                manifest_path: pool_ref.path.clone(),
                priority: pool_ref.priority,
                resolved_path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            }
        })
        .collect()
}

pub(crate) fn query_native_project_pools(
    root: &Path,
) -> Result<Vec<NativeProjectInspectPoolRefView>> {
    let project = load_native_project(root)?;
    Ok(collect_native_project_pool_ref_views(&project))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectLibraryObjectsQueryView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) model_revision: String,
    pub(crate) object_count: usize,
    pub(crate) objects: Vec<NativeProjectLibraryObjectView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectLibraryObjectView {
    pub(crate) object_uuid: Uuid,
    pub(crate) object_kind: String,
    pub(crate) pool_path: String,
    pub(crate) relative_path: String,
    pub(crate) object_revision: u64,
    pub(crate) schema_version: Option<u64>,
    pub(crate) source_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolModelsQueryView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) model_revision: String,
    pub(crate) model_count: usize,
    pub(crate) models: Vec<NativeProjectPoolModelView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolModelView {
    pub(crate) pool_path: String,
    pub(crate) role: String,
    pub(crate) relative_path: String,
    pub(crate) sha256: String,
    pub(crate) computed_sha256: String,
    pub(crate) hash_matches: bool,
    pub(crate) extension: String,
    pub(crate) size_bytes: u64,
    pub(crate) model_uuid: Uuid,
    pub(crate) referenced: bool,
    pub(crate) orphaned: bool,
    pub(crate) attachment_count: usize,
    pub(crate) attachments: Vec<NativeProjectPoolModelAttachmentRefView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolModelAttachmentRefView {
    pub(crate) part_uuid: Uuid,
    pub(crate) attachment_uuid: Uuid,
    pub(crate) relative_path: String,
}

pub(crate) fn query_native_project_pool_library_objects(
    root: &Path,
    pool_filter: Option<&str>,
    kind_filter: Option<&str>,
    object_filter: Option<Uuid>,
    include_payload: bool,
) -> Result<NativeProjectLibraryObjectsQueryView> {
    let model = ProjectResolver::new(root).resolve()?;
    let mut objects = Vec::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::Pool)
    {
        let Some((pool_path, object_kind, object_uuid)) =
            parse_pool_library_object_path(&shard.relative_path)
        else {
            continue;
        };
        if pool_filter.is_some_and(|pool| pool != pool_path)
            || kind_filter.is_some_and(|kind| kind != object_kind)
            || object_filter.is_some_and(|object| object != object_uuid)
        {
            continue;
        }
        let Some(object) = model.objects.get(&object_uuid) else {
            continue;
        };
        if object.source_shard_id != shard.shard_id || object.kind != object_kind {
            continue;
        }
        let payload = if include_payload {
            Some(model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?)
        } else {
            None
        };
        objects.push(NativeProjectLibraryObjectView {
            object_uuid,
            object_kind: object_kind.to_string(),
            pool_path: pool_path.to_string(),
            relative_path: shard.relative_path.clone(),
            object_revision: object.object_revision.0,
            schema_version: shard.schema_version,
            source_hash: shard.content_hash.clone(),
            payload,
        });
    }
    Ok(NativeProjectLibraryObjectsQueryView {
        contract: "native_project_library_objects_query_v1",
        project_id: model.project.project_id,
        model_revision: model.model_revision.0,
        object_count: objects.len(),
        objects,
    })
}

pub(crate) fn query_native_project_pool_models(
    root: &Path,
    pool_filter: Option<&str>,
    role_filter: Option<&str>,
    sha_filter: Option<&str>,
) -> Result<NativeProjectPoolModelsQueryView> {
    let model = ProjectResolver::new(root).resolve()?;
    let project = load_native_project(root)?;
    let attachment_refs = collect_model_attachment_refs(&model)?;
    let mut models = Vec::new();
    for pool_ref in &project.manifest.pools {
        let pool_path = pool_ref.path.as_str();
        if pool_filter.is_some_and(|pool| pool != pool_path) {
            continue;
        }
        let models_dir = root.join(pool_path).join("models");
        if !models_dir.exists() {
            continue;
        }
        for role_entry in fs::read_dir(&models_dir)
            .with_context(|| format!("failed to read model directory {}", models_dir.display()))?
        {
            let role_entry = role_entry?;
            let role_path = role_entry.path();
            if !role_path.is_dir() {
                continue;
            }
            let role = role_entry.file_name().to_string_lossy().to_string();
            if role_filter.is_some_and(|filter| filter != role) {
                continue;
            }
            for model_entry in fs::read_dir(&role_path).with_context(|| {
                format!(
                    "failed to read model role directory {}",
                    role_path.display()
                )
            })? {
                let model_entry = model_entry?;
                let path = model_entry.path();
                if !path.is_file() {
                    continue;
                }
                let Some((sha256, extension)) = parse_model_blob_filename(&path) else {
                    continue;
                };
                if sha_filter.is_some_and(|filter| filter != sha256) {
                    continue;
                }
                let bytes = fs::read(&path)
                    .with_context(|| format!("failed to read model blob {}", path.display()))?;
                let computed_sha256 = format!("{:x}", Sha256::digest(&bytes));
                let relative_path = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let model_uuid = Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("datum-eda:pool-model:{sha256}").as_bytes(),
                );
                let hash_matches = sha256 == computed_sha256;
                let attachments = attachment_refs
                    .get(&sha256.to_string())
                    .cloned()
                    .unwrap_or_default();
                let referenced = !attachments.is_empty();
                models.push(NativeProjectPoolModelView {
                    pool_path: pool_path.to_string(),
                    role: role.clone(),
                    relative_path,
                    sha256: sha256.to_string(),
                    computed_sha256,
                    hash_matches,
                    extension: extension.to_string(),
                    size_bytes: bytes.len() as u64,
                    model_uuid,
                    referenced,
                    orphaned: !referenced,
                    attachment_count: attachments.len(),
                    attachments,
                });
            }
        }
    }
    models.sort_by(|a, b| {
        a.pool_path
            .cmp(&b.pool_path)
            .then_with(|| a.role.cmp(&b.role))
            .then_with(|| a.sha256.cmp(&b.sha256))
            .then_with(|| a.extension.cmp(&b.extension))
    });
    Ok(NativeProjectPoolModelsQueryView {
        contract: "native_project_pool_models_query_v1",
        project_id: model.project.project_id,
        model_revision: model.model_revision.0,
        model_count: models.len(),
        models,
    })
}

fn collect_model_attachment_refs(
    model: &DesignModel,
) -> Result<std::collections::BTreeMap<String, Vec<NativeProjectPoolModelAttachmentRefView>>> {
    let mut refs: std::collections::BTreeMap<String, Vec<NativeProjectPoolModelAttachmentRefView>> =
        std::collections::BTreeMap::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|shard| shard.kind == SourceShardKind::Pool)
    {
        let Some((_, object_kind, object_uuid)) =
            parse_pool_library_object_path(&shard.relative_path)
        else {
            continue;
        };
        if object_kind != "parts" {
            continue;
        }
        let payload =
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let Some(models) = payload
            .get("behavioural_models")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for attachment in models {
            let Some(sha256) = attachment
                .get("provenance")
                .and_then(|provenance| provenance.get("sha256"))
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let Some(attachment_uuid) = attachment
                .get("uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|uuid| Uuid::parse_str(uuid).ok())
            else {
                continue;
            };
            refs.entry(sha256.to_string()).or_default().push(
                NativeProjectPoolModelAttachmentRefView {
                    part_uuid: object_uuid,
                    attachment_uuid,
                    relative_path: shard.relative_path.clone(),
                },
            );
        }
    }
    for attachment_refs in refs.values_mut() {
        attachment_refs.sort_by(|a, b| {
            a.part_uuid
                .cmp(&b.part_uuid)
                .then_with(|| a.attachment_uuid.cmp(&b.attachment_uuid))
        });
    }
    Ok(refs)
}

fn parse_model_blob_filename(path: &Path) -> Option<(&str, &str)> {
    let file_name = path.file_name()?.to_str()?;
    let (sha256, extension) = file_name.rsplit_once('.')?;
    if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    if extension.is_empty() {
        return None;
    }
    Some((sha256, extension))
}

fn parse_pool_library_object_path(relative_path: &str) -> Option<(&str, &str, Uuid)> {
    let parts = relative_path.split('/').collect::<Vec<_>>();
    if parts.len() < 3 || !parts.last()?.ends_with(".json") {
        return None;
    }
    let object_file = parts.last()?.strip_suffix(".json")?;
    let object_uuid = Uuid::parse_str(object_file).ok()?;
    let object_kind = parts.get(parts.len() - 2)?;
    let pool_path_end = relative_path.rfind(&format!("/{object_kind}/{object_file}.json"))?;
    Some((&relative_path[..pool_path_end], object_kind, object_uuid))
}
