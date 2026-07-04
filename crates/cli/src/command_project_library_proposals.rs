use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
    pool_entity_payload, pool_padstack_payload, pool_symbol_payload, pool_unit_payload,
};
use eda_engine::api::native_write::library_pin_pad_map::{
    pin_pad_map_payload, set_part_default_pin_pad_map_spec,
};
use eda_engine::substrate::{
    ProjectResolver, ProposalCreateRequest, ProposalSource,
    create_draft_proposal_from_batch,
};
use uuid::Uuid;

use super::command_project_library::{
    pool_library_relative_path, validate_pool_library_object_kind,
    validate_project_local_pool_path,
};
use super::command_project_library_payload::{
    read_pool_library_object_payload, read_project_pool_object_payload,
};
use super::command_project_library_pin_pad_map::{
    parse_mapping_entries, validate_pin_pad_map_payload,
};
use super::command_project_proposals::{
    NativeProjectProposalCreateView, validate_proposal_in_model,
};

use crate::command_project::cli_commit_source;

pub(crate) fn propose_create_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source_path: &Path,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let object = read_pool_library_object_payload(source_path, object_id)?;
    propose_create_pool_library_object_value(
        root,
        pool_path,
        object_kind,
        object_id,
        object,
        proposal_id,
        rationale,
        "create_pool_library_object_proposal",
        "Create native pool library object",
    )
}

pub(crate) fn propose_create_native_project_pool_unit(
    root: &Path,
    pool_path: &str,
    unit_id: Uuid,
    name: String,
    manufacturer: String,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let object = pool_unit_payload(unit_id, &name, &manufacturer);
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "units",
        unit_id,
        object,
        proposal_id,
        rationale,
        "create_pool_unit_proposal",
        "Create native pool unit",
    )
}

pub(crate) fn propose_create_native_project_pool_symbol(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    unit_id: Uuid,
    name: String,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
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
    let object = pool_symbol_payload(symbol_id, unit_id, &name);
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "symbols",
        symbol_id,
        object,
        proposal_id,
        rationale,
        "create_pool_symbol_proposal",
        "Create native pool symbol",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_create_native_project_pool_entity(
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
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
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
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "entities",
        entity_id,
        object,
        proposal_id,
        rationale,
        "create_pool_entity_proposal",
        "Create native pool entity",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_create_native_project_pool_padstack(
    root: &Path,
    pool_path: &str,
    padstack_id: Uuid,
    name: String,
    aperture: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    drill_nm: Option<i64>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
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
    let object = pool_padstack_payload(padstack_id, &name, aperture_value, drill_nm);
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "padstacks",
        padstack_id,
        object,
        proposal_id,
        rationale,
        "create_pool_padstack_proposal",
        "Create native pool padstack",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_create_native_project_pool_pin_pad_map(
    root: &Path,
    pool_path: &str,
    map_id: Uuid,
    part_id: Uuid,
    footprint_id: Option<Uuid>,
    entries: Vec<String>,
    set_default: bool,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let mappings = parse_mapping_entries(entries, &model, part_id)?;
    validate_pin_pad_map_payload(&model, part_id, footprint_id, &mappings)?;
    let object = pin_pad_map_payload(map_id, part_id, footprint_id, &mappings);
    let mut operations = vec![PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new(pool_path, "pin_pad_maps", map_id),
        object,
    }];
    if set_default {
        operations.push(set_part_default_pin_pad_map_spec(
            &model, pool_path, part_id, map_id,
        )?);
    }
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-proposal",
            cli_commit_source()?,
            format!("propose native pool PinPadMap {map_id} for part {part_id}"),
        ),
        None,
        operations,
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch: prepared.batch,
            rationale: rationale
                .unwrap_or("Create native pool PinPadMap")
                .to_string(),
            source: ProposalSource::Tool,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "create_pool_pin_pad_map_proposal",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

pub(crate) fn propose_set_native_project_pool_pin_pad_map(
    root: &Path,
    pool_path: &str,
    map_id: Uuid,
    mode: String,
    entries: Vec<String>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let replace = match mode.as_str() {
        "merge" => false,
        "replace" => true,
        other => bail!("unsupported PinPadMap mode {other}; expected merge or replace"),
    };
    let relative_path = pool_library_relative_path(pool_path, "pin_pad_maps", map_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, map_id)?;
    let part_id = previous_object
        .get("part")
        .and_then(serde_json::Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or_else(|| anyhow::anyhow!("pin_pad_map missing part"))?;
    let footprint_id = previous_object
        .get("footprint")
        .and_then(serde_json::Value::as_str)
        .map(Uuid::parse_str)
        .transpose()
        .context("pin_pad_map has invalid footprint uuid")?;
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let entries = parse_mapping_entries(entries, &model, part_id)?;
    let mut object = previous_object.clone();
    let mappings = object
        .as_object_mut()
        .context("pin_pad_map payload must be a JSON object")?
        .entry("mappings")
        .or_insert_with(|| serde_json::json!({}));
    if replace {
        *mappings = serde_json::json!({});
    }
    let mappings = mappings
        .as_object_mut()
        .context("pin_pad_map mappings field must be an object")?;
    for entry in entries {
        mappings.insert(
            entry.pad.to_string(),
            serde_json::json!({"gate": entry.gate, "pin": entry.pin}),
        );
    }
    let merged = mappings
        .iter()
        .map(|(pad, entry)| {
            let pad = Uuid::parse_str(pad)
                .with_context(|| format!("pin_pad_map mapping key {pad} is not a UUID"))?;
            let gate = entry
                .get("gate")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok())
                .ok_or_else(|| anyhow::anyhow!("pin_pad_map mapping missing gate"))?;
            let pin = entry
                .get("pin")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok())
                .ok_or_else(|| anyhow::anyhow!("pin_pad_map mapping missing pin"))?;
            Ok(super::command_project_library_pin_pad_map::PinPadMapEntryInput { pad, gate, pin })
        })
        .collect::<Result<Vec<_>>>()?;
    validate_pin_pad_map_payload(&model, part_id, footprint_id, &merged)?;
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-proposal",
            cli_commit_source()?,
            format!("propose native pool PinPadMap update {map_id}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(map_id, "pin_pad_maps", relative_path),
            object,
        }],
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch: prepared.batch,
            rationale: rationale
                .unwrap_or("Set native pool PinPadMap mappings")
                .to_string(),
            source: ProposalSource::Tool,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "set_pool_pin_pad_map_proposal",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

fn positive_required_dimension(value: Option<i64>, name: &str) -> Result<i64> {
    match value {
        Some(value) if value > 0 => Ok(value),
        Some(_) => bail!("padstack {name} must be positive"),
        None => bail!("padstack {name} is required"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn propose_create_pool_library_object_value(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    object: serde_json::Value,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
    action: &'static str,
    default_rationale: &'static str,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-proposal",
            cli_commit_source()?,
            format!("propose native pool library object {object_id}"),
        ),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object,
        }],
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch: prepared.batch,
            rationale: rationale.unwrap_or(default_rationale).to_string(),
            source: ProposalSource::Tool,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}
