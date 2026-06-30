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

fn positive_required_dimension(value: Option<i64>, name: &str) -> Result<i64> {
    match value {
        Some(value) if value > 0 => Ok(value),
        Some(_) => bail!("padstack {name} must be positive"),
        None => bail!("padstack {name} is required"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_set_native_project_pool_package_pad(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    pad_id: Uuid,
    padstack_id: Uuid,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    if layer <= 0 {
        bail!("package pad layer must be positive");
    }
    let pad_name = pad_name.trim().to_string();
    if pad_name.is_empty() {
        bail!("package pad name must not be empty");
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&padstack_id)
        .filter(|object| object.domain == "pool" && object.kind == "padstacks")
        .is_none()
    {
        bail!("missing pool padstack {padstack_id} for package {package_id}");
    }
    if model
        .objects
        .get(&package_id)
        .filter(|object| object.domain == "pool" && object.kind == "packages")
        .is_none()
    {
        bail!("missing pool package {package_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, package_id)?;
    let mut object = previous_object.clone();
    let pads = object
        .as_object_mut()
        .context("pool package payload must be a JSON object")?
        .entry("pads")
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads
        .as_object_mut()
        .context("pool package pads field must be an object")?;
    if pads.contains_key(&pad_id.to_string()) {
        bail!("pool package {package_id} already has pad {pad_id}");
    }
    pads.insert(
        pad_id.to_string(),
        serde_json::json!({
            "uuid": pad_id,
            "name": pad_name,
            "position": {"x": x_nm, "y": y_nm},
            "padstack": padstack_id,
            "layer": layer
        }),
    );
    let mut model = ProjectResolver::new(root).resolve()?;
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: format!("propose native pool package pad {pad_id}"),
            },
            operations: vec![Operation::SetPoolLibraryObject {
                object_id: package_id,
                relative_path,
                object_kind: "packages".to_string(),
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
                .unwrap_or("Set native pool package pad")
                .to_string(),
            source: ProposalSource::Tool,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "set_pool_package_pad_proposal",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_set_native_project_pool_package_courtyard_rect(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = courtyard_rect_vertices(min_x_nm, min_y_nm, max_x_nm, max_y_nm)?;
    let object = package_object_with_courtyard(root, pool_path, package_id, vertices)?;
    propose_set_pool_package_object_value(
        root,
        pool_path,
        package_id,
        object,
        proposal_id,
        rationale,
        "set_pool_package_courtyard_rect_proposal",
        "Set native pool package courtyard rectangle",
    )
}

pub(crate) fn propose_set_native_project_pool_package_courtyard_polygon(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    vertices: &str,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = parse_vertices(vertices, "package courtyard polygon")?;
    if vertices.len() < 3 {
        bail!("package courtyard polygon must have at least 3 vertices");
    }
    let object = package_object_with_courtyard(root, pool_path, package_id, vertices)?;
    propose_set_pool_package_object_value(
        root,
        pool_path,
        package_id,
        object,
        proposal_id,
        rationale,
        "set_pool_package_courtyard_polygon_proposal",
        "Set native pool package courtyard polygon",
    )
}

fn package_object_with_courtyard(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    vertices: Vec<serde_json::Value>,
) -> Result<serde_json::Value> {
    ensure_pool_package_exists(root, package_id)?;
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let mut object = read_project_pool_object_payload(root, &relative_path, package_id)?;
    object
        .as_object_mut()
        .context("pool package payload must be a JSON object")?
        .insert(
            "courtyard".to_string(),
            serde_json::json!({"vertices": vertices, "closed": true}),
        );
    Ok(object)
}

#[allow(clippy::too_many_arguments)]
fn propose_set_pool_package_object_value(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    object: serde_json::Value,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
    action: &'static str,
    default_rationale: &'static str,
) -> Result<NativeProjectProposalCreateView> {
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, package_id)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: format!("propose native pool package update {package_id}"),
            },
            operations: vec![Operation::SetPoolLibraryObject {
                object_id: package_id,
                relative_path,
                object_kind: "packages".to_string(),
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

fn ensure_pool_package_exists(root: &Path, package_id: Uuid) -> Result<()> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&package_id)
        .filter(|object| object.domain == "pool" && object.kind == "packages")
        .is_none()
    {
        bail!("missing pool package {package_id}");
    }
    Ok(())
}

fn parse_vertices(vertices: &str, geometry_name: &str) -> Result<Vec<serde_json::Value>> {
    let vertices = vertices.trim();
    if vertices.is_empty() {
        bail!("{geometry_name} vertices must not be empty");
    }
    vertices
        .split(';')
        .enumerate()
        .map(|(index, pair)| {
            let mut coords = pair.split(',');
            let x = coords
                .next()
                .context("missing x coordinate")?
                .trim()
                .parse::<i64>()
                .with_context(|| format!("invalid x coordinate in vertex {}", index + 1))?;
            let y = coords
                .next()
                .context("missing y coordinate")?
                .trim()
                .parse::<i64>()
                .with_context(|| format!("invalid y coordinate in vertex {}", index + 1))?;
            if coords.next().is_some() {
                bail!("{geometry_name} vertex {} must be x,y", index + 1);
            }
            Ok(serde_json::json!({"x": x, "y": y}))
        })
        .collect()
}

fn courtyard_rect_vertices(
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
) -> Result<Vec<serde_json::Value>> {
    if min_x_nm >= max_x_nm {
        bail!("package courtyard min-x-nm must be less than max-x-nm");
    }
    if min_y_nm >= max_y_nm {
        bail!("package courtyard min-y-nm must be less than max-y-nm");
    }
    Ok(vec![
        serde_json::json!({"x": min_x_nm, "y": min_y_nm}),
        serde_json::json!({"x": max_x_nm, "y": min_y_nm}),
        serde_json::json!({"x": max_x_nm, "y": max_y_nm}),
        serde_json::json!({"x": min_x_nm, "y": max_y_nm}),
    ])
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
