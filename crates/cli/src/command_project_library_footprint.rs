use std::path::Path;

use crate::IpcDensityLevelArg;
use anyhow::{Context, Result, bail};
use eda_engine::pool::{
    GeneratedIpcFootprint, IpcDensityLevel, IpcSourceDimensions, IpcTwoTerminalChipSpec,
    generate_ipc7351b_two_terminal_chip,
};
use eda_engine::substrate::{Operation, ProjectResolver};
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations, next_pool_priority,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_payload::read_project_pool_object_payload;
use super::load_native_project_with_resolved_board;

pub(crate) fn create_native_project_pool_footprint(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    package_id: Uuid,
    name: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, package_id, "packages", "package")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
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
    let project = load_native_project_with_resolved_board(root)?;
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
        relative_path: relative_path.clone(),
        object_kind: "footprints".to_string(),
        object,
    });
    commit_pool_library_operations(
        root,
        format!("create native pool footprint {footprint_id} for package {package_id}"),
        operations,
    )?;
    pool_library_mutation_view(
        root,
        "create_footprint",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_native_project_ipc7351b_two_terminal_chip(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    package_id: Uuid,
    padstack_id: Uuid,
    pad_a_id: Uuid,
    pad_b_id: Uuid,
    name: Option<String>,
    metric_code: String,
    body_length_nm: i64,
    body_width_nm: i64,
    terminal_length_nm: i64,
    terminal_width_nm: i64,
    density: IpcDensityLevelArg,
    mask_expansion_nm: i64,
    paste_reduction_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let operations = ipc7351b_two_terminal_chip_operations(
        root,
        pool_path,
        footprint_id,
        package_id,
        padstack_id,
        pad_a_id,
        pad_b_id,
        name,
        metric_code,
        body_length_nm,
        body_width_nm,
        terminal_length_nm,
        terminal_width_nm,
        density,
        mask_expansion_nm,
        paste_reduction_nm,
    )?;
    commit_pool_library_operations(
        root,
        format!("generate IPC-7351B two-terminal chip footprint {footprint_id}"),
        operations,
    )?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    pool_library_mutation_view(
        root,
        "generate_ipc7351b_two_terminal_chip",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn ipc7351b_two_terminal_chip_operations(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    package_id: Uuid,
    padstack_id: Uuid,
    pad_a_id: Uuid,
    pad_b_id: Uuid,
    name: Option<String>,
    metric_code: String,
    body_length_nm: i64,
    body_width_nm: i64,
    terminal_length_nm: i64,
    terminal_width_nm: i64,
    density: IpcDensityLevelArg,
    mask_expansion_nm: i64,
    paste_reduction_nm: i64,
) -> Result<Vec<Operation>> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, package_id, "packages", "package")?;
    let generated = generate_ipc7351b_two_terminal_chip(IpcTwoTerminalChipSpec {
        footprint_uuid: footprint_id,
        package_uuid: package_id,
        padstack_uuid: padstack_id,
        pad_a_uuid: pad_a_id,
        pad_b_uuid: pad_b_id,
        name,
        metric_code,
        dimensions: IpcSourceDimensions {
            body_length_nm,
            body_width_nm,
            terminal_length_nm,
            terminal_width_nm,
        },
        density_level: ipc_density_level(density),
        mask_expansion_nm,
        paste_reduction_nm,
    })
    .map_err(anyhow::Error::msg)?;
    ipc_generated_footprint_operations(root, pool_path, generated)
}

fn ipc_generated_footprint_operations(
    root: &Path,
    pool_path: &str,
    generated: GeneratedIpcFootprint,
) -> Result<Vec<Operation>> {
    let project = load_native_project_with_resolved_board(root)?;
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
        object_id: generated.padstack.uuid,
        relative_path: pool_library_relative_path(pool_path, "padstacks", generated.padstack.uuid),
        object_kind: "padstacks".to_string(),
        object: pool_object_value_with_schema(serde_json::to_value(generated.padstack)?)?,
    });
    operations.push(Operation::CreatePoolLibraryObject {
        object_id: generated.footprint.uuid,
        relative_path: pool_library_relative_path(
            pool_path,
            "footprints",
            generated.footprint.uuid,
        ),
        object_kind: "footprints".to_string(),
        object: pool_object_value_with_schema(serde_json::to_value(generated.footprint)?)?,
    });
    Ok(operations)
}

fn pool_object_value_with_schema(mut object: serde_json::Value) -> Result<serde_json::Value> {
    object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("generated pool object must serialize as a JSON object"))?
        .insert("schema_version".to_string(), serde_json::json!(1));
    Ok(object)
}

pub(crate) fn ipc_density_level(value: IpcDensityLevelArg) -> IpcDensityLevel {
    match value {
        IpcDensityLevelArg::Most => IpcDensityLevel::Most,
        IpcDensityLevelArg::Nominal => IpcDensityLevel::Nominal,
        IpcDensityLevelArg::Least => IpcDensityLevel::Least,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_footprint_pad(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    pad_id: Uuid,
    padstack_id: Uuid,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if pad_name.trim().is_empty() {
        bail!("footprint pad name must be non-empty");
    }
    if layer <= 0 {
        bail!("footprint pad layer must be positive");
    }
    ensure_pool_object_exists(root, padstack_id, "padstacks", "padstack")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("footprint {footprint_id} payload is not an object"))?;
    let pads = object_map
        .entry("pads".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("footprint {footprint_id} pads field is not an object"))?;
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
    commit_pool_library_operations(
        root,
        format!("set native pool footprint {footprint_id} pad {pad_id}"),
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path: relative_path.clone(),
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_footprint_pad",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_footprint_courtyard_rect(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let vertices = courtyard_rect_vertices(min_x_nm, min_y_nm, max_x_nm, max_y_nm)?;
    set_native_project_pool_footprint_courtyard(
        root,
        pool_path,
        footprint_id,
        vertices,
        "set_footprint_courtyard_rect",
        format!("set native pool footprint {footprint_id} rectangular courtyard"),
    )
}

pub(crate) fn set_native_project_pool_footprint_courtyard_polygon(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: &str,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let vertices = parse_vertices(vertices, "footprint courtyard polygon")?;
    if vertices.len() < 3 {
        bail!("footprint courtyard polygon must have at least 3 vertices");
    }
    set_native_project_pool_footprint_courtyard(
        root,
        pool_path,
        footprint_id,
        vertices,
        "set_footprint_courtyard_polygon",
        format!("set native pool footprint {footprint_id} courtyard polygon"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_footprint_silkscreen_line(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let primitive =
        footprint_silkscreen_line_primitive(from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm)?;
    let (relative_path, previous_object, object) =
        footprint_object_with_appended_silkscreen(root, pool_path, footprint_id, primitive)?;
    commit_pool_library_operations(
        root,
        format!("add native pool footprint {footprint_id} silkscreen line"),
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path: relative_path.clone(),
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_footprint_silkscreen_line",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_footprint_silkscreen_rect(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let primitive =
        footprint_silkscreen_rect_primitive(min_x_nm, min_y_nm, max_x_nm, max_y_nm, width_nm)?;
    commit_footprint_silkscreen_primitive(
        root,
        pool_path,
        footprint_id,
        primitive,
        "add_footprint_silkscreen_rect",
        format!("add native pool footprint {footprint_id} silkscreen rect"),
    )
}

pub(crate) fn add_native_project_pool_footprint_silkscreen_circle(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let primitive =
        footprint_silkscreen_circle_primitive(center_x_nm, center_y_nm, radius_nm, width_nm)?;
    commit_footprint_silkscreen_primitive(
        root,
        pool_path,
        footprint_id,
        primitive,
        "add_footprint_silkscreen_circle",
        format!("add native pool footprint {footprint_id} silkscreen circle"),
    )
}

pub(crate) fn add_native_project_pool_footprint_silkscreen_polygon(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: &str,
    closed: bool,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let primitive = footprint_silkscreen_polygon_primitive(vertices, closed, width_nm)?;
    commit_footprint_silkscreen_primitive(
        root,
        pool_path,
        footprint_id,
        primitive,
        "add_footprint_silkscreen_polygon",
        format!("add native pool footprint {footprint_id} silkscreen polygon"),
    )
}

pub(crate) fn footprint_object_with_courtyard(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: Vec<serde_json::Value>,
) -> Result<(String, serde_json::Value, serde_json::Value)> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, footprint_id, "footprints", "footprint")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let mut object = previous_object.clone();
    object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("footprint {footprint_id} payload is not an object"))?
        .insert(
            "courtyard".to_string(),
            serde_json::json!({"vertices": vertices, "closed": true}),
        );
    Ok((relative_path, previous_object, object))
}

pub(crate) fn footprint_object_with_appended_silkscreen(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    primitive: serde_json::Value,
) -> Result<(String, serde_json::Value, serde_json::Value)> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, footprint_id, "footprints", "footprint")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let mut object = previous_object.clone();
    append_silkscreen_primitive(&mut object, primitive)?;
    Ok((relative_path, previous_object, object))
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

fn set_native_project_pool_footprint_courtyard(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    vertices: Vec<serde_json::Value>,
    action: &'static str,
    reason: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let (relative_path, previous_object, object) =
        footprint_object_with_courtyard(root, pool_path, footprint_id, vertices)?;
    commit_pool_library_operations(
        root,
        reason,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path: relative_path.clone(),
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        action,
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

pub(crate) fn parse_vertices(vertices: &str, label: &str) -> Result<Vec<serde_json::Value>> {
    let mut parsed = Vec::new();
    for vertex in vertices.split(';') {
        let vertex = vertex.trim();
        if vertex.is_empty() {
            continue;
        }
        let Some((x, y)) = vertex.split_once(',') else {
            bail!("{label} vertex '{vertex}' must be formatted as x,y");
        };
        let x = x
            .trim()
            .parse::<i64>()
            .with_context(|| format!("{label} vertex '{vertex}' has invalid x coordinate"))?;
        let y = y
            .trim()
            .parse::<i64>()
            .with_context(|| format!("{label} vertex '{vertex}' has invalid y coordinate"))?;
        parsed.push(serde_json::json!({"x": x, "y": y}));
    }
    Ok(parsed)
}

pub(crate) fn courtyard_rect_vertices(
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
) -> Result<Vec<serde_json::Value>> {
    if min_x_nm >= max_x_nm {
        bail!("footprint courtyard min-x-nm must be less than max-x-nm");
    }
    if min_y_nm >= max_y_nm {
        bail!("footprint courtyard min-y-nm must be less than max-y-nm");
    }
    Ok(vec![
        serde_json::json!({"x": min_x_nm, "y": min_y_nm}),
        serde_json::json!({"x": max_x_nm, "y": min_y_nm}),
        serde_json::json!({"x": max_x_nm, "y": max_y_nm}),
        serde_json::json!({"x": min_x_nm, "y": max_y_nm}),
    ])
}

pub(crate) fn footprint_silkscreen_line_primitive(
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    if from_x_nm == to_x_nm && from_y_nm == to_y_nm {
        bail!("footprint silkscreen line must have distinct endpoints");
    }
    if width_nm <= 0 {
        bail!("footprint silkscreen line width-nm must be positive");
    }
    Ok(serde_json::json!({
        "Line": {
            "from": {"x": from_x_nm, "y": from_y_nm},
            "to": {"x": to_x_nm, "y": to_y_nm},
            "width": width_nm
        }
    }))
}

pub(crate) fn footprint_silkscreen_rect_primitive(
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    if min_x_nm >= max_x_nm {
        bail!("footprint silkscreen rect min-x-nm must be less than max-x-nm");
    }
    if min_y_nm >= max_y_nm {
        bail!("footprint silkscreen rect min-y-nm must be less than max-y-nm");
    }
    if width_nm <= 0 {
        bail!("footprint silkscreen rect width-nm must be positive");
    }
    Ok(serde_json::json!({
        "Rect": {
            "min": {"x": min_x_nm, "y": min_y_nm},
            "max": {"x": max_x_nm, "y": max_y_nm},
            "width": width_nm
        }
    }))
}

pub(crate) fn footprint_silkscreen_circle_primitive(
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    if radius_nm <= 0 {
        bail!("footprint silkscreen circle radius-nm must be positive");
    }
    if width_nm <= 0 {
        bail!("footprint silkscreen circle width-nm must be positive");
    }
    Ok(serde_json::json!({
        "Circle": {
            "center": {"x": center_x_nm, "y": center_y_nm},
            "radius": radius_nm,
            "width": width_nm
        }
    }))
}

pub(crate) fn footprint_silkscreen_polygon_primitive(
    vertices: &str,
    closed: bool,
    width_nm: i64,
) -> Result<serde_json::Value> {
    let vertices = parse_vertices(vertices, "footprint silkscreen polygon")?;
    if closed && vertices.len() < 3 {
        bail!("footprint silkscreen closed polygon must have at least 3 vertices");
    }
    if !closed && vertices.len() < 2 {
        bail!("footprint silkscreen polyline must have at least 2 vertices");
    }
    if width_nm <= 0 {
        bail!("footprint silkscreen polygon width-nm must be positive");
    }
    Ok(serde_json::json!({
        "Polygon": {
            "polygon": {"vertices": vertices, "closed": closed},
            "width": width_nm
        }
    }))
}

fn commit_footprint_silkscreen_primitive(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    primitive: serde_json::Value,
    action: &'static str,
    reason: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let (relative_path, previous_object, object) =
        footprint_object_with_appended_silkscreen(root, pool_path, footprint_id, primitive)?;
    commit_pool_library_operations(
        root,
        reason,
        vec![Operation::SetPoolLibraryObject {
            object_id: footprint_id,
            relative_path: relative_path.clone(),
            object_kind: "footprints".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        action,
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

fn append_silkscreen_primitive(
    object: &mut serde_json::Value,
    primitive: serde_json::Value,
) -> Result<()> {
    let silkscreen = object
        .as_object_mut()
        .context("pool footprint payload must be a JSON object")?
        .entry("silkscreen")
        .or_insert_with(|| serde_json::json!([]));
    silkscreen
        .as_array_mut()
        .context("pool footprint silkscreen field must be an array")?
        .push(primitive);
    Ok(())
}
