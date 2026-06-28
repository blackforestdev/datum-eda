use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::{PadAperture, PadShape, PlacedPad};
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use uuid::Uuid;

use super::{
    NativeProjectBoardPadMutationReportView,
    command_project_operation_guards::guarded_existing_object_operation,
    load_native_project_with_resolved_board, native_project_board_pad_report,
};

pub(crate) fn query_native_project_board_pads(root: &Path) -> Result<Vec<PlacedPad>> {
    let project = load_native_project_with_resolved_board(root)?;
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

pub(crate) fn set_native_project_board_pad_net(
    root: &Path,
    pad_uuid: Uuid,
    net_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_board_pad_operation(
        root,
        if net_uuid.is_some() {
            "set board pad net"
        } else {
            "clear board pad net"
        },
        Operation::SetBoardPad {
            pad_id: pad_uuid,
            pad: serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
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
        "oval" => Ok(PadShape::Oval),
        "roundrect" => Ok(PadShape::RoundRect),
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
    let project = load_native_project_with_resolved_board(root)?;
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
        copper_layers: vec![layer],
        shape,
        diameter: diameter_nm.unwrap_or(0),
        width: width_nm.unwrap_or(0),
        height: height_nm.unwrap_or(0),
        drill: 0,
        rotation: 0,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        roundrect_rratio_ppm: 250_000,
    };
    validate_native_board_pad_geometry(&pad)?;
    commit_board_pad_operation(
        root,
        "place board pad",
        Operation::CreateBoardPad {
            pad_id: pad_uuid,
            pad: serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_board_pad_operation(
        root,
        "edit board pad",
        Operation::SetBoardPad {
            pad_id: pad_uuid,
            pad: serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .pads
        .get(&pad_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let pad: PlacedPad = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    commit_board_pad_operation(
        root,
        "delete board pad",
        Operation::DeleteBoardPad {
            pad_id: pad_uuid,
            pad: value,
        },
    )?;
    Ok(native_project_board_pad_report(
        "delete_board_pad",
        &project,
        pad,
    ))
}

fn commit_board_pad_operation(root: &Path, reason: &str, operation: Operation) -> Result<()> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let expected_model_revision = model.model_revision.clone();
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: reason.to_string(),
            },
            operations: guarded_existing_object_operation(&model, operation)?,
        },
    )?;
    Ok(())
}
