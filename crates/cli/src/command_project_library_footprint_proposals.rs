use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
    ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use uuid::Uuid;

use super::command_project_library::{
    next_pool_priority, pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_footprint::{
    courtyard_rect_vertices, footprint_object_with_appended_silkscreen,
    footprint_object_with_courtyard, footprint_silkscreen_circle_primitive,
    footprint_silkscreen_line_primitive, footprint_silkscreen_polygon_primitive,
    footprint_silkscreen_rect_primitive, parse_vertices,
};
use super::command_project_library_payload::read_project_pool_object_payload;
use super::command_project_operation_guards::guarded_operation_batch;
use super::command_project_proposals::{
    NativeProjectProposalCreateView, validate_proposal_in_model,
};

pub(crate) fn propose_create_native_project_pool_footprint(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    package_id: Uuid,
    name: String,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, package_id, "packages", "package")?;
    let object = serde_json::json!({
        "schema_version": 1,
        "uuid": footprint_id,
        "name": name,
        "package": package_id,
        "pads": {},
        "courtyard": {"vertices": [], "closed": true},
        "silkscreen": [],
        "fab": [],
        "assembly": [],
        "mechanical": [],
        "models_3d": [],
        "standards_basis": null,
        "process_aperture_policy": null,
        "tags": []
    });
    let project = super::load_native_project_with_resolved_board(root)?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
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
        object_id: footprint_id,
        relative_path,
        object_kind: "footprints".to_string(),
        object,
    });
    create_pool_footprint_proposal_from_operations(
        root,
        operations,
        proposal_id,
        rationale.unwrap_or("Create native pool footprint"),
        "create_pool_footprint_proposal",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_set_native_project_pool_footprint_pad(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
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
        bail!("footprint pad layer must be positive");
    }
    let pad_name = pad_name.trim().to_string();
    if pad_name.is_empty() {
        bail!("footprint pad name must not be empty");
    }
    ensure_pool_object_exists(root, padstack_id, "padstacks", "padstack")?;
    ensure_pool_object_exists(root, footprint_id, "footprints", "footprint")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let mut object = previous_object.clone();
    let pads = object
        .as_object_mut()
        .context("pool footprint payload must be a JSON object")?
        .entry("pads")
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads
        .as_object_mut()
        .context("pool footprint pads field must be an object")?;
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
    create_pool_footprint_proposal_from_operations(
        root,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path,
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
        proposal_id,
        rationale.unwrap_or("Set native pool footprint pad"),
        "set_pool_footprint_pad_proposal",
    )
}

pub(crate) fn propose_set_native_project_pool_footprint_courtyard_rect(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let vertices = courtyard_rect_vertices(min_x_nm, min_y_nm, max_x_nm, max_y_nm)?;
    create_pool_footprint_courtyard_proposal(
        root,
        pool_path,
        footprint_id,
        vertices,
        proposal_id,
        rationale.unwrap_or("Set native pool footprint courtyard rectangle"),
        "set_pool_footprint_courtyard_rect_proposal",
    )
}

pub(crate) fn propose_set_native_project_pool_footprint_courtyard_polygon(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: &str,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let vertices = parse_vertices(vertices, "footprint courtyard polygon")?;
    if vertices.len() < 3 {
        bail!("footprint courtyard polygon must have at least 3 vertices");
    }
    create_pool_footprint_courtyard_proposal(
        root,
        pool_path,
        footprint_id,
        vertices,
        proposal_id,
        rationale.unwrap_or("Set native pool footprint courtyard polygon"),
        "set_pool_footprint_courtyard_polygon_proposal",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_add_native_project_pool_footprint_silkscreen_line(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let primitive =
        footprint_silkscreen_line_primitive(from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm)?;
    let (relative_path, previous_object, object) =
        footprint_object_with_appended_silkscreen(root, pool_path, footprint_id, primitive)?;
    create_pool_footprint_proposal_from_operations(
        root,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path,
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
        proposal_id,
        rationale.unwrap_or("Add native pool footprint silkscreen line"),
        "add_pool_footprint_silkscreen_line_proposal",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_add_native_project_pool_footprint_silkscreen_rect(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let primitive =
        footprint_silkscreen_rect_primitive(min_x_nm, min_y_nm, max_x_nm, max_y_nm, width_nm)?;
    create_pool_footprint_silkscreen_proposal(
        root,
        pool_path,
        footprint_id,
        primitive,
        proposal_id,
        rationale.unwrap_or("Add native pool footprint silkscreen rectangle"),
        "add_pool_footprint_silkscreen_rect_proposal",
    )
}

pub(crate) fn propose_add_native_project_pool_footprint_silkscreen_circle(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let primitive =
        footprint_silkscreen_circle_primitive(center_x_nm, center_y_nm, radius_nm, width_nm)?;
    create_pool_footprint_silkscreen_proposal(
        root,
        pool_path,
        footprint_id,
        primitive,
        proposal_id,
        rationale.unwrap_or("Add native pool footprint silkscreen circle"),
        "add_pool_footprint_silkscreen_circle_proposal",
    )
}

pub(crate) fn propose_add_native_project_pool_footprint_silkscreen_polygon(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: &str,
    closed: bool,
    width_nm: i64,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let primitive = footprint_silkscreen_polygon_primitive(vertices, closed, width_nm)?;
    create_pool_footprint_silkscreen_proposal(
        root,
        pool_path,
        footprint_id,
        primitive,
        proposal_id,
        rationale.unwrap_or("Add native pool footprint silkscreen polygon"),
        "add_pool_footprint_silkscreen_polygon_proposal",
    )
}

fn ensure_pool_object_exists(root: &Path, object_id: Uuid, kind: &str, label: &str) -> Result<()> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == kind)
        .is_none()
    {
        bail!("missing pool {label} {object_id}");
    }
    Ok(())
}

fn create_pool_footprint_courtyard_proposal(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: Vec<serde_json::Value>,
    proposal_id: Option<Uuid>,
    rationale: &str,
    action: &'static str,
) -> Result<NativeProjectProposalCreateView> {
    let (relative_path, previous_object, object) =
        footprint_object_with_courtyard(root, pool_path, footprint_id, vertices)?;
    create_pool_footprint_proposal_from_operations(
        root,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path,
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
        proposal_id,
        rationale,
        action,
    )
}

fn create_pool_footprint_silkscreen_proposal(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    primitive: serde_json::Value,
    proposal_id: Option<Uuid>,
    rationale: &str,
    action: &'static str,
) -> Result<NativeProjectProposalCreateView> {
    let (relative_path, previous_object, object) =
        footprint_object_with_appended_silkscreen(root, pool_path, footprint_id, primitive)?;
    create_pool_footprint_proposal_from_operations(
        root,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path,
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
        proposal_id,
        rationale,
        action,
    )
}

fn create_pool_footprint_proposal_from_operations(
    root: &Path,
    operations: Vec<Operation>,
    proposal_id: Option<Uuid>,
    rationale: &str,
    action: &'static str,
) -> Result<NativeProjectProposalCreateView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let batch = guarded_operation_batch(
        &model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-proposal".to_string(),
                source: CommitSource::Cli,
                reason: "propose native pool footprint update".to_string(),
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
            rationale: rationale.to_string(),
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
