use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
    ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use uuid::Uuid;

use super::command_project_library::{
    next_pool_priority, pool_library_relative_path, validate_pool_library_object_kind,
    validate_project_local_pool_path,
};
use super::command_project_library_payload::{
    read_pool_library_object_payload, read_project_pool_object_payload,
};
use super::command_project_library_pin_pad_map::{
    mappings_as_json, parse_mapping_entries, set_part_default_pin_pad_map_operation,
    validate_pin_pad_map_payload,
};
use super::command_project_operation_guards::guarded_operation_batch;
use super::command_project_proposals::{
    NativeProjectProposalCreateView, validate_proposal_in_model,
};

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
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": unit_id,
        "name": name,
        "manufacturer": manufacturer,
        "pins": {},
        "tags": []
    });
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
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": symbol_id,
        "name": name,
        "unit": unit_id
    });
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
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": entity_id,
        "name": name,
        "prefix": prefix,
        "manufacturer": manufacturer,
        "gates": {
            gate_id.to_string(): {
                "uuid": gate_id,
                "name": gate_name,
                "unit": unit_id,
                "symbol": symbol_id
            }
        },
        "tags": []
    });
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
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": padstack_id,
        "name": name,
        "aperture": aperture_value,
        "drill_nm": drill_nm
    });
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
    let relative_path = pool_library_relative_path(pool_path, "pin_pad_maps", map_id);
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": map_id,
        "part": part_id,
        "footprint": footprint_id,
        "mappings": mappings_as_json(&mappings),
        "tags": []
    });
    let mut operations = vec![Operation::CreatePoolLibraryObject {
        object_id: map_id,
        relative_path,
        object_kind: "pin_pad_maps".to_string(),
        object,
    }];
    if set_default {
        operations.push(set_part_default_pin_pad_map_operation(
            root, pool_path, part_id, map_id,
        )?);
    }
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: format!("propose native pool PinPadMap {map_id} for part {part_id}"),
            },
            operations,
        },
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
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
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: format!("propose native pool PinPadMap update {map_id}"),
            },
            operations: vec![Operation::SetPoolLibraryObject {
                object_id: map_id,
                relative_path,
                object_kind: "pin_pad_maps".to_string(),
                previous_object,
                object,
            }],
        },
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
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
    let project = super::load_native_project_with_resolved_board(root)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
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
    operations.push(Operation::CreatePoolLibraryObject {
        object_id,
        relative_path,
        object_kind: object_kind.to_string(),
        object,
    });
    let mut model = ProjectResolver::new(root).resolve()?;
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: format!("propose native pool library object {object_id}"),
            },
            operations,
        },
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
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
