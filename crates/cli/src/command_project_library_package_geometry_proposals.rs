use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};
use eda_engine::substrate::{
    CommitSource, ProjectResolver, ProposalCreateRequest, ProposalSource,
    create_draft_proposal_from_batch,
};
use uuid::Uuid;

use super::command_project_library::{
    pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_payload::read_project_pool_object_payload;
use super::command_project_proposals::{
    NativeProjectProposalCreateView, validate_proposal_in_model,
};

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
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-proposal",
            CommitSource::Cli,
            format!("propose native pool package pad {pad_id}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                package_id,
                "packages",
                relative_path,
            ),
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
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-proposal",
            CommitSource::Cli,
            format!("propose native pool package update {package_id}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "packages", package_id),
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
