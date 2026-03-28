use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::{PadAperture, PadShape, PlacedPad};
use eda_engine::ir::geometry::Point;
use uuid::Uuid;

use super::{
    NativeComponentPad, NativeProjectBoardPadMutationReportView, load_native_project,
    native_project_board_pad_report, write_canonical_json,
};

pub(crate) fn query_native_project_board_pads(root: &Path) -> Result<Vec<PlacedPad>> {
    let project = load_native_project(root)?;
    let mut pads = project
        .board
        .pads
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board pad"))
        .collect::<Result<Vec<PlacedPad>>>()?;
    pads.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

fn query_native_project_component_pads(root: &Path) -> Result<Vec<PlacedPad>> {
    let project = load_native_project(root)?;
    let mut pads = Vec::new();
    for (component_key, component_pads) in &project.board.component_pads {
        let component_uuid = Uuid::parse_str(component_key).with_context(|| {
            format!(
                "failed to parse component UUID in {}",
                project.board_path.display()
            )
        })?;
        for pad in component_pads {
            if let Some(resolved) = native_component_pad_to_placed_pad(component_uuid, pad) {
                pads.push(resolved);
            }
        }
    }
    pads.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

pub(crate) fn query_native_project_emitted_copper_pads(root: &Path) -> Result<Vec<PlacedPad>> {
    let mut pads = query_native_project_board_pads(root)?;
    pads.extend(query_native_project_component_pads(root)?);
    pads.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

fn native_component_pad_to_placed_pad(
    component_uuid: Uuid,
    pad: &NativeComponentPad,
) -> Option<PlacedPad> {
    let shape = pad.shape?;
    Some(PlacedPad {
        uuid: pad.uuid,
        package: component_uuid,
        name: pad.name.clone(),
        net: None,
        position: Point {
            x: pad.position.x,
            y: pad.position.y,
        },
        layer: pad.layer,
        shape,
        diameter: pad.diameter_nm,
        width: pad.width_nm,
        height: pad.height_nm,
    })
}

pub(crate) fn set_native_project_board_pad_net(
    root: &Path,
    pad_uuid: Uuid,
    net_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    if let Some(net_uuid) = net_uuid
        && !project.board.nets.contains_key(&net_uuid.to_string())
    {
        bail!("board net not found in native project: {net_uuid}");
    }
    let key = pad_uuid.to_string();
    let entry = project
        .board
        .pads
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let mut pad: PlacedPad = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    pad.net = net_uuid;
    project.board.pads.insert(
        key,
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        if net_uuid.is_some() {
            "set_board_pad_net"
        } else {
            "clear_board_pad_net"
        },
        &project,
        pad,
    ))
}

fn parse_native_board_pad_shape(shape: &str) -> Result<PadShape> {
    match shape {
        "circle" => Ok(PadShape::Circle),
        "rect" => Ok(PadShape::Rect),
        _ => bail!("unsupported board pad shape: {shape}"),
    }
}

fn validate_native_board_pad_geometry(pad: &PlacedPad) -> Result<()> {
    match pad.aperture() {
        PadAperture::Circle { diameter_nm } if diameter_nm <= 0 => {
            bail!("board pad circular diameter must be positive");
        }
        PadAperture::Rect { width_nm, .. } if width_nm <= 0 => {
            bail!("board pad rectangular width must be positive");
        }
        PadAperture::Rect { height_nm, .. } if height_nm <= 0 => {
            bail!("board pad rectangular height must be positive");
        }
        _ => Ok(()),
    }
}

pub(crate) fn place_native_project_board_pad(
    root: &Path,
    package_uuid: Uuid,
    name: String,
    position: Point,
    layer: i32,
    shape: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    net_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    if let Some(net_uuid) = net_uuid
        && !project.board.nets.contains_key(&net_uuid.to_string())
    {
        bail!("board net not found in native project: {net_uuid}");
    }
    let pad_uuid = Uuid::new_v4();
    let shape = shape
        .as_deref()
        .map(parse_native_board_pad_shape)
        .transpose()?
        .unwrap_or(PadShape::Circle);
    let pad = PlacedPad {
        uuid: pad_uuid,
        package: package_uuid,
        name,
        net: net_uuid,
        position,
        layer,
        shape,
        diameter: diameter_nm.unwrap_or(0),
        width: width_nm.unwrap_or(0),
        height: height_nm.unwrap_or(0),
    };
    validate_native_board_pad_geometry(&pad)?;
    project.board.pads.insert(
        pad_uuid.to_string(),
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "place_board_pad",
        &project,
        pad,
    ))
}

pub(crate) fn edit_native_project_board_pad(
    root: &Path,
    pad_uuid: Uuid,
    position: Option<Point>,
    layer: Option<i32>,
    shape: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = pad_uuid.to_string();
    let entry = project
        .board
        .pads
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let mut pad: PlacedPad = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    if let Some(position) = position {
        pad.position = position;
    }
    if let Some(layer) = layer {
        pad.layer = layer;
    }
    if let Some(shape) = shape {
        pad.shape = parse_native_board_pad_shape(&shape)?;
    }
    if let Some(diameter_nm) = diameter_nm {
        pad.diameter = diameter_nm;
    }
    if let Some(width_nm) = width_nm {
        pad.width = width_nm;
    }
    if let Some(height_nm) = height_nm {
        pad.height = height_nm;
    }
    validate_native_board_pad_geometry(&pad)?;
    project.board.pads.insert(
        key,
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "edit_board_pad",
        &project,
        pad,
    ))
}

pub(crate) fn delete_native_project_board_pad(
    root: &Path,
    pad_uuid: Uuid,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .pads
        .remove(&pad_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let pad: PlacedPad = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "delete_board_pad",
        &project,
        pad,
    ))
}
