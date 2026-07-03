use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_footprint::{
    footprint_object_with_appended_silkscreen, footprint_object_with_courtyard,
};
use super::command_project_library_package_pad::legacy_target_footprint_for_package;
use super::command_project_library_payload::read_project_pool_object_payload;

pub(crate) fn set_native_project_pool_package_courtyard_rect(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = courtyard_rect_vertices(min_x_nm, min_y_nm, max_x_nm, max_y_nm)?;
    ensure_pool_package_exists(root, package_id)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let (footprint_id, _) = legacy_target_footprint_for_package(pool_path, package_id, &model)?;
    let (relative_path, object) =
        footprint_object_with_courtyard(root, pool_path, footprint_id, vertices)?;
    commit_pool_library_operations(
        root,
        format!(
            "route legacy package rectangular courtyard on package {package_id} to footprint {footprint_id}"
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                relative_path.clone(),
            ),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_package_courtyard_rect",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_package_courtyard_polygon(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    vertices: &str,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = parse_vertices(vertices, "package courtyard polygon")?;
    if vertices.len() < 3 {
        bail!("package courtyard polygon must have at least 3 vertices");
    }
    ensure_pool_package_exists(root, package_id)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let (footprint_id, _) = legacy_target_footprint_for_package(pool_path, package_id, &model)?;
    let (relative_path, object) =
        footprint_object_with_courtyard(root, pool_path, footprint_id, vertices)?;
    commit_pool_library_operations(
        root,
        format!(
            "route legacy package courtyard polygon on package {package_id} to footprint {footprint_id}"
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                relative_path.clone(),
            ),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_package_courtyard_polygon",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_package_silkscreen_line(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if from_x_nm == to_x_nm && from_y_nm == to_y_nm {
        bail!("package silkscreen line must have distinct endpoints");
    }
    if width_nm <= 0 {
        bail!("package silkscreen line width-nm must be positive");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Line": {
                "from": {"x": from_x_nm, "y": from_y_nm},
                "to": {"x": to_x_nm, "y": to_y_nm},
                "width": width_nm
            }
        }),
        "add_package_silkscreen_line",
        format!("add native pool package silkscreen line to package {package_id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_package_silkscreen_rect(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if min_x_nm >= max_x_nm {
        bail!("package silkscreen rect min-x-nm must be less than max-x-nm");
    }
    if min_y_nm >= max_y_nm {
        bail!("package silkscreen rect min-y-nm must be less than max-y-nm");
    }
    if width_nm <= 0 {
        bail!("package silkscreen rect width-nm must be positive");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Rect": {
                "min": {"x": min_x_nm, "y": min_y_nm},
                "max": {"x": max_x_nm, "y": max_y_nm},
                "width": width_nm
            }
        }),
        "add_package_silkscreen_rect",
        format!("add native pool package silkscreen rect to package {package_id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_package_silkscreen_circle(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if radius_nm <= 0 {
        bail!("package silkscreen circle radius-nm must be positive");
    }
    if width_nm <= 0 {
        bail!("package silkscreen circle width-nm must be positive");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Circle": {
                "center": {"x": center_x_nm, "y": center_y_nm},
                "radius": radius_nm,
                "width": width_nm
            }
        }),
        "add_package_silkscreen_circle",
        format!("add native pool package silkscreen circle to package {package_id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_package_silkscreen_arc(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    start_angle: i32,
    end_angle: i32,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if radius_nm <= 0 {
        bail!("package silkscreen arc radius-nm must be positive");
    }
    if width_nm <= 0 {
        bail!("package silkscreen arc width-nm must be positive");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Arc": {
                "arc": {
                    "center": {"x": center_x_nm, "y": center_y_nm},
                    "radius": radius_nm,
                    "start_angle": start_angle,
                    "end_angle": end_angle
                },
                "width": width_nm
            }
        }),
        "add_package_silkscreen_arc",
        format!("add native pool package silkscreen arc to package {package_id}"),
    )
}

pub(crate) fn add_native_project_pool_package_silkscreen_polygon(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    vertices: &str,
    closed: bool,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = parse_vertices(vertices, "package silkscreen polygon")?;
    if closed && vertices.len() < 3 {
        bail!("package silkscreen closed polygon must have at least 3 vertices");
    }
    if !closed && vertices.len() < 2 {
        bail!("package silkscreen polyline must have at least 2 vertices");
    }
    if width_nm <= 0 {
        bail!("package silkscreen polygon width-nm must be positive");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Polygon": {
                "polygon": {"vertices": vertices, "closed": closed},
                "width": width_nm
            }
        }),
        "add_package_silkscreen_polygon",
        format!("add native pool package silkscreen polygon to package {package_id}"),
    )
}

pub(crate) fn add_native_project_pool_package_silkscreen_text(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    text: String,
    x_nm: i64,
    y_nm: i64,
    rotation: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let text = text.trim().to_string();
    if text.is_empty() {
        bail!("package silkscreen text must not be empty");
    }
    commit_legacy_package_silkscreen_primitive(
        root,
        pool_path,
        package_id,
        serde_json::json!({
            "Text": {
                "text": text,
                "position": {"x": x_nm, "y": y_nm},
                "rotation": rotation
            }
        }),
        "add_package_silkscreen_text",
        format!("add native pool package silkscreen text to package {package_id}"),
    )
}

fn commit_legacy_package_silkscreen_primitive(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    primitive: serde_json::Value,
    action: &'static str,
    reason: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    ensure_pool_package_exists(root, package_id)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let (footprint_id, _) = legacy_target_footprint_for_package(pool_path, package_id, &model)?;
    let (relative_path, object) =
        footprint_object_with_appended_silkscreen(root, pool_path, footprint_id, primitive)?;
    commit_pool_library_operations(
        root,
        reason,
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                relative_path.clone(),
            ),
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

pub(crate) fn add_native_project_pool_package_model_3d(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    model_path: &str,
    model_format: Option<String>,
    transform_json: Option<&str>,
    tx_nm: Option<i64>,
    ty_nm: Option<i64>,
    tz_nm: Option<i64>,
    roll_tenths_deg: Option<i32>,
    pitch_tenths_deg: Option<i32>,
    yaw_tenths_deg: Option<i32>,
    scale: Option<&str>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let model_path = validate_model_path(model_path)?;
    let model_format = parse_model_format(model_format.as_deref(), &model_path)?;
    let transform = parse_transform3d(
        transform_json,
        tx_nm,
        ty_nm,
        tz_nm,
        roll_tenths_deg,
        pitch_tenths_deg,
        yaw_tenths_deg,
        scale,
    )?;
    ensure_pool_package_exists(root, package_id)?;
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, package_id)?;
    let mut object = previous_object.clone();
    append_model_3d(
        &mut object,
        serde_json::to_value(eda_engine::pool::ModelRef {
            path: model_path,
            format: model_format,
            transform,
            provenance: None,
        })?,
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool package 3D model to package {package_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "packages", package_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_package_model_3d",
        pool_path,
        "packages",
        package_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_package_body_heights(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    clear: bool,
    body_height_nm: Option<i64>,
    body_height_mounted_nm: Option<i64>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if !clear && body_height_nm.is_none() && body_height_mounted_nm.is_none() {
        bail!("set-pool-package-body-heights requires --clear or at least one height field");
    }
    validate_optional_positive_height(body_height_nm, "body-height-nm")?;
    validate_optional_positive_height(body_height_mounted_nm, "body-height-mounted-nm")?;
    ensure_pool_package_exists(root, package_id)?;
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, package_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .context("pool package payload must be a JSON object")?;
    if clear {
        object_map.insert("body_height_nm".to_string(), serde_json::Value::Null);
        object_map.insert(
            "body_height_mounted_nm".to_string(),
            serde_json::Value::Null,
        );
    }
    if let Some(body_height_nm) = body_height_nm {
        object_map.insert(
            "body_height_nm".to_string(),
            serde_json::Value::Number(body_height_nm.into()),
        );
    }
    if let Some(body_height_mounted_nm) = body_height_mounted_nm {
        object_map.insert(
            "body_height_mounted_nm".to_string(),
            serde_json::Value::Number(body_height_mounted_nm.into()),
        );
    }
    commit_pool_library_operations(
        root,
        format!("set native pool package body heights on package {package_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "packages", package_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_package_body_heights",
        pool_path,
        "packages",
        package_id,
        &relative_path,
    )
}

fn parse_model_format(
    format: Option<&str>,
    model_path: &str,
) -> Result<eda_engine::pool::ModelFormat> {
    let inferred = || {
        let extension = Path::new(model_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "step" | "stp" => Ok(eda_engine::pool::ModelFormat::Step),
            "wrl" | "vrml" => Ok(eda_engine::pool::ModelFormat::Wrl),
            "igs" | "iges" => Ok(eda_engine::pool::ModelFormat::Iges),
            "obj" => Ok(eda_engine::pool::ModelFormat::Obj),
            "gltf" | "glb" => Ok(eda_engine::pool::ModelFormat::Gltf),
            _ => bail!("package model format could not be inferred; pass --format"),
        }
    };
    let Some(format) = format.map(str::trim).filter(|format| !format.is_empty()) else {
        return inferred();
    };
    match format.to_ascii_lowercase().as_str() {
        "step" | "stp" => Ok(eda_engine::pool::ModelFormat::Step),
        "wrl" | "vrml" => Ok(eda_engine::pool::ModelFormat::Wrl),
        "iges" | "igs" => Ok(eda_engine::pool::ModelFormat::Iges),
        "obj" => Ok(eda_engine::pool::ModelFormat::Obj),
        "gltf" | "glb" => Ok(eda_engine::pool::ModelFormat::Gltf),
        _ => bail!("package model format is not a supported ModelFormat value"),
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_transform3d(
    transform_json: Option<&str>,
    tx_nm: Option<i64>,
    ty_nm: Option<i64>,
    tz_nm: Option<i64>,
    roll_tenths_deg: Option<i32>,
    pitch_tenths_deg: Option<i32>,
    yaw_tenths_deg: Option<i32>,
    scale: Option<&str>,
) -> Result<eda_engine::pool::Transform3D> {
    let mut transform = transform_json
        .map(|value| {
            serde_json::from_str::<eda_engine::pool::Transform3D>(value)
                .context("invalid package model transform JSON")
        })
        .transpose()?
        .unwrap_or_default();
    if let Some(tx_nm) = tx_nm {
        transform.translation_nm.x = tx_nm;
    }
    if let Some(ty_nm) = ty_nm {
        transform.translation_nm.y = ty_nm;
    }
    if let Some(tz_nm) = tz_nm {
        transform.translation_nm.z = tz_nm;
    }
    if let Some(roll_tenths_deg) = roll_tenths_deg {
        transform.rotation_tenths_deg.roll_tenths_deg = roll_tenths_deg;
    }
    if let Some(pitch_tenths_deg) = pitch_tenths_deg {
        transform.rotation_tenths_deg.pitch_tenths_deg = pitch_tenths_deg;
    }
    if let Some(yaw_tenths_deg) = yaw_tenths_deg {
        transform.rotation_tenths_deg.yaw_tenths_deg = yaw_tenths_deg;
    }
    if let Some(scale) = scale {
        let scale = scale.trim();
        if scale.is_empty() {
            bail!("package model scale must be a positive JSON number");
        }
        let parsed: serde_json::Number = serde_json::from_str(scale)
            .context("package model scale must be a positive JSON number")?;
        if !parsed.as_f64().is_some_and(|value| value > 0.0) {
            bail!("package model scale must be a positive JSON number");
        }
        transform.scale = parsed;
    }
    Ok(transform)
}

fn validate_optional_positive_height(value: Option<i64>, name: &str) -> Result<()> {
    if value.is_some_and(|value| value <= 0) {
        bail!("{name} must be positive");
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

fn validate_model_path(model_path: &str) -> Result<String> {
    let model_path = model_path.trim();
    if model_path.is_empty() {
        bail!("package model path must not be empty");
    }
    let path = Path::new(model_path);
    if path.is_absolute() {
        bail!("package model path must be relative");
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("package model path must not contain traversal or special components");
    }
    Ok(model_path.to_string())
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

fn append_model_3d(object: &mut serde_json::Value, model: serde_json::Value) -> Result<()> {
    let models = object
        .as_object_mut()
        .context("pool package payload must be a JSON object")?
        .entry("models_3d")
        .or_insert_with(|| serde_json::json!([]));
    models
        .as_array_mut()
        .context("pool package models_3d field must be an array")?
        .push(model);
    Ok(())
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
