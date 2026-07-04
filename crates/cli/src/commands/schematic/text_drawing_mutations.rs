use super::connectivity_mutations::commit_schematic_write;
use super::*;
use eda_engine::api::native_write::schematic_sheets::{
    build_create_schematic_drawing, build_create_schematic_text, build_delete_schematic_drawing,
    build_delete_schematic_text, build_set_schematic_drawing, build_set_schematic_text,
};

pub(crate) fn place_native_project_text(
    root: &Path,
    sheet_uuid: Uuid,
    text: String,
    position: Point,
    rotation_deg: i32,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let text_uuid = Uuid::new_v4();
    let text_object = SchematicText {
        uuid: text_uuid,
        text: text.clone(),
        position,
        rotation: rotation_deg,
    };
    commit_schematic_write(root, "place schematic text", |model, provenance| {
        build_create_schematic_text(model, provenance, sheet_uuid, &text_object)
    })?;

    Ok(NativeProjectTextMutationReportView {
        action: "place_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
        text_uuid: text_uuid.to_string(),
        text,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
    })
}

pub(crate) fn edit_native_project_text(
    root: &Path,
    text_uuid: Uuid,
    text: Option<String>,
    position: Option<Point>,
    rotation_deg: Option<i32>,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut text_object) =
        load_native_text_mutation_target(&project, text_uuid)?;
    if let Some(text) = text {
        text_object.text = text;
    }
    if let Some(position) = position {
        text_object.position = position;
    }
    if let Some(rotation_deg) = rotation_deg {
        text_object.rotation = rotation_deg;
    }
    commit_schematic_write(root, "edit schematic text", |model, provenance| {
        build_set_schematic_text(model, provenance, sheet_uuid, &text_object)
    })?;

    Ok(NativeProjectTextMutationReportView {
        action: "edit_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        text_uuid: text_object.uuid.to_string(),
        text: text_object.text,
        x_nm: text_object.position.x,
        y_nm: text_object.position.y,
        rotation_deg: text_object.rotation,
    })
}

pub(crate) fn delete_native_project_text(
    root: &Path,
    text_uuid: Uuid,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, text_object) =
        load_native_text_mutation_target(&project, text_uuid)?;
    commit_schematic_write(root, "delete schematic text", |model, provenance| {
        build_delete_schematic_text(model, provenance, sheet_uuid, &text_object)
    })?;

    Ok(NativeProjectTextMutationReportView {
        action: "delete_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        text_uuid: text_object.uuid.to_string(),
        text: text_object.text,
        x_nm: text_object.position.x,
        y_nm: text_object.position.y,
        rotation_deg: text_object.rotation,
    })
}

fn schematic_sheet_path(project: &LoadedNativeProject, sheet_uuid: Uuid) -> Result<PathBuf> {
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    Ok(project.root.join("schematic").join(relative_path))
}

fn drawing_report(
    project: &LoadedNativeProject,
    action: &str,
    sheet_uuid: Uuid,
    sheet_path: &Path,
    drawing_uuid: Uuid,
    kind: &str,
    from: Point,
    to: Point,
) -> NativeProjectDrawingMutationReportView {
    NativeProjectDrawingMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: kind.to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    }
}

pub(crate) fn place_native_project_drawing_line(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_path = schematic_sheet_path(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    let drawing = SchematicPrimitive::Line {
        uuid: drawing_uuid,
        from,
        to,
    };
    commit_schematic_write(root, "place schematic drawing line", |model, provenance| {
        build_create_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;

    Ok(drawing_report(
        &project,
        "place_drawing_line",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "line",
        from,
        to,
    ))
}

pub(crate) fn place_native_project_drawing_rect(
    root: &Path,
    sheet_uuid: Uuid,
    min: Point,
    max: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_path = schematic_sheet_path(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    let drawing = SchematicPrimitive::Rect {
        uuid: drawing_uuid,
        min,
        max,
    };
    commit_schematic_write(root, "place schematic drawing rect", |model, provenance| {
        build_create_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;
    Ok(drawing_report(
        &project,
        "place_drawing_rect",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "rect",
        min,
        max,
    ))
}

pub(crate) fn place_native_project_drawing_circle(
    root: &Path,
    sheet_uuid: Uuid,
    center: Point,
    radius: i64,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_path = schematic_sheet_path(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    let drawing = SchematicPrimitive::Circle {
        uuid: drawing_uuid,
        center,
        radius,
    };
    commit_schematic_write(
        root,
        "place schematic drawing circle",
        |model, provenance| build_create_schematic_drawing(model, provenance, sheet_uuid, &drawing),
    )?;
    Ok(drawing_report(
        &project,
        "place_drawing_circle",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "circle",
        center,
        Point {
            x: center.x + radius,
            y: center.y,
        },
    ))
}

pub(crate) fn place_native_project_drawing_arc(
    root: &Path,
    sheet_uuid: Uuid,
    arc: Arc,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_path = schematic_sheet_path(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    let drawing = SchematicPrimitive::Arc {
        uuid: drawing_uuid,
        arc,
    };
    commit_schematic_write(root, "place schematic drawing arc", |model, provenance| {
        build_create_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;
    Ok(drawing_report(
        &project,
        "place_drawing_arc",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "arc",
        arc.center,
        Point {
            x: arc.radius,
            y: i64::from(arc.end_angle),
        },
    ))
}

pub(crate) fn edit_native_project_drawing_line(
    root: &Path,
    drawing_uuid: Uuid,
    from: Option<Point>,
    to: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_from, current_to) = match drawing {
        SchematicPrimitive::Line { from, to, .. } => (from, to),
        _ => bail!("drawing is not a line: {drawing_uuid}"),
    };
    let from = from.unwrap_or(current_from);
    let to = to.unwrap_or(current_to);
    let drawing = SchematicPrimitive::Line {
        uuid: drawing_uuid,
        from,
        to,
    };
    commit_schematic_write(root, "edit schematic drawing line", |model, provenance| {
        build_set_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;

    Ok(drawing_report(
        &project,
        "edit_drawing_line",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "line",
        from,
        to,
    ))
}

pub(crate) fn edit_native_project_drawing_rect(
    root: &Path,
    drawing_uuid: Uuid,
    min: Option<Point>,
    max: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_min, current_max) = match drawing {
        SchematicPrimitive::Rect { min, max, .. } => (min, max),
        _ => bail!("drawing is not a rect: {drawing_uuid}"),
    };
    let min = min.unwrap_or(current_min);
    let max = max.unwrap_or(current_max);
    let drawing = SchematicPrimitive::Rect {
        uuid: drawing_uuid,
        min,
        max,
    };
    commit_schematic_write(root, "edit schematic drawing rect", |model, provenance| {
        build_set_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;
    Ok(drawing_report(
        &project,
        "edit_drawing_rect",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "rect",
        min,
        max,
    ))
}

pub(crate) fn edit_native_project_drawing_circle(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_center, current_radius) = match drawing {
        SchematicPrimitive::Circle { center, radius, .. } => (center, radius),
        _ => bail!("drawing is not a circle: {drawing_uuid}"),
    };
    let center = center.unwrap_or(current_center);
    let radius = radius.unwrap_or(current_radius);
    let drawing = SchematicPrimitive::Circle {
        uuid: drawing_uuid,
        center,
        radius,
    };
    commit_schematic_write(
        root,
        "edit schematic drawing circle",
        |model, provenance| build_set_schematic_drawing(model, provenance, sheet_uuid, &drawing),
    )?;
    Ok(drawing_report(
        &project,
        "edit_drawing_circle",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "circle",
        center,
        Point {
            x: center.x + radius,
            y: center.y,
        },
    ))
}

pub(crate) fn edit_native_project_drawing_arc(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
    start_angle: Option<i32>,
    end_angle: Option<i32>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let current_arc = match drawing {
        SchematicPrimitive::Arc { arc, .. } => arc,
        _ => bail!("drawing is not an arc: {drawing_uuid}"),
    };
    let arc = Arc {
        center: center.unwrap_or(current_arc.center),
        radius: radius.unwrap_or(current_arc.radius),
        start_angle: start_angle.unwrap_or(current_arc.start_angle),
        end_angle: end_angle.unwrap_or(current_arc.end_angle),
    };
    let drawing = SchematicPrimitive::Arc {
        uuid: drawing_uuid,
        arc,
    };
    commit_schematic_write(root, "edit schematic drawing arc", |model, provenance| {
        build_set_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;
    Ok(drawing_report(
        &project,
        "edit_drawing_arc",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        "arc",
        arc.center,
        Point {
            x: arc.radius,
            y: i64::from(arc.end_angle),
        },
    ))
}

pub(crate) fn delete_native_project_drawing(
    root: &Path,
    drawing_uuid: Uuid,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (kind, from, to) = match drawing {
        SchematicPrimitive::Line { from, to, .. } => ("line".to_string(), from, to),
        SchematicPrimitive::Rect { min, max, .. } => ("rect".to_string(), min, max),
        SchematicPrimitive::Circle { center, radius, .. } => (
            "circle".to_string(),
            center,
            Point {
                x: center.x + radius,
                y: center.y,
            },
        ),
        SchematicPrimitive::Arc { arc, .. } => ("arc".to_string(), arc.center, arc.center),
    };
    let (_, _, _, drawing) = load_native_drawing_mutation_target(&project, drawing_uuid)?;
    commit_schematic_write(root, "delete schematic drawing", |model, provenance| {
        build_delete_schematic_drawing(model, provenance, sheet_uuid, &drawing)
    })?;

    Ok(drawing_report(
        &project,
        "delete_drawing",
        sheet_uuid,
        &sheet_path,
        drawing_uuid,
        &kind,
        from,
        to,
    ))
}
