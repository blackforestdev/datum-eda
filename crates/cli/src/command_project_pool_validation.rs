use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
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
            "padstacks",
            "pin_pad_maps",
        ] {
            read_pool_kind(root, &pool_root, &pool.path, kind, &mut graph, issues)?;
        }
        read_pool_model_blobs(root, &pool_root, &mut graph, issues)?;
    }
    validate_pool_refs(&graph, issues);
    Ok(())
}

#[derive(Default)]
struct PoolValidationGraph {
    units: BTreeMap<Uuid, serde_json::Value>,
    symbols: BTreeMap<Uuid, serde_json::Value>,
    entities: BTreeMap<Uuid, serde_json::Value>,
    parts: BTreeMap<Uuid, serde_json::Value>,
    packages: BTreeMap<Uuid, serde_json::Value>,
    padstacks: BTreeMap<Uuid, serde_json::Value>,
    pin_pad_maps: BTreeMap<Uuid, serde_json::Value>,
    model_blobs: BTreeMap<String, PoolModelBlob>,
    seen: BTreeMap<Uuid, String>,
    subjects: BTreeMap<Uuid, String>,
}

#[derive(Debug, Clone)]
struct PoolModelBlob {
    model_uuid: Uuid,
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
    if let Some(previous) = graph.seen.insert(object_id, subject.to_string()) {
        push_issue(
            issues,
            "error",
            "duplicate_uuid_within_type",
            subject.to_string(),
            format!("pool object UUID {object_id} already appeared at {previous}"),
        );
    }
    graph.subjects.insert(object_id, subject.to_string());
    match kind {
        "units" => {
            graph.units.insert(object_id, value.clone());
        }
        "symbols" => {
            graph.symbols.insert(object_id, value.clone());
        }
        "entities" => {
            graph.entities.insert(object_id, value.clone());
        }
        "parts" => {
            graph.parts.insert(object_id, value.clone());
        }
        "packages" => {
            graph.packages.insert(object_id, value.clone());
        }
        "padstacks" => {
            graph.padstacks.insert(object_id, value.clone());
        }
        "pin_pad_maps" => {
            graph.pin_pad_maps.insert(object_id, value.clone());
        }
        _ => {}
    }
}

fn validate_pool_refs(
    graph: &PoolValidationGraph,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    for (symbol_id, symbol) in &graph.symbols {
        let subject = graph.subjects.get(symbol_id).cloned().unwrap_or_default();
        validate_uuid_ref(
            symbol,
            "unit",
            &graph.units.keys().copied().collect(),
            &subject,
            "symbol",
            issues,
        );
    }
    for (entity_id, entity) in &graph.entities {
        let subject = graph.subjects.get(entity_id).cloned().unwrap_or_default();
        let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (gate_key, gate) in gates {
            let gate_subject = format!("{subject}#gates/{gate_key}");
            validate_uuid_ref(
                gate,
                "unit",
                &graph.units.keys().copied().collect(),
                &gate_subject,
                "entity gate",
                issues,
            );
            validate_uuid_ref(
                gate,
                "symbol",
                &graph.symbols.keys().copied().collect(),
                &gate_subject,
                "entity gate",
                issues,
            );
        }
    }
    for (part_id, part) in &graph.parts {
        let subject = graph.subjects.get(part_id).cloned().unwrap_or_default();
        let package_id = parse_uuid_field(part, "package", &subject, "part", issues);
        validate_uuid_ref(
            part,
            "entity",
            &graph.entities.keys().copied().collect(),
            &subject,
            "part",
            issues,
        );
        validate_uuid_ref(
            part,
            "package",
            &graph.packages.keys().copied().collect(),
            &subject,
            "part",
            issues,
        );
        if let Some(pad_map) = part.get("pad_map").and_then(serde_json::Value::as_object) {
            for (map_key, entry) in pad_map {
                let map_subject = format!("{subject}#pad_map/{map_key}");
                if let Some(package_id) = package_id {
                    validate_part_pad_map_key(map_key, package_id, graph, &map_subject, issues);
                }
                validate_part_pad_map_entry(entry, part, graph, &map_subject, issues);
            }
        }
        validate_part_model_attachments(part, graph, &subject, issues);
    }
    for (package_id, package) in &graph.packages {
        let subject = graph.subjects.get(package_id).cloned().unwrap_or_default();
        let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (pad_key, pad) in pads {
            let pad_subject = format!("{subject}#pads/{pad_key}");
            validate_uuid_ref(
                pad,
                "padstack",
                &graph.padstacks.keys().copied().collect(),
                &pad_subject,
                "package pad",
                issues,
            );
        }
    }
    for (map_id, map) in &graph.pin_pad_maps {
        let subject = graph.subjects.get(map_id).cloned().unwrap_or_default();
        validate_uuid_ref(
            map,
            "part",
            &graph.parts.keys().copied().collect(),
            &subject,
            "pin_pad_map",
            issues,
        );
    }
}

fn validate_part_model_attachments(
    part: &serde_json::Value,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(models) = part
        .get("behavioural_models")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };
    for (index, attachment) in models.iter().enumerate() {
        let attachment_subject = format!("{subject}#behavioural_models/{index}");
        let Some(model_uuid) = parse_uuid_field(
            attachment,
            "model_uuid",
            &attachment_subject,
            "model attachment",
            issues,
        ) else {
            continue;
        };
        let Some(provenance) = attachment.get("provenance") else {
            continue;
        };
        if provenance.is_null() {
            continue;
        }
        let Some(sha256) = provenance.get("sha256").and_then(serde_json::Value::as_str) else {
            push_issue(
                issues,
                "error",
                "missing_required_field",
                attachment_subject,
                "model attachment provenance missing sha256",
            );
            continue;
        };
        let Some(blob) = graph.model_blobs.get(sha256) else {
            push_issue(
                issues,
                "error",
                "missing_model_blob",
                attachment_subject,
                format!("model attachment references missing pool model blob sha256 {sha256}"),
            );
            continue;
        };
        if model_uuid != blob.model_uuid {
            push_issue(
                issues,
                "error",
                "bad_deterministic_model_uuid",
                attachment_subject,
                "model attachment model_uuid does not match deterministic UUID",
            );
        }
    }
}

fn validate_part_pad_map_key(
    map_key: &str,
    package_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(pad_id) = Uuid::parse_str(map_key).ok() else {
        push_issue(
            issues,
            "error",
            "invalid_uuid_key",
            subject.to_string(),
            format!("part pad_map key `{map_key}` is not a valid UUID"),
        );
        return;
    };
    let Some(package) = graph.packages.get(&package_id) else {
        return;
    };
    let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
        return;
    };
    if !pads.contains_key(&pad_id.to_string()) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map key {pad_id} references missing package pad"),
        );
    }
}

fn validate_part_pad_map_entry(
    entry: &serde_json::Value,
    part: &serde_json::Value,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let gate_id = validate_nested_uuid_field(entry, "gate", subject, "part pad_map", issues);
    let pin_id = validate_nested_uuid_field(entry, "pin", subject, "part pad_map", issues);
    let Some(entity_id) = parse_uuid_field(part, "entity", subject, "part", issues) else {
        return;
    };
    let Some(entity) = graph.entities.get(&entity_id) else {
        return;
    };
    let Some(gate_id) = gate_id else {
        return;
    };
    let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
        return;
    };
    let Some(gate) = gates.get(&gate_id.to_string()) else {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map references missing entity gate {gate_id}"),
        );
        return;
    };
    let Some(unit_id) = parse_uuid_field(gate, "unit", subject, "entity gate", issues) else {
        return;
    };
    if !graph.units.contains_key(&unit_id) {
        return;
    }
    if let Some(pin_id) = pin_id {
        validate_unit_pin_exists(unit_id, pin_id, graph, subject, issues);
    }
}

fn validate_unit_pin_exists(
    unit_id: Uuid,
    pin_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(unit) = graph.units.get(&unit_id) else {
        return;
    };
    let Some(pins) = unit.get("pins").and_then(serde_json::Value::as_object) else {
        return;
    };
    if !pins.contains_key(&pin_id.to_string()) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map references missing unit pin {pin_id}"),
        );
    }
}

fn validate_uuid_ref(
    value: &serde_json::Value,
    field: &str,
    allowed: &BTreeSet<Uuid>,
    subject: &str,
    label: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(reference) = value.get(field) else {
        return;
    };
    let Some(reference) = parse_uuid_value(reference, subject, field, issues) else {
        return;
    };
    if !allowed.contains(&reference) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("{label} references missing {field} {reference}"),
        );
    }
}

fn validate_nested_uuid_field(
    value: &serde_json::Value,
    field: &str,
    subject: &str,
    label: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    if let Some(reference) = value.get(field) {
        parse_uuid_value(reference, subject, field, issues)
    } else {
        push_issue(
            issues,
            "error",
            "missing_required_field",
            subject.to_string(),
            format!("{label} missing {field}"),
        );
        None
    }
}

fn parse_uuid_field(
    value: &serde_json::Value,
    field: &str,
    subject: &str,
    label: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    if let Some(reference) = value.get(field) {
        parse_uuid_value(reference, subject, field, issues)
    } else {
        push_issue(
            issues,
            "error",
            "missing_required_field",
            subject.to_string(),
            format!("{label} missing {field}"),
        );
        None
    }
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
