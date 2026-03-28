use super::*;

pub(crate) fn place_native_project_text(
    root: &Path,
    sheet_uuid: Uuid,
    text: String,
    position: Point,
    rotation_deg: i32,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing in {}", sheet_path.display()))?;

    let text_uuid = Uuid::new_v4();
    texts.insert(
        text_uuid.to_string(),
        serde_json::to_value(SchematicText {
            uuid: text_uuid,
            text: text.clone(),
            position,
            rotation: rotation_deg,
        })
        .expect("native text serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectTextMutationReportView {
        action: "place_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
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
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut text_object) =
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
    write_text_into_sheet(&mut sheet_value, &text_object)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

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
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, text_object) =
        load_native_text_mutation_target(&project, text_uuid)?;
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing in {}", sheet_path.display()))?;
    texts.remove(&text_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

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

pub(crate) fn place_native_project_drawing_line(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display())
        })?;

    let drawing_uuid = Uuid::new_v4();
    drawings.insert(
        drawing_uuid.to_string(),
        serde_json::to_value(SchematicPrimitive::Line {
            uuid: drawing_uuid,
            from,
            to,
        })
        .expect("native drawing serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_line".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "line".to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn place_native_project_drawing_rect(
    root: &Path,
    sheet_uuid: Uuid,
    min: Point,
    max: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Rect {
            uuid: drawing_uuid,
            min,
            max,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_rect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "rect".to_string(),
        from_x_nm: min.x,
        from_y_nm: min.y,
        to_x_nm: max.x,
        to_y_nm: max.y,
    })
}

pub(crate) fn place_native_project_drawing_circle(
    root: &Path,
    sheet_uuid: Uuid,
    center: Point,
    radius: i64,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Circle {
            uuid: drawing_uuid,
            center,
            radius,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_circle".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "circle".to_string(),
        from_x_nm: center.x,
        from_y_nm: center.y,
        to_x_nm: center.x + radius,
        to_y_nm: center.y,
    })
}

pub(crate) fn place_native_project_drawing_arc(
    root: &Path,
    sheet_uuid: Uuid,
    arc: Arc,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Arc {
            uuid: drawing_uuid,
            arc,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_arc".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "arc".to_string(),
        from_x_nm: arc.center.x,
        from_y_nm: arc.center.y,
        to_x_nm: arc.radius,
        to_y_nm: i64::from(arc.end_angle),
    })
}

pub(crate) fn edit_native_project_drawing_line(
    root: &Path,
    drawing_uuid: Uuid,
    from: Option<Point>,
    to: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_from, current_to) = match drawing {
        SchematicPrimitive::Line { from, to, .. } => (from, to),
        _ => bail!("drawing is not a line: {drawing_uuid}"),
    };
    let from = from.unwrap_or(current_from);
    let to = to.unwrap_or(current_to);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Line {
            uuid: drawing_uuid,
            from,
            to,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_line".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "line".to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn edit_native_project_drawing_rect(
    root: &Path,
    drawing_uuid: Uuid,
    min: Option<Point>,
    max: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_min, current_max) = match drawing {
        SchematicPrimitive::Rect { min, max, .. } => (min, max),
        _ => bail!("drawing is not a rect: {drawing_uuid}"),
    };
    let min = min.unwrap_or(current_min);
    let max = max.unwrap_or(current_max);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Rect {
            uuid: drawing_uuid,
            min,
            max,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_rect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "rect".to_string(),
        from_x_nm: min.x,
        from_y_nm: min.y,
        to_x_nm: max.x,
        to_y_nm: max.y,
    })
}

pub(crate) fn edit_native_project_drawing_circle(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_center, current_radius) = match drawing {
        SchematicPrimitive::Circle { center, radius, .. } => (center, radius),
        _ => bail!("drawing is not a circle: {drawing_uuid}"),
    };
    let center = center.unwrap_or(current_center);
    let radius = radius.unwrap_or(current_radius);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Circle {
            uuid: drawing_uuid,
            center,
            radius,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_circle".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "circle".to_string(),
        from_x_nm: center.x,
        from_y_nm: center.y,
        to_x_nm: center.x + radius,
        to_y_nm: center.y,
    })
}

pub(crate) fn edit_native_project_drawing_arc(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
    start_angle: Option<i32>,
    end_angle: Option<i32>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
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
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Arc {
            uuid: drawing_uuid,
            arc,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_arc".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "arc".to_string(),
        from_x_nm: arc.center.x,
        from_y_nm: arc.center.y,
        to_x_nm: arc.radius,
        to_y_nm: i64::from(arc.end_angle),
    })
}

pub(crate) fn delete_native_project_drawing(
    root: &Path,
    drawing_uuid: Uuid,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display())
        })?;
    drawings.remove(&drawing_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

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

    Ok(NativeProjectDrawingMutationReportView {
        action: "delete_drawing".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind,
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}
