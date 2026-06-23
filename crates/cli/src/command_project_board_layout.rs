use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::{BoardText, Keepout, StackupLayer};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use eda_engine::text::{TextFamilyId, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign};
use uuid::Uuid;

use super::{
    NativeOutline, NativePoint, NativeProjectBoardKeepoutMutationReportView,
    NativeProjectBoardNameMutationReportView, NativeProjectBoardOutlineMutationReportView,
    NativeProjectBoardStackupMutationReportView, NativeProjectBoardTextMutationReportView,
    load_native_project, load_native_project_with_resolved_board,
};

pub(crate) fn query_native_project_board_texts(root: &Path) -> Result<Vec<BoardText>> {
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    render_intent: Option<String>,
    family: Option<String>,
    style: Option<String>,
    style_class: Option<String>,
    h_align: Option<String>,
    v_align: Option<String>,
    mirrored: bool,
    keep_upright: bool,
    line_spacing_ratio_ppm: i32,
    bold: bool,
    italic: bool,
    layer: i32,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let text_uuid = Uuid::new_v4();
    if height_nm <= 0 {
        bail!("board text height must be positive");
    }
    if stroke_width_nm <= 0 {
        bail!("board text stroke width must be positive");
    }
    let render_intent = parse_text_render_intent(render_intent.as_deref())?;
    let h_align = parse_text_h_align(h_align.as_deref())?;
    let v_align = parse_text_v_align(v_align.as_deref())?;
    validate_line_spacing_ratio(line_spacing_ratio_ppm)?;
    let family_source = if family.is_some() {
        eda_engine::text::TextFamilySource::Explicit
    } else {
        eda_engine::text::TextFamilySource::ImplicitDefault
    };
    let board_text = BoardText {
        uuid: text_uuid,
        text: text.clone(),
        position,
        rotation: rotation_deg,
        render_intent,
        family: family
            .map(TextFamilyId)
            .unwrap_or_else(TextFamilyId::default),
        family_source,
        style: style.map(TextStyleId).unwrap_or_else(TextStyleId::default),
        h_align,
        v_align,
        mirrored,
        keep_upright,
        line_spacing_ratio_ppm,
        italic,
        bold,
        style_class,
        height_nm,
        stroke_width_nm,
        layer,
    };
    commit_board_layout_operation(
        root,
        "place board text",
        Operation::CreateBoardText {
            text_id: text_uuid,
            text: serde_json::to_value(&board_text)
                .expect("native board text serialization must succeed"),
        },
    )?;
    let project = load_native_project(root)?;
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
        render_intent: render_intent_to_string(board_text.render_intent),
        family: board_text.family.0.clone(),
        style: board_text.style.0.clone(),
        style_class: board_text.style_class.clone(),
        h_align: text_h_align_to_string(board_text.h_align),
        v_align: text_v_align_to_string(board_text.v_align),
        mirrored: board_text.mirrored,
        keep_upright: board_text.keep_upright,
        line_spacing_ratio_ppm: board_text.line_spacing_ratio_ppm,
        bold: board_text.bold,
        italic: board_text.italic,
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
    render_intent: Option<String>,
    family: Option<String>,
    style: Option<String>,
    style_class: Option<String>,
    h_align: Option<String>,
    v_align: Option<String>,
    mirrored: Option<bool>,
    keep_upright: Option<bool>,
    line_spacing_ratio_ppm: Option<i32>,
    bold: Option<bool>,
    italic: Option<bool>,
    layer: Option<i32>,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let project = load_native_project(root)?;
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
    if let Some(render_intent) = render_intent {
        board_text.render_intent = parse_text_render_intent(Some(&render_intent))?;
    }
    if let Some(family) = family {
        board_text.family = TextFamilyId(family);
        board_text.family_source = eda_engine::text::TextFamilySource::Explicit;
    }
    if let Some(style) = style {
        board_text.style = TextStyleId(style);
    }
    if let Some(style_class) = style_class {
        board_text.style_class = Some(style_class);
    }
    if let Some(h_align) = h_align {
        board_text.h_align = parse_text_h_align(Some(&h_align))?;
    }
    if let Some(v_align) = v_align {
        board_text.v_align = parse_text_v_align(Some(&v_align))?;
    }
    if let Some(mirrored) = mirrored {
        board_text.mirrored = mirrored;
    }
    if let Some(keep_upright) = keep_upright {
        board_text.keep_upright = keep_upright;
    }
    if let Some(line_spacing_ratio_ppm) = line_spacing_ratio_ppm {
        validate_line_spacing_ratio(line_spacing_ratio_ppm)?;
        board_text.line_spacing_ratio_ppm = line_spacing_ratio_ppm;
    }
    if let Some(bold) = bold {
        board_text.bold = bold;
    }
    if let Some(italic) = italic {
        board_text.italic = italic;
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
    validate_line_spacing_ratio(board_text.line_spacing_ratio_ppm)?;
    commit_board_layout_operation(
        root,
        "edit board text",
        Operation::SetBoardText {
            text_id: text_uuid,
            text: serde_json::to_value(&board_text)
                .expect("native board text serialization must succeed"),
        },
    )?;
    let project = load_native_project(root)?;
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
        render_intent: render_intent_to_string(board_text.render_intent),
        family: board_text.family.0.clone(),
        style: board_text.style.0.clone(),
        style_class: board_text.style_class.clone(),
        h_align: text_h_align_to_string(board_text.h_align),
        v_align: text_v_align_to_string(board_text.v_align),
        mirrored: board_text.mirrored,
        keep_upright: board_text.keep_upright,
        line_spacing_ratio_ppm: board_text.line_spacing_ratio_ppm,
        bold: board_text.bold,
        italic: board_text.italic,
        layer: board_text.layer,
    })
}

pub(crate) fn delete_native_project_board_text(
    root: &Path,
    text_uuid: Uuid,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let project = load_native_project(root)?;
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
    let value = project.board.texts[index].clone();
    let board_text: BoardText = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board text in {}",
            project.board_path.display()
        )
    })?;
    commit_board_layout_operation(
        root,
        "delete board text",
        Operation::DeleteBoardText {
            text_id: text_uuid,
            text: value,
        },
    )?;
    let project = load_native_project(root)?;
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
        render_intent: render_intent_to_string(board_text.render_intent),
        family: board_text.family.0.clone(),
        style: board_text.style.0.clone(),
        style_class: board_text.style_class.clone(),
        h_align: text_h_align_to_string(board_text.h_align),
        v_align: text_v_align_to_string(board_text.v_align),
        mirrored: board_text.mirrored,
        keep_upright: board_text.keep_upright,
        line_spacing_ratio_ppm: board_text.line_spacing_ratio_ppm,
        bold: board_text.bold,
        italic: board_text.italic,
        layer: board_text.layer,
    })
}

pub(crate) fn commit_board_layout_operation(
    root: &Path,
    reason: &str,
    operation: Operation,
) -> Result<()> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let expected_model_revision = model.model_revision.clone();
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(expected_model_revision),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: reason.to_string(),
                },
                operations: vec![operation],
            },
        )
        .with_context(|| format!("failed to commit {reason}"))?;
    Ok(())
}

fn parse_text_render_intent(value: Option<&str>) -> Result<TextRenderIntent> {
    match value.unwrap_or("manufacturing") {
        "manufacturing" => Ok(TextRenderIntent::Manufacturing),
        "annotation" => Ok(TextRenderIntent::Annotation),
        "branding" => Ok(TextRenderIntent::Branding),
        "documentation" => Ok(TextRenderIntent::Documentation),
        "ui_preview" => Ok(TextRenderIntent::UiPreview),
        other => bail!(
            "unsupported board text render intent '{}'; expected one of: manufacturing, annotation, branding, documentation, ui_preview",
            other
        ),
    }
}

fn render_intent_to_string(intent: TextRenderIntent) -> String {
    match intent {
        TextRenderIntent::Manufacturing => "manufacturing",
        TextRenderIntent::Annotation => "annotation",
        TextRenderIntent::Branding => "branding",
        TextRenderIntent::Documentation => "documentation",
        TextRenderIntent::UiPreview => "ui_preview",
    }
    .to_string()
}

fn parse_text_h_align(value: Option<&str>) -> Result<TextHAlign> {
    match value.unwrap_or("left") {
        "left" => Ok(TextHAlign::Left),
        "center" => Ok(TextHAlign::Center),
        "right" => Ok(TextHAlign::Right),
        other => bail!(
            "unsupported board text horizontal alignment '{}'; expected one of: left, center, right",
            other
        ),
    }
}

fn parse_text_v_align(value: Option<&str>) -> Result<TextVAlign> {
    match value.unwrap_or("bottom") {
        "top" => Ok(TextVAlign::Top),
        "center" => Ok(TextVAlign::Center),
        "bottom" => Ok(TextVAlign::Bottom),
        other => bail!(
            "unsupported board text vertical alignment '{}'; expected one of: top, center, bottom",
            other
        ),
    }
}

fn validate_line_spacing_ratio(line_spacing_ratio_ppm: i32) -> Result<()> {
    if line_spacing_ratio_ppm <= 0 {
        bail!("board text line spacing ratio must be positive");
    }
    Ok(())
}

fn text_h_align_to_string(align: TextHAlign) -> String {
    match align {
        TextHAlign::Left => "left",
        TextHAlign::Center => "center",
        TextHAlign::Right => "right",
    }
    .to_string()
}

fn text_v_align_to_string(align: TextVAlign) -> String {
    match align {
        TextVAlign::Top => "top",
        TextVAlign::Center => "center",
        TextVAlign::Bottom => "bottom",
    }
    .to_string()
}

pub(crate) fn place_native_project_board_keepout(
    root: &Path,
    kind: String,
    layers: Vec<i32>,
    polygon: Polygon,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let keepout_uuid = Uuid::new_v4();
    let keepout = Keepout {
        uuid: keepout_uuid,
        polygon,
        layers,
        kind: kind.clone(),
    };
    commit_board_layout_operation(
        root,
        "place board keepout",
        Operation::CreateBoardKeepout {
            keepout_id: keepout_uuid,
            keepout: serde_json::to_value(&keepout)
                .expect("native board keepout serialization must succeed"),
        },
    )?;
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
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
    commit_board_layout_operation(
        root,
        "edit board keepout",
        Operation::SetBoardKeepout {
            keepout_id: keepout_uuid,
            keepout: serde_json::to_value(&keepout)
                .expect("native board keepout serialization must succeed"),
        },
    )?;
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
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
    let value = project.board.keepouts[index].clone();
    let keepout: Keepout = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board keepout in {}",
            project.board_path.display()
        )
    })?;
    commit_board_layout_operation(
        root,
        "delete board keepout",
        Operation::DeleteBoardKeepout {
            keepout_id: keepout_uuid,
            keepout: value,
        },
    )?;
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
    let outline = NativeOutline {
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
    commit_board_layout_operation(
        root,
        "set board outline",
        Operation::SetBoardOutline {
            board_id: project.board.uuid,
            outline: serde_json::to_value(&outline)
                .expect("native board outline serialization must succeed"),
        },
    )?;
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
    let stackup = serde_json::json!({
        "layers": layers
            .into_iter()
            .map(|layer| {
                serde_json::to_value(layer)
                    .expect("native board stackup serialization must succeed")
            })
            .collect::<Vec<_>>(),
    });
    let layer_count = stackup["layers"].as_array().map_or(0, Vec::len);
    commit_board_layout_operation(
        root,
        "set board stackup",
        Operation::SetBoardStackup {
            board_id: project.board.uuid,
            stackup,
        },
    )?;
    let project = load_native_project(root)?;
    Ok(NativeProjectBoardStackupMutationReportView {
        action: "set_board_stackup".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        layer_count,
    })
}

pub(crate) fn set_native_project_board_name(
    root: &Path,
    name: String,
) -> Result<NativeProjectBoardNameMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("board name must not be empty");
    }
    let project = load_native_project_with_resolved_board(root)?;
    commit_board_layout_operation(
        root,
        "set board name",
        Operation::SetBoardName {
            board_id: project.board.uuid,
            name,
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardNameMutationReportView {
        action: "set_board_name".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        board_uuid: project.board.uuid.to_string(),
        name: project.board.name,
    })
}
