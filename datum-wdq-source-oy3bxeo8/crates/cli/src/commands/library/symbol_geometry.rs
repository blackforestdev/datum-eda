use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::payload::read_project_pool_object_payload;

// CLI command handler threads individually parsed flag values.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_symbol_line(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if from_x_nm == to_x_nm && from_y_nm == to_y_nm {
        bail!("pool symbol line must have distinct endpoints");
    }
    if width_nm <= 0 {
        bail!("pool symbol line width-nm must be positive");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
        serde_json::json!({
            "Line": {
                "from": {"x": from_x_nm, "y": from_y_nm},
                "to": {"x": to_x_nm, "y": to_y_nm},
                "width": width_nm
            }
        }),
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol line to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_line",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

pub(crate) fn add_native_project_pool_symbol_polygon(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    vertices: &str,
    closed: bool,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let vertices = parse_symbol_vertices(vertices)?;
    if closed && vertices.len() < 3 {
        bail!("pool symbol closed polygon must have at least 3 vertices");
    }
    if !closed && vertices.len() < 2 {
        bail!("pool symbol polyline must have at least 2 vertices");
    }
    if width_nm <= 0 {
        bail!("pool symbol polygon width-nm must be positive");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
        serde_json::json!({
            "Polygon": {
                "polygon": {"vertices": vertices, "closed": closed},
                "width": width_nm
            }
        }),
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol polygon to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_polygon",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_symbol_rect(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if min_x_nm >= max_x_nm {
        bail!("pool symbol rect min-x-nm must be less than max-x-nm");
    }
    if min_y_nm >= max_y_nm {
        bail!("pool symbol rect min-y-nm must be less than max-y-nm");
    }
    if width_nm <= 0 {
        bail!("pool symbol rect width-nm must be positive");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
        serde_json::json!({
            "Rect": {
                "min": {"x": min_x_nm, "y": min_y_nm},
                "max": {"x": max_x_nm, "y": max_y_nm},
                "width": width_nm
            }
        }),
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol rect to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_rect",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

pub(crate) fn add_native_project_pool_symbol_circle(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if radius_nm <= 0 {
        bail!("pool symbol circle radius-nm must be positive");
    }
    if width_nm <= 0 {
        bail!("pool symbol circle width-nm must be positive");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
        serde_json::json!({
            "Circle": {
                "center": {"x": center_x_nm, "y": center_y_nm},
                "radius": radius_nm,
                "width": width_nm
            }
        }),
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol circle to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_circle",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_native_project_pool_symbol_arc(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    start_angle: i32,
    end_angle: i32,
    width_nm: i64,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if radius_nm <= 0 {
        bail!("pool symbol arc radius-nm must be positive");
    }
    if width_nm <= 0 {
        bail!("pool symbol arc width-nm must be positive");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
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
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol arc to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_arc",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

pub(crate) fn add_native_project_pool_symbol_text(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    text: String,
    x_nm: i64,
    y_nm: i64,
    rotation: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let text = text.trim().to_string();
    if text.is_empty() {
        bail!("pool symbol text must not be empty");
    }
    ensure_pool_symbol_exists(root, symbol_id)?;
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    append_symbol_drawing(
        &mut object,
        serde_json::json!({
            "Text": {
                "text": text,
                "position": {"x": x_nm, "y": y_nm},
                "rotation": rotation
            }
        }),
    )?;
    commit_pool_library_operations(
        root,
        format!("add native pool symbol text to symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "add_symbol_text",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

// CLI command handler threads individually parsed flag values.
#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_symbol_pin_anchor(
    root: &Path,
    pool_path: &str,
    symbol_id: Uuid,
    pin_id: Uuid,
    x_nm: i64,
    y_nm: i64,
    orientation: String,
    length_nm: Option<i64>,
    decoration: String,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let orientation = validate_symbol_pin_orientation(orientation)?;
    let decoration = validate_symbol_pin_decoration(decoration)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let symbol = materialized_pool_object(&model, symbol_id, "symbols")?;
    let unit_id = uuid_field(&symbol, "unit", "pool symbol")?;
    let unit = materialized_pool_object(&model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pins map"))?;
    if !unit_pins.contains_key(&pin_id.to_string()) {
        bail!("pool unit {unit_id} has no pin {pin_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "symbols", symbol_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, symbol_id)?;
    let mut object = previous_object.clone();
    set_symbol_pin_anchor(
        &mut object,
        pin_id,
        x_nm,
        y_nm,
        &orientation,
        length_nm,
        &decoration,
    )?;
    commit_pool_library_operations(
        root,
        format!("set native pool symbol pin anchor {pin_id} on symbol {symbol_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "symbols", symbol_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_symbol_pin_anchor",
        pool_path,
        "symbols",
        symbol_id,
        &relative_path,
    )
}

fn ensure_pool_symbol_exists(root: &Path, symbol_id: Uuid) -> Result<()> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&symbol_id)
        .filter(|object| object.domain == "pool" && object.kind == "symbols")
        .is_none()
    {
        bail!("missing pool symbol {symbol_id}");
    }
    Ok(())
}

fn materialized_pool_object(
    model: &eda_engine::substrate::DesignModel,
    object_id: Uuid,
    object_kind: &str,
) -> Result<serde_json::Value> {
    let object = model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == object_kind)
        .ok_or_else(|| anyhow::anyhow!("missing pool {object_kind} {object_id}"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == object.source_shard_id)
        .ok_or_else(|| {
            anyhow::anyhow!("missing source shard for pool {object_kind} {object_id}")
        })?;
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .with_context(|| format!("failed to materialize pool {object_kind} {object_id}"))
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} has invalid {field} uuid {raw}"))
}

fn set_symbol_pin_anchor(
    object: &mut serde_json::Value,
    pin_id: Uuid,
    x_nm: i64,
    y_nm: i64,
    orientation: &str,
    length_nm: Option<i64>,
    decoration: &str,
) -> Result<()> {
    let anchors = object
        .as_object_mut()
        .context("pool symbol payload must be a JSON object")?
        .entry("pin_anchors".to_string())
        .or_insert_with(|| serde_json::json!([]));
    let anchors = anchors
        .as_array_mut()
        .context("pool symbol pin_anchors field must be an array")?;
    anchors.retain(|anchor| {
        anchor.get("pin").and_then(serde_json::Value::as_str) != Some(pin_id.to_string().as_str())
    });
    anchors.push(serde_json::json!({
        "pin": pin_id,
        "position": {"x": x_nm, "y": y_nm},
        "style": {
            "orientation": orientation,
            "length_nm": length_nm,
            "decoration": decoration
        }
    }));
    Ok(())
}

fn validate_symbol_pin_orientation(orientation: String) -> Result<String> {
    match orientation.as_str() {
        "Left" | "Right" | "Up" | "Down" => Ok(orientation),
        other => {
            bail!("unsupported symbol pin orientation {other}; expected Left, Right, Up, or Down")
        }
    }
}

fn validate_symbol_pin_decoration(decoration: String) -> Result<String> {
    match decoration.as_str() {
        "none" | "inverted" | "clock" | "inverted_clock" => Ok(decoration),
        other => bail!(
            "unsupported symbol pin decoration {other}; expected none, inverted, clock, or inverted_clock"
        ),
    }
}

fn append_symbol_drawing(object: &mut serde_json::Value, drawing: serde_json::Value) -> Result<()> {
    let drawings = object
        .as_object_mut()
        .context("pool symbol payload must be a JSON object")?
        .entry("drawings".to_string())
        .or_insert_with(|| serde_json::json!([]));
    drawings
        .as_array_mut()
        .context("pool symbol drawings field must be an array")?
        .push(drawing);
    Ok(())
}

fn parse_symbol_vertices(vertices: &str) -> Result<Vec<serde_json::Value>> {
    let vertices = vertices.trim();
    if vertices.is_empty() {
        bail!("pool symbol polygon vertices must not be empty");
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
                bail!("pool symbol polygon vertex {} must be x,y", index + 1);
            }
            Ok(serde_json::json!({"x": x, "y": y}))
        })
        .collect()
}
