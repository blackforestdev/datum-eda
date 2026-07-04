use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::pool::{LibraryGraph as PoolValidationGraph, LibraryModelBlob as PoolModelBlob};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    NativeProjectPoolRef, NativeProjectValidationIssueView, resolve_native_project_pool_path,
};

pub(crate) fn validate_native_project_pools(
    root: &Path,
    pools: &[NativeProjectPoolRef],
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()> {
    let mut graph = PoolValidationGraph::default();
    for pool in pools {
        let Some(pool_root) = validate_pool_path(root, &pool.path, issues) else {
            continue;
        };
        for kind in [
            "units",
            "symbols",
            "entities",
            "parts",
            "packages",
            "footprints",
            "padstacks",
            "pin_pad_maps",
        ] {
            read_pool_kind(root, &pool_root, &pool.path, kind, &mut graph, issues)?;
        }
        read_pool_model_blobs(root, &pool_root, &mut graph, issues)?;
    }
    for diagnostic in graph.validation_report().diagnostics {
        push_issue(
            issues,
            diagnostic.severity,
            diagnostic.code,
            diagnostic.subject,
            diagnostic.message,
        );
    }
    Ok(())
}

fn validate_pool_path(
    root: &Path,
    pool_path: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<PathBuf> {
    let path = PathBuf::from(pool_path);
    if pool_path.trim().is_empty() {
        push_issue(
            issues,
            "error",
            "invalid_pool_path",
            pool_path.to_string(),
            "pool path must be non-empty",
        );
        return None;
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        push_issue(
            issues,
            "error",
            "invalid_pool_path",
            pool_path.to_string(),
            "pool path must not contain parent-directory components",
        );
        return None;
    }
    let pool_root = resolve_native_project_pool_path(root, pool_path);
    if !pool_root.exists() {
        push_issue(
            issues,
            "error",
            "missing_pool_directory",
            pool_path.to_string(),
            "declared pool path does not exist",
        );
        return None;
    }
    if !pool_root.is_dir() {
        push_issue(
            issues,
            "error",
            "invalid_pool_path",
            pool_path.to_string(),
            "pool path exists but is not a directory",
        );
        return None;
    }
    Some(pool_root)
}

fn read_pool_kind(
    root: &Path,
    pool_root: &Path,
    pool_path: &str,
    kind: &str,
    graph: &mut PoolValidationGraph,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()> {
    let directory = pool_root.join(kind);
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let subject = format!("{pool_path}/{kind}/{filename}");
        let value = match read_json_value(&path) {
            Ok(value) => value,
            Err(error) => {
                push_issue(
                    issues,
                    "error",
                    "invalid_json",
                    relative_subject(root, &path),
                    format!("failed to parse pool {kind} object: {error:#}"),
                );
                continue;
            }
        };
        validate_pool_object(root, &path, &subject, kind, &value, graph, issues);
    }
    Ok(())
}

fn read_pool_model_blobs(
    root: &Path,
    pool_root: &Path,
    graph: &mut PoolValidationGraph,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()> {
    let models_root = pool_root.join("models");
    let Ok(role_entries) = std::fs::read_dir(&models_root) else {
        return Ok(());
    };
    for role_entry in role_entries.filter_map(|entry| entry.ok()) {
        let role_path = role_entry.path();
        if !role_path.is_dir() {
            continue;
        }
        let mut model_paths = std::fs::read_dir(&role_path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        model_paths.sort();
        for path in model_paths {
            let subject = relative_subject(root, &path);
            let Some((sha256, _)) = parse_model_blob_filename(&path) else {
                push_issue(
                    issues,
                    "error",
                    "invalid_pool_model_filename",
                    subject,
                    "pool model filename must be <sha256>.<ext>",
                );
                continue;
            };
            let bytes = std::fs::read(&path)?;
            let computed_sha256 = format!("{:x}", Sha256::digest(&bytes));
            if sha256 != computed_sha256 {
                push_issue(
                    issues,
                    "error",
                    "model_blob_hash_mismatch",
                    relative_subject(root, &path),
                    "pool model filename hash does not match file bytes",
                );
            }
            graph
                .model_blobs
                .entry(sha256.to_string())
                .or_insert(PoolModelBlob {
                    model_uuid: deterministic_model_uuid(sha256),
                });
        }
    }
    Ok(())
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

fn deterministic_model_uuid(sha256: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:pool-model:{sha256}").as_bytes(),
    )
}

fn validate_pool_object(
    root: &Path,
    path: &Path,
    subject: &str,
    kind: &str,
    value: &serde_json::Value,
    graph: &mut PoolValidationGraph,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    match value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => {}
        Some(version) => push_issue(
            issues,
            "error",
            "invalid_schema_version",
            relative_subject(root, path),
            format!("unsupported schema_version {version}; expected 1"),
        ),
        None => push_issue(
            issues,
            "error",
            "missing_schema_version",
            relative_subject(root, path),
            "pool object missing schema_version",
        ),
    }
    let Some(object_id) = parse_payload_uuid(value, subject, issues) else {
        return;
    };
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if filename != format!("{object_id}.json") {
        push_issue(
            issues,
            "error",
            "uuid_filename_mismatch",
            relative_subject(root, path),
            format!("pool {kind} filename {filename} does not match payload UUID {object_id}"),
        );
    }
    graph.insert_pool_object(kind, object_id, subject.to_string(), value.clone());
}

fn parse_payload_uuid(
    value: &serde_json::Value,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    let Some(uuid) = value.get("uuid") else {
        push_issue(
            issues,
            "error",
            "missing_required_field",
            subject.to_string(),
            "pool object missing uuid",
        );
        return None;
    };
    parse_uuid_value(uuid, subject, "uuid", issues)
}

fn parse_uuid_value(
    value: &serde_json::Value,
    subject: &str,
    field: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    match value.as_str().and_then(|value| Uuid::parse_str(value).ok()) {
        Some(uuid) => Some(uuid),
        None => {
            push_issue(
                issues,
                "error",
                "invalid_uuid",
                subject.to_string(),
                format!("field {field} is not a valid UUID"),
            );
            None
        }
    }
}

fn read_json_value(path: &Path) -> Result<serde_json::Value> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn relative_subject(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn push_issue(
    issues: &mut Vec<NativeProjectValidationIssueView>,
    severity: &str,
    code: &str,
    subject: String,
    message: impl Into<String>,
) {
    issues.push(NativeProjectValidationIssueView {
        severity: severity.to_string(),
        code: code.to_string(),
        subject,
        message: message.into(),
    });
}
