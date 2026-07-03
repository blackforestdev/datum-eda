use std::path::{Component, Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
    derive_pool_model_attachment_uuid, derive_pool_model_uuid, pool_entity_payload,
    pool_padstack_payload, pool_part_payload, pool_symbol_payload, pool_unit_payload,
    write_pool_model_blob,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::command_project_library_payload::{
    read_pool_library_object_payload, read_project_pool_object_payload,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolLibraryObjectMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) object_uuid: Uuid,
    pub(crate) object_kind: String,
    pub(crate) pool_path: String,
    pub(crate) relative_path: String,
    pub(crate) object_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolModelGcView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) pool_path: String,
    pub(crate) applied: bool,
    pub(crate) planned_count: usize,
    pub(crate) deleted_count: usize,
    pub(crate) skipped_count: usize,
    pub(crate) entries: Vec<NativeProjectPoolModelGcEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPoolModelGcEntryView {
    pub(crate) role: String,
    pub(crate) sha256: String,
    pub(crate) relative_path: String,
    pub(crate) size_bytes: u64,
    pub(crate) deleted: bool,
    pub(crate) skipped_reason: Option<String>,
}

pub(crate) fn create_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source: &Path,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let object = read_pool_library_object_payload(source, object_id)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    commit_pool_library_operations(
        root,
        format!("create native pool library object {object_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}

pub(crate) fn create_native_project_pool_unit(
    root: &Path,
    pool_path: &str,
    unit_id: Uuid,
    name: String,
    manufacturer: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let relative_path = pool_library_relative_path(pool_path, "units", unit_id);
    let object = pool_unit_payload(unit_id, &name, &manufacturer);
    commit_pool_library_operations(
        root,
        format!("create native pool unit {unit_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "units", unit_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_unit",
        pool_path,
        "units",
        unit_id,
        &relative_path,
    )
}

pub(crate) fn create_native_project_pool_symbol(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    unit_id: Uuid,
    name: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let unit = model
        .objects
        .get(&unit_id)
        .filter(|object| object.domain == "pool" && object.kind == "units");
    if unit.is_none() {
        bail!("missing pool unit {unit_id} for symbol {symbol_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let object = pool_symbol_payload(symbol_id, unit_id, &name);
    commit_pool_library_operations(
        root,
        format!("create native pool symbol {symbol_id} for unit {unit_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_symbol",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

pub(crate) fn create_native_project_pool_entity(
    root: &Path,
    pool_path: &str,
    entity_id: Uuid,
    gate_id: Uuid,
    unit_id: Uuid,
    symbol_id: Uuid,
    name: String,
    prefix: String,
    manufacturer: String,
    gate_name: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&unit_id)
        .filter(|object| object.domain == "pool" && object.kind == "units")
        .is_none()
    {
        bail!("missing pool unit {unit_id} for entity {entity_id}");
    }
    let symbol_object = model
        .objects
        .get(&symbol_id)
        .filter(|object| object.domain == "pool" && object.kind == "symbols")
        .ok_or_else(|| anyhow::anyhow!("missing pool symbol {symbol_id} for entity {entity_id}"))?;
    let symbol_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == symbol_object.source_shard_id)
        .ok_or_else(|| anyhow::anyhow!("missing source shard for pool symbol {symbol_id}"))?;
    let symbol =
        model.materialized_source_shard_value_by_relative_path(&symbol_shard.relative_path)?;
    if symbol.get("unit").and_then(serde_json::Value::as_str) != Some(unit_id.to_string().as_str())
    {
        bail!("pool symbol {symbol_id} does not reference unit {unit_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "entities", entity_id);
    let object = pool_entity_payload(
        entity_id,
        gate_id,
        unit_id,
        symbol_id,
        &name,
        &prefix,
        &manufacturer,
        &gate_name,
    );
    commit_pool_library_operations(
        root,
        format!("create native pool entity {entity_id} for unit {unit_id} and symbol {symbol_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "entities", entity_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_entity",
        pool_path,
        "entities",
        entity_id,
        &relative_path,
    )
}

pub(crate) fn create_native_project_pool_padstack(
    root: &Path,
    pool_path: &str,
    padstack_id: Uuid,
    name: String,
    aperture: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    drill_nm: Option<i64>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if drill_nm.is_some_and(|value| value <= 0) {
        bail!("padstack drill-nm must be positive");
    }
    let aperture_value = match aperture.as_deref() {
        None => {
            if diameter_nm.is_some() || width_nm.is_some() || height_nm.is_some() {
                bail!("padstack aperture dimensions require --aperture circle or rect");
            }
            serde_json::Value::Null
        }
        Some("circle") => {
            if width_nm.is_some() || height_nm.is_some() {
                bail!("circle padstack aperture does not accept width-nm or height-nm");
            }
            let diameter = positive_required_dimension(diameter_nm, "diameter-nm")?;
            serde_json::json!({"circle": {"diameter_nm": diameter}})
        }
        Some("rect") => {
            if diameter_nm.is_some() {
                bail!("rect padstack aperture does not accept diameter-nm");
            }
            let width = positive_required_dimension(width_nm, "width-nm")?;
            let height = positive_required_dimension(height_nm, "height-nm")?;
            serde_json::json!({"rect": {"width_nm": width, "height_nm": height}})
        }
        Some(kind) => bail!("unsupported padstack aperture {kind}; expected circle or rect"),
    };
    let relative_path = pool_library_relative_path(pool_path, "padstacks", padstack_id);
    let object = pool_padstack_payload(padstack_id, &name, aperture_value, drill_nm);
    commit_pool_library_operations(
        root,
        format!("create native pool padstack {padstack_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "padstacks", padstack_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_padstack",
        pool_path,
        "padstacks",
        padstack_id,
        &relative_path,
    )
}

pub(crate) fn create_native_project_pool_part(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    entity_id: Uuid,
    package_id: Uuid,
    mpn: String,
    manufacturer: String,
    value: String,
    description: String,
    datasheet: String,
    lifecycle: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let lifecycle = validate_part_lifecycle(lifecycle)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&entity_id)
        .filter(|object| object.domain == "pool" && object.kind == "entities")
        .is_none()
    {
        bail!("missing pool entity {entity_id} for part {part_id}");
    }
    if model
        .objects
        .get(&package_id)
        .filter(|object| object.domain == "pool" && object.kind == "packages")
        .is_none()
    {
        bail!("missing pool package {package_id} for part {part_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let object = pool_part_payload(
        part_id,
        entity_id,
        package_id,
        &mpn,
        &manufacturer,
        &value,
        &description,
        &datasheet,
        &lifecycle,
    );
    commit_pool_library_operations(
        root,
        format!(
            "create native pool part {part_id} for entity {entity_id} and package {package_id}"
        ),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_part",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_part_metadata(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mpn: Option<String>,
    manufacturer: Option<String>,
    manufacturer_jep106: Option<u16>,
    value: Option<String>,
    description: Option<String>,
    datasheet: Option<String>,
    lifecycle: Option<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if mpn.is_none()
        && manufacturer.is_none()
        && manufacturer_jep106.is_none()
        && value.is_none()
        && description.is_none()
        && datasheet.is_none()
        && lifecycle.is_none()
    {
        bail!("set-pool-part-metadata requires at least one metadata field");
    }
    let lifecycle = lifecycle.map(validate_part_lifecycle).transpose()?;
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    if let Some(mpn) = mpn {
        object_map.insert("mpn".to_string(), serde_json::Value::String(mpn));
    }
    if let Some(manufacturer) = manufacturer {
        object_map.insert(
            "manufacturer".to_string(),
            serde_json::Value::String(manufacturer),
        );
    }
    if let Some(manufacturer_jep106) = manufacturer_jep106 {
        if manufacturer_jep106 > 2047 {
            bail!("manufacturer-jep106 must be a valid 11-bit JEP106 code");
        }
        object_map.insert(
            "manufacturer_jep106".to_string(),
            serde_json::Value::Number(serde_json::Number::from(manufacturer_jep106)),
        );
    }
    if let Some(value) = value {
        object_map.insert("value".to_string(), serde_json::Value::String(value));
    }
    if let Some(description) = description {
        object_map.insert(
            "description".to_string(),
            serde_json::Value::String(description),
        );
    }
    if let Some(datasheet) = datasheet {
        object_map.insert(
            "datasheet".to_string(),
            serde_json::Value::String(datasheet),
        );
    }
    if let Some(lifecycle) = lifecycle {
        object_map.insert(
            "lifecycle".to_string(),
            serde_json::Value::String(lifecycle),
        );
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} metadata"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_metadata",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_part_parametric(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    params: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let mut parsed_params = serde_json::Map::new();
    for param in params {
        let (key, value) = param
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("part parametric entry must be key=value"))?;
        let key = key.trim();
        if key.is_empty() {
            bail!("part parametric key must be non-empty");
        }
        if parsed_params
            .insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            )
            .is_some()
        {
            bail!("duplicate part parametric key {key}");
        }
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    match mode.as_str() {
        "merge" => {
            let parametric = object_map
                .entry("parametric".to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            let parametric_map = parametric.as_object_mut().ok_or_else(|| {
                anyhow::anyhow!("part {part_id} parametric field is not an object")
            })?;
            for (key, value) in parsed_params {
                parametric_map.insert(key, value);
            }
        }
        "replace" => {
            object_map.insert(
                "parametric".to_string(),
                serde_json::Value::Object(parsed_params),
            );
        }
        other => bail!("unsupported part parametric mode {other}; expected merge or replace"),
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} parametric fields"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_parametric",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_part_orderable_mpns(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    mpns: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let mut orderable_mpns = Vec::new();
    for mpn in mpns {
        let mpn = mpn.trim();
        if mpn.is_empty() {
            bail!("part orderable MPN must be non-empty");
        }
        if orderable_mpns.iter().any(|existing| existing == mpn) {
            bail!("duplicate part orderable MPN {mpn}");
        }
        orderable_mpns.push(mpn.to_string());
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    match mode.as_str() {
        "merge" => {
            let existing = object_map
                .entry("orderable_mpns".to_string())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            let existing_array = existing.as_array_mut().ok_or_else(|| {
                anyhow::anyhow!("part {part_id} orderable_mpns field is not an array")
            })?;
            for mpn in orderable_mpns {
                if !existing_array
                    .iter()
                    .any(|existing| existing.as_str() == Some(mpn.as_str()))
                {
                    existing_array.push(serde_json::Value::String(mpn));
                }
            }
        }
        "replace" => {
            object_map.insert(
                "orderable_mpns".to_string(),
                serde_json::Value::Array(
                    orderable_mpns
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
        other => bail!("unsupported part orderable MPN mode {other}; expected merge or replace"),
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} orderable MPNs"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_orderable_mpns",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_part_packaging_options(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    options: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let mut packaging_options = Vec::new();
    for option in options {
        let value: serde_json::Value = serde_json::from_str(&option)
            .with_context(|| "part packaging option must be a JSON object")?;
        if !value.is_object() {
            bail!("part packaging option must be a JSON object");
        }
        let parsed: eda_engine::pool::PackagingOption = serde_json::from_value(value.clone())
            .with_context(|| "part packaging option must match a supported packaging schema")?;
        let normalized = serde_json::to_value(parsed)?;
        if packaging_options
            .iter()
            .any(|existing| existing == &normalized)
        {
            bail!("duplicate part packaging option {option}");
        }
        packaging_options.push(normalized);
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    match mode.as_str() {
        "merge" => {
            let existing = object_map
                .entry("packaging_options".to_string())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            let existing_array = existing.as_array_mut().ok_or_else(|| {
                anyhow::anyhow!("part {part_id} packaging_options field is not an array")
            })?;
            for option in packaging_options {
                if !existing_array.iter().any(|existing| existing == &option) {
                    existing_array.push(option);
                }
            }
        }
        "replace" => {
            object_map.insert(
                "packaging_options".to_string(),
                serde_json::Value::Array(packaging_options),
            );
        }
        other => bail!("unsupported part packaging option mode {other}; expected merge or replace"),
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} packaging options"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_packaging_options",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_part_behavioural_models(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    models: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let mut behavioural_models = Vec::new();
    for model in models {
        let value: serde_json::Value = serde_json::from_str(&model)
            .with_context(|| "part behavioural model attachment must be a JSON object")?;
        if !value.is_object() {
            bail!("part behavioural model attachment must be a JSON object");
        }
        let parsed: eda_engine::pool::ModelAttachment = serde_json::from_value(value.clone())
            .with_context(
                || "part behavioural model attachment must match ModelAttachment schema",
            )?;
        validate_model_attachment(&parsed)?;
        let normalized = serde_json::to_value(parsed)?;
        if behavioural_models
            .iter()
            .any(|existing| existing == &normalized)
        {
            bail!("duplicate part behavioural model attachment {model}");
        }
        behavioural_models.push(normalized);
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    match mode.as_str() {
        "merge" => {
            let existing = object_map
                .entry("behavioural_models".to_string())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            let existing_array = existing.as_array_mut().ok_or_else(|| {
                anyhow::anyhow!("part {part_id} behavioural_models field is not an array")
            })?;
            for model in behavioural_models {
                if !existing_array.iter().any(|existing| existing == &model) {
                    existing_array.push(model);
                }
            }
        }
        "replace" => {
            object_map.insert(
                "behavioural_models".to_string(),
                serde_json::Value::Array(behavioural_models),
            );
        }
        other => {
            bail!("unsupported part behavioural model mode {other}; expected merge or replace")
        }
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} behavioural model attachments"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_behavioural_models",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

fn validate_model_attachment(model: &eda_engine::pool::ModelAttachment) -> Result<()> {
    if model.model_names.iter().any(|name| name.trim().is_empty()) {
        bail!("part behavioural model names must be non-empty");
    }
    if model.encryption_scheme.is_some() && !model.encrypted {
        bail!("part behavioural model encryption scheme requires encrypted=true");
    }
    if model.provenance.as_ref().is_some_and(|provenance| {
        provenance.source.trim().is_empty() || provenance.sha256.trim().is_empty()
    }) {
        bail!("part behavioural model provenance source and sha256 must be non-empty");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach_native_project_pool_part_model(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    source: &Path,
    role: String,
    dialect: Option<String>,
    model_names: Vec<String>,
    encrypted: bool,
    encryption_scheme: Option<String>,
    vendor: Option<String>,
    fetched_at: Option<String>,
    format_metadata_json: Option<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if !source.is_file() {
        bail!("model source file does not exist: {}", source.display());
    }
    let role = parse_json_string_enum::<eda_engine::pool::ModelRole>(&role, "role")?;
    let dialect = dialect
        .map(|dialect| {
            parse_json_string_enum::<eda_engine::pool::SpiceDialect>(&dialect, "dialect")
        })
        .transpose()?;
    let encryption_scheme = encryption_scheme
        .map(|scheme| parse_encryption_scheme(&scheme))
        .transpose()?;
    let format_metadata = format_metadata_json
        .map(|metadata| {
            let value: serde_json::Value = serde_json::from_str(&metadata)
                .with_context(|| "format-metadata-json must be a ModelFormatMetadata object")?;
            serde_json::from_value::<eda_engine::pool::ModelFormatMetadata>(value)
                .with_context(|| "format-metadata-json must match ModelFormatMetadata schema")
        })
        .transpose()?
        .unwrap_or(eda_engine::pool::ModelFormatMetadata::None);
    let model_names = normalize_model_names(model_names)?;
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let model_bytes = fs::read(source)
        .with_context(|| format!("failed to read model source {}", source.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(&model_bytes));
    let role_dir = model_role_directory(&role);
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::trim)
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| default_model_extension(&role));
    let model_relative_path = format!("{pool_path}/models/{role_dir}/{sha256}.{extension}");
    write_pool_model_blob(root, &model_relative_path, &model_bytes)?;
    let model_uuid = derive_pool_model_uuid(&sha256);
    let attachment_uuid =
        derive_pool_model_attachment_uuid(part_id, model_uuid, &role, &model_names);
    let provenance = eda_engine::pool::ModelProvenance {
        source: source.to_string_lossy().to_string(),
        vendor: normalize_optional_string(vendor),
        fetched_at: normalize_optional_string(fetched_at),
        sha256: sha256.clone(),
    };
    let attachment = eda_engine::pool::ModelAttachment {
        uuid: attachment_uuid,
        model_uuid,
        role,
        dialect,
        model_names,
        encrypted,
        encryption_scheme,
        provenance: Some(provenance),
        format_metadata,
        reviewed: None,
        notes: Vec::new(),
    };
    validate_model_attachment(&attachment)?;
    let attachment_value = serde_json::to_value(attachment)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    let existing = object_map
        .entry("behavioural_models".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let existing_array = existing.as_array_mut().ok_or_else(|| {
        anyhow::anyhow!("part {part_id} behavioural_models field is not an array")
    })?;
    if !existing_array
        .iter()
        .any(|existing| existing == &attachment_value)
    {
        existing_array.push(attachment_value);
    }
    let attachments = existing_array.clone();
    commit_pool_library_operations(
        root,
        format!("attach behavioural model {sha256} to native pool part {part_id}"),
        None,
        vec![PoolLibraryOperationSpec::AttachPartModel {
            part_id,
            relative_path: relative_path.clone(),
            attachments,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "attach_part_model",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn detach_native_project_pool_part_model(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    attachment_id: Option<Uuid>,
    model_id: Option<Uuid>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    match (attachment_id, model_id) {
        (Some(_), Some(_)) => {
            bail!("detach-pool-part-model accepts --attachment or --model, not both")
        }
        (None, None) => bail!("detach-pool-part-model requires --attachment or --model"),
        _ => {}
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    let models = object_map
        .entry("behavioural_models".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let models = models.as_array_mut().ok_or_else(|| {
        anyhow::anyhow!("part {part_id} behavioural_models field is not an array")
    })?;
    let previous_attachments = models.clone();
    let mut attachments = Vec::new();
    for model in models.iter() {
        if let Some(attachment_id) = attachment_id {
            if model
                .get("uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|uuid| Uuid::parse_str(uuid).ok())
                == Some(attachment_id)
            {
                continue;
            }
        }
        if let Some(model_id) = model_id {
            if model
                .get("model_uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|uuid| Uuid::parse_str(uuid).ok())
                == Some(model_id)
            {
                continue;
            }
        }
        attachments.push(model.clone());
    }
    if attachments.len() == previous_attachments.len() {
        bail!("part {part_id} has no matching behavioural model attachment");
    }
    commit_pool_library_operations(
        root,
        format!("detach behavioural model from native pool part {part_id}"),
        None,
        vec![PoolLibraryOperationSpec::DetachPartModel {
            part_id,
            relative_path: relative_path.clone(),
            attachments,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "detach_part_model",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn gc_native_project_pool_models(
    root: &Path,
    pool_path: &str,
    role_filter: Option<&str>,
    sha_filter: Option<&str>,
    apply: bool,
) -> Result<NativeProjectPoolModelGcView> {
    validate_project_local_pool_path(pool_path)?;
    let models =
        super::query_native_project_pool_models(root, Some(pool_path), role_filter, sha_filter)?;
    let mut entries = Vec::new();
    let mut deleted_count = 0usize;
    let mut skipped_count = 0usize;
    for model in models.models.into_iter().filter(|model| model.orphaned) {
        let target = root.join(&model.relative_path);
        let mut deleted = false;
        let mut skipped_reason = None;
        if model.role == "ami" {
            skipped_count += 1;
            skipped_reason = Some("ami_bundle_gc_deferred".to_string());
        } else if !model.hash_matches {
            skipped_count += 1;
            skipped_reason = Some("hash_mismatch".to_string());
        } else if !target.is_file() {
            skipped_count += 1;
            skipped_reason = Some("not_regular_file".to_string());
        } else if apply {
            fs::remove_file(&target).with_context(|| {
                format!("failed to delete orphaned pool model {}", target.display())
            })?;
            deleted = true;
            deleted_count += 1;
        }
        entries.push(NativeProjectPoolModelGcEntryView {
            role: model.role,
            sha256: model.sha256,
            relative_path: model.relative_path,
            size_bytes: model.size_bytes,
            deleted,
            skipped_reason,
        });
    }
    let planned_count = entries.len();
    Ok(NativeProjectPoolModelGcView {
        contract: "native_project_pool_model_gc_v1",
        action: "gc_pool_models",
        pool_path: pool_path.to_string(),
        applied: apply,
        planned_count,
        deleted_count,
        skipped_count,
        entries,
    })
}

fn parse_json_string_enum<T>(value: &str, name: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .with_context(|| format!("{name} is not a supported enum value"))
}

fn parse_encryption_scheme(value: &str) -> Result<eda_engine::pool::EncryptionScheme> {
    serde_json::from_str::<eda_engine::pool::EncryptionScheme>(value)
        .or_else(|_| {
            parse_json_string_enum::<eda_engine::pool::EncryptionScheme>(value, "encryption-scheme")
        })
        .with_context(|| "encryption-scheme must match EncryptionScheme schema")
}

fn normalize_model_names(model_names: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    for name in model_names {
        let name = name.trim();
        if name.is_empty() {
            bail!("model-name must be non-empty");
        }
        if normalized.iter().any(|existing| existing == name) {
            bail!("duplicate model-name {name}");
        }
        normalized.push(name.to_string());
    }
    Ok(normalized)
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn model_role_directory(role: &eda_engine::pool::ModelRole) -> &'static str {
    match role {
        eda_engine::pool::ModelRole::Spice => "spice",
        eda_engine::pool::ModelRole::Ibis => "ibis",
        eda_engine::pool::ModelRole::IbisIss => "ibis_iss",
        eda_engine::pool::ModelRole::IbisAmi => "ami",
        eda_engine::pool::ModelRole::Touchstone => "touchstone",
        eda_engine::pool::ModelRole::VerilogA => "verilog_a",
        eda_engine::pool::ModelRole::VerilogAms => "verilog_ams",
        eda_engine::pool::ModelRole::VhdlAms => "vhdl_ams",
        eda_engine::pool::ModelRole::CompactThermal => "thermal",
    }
}

fn default_model_extension(role: &eda_engine::pool::ModelRole) -> &'static str {
    match role {
        eda_engine::pool::ModelRole::Spice => "lib",
        eda_engine::pool::ModelRole::Ibis => "ibs",
        eda_engine::pool::ModelRole::IbisIss => "pkg",
        eda_engine::pool::ModelRole::IbisAmi => "ami",
        eda_engine::pool::ModelRole::Touchstone => "snp",
        eda_engine::pool::ModelRole::VerilogA => "va",
        eda_engine::pool::ModelRole::VerilogAms => "vams",
        eda_engine::pool::ModelRole::VhdlAms => "vhd",
        eda_engine::pool::ModelRole::CompactThermal => "xml",
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_part_thermal(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    clear: bool,
    theta_ja_c_per_w: Option<String>,
    theta_jc_top_c_per_w: Option<String>,
    theta_jc_bot_c_per_w: Option<String>,
    theta_jb_c_per_w: Option<String>,
    max_junction_c: Option<String>,
    thermal_reference: Option<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if !clear
        && theta_ja_c_per_w.is_none()
        && theta_jc_top_c_per_w.is_none()
        && theta_jc_bot_c_per_w.is_none()
        && theta_jb_c_per_w.is_none()
        && max_junction_c.is_none()
        && thermal_reference.is_none()
    {
        bail!("set-pool-part-thermal requires --clear or at least one thermal field");
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    if clear
        && theta_ja_c_per_w.is_none()
        && theta_jc_top_c_per_w.is_none()
        && theta_jc_bot_c_per_w.is_none()
        && theta_jb_c_per_w.is_none()
        && max_junction_c.is_none()
        && thermal_reference.is_none()
    {
        object_map.insert("thermal".to_string(), serde_json::Value::Null);
    } else {
        let mut thermal = if clear {
            serde_json::Map::new()
        } else {
            object_map
                .get("thermal")
                .and_then(|value| value.as_object().cloned())
                .unwrap_or_default()
        };
        set_optional_thermal_number(
            &mut thermal,
            "theta_ja_c_per_w",
            theta_ja_c_per_w,
            "theta-ja-c-per-w",
        )?;
        set_optional_thermal_number(
            &mut thermal,
            "theta_jc_top_c_per_w",
            theta_jc_top_c_per_w,
            "theta-jc-top-c-per-w",
        )?;
        set_optional_thermal_number(
            &mut thermal,
            "theta_jc_bot_c_per_w",
            theta_jc_bot_c_per_w,
            "theta-jc-bot-c-per-w",
        )?;
        set_optional_thermal_number(
            &mut thermal,
            "theta_jb_c_per_w",
            theta_jb_c_per_w,
            "theta-jb-c-per-w",
        )?;
        set_optional_thermal_number(
            &mut thermal,
            "max_junction_c",
            max_junction_c,
            "max-junction-c",
        )?;
        if let Some(reference) = thermal_reference {
            let reference = reference.trim();
            if reference.is_empty() {
                bail!("thermal-reference must be non-empty");
            }
            thermal.insert(
                "thermal_reference".to_string(),
                serde_json::Value::String(reference.to_string()),
            );
        }
        let parsed: eda_engine::pool::ThermalSpec =
            serde_json::from_value(serde_json::Value::Object(thermal.clone()))
                .with_context(|| "part thermal spec must match ThermalSpec schema")?;
        object_map.insert("thermal".to_string(), serde_json::to_value(parsed)?);
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} thermal metadata"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_thermal",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

fn set_optional_thermal_number(
    thermal: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<String>,
    cli_name: &str,
) -> Result<()> {
    if let Some(value) = value {
        let number = parse_non_negative_json_number(&value, cli_name)?;
        thermal.insert(key.to_string(), serde_json::Value::Number(number));
    }
    Ok(())
}

fn parse_non_negative_json_number(value: &str, cli_name: &str) -> Result<serde_json::Number> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{cli_name} must be a non-negative JSON number");
    }
    let number: serde_json::Number = serde_json::from_str(trimmed)
        .with_context(|| format!("{cli_name} must be a non-negative JSON number"))?;
    if number.as_f64().is_some_and(|value| value >= 0.0) {
        Ok(number)
    } else {
        bail!("{cli_name} must be a non-negative JSON number");
    }
}

pub(crate) fn set_native_project_pool_part_supply_chain(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    clear: bool,
    checked_at: Option<String>,
    offers: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if !clear && checked_at.is_none() && offers.is_empty() {
        bail!("set-pool-part-supply-chain requires --clear, --checked-at, or at least one --offer");
    }
    let checked_at = normalize_optional_string(checked_at);
    let mut parsed_offers = Vec::new();
    for offer in offers {
        let value: serde_json::Value = serde_json::from_str(&offer)
            .with_context(|| "part supply-chain offer must be a JSON object")?;
        if !value.is_object() {
            bail!("part supply-chain offer must be a JSON object");
        }
        let parsed: eda_engine::pool::SupplyOffer = serde_json::from_value(value)
            .with_context(|| "part supply-chain offer must match SupplyOffer schema")?;
        validate_supply_offer(&parsed)?;
        let normalized = serde_json::to_value(parsed)?;
        if parsed_offers.iter().any(|existing| existing == &normalized) {
            bail!("duplicate part supply-chain offer {offer}");
        }
        parsed_offers.push(normalized);
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    if clear {
        object_map.insert("supply_chain_offers".to_string(), serde_json::Value::Null);
        object_map.insert(
            "last_supply_chain_check".to_string(),
            serde_json::Value::Null,
        );
    }
    if !parsed_offers.is_empty() {
        object_map.insert(
            "supply_chain_offers".to_string(),
            serde_json::Value::Array(parsed_offers),
        );
    }
    if let Some(checked_at) = checked_at {
        object_map.insert(
            "last_supply_chain_check".to_string(),
            serde_json::Value::String(checked_at),
        );
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} supply-chain cache"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_supply_chain",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

fn validate_supply_offer(offer: &eda_engine::pool::SupplyOffer) -> Result<()> {
    if offer.distributor.trim().is_empty() {
        bail!("part supply-chain offer distributor must be non-empty");
    }
    if offer.link.trim().is_empty() {
        bail!("part supply-chain offer link must be non-empty");
    }
    for price_break in &offer.price_breaks {
        if price_break.qty == 0 {
            bail!("part supply-chain price-break qty must be positive");
        }
        if price_break.price.as_f64().is_some_and(|price| price < 0.0) {
            bail!("part supply-chain price-break price must be non-negative");
        }
        if price_break.currency.trim().is_empty() {
            bail!("part supply-chain price-break currency must be non-empty");
        }
    }
    Ok(())
}

pub(crate) fn set_native_project_pool_part_tags(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    tags: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let mut parsed_tags = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            bail!("part tag must be non-empty");
        }
        if parsed_tags.iter().any(|existing| existing == tag) {
            bail!("duplicate part tag {tag}");
        }
        parsed_tags.push(tag.to_string());
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    match mode.as_str() {
        "merge" => {
            let existing = object_map
                .entry("tags".to_string())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            let existing_array = existing
                .as_array_mut()
                .ok_or_else(|| anyhow::anyhow!("part {part_id} tags field is not an array"))?;
            for tag in parsed_tags {
                if !existing_array
                    .iter()
                    .any(|existing| existing.as_str() == Some(tag.as_str()))
                {
                    existing_array.push(serde_json::Value::String(tag));
                }
            }
        }
        "replace" => {
            object_map.insert(
                "tags".to_string(),
                serde_json::Value::Array(
                    parsed_tags
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
        other => bail!("unsupported part tag mode {other}; expected merge or replace"),
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} tags"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_tags",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}

pub(crate) fn delete_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    // Validate existence and payload uuid exactly as the pre-migration CLI
    // did; the facade re-sources the stored payload from the same model view.
    read_project_pool_object_payload(root, &relative_path, object_id)?;
    commit_pool_library_operations(
        root,
        format!("delete native pool library object {object_id}"),
        None,
        vec![PoolLibraryOperationSpec::Delete {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
        }],
    )?;
    pool_library_mutation_view(
        root,
        "delete",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source: &Path,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    // Validate existence and payload uuid exactly as the pre-migration CLI
    // did; the facade re-sources the previous payload from the same model view.
    read_project_pool_object_payload(root, &relative_path, object_id)?;
    let object = read_pool_library_object_payload(source, object_id)?;
    commit_pool_library_operations(
        root,
        format!("set native pool library object {object_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}

/// Author and commit one pool-library write through the native write facade:
/// resolve, build the typed batch (with the ensure-pool-ref rule applied by
/// the engine when `ensure_pool_ref` names a pool), and commit through the
/// single journaled path.
pub(super) fn commit_pool_library_operations(
    root: &Path,
    reason: String,
    ensure_pool_ref: Option<&str>,
    operations: Vec<PoolLibraryOperationSpec>,
) -> Result<()> {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new("datum-eda-cli", pool_library_commit_source()?, reason),
        ensure_pool_ref,
        operations,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(())
}

fn pool_library_commit_source() -> Result<CommitSource> {
    let Ok(value) = env::var("DATUM_COMMIT_SOURCE") else {
        return Ok(CommitSource::Cli);
    };
    match value.as_str() {
        "cli" => Ok(CommitSource::Cli),
        "tool" => Ok(CommitSource::Tool),
        "assistant" => Ok(CommitSource::Assistant),
        _ => bail!(
            "unsupported DATUM_COMMIT_SOURCE for pool-library authoring: {value}; expected cli, tool, or assistant"
        ),
    }
}

pub(super) fn pool_library_mutation_view(
    root: &Path,
    action: &'static str,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    relative_path: &str,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    Ok(NativeProjectPoolLibraryObjectMutationView {
        contract: "native_project_pool_library_object_mutation_v1",
        action,
        project_id: model.project.project_id,
        object_uuid: object_id,
        object_kind: object_kind.to_string(),
        pool_path: pool_path.to_string(),
        relative_path: relative_path.to_string(),
        object_path: root.join(relative_path).display().to_string(),
    })
}

pub(super) fn validate_project_local_pool_path(pool_path: &str) -> Result<()> {
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

pub(super) fn validate_pool_library_object_kind(kind: &str) -> Result<()> {
    const ALLOWED_KINDS: &[&str] = &[
        "units",
        "symbols",
        "entities",
        "parts",
        "packages",
        "footprints",
        "padstacks",
        "pin_pad_maps",
    ];
    if ALLOWED_KINDS.contains(&kind) {
        Ok(())
    } else {
        bail!("unsupported pool library object kind {kind}");
    }
}

fn positive_required_dimension(value: Option<i64>, name: &str) -> Result<i64> {
    match value {
        Some(value) if value > 0 => Ok(value),
        Some(_) => bail!("padstack {name} must be positive"),
        None => bail!("padstack {name} is required"),
    }
}

fn validate_part_lifecycle(lifecycle: String) -> Result<String> {
    match lifecycle.as_str() {
        "Active" | "Nrnd" | "Eol" | "Obsolete" | "Unknown" => Ok(lifecycle),
        other => bail!(
            "unsupported part lifecycle {other}; expected Active, Nrnd, Eol, Obsolete, or Unknown"
        ),
    }
}

/// Thin shim over the engine facade's canonical pool shard path rule.
pub(super) fn pool_library_relative_path(
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
) -> String {
    eda_engine::api::native_write::library::pool_library_relative_path(
        pool_path,
        object_kind,
        object_id,
    )
}
