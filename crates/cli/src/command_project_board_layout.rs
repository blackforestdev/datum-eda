use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::{BoardText, Keepout, StackupLayer};
use eda_engine::ir::geometry::{Point, Polygon};
use uuid::Uuid;

use super::{
    NativeOutline, NativePoint, NativeProjectBoardKeepoutMutationReportView,
    NativeProjectBoardOutlineMutationReportView, NativeProjectBoardStackupMutationReportView,
    NativeProjectBoardTextMutationReportView, NativeStackup, load_native_project,
    write_canonical_json,
};

pub(crate) fn query_native_project_board_texts(root: &Path) -> Result<Vec<BoardText>> {
    let project = load_native_project(root)?;
    let mut texts = project
        .board
        .texts
        .into_iter()
        .map(|entry| serde_json::from_value(entry).context("failed to parse board text"))
        .collect::<Result<Vec<BoardText>>>()?;
    texts.sort_by(|a, b| a.text.cmp(&b.text).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(texts)
}

pub(crate) fn query_native_project_board_keepouts(root: &Path) -> Result<Vec<Keepout>> {
    let project = load_native_project(root)?;
    let mut keepouts = project
        .board
        .keepouts
        .into_iter()
        .map(|entry| serde_json::from_value(entry).context("failed to parse board keepout"))
        .collect::<Result<Vec<Keepout>>>()?;
    keepouts.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(keepouts)
}

pub(crate) fn query_native_project_board_outline(root: &Path) -> Result<Polygon> {
    let project = load_native_project(root)?;
    Ok(Polygon {
        vertices: project
            .board
            .outline
            .vertices
            .iter()
            .map(|point| Point {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: project.board.outline.closed,
    })
}

pub(crate) fn query_native_project_board_stackup(root: &Path) -> Result<Vec<StackupLayer>> {
    let project = load_native_project(root)?;
    project
        .board
        .stackup
        .layers
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()
}

pub(crate) fn place_native_project_board_text(
    root: &Path,
    text: String,
    position: Point,
    rotation_deg: i32,
    height_nm: i64,
    stroke_width_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let text_uuid = Uuid::new_v4();
    if height_nm <= 0 {
        bail!("board text height must be positive");
    }
    if stroke_width_nm <= 0 {
        bail!("board text stroke width must be positive");
    }
    let board_text = BoardText {
        uuid: text_uuid,
        text: text.clone(),
        position,
        rotation: rotation_deg,
        height_nm,
        stroke_width_nm,
        layer,
    };
    project.board.texts.push(
        serde_json::to_value(&board_text).expect("native board text serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "place_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: text_uuid.to_string(),
        text,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
        height_nm,
        stroke_width_nm,
        layer,
    })
}

pub(crate) fn edit_native_project_board_text(
    root: &Path,
    text_uuid: Uuid,
    value: Option<String>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    rotation_deg: Option<i32>,
    height_nm: Option<i64>,
    stroke_width_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .texts
        .iter()
        .position(|entry| {
            serde_json::from_value::<BoardText>(entry.clone())
                .map(|text| text.uuid == text_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("board text not found in native project: {text_uuid}"))?;
    let mut board_text: BoardText = serde_json::from_value(project.board.texts[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board text in {}",
                project.board_path.display()
            )
        })?;
    if let Some(value) = value {
        board_text.text = value;
    }
    match (x_nm, y_nm) {
        (None, None) => {}
        (Some(x), Some(y)) => board_text.position = Point { x, y },
        _ => bail!("board text position requires both --x-nm and --y-nm"),
    }
    if let Some(rotation_deg) = rotation_deg {
        board_text.rotation = rotation_deg;
    }
    if let Some(height_nm) = height_nm {
        board_text.height_nm = height_nm;
    }
    if let Some(stroke_width_nm) = stroke_width_nm {
        board_text.stroke_width_nm = stroke_width_nm;
    }
    if let Some(layer) = layer {
        board_text.layer = layer;
    }
    if board_text.height_nm <= 0 {
        bail!("board text height must be positive");
    }
    if board_text.stroke_width_nm <= 0 {
        bail!("board text stroke width must be positive");
    }
    project.board.texts[index] =
        serde_json::to_value(&board_text).expect("native board text serialization must succeed");
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "edit_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: board_text.uuid.to_string(),
        text: board_text.text,
        x_nm: board_text.position.x,
        y_nm: board_text.position.y,
        rotation_deg: board_text.rotation,
        height_nm: board_text.height_nm,
        stroke_width_nm: board_text.stroke_width_nm,
        layer: board_text.layer,
    })
}

pub(crate) fn delete_native_project_board_text(
    root: &Path,
    text_uuid: Uuid,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .texts
        .iter()
        .position(|entry| {
            serde_json::from_value::<BoardText>(entry.clone())
                .map(|text| text.uuid == text_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("board text not found in native project: {text_uuid}"))?;
    let board_text: BoardText = serde_json::from_value(project.board.texts.remove(index))
        .with_context(|| {
            format!(
                "failed to parse board text in {}",
                project.board_path.display()
            )
        })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "delete_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: board_text.uuid.to_string(),
        text: board_text.text,
        x_nm: board_text.position.x,
        y_nm: board_text.position.y,
        rotation_deg: board_text.rotation,
        height_nm: board_text.height_nm,
        stroke_width_nm: board_text.stroke_width_nm,
        layer: board_text.layer,
    })
}

pub(crate) fn place_native_project_board_keepout(
    root: &Path,
    kind: String,
    layers: Vec<i32>,
    polygon: Polygon,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let keepout_uuid = Uuid::new_v4();
    let keepout = Keepout {
        uuid: keepout_uuid,
        polygon,
        layers,
        kind: kind.clone(),
    };
    project.board.keepouts.push(
        serde_json::to_value(&keepout).expect("native board keepout serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "place_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout_uuid.to_string(),
        kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn edit_native_project_board_keepout(
    root: &Path,
    keepout_uuid: Uuid,
    kind: Option<String>,
    layers: Vec<i32>,
    polygon: Option<Polygon>,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .keepouts
        .iter()
        .position(|entry| {
            serde_json::from_value::<Keepout>(entry.clone())
                .map(|keepout| keepout.uuid == keepout_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board keepout not found in native project: {keepout_uuid}")
        })?;
    let mut keepout: Keepout = serde_json::from_value(project.board.keepouts[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board keepout in {}",
                project.board_path.display()
            )
        })?;
    if let Some(kind) = kind {
        keepout.kind = kind;
    }
    if !layers.is_empty() {
        keepout.layers = layers;
    }
    if let Some(polygon) = polygon {
        keepout.polygon = polygon;
    }
    project.board.keepouts[index] =
        serde_json::to_value(&keepout).expect("native board keepout serialization must succeed");
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "edit_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout.uuid.to_string(),
        kind: keepout.kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn delete_native_project_board_keepout(
    root: &Path,
    keepout_uuid: Uuid,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .keepouts
        .iter()
        .position(|entry| {
            serde_json::from_value::<Keepout>(entry.clone())
                .map(|keepout| keepout.uuid == keepout_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board keepout not found in native project: {keepout_uuid}")
        })?;
    let keepout: Keepout = serde_json::from_value(project.board.keepouts.remove(index))
        .with_context(|| {
            format!(
                "failed to parse board keepout in {}",
                project.board_path.display()
            )
        })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "delete_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout.uuid.to_string(),
        kind: keepout.kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn set_native_project_board_outline(
    root: &Path,
    polygon: Polygon,
) -> Result<NativeProjectBoardOutlineMutationReportView> {
    let mut project = load_native_project(root)?;
    project.board.outline = NativeOutline {
        vertices: polygon
            .vertices
            .iter()
            .map(|point| NativePoint {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: polygon.closed,
    };
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardOutlineMutationReportView {
        action: "set_board_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        vertex_count: polygon.vertices.len(),
        closed: polygon.closed,
    })
}

pub(crate) fn set_native_project_board_stackup(
    root: &Path,
    layers: Vec<StackupLayer>,
) -> Result<NativeProjectBoardStackupMutationReportView> {
    let mut project = load_native_project(root)?;
    project.board.stackup = NativeStackup {
        layers: layers
            .into_iter()
            .map(|layer| {
                serde_json::to_value(layer)
                    .expect("native board stackup serialization must succeed")
            })
            .collect(),
    };
    let layer_count = project.board.stackup.layers.len();
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardStackupMutationReportView {
        action: "set_board_stackup".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        layer_count,
    })
}
