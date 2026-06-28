use super::command_project_operation_guards::guarded_existing_object_operation;
use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

pub(crate) fn query_native_project_board_net_classes(root: &Path) -> Result<Vec<NetClass>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut net_classes = project
        .board
        .net_classes
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board net class"))
        .collect::<Result<Vec<NetClass>>>()?;
    net_classes.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(net_classes)
}

pub(crate) fn query_native_project_board_net_class(
    root: &Path,
    net_class_uuid: Uuid,
) -> Result<NetClass> {
    let project = load_native_project_with_resolved_board(root)?;
    let key = net_class_uuid.to_string();
    let entry = project
        .board
        .net_classes
        .get(&key)
        .cloned()
        .with_context(|| {
            format!("board net class not found in native project: {net_class_uuid}")
        })?;
    serde_json::from_value(entry).context("failed to parse board net class")
}

pub(crate) fn query_native_project_board_dimensions(root: &Path) -> Result<Vec<Dimension>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut dimensions = project
        .board
        .dimensions
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board dimension"))
        .collect::<Result<Vec<Dimension>>>()?;
    dimensions.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(dimensions)
}

pub(crate) fn place_native_project_board_net_class(
    root: &Path,
    name: String,
    clearance_nm: i64,
    track_width_nm: i64,
    via_drill_nm: i64,
    via_diameter_nm: i64,
    diffpair_width_nm: i64,
    diffpair_gap_nm: i64,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let net_class_uuid = Uuid::new_v4();
    let net_class = NetClass {
        uuid: net_class_uuid,
        name,
        clearance: clearance_nm,
        track_width: track_width_nm,
        via_drill: via_drill_nm,
        via_diameter: via_diameter_nm,
        diffpair_width: diffpair_width_nm,
        diffpair_gap: diffpair_gap_nm,
    };
    commit_board_operation(
        root,
        "place board net class",
        Operation::CreateBoardNetClass {
            net_class_id: net_class_uuid,
            net_class: serde_json::to_value(&net_class)
                .expect("native board net class serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_net_class_report(
        "place_board_net_class",
        &project,
        net_class,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn edit_native_project_board_net_class(
    root: &Path,
    net_class_uuid: Uuid,
    name: Option<String>,
    clearance_nm: Option<i64>,
    track_width_nm: Option<i64>,
    via_drill_nm: Option<i64>,
    via_diameter_nm: Option<i64>,
    diffpair_width_nm: Option<i64>,
    diffpair_gap_nm: Option<i64>,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let key = net_class_uuid.to_string();
    let entry = project
        .board
        .net_classes
        .get(&key)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board net class not found in native project: {net_class_uuid}")
        })?;
    let mut net_class: NetClass = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board net class in {}",
            project.board_path.display()
        )
    })?;
    if let Some(name) = name {
        net_class.name = name;
    }
    if let Some(clearance_nm) = clearance_nm {
        net_class.clearance = clearance_nm;
    }
    if let Some(track_width_nm) = track_width_nm {
        net_class.track_width = track_width_nm;
    }
    if let Some(via_drill_nm) = via_drill_nm {
        net_class.via_drill = via_drill_nm;
    }
    if let Some(via_diameter_nm) = via_diameter_nm {
        net_class.via_diameter = via_diameter_nm;
    }
    if let Some(diffpair_width_nm) = diffpair_width_nm {
        net_class.diffpair_width = diffpair_width_nm;
    }
    if let Some(diffpair_gap_nm) = diffpair_gap_nm {
        net_class.diffpair_gap = diffpair_gap_nm;
    }
    commit_board_operation(
        root,
        "edit board net class",
        Operation::SetBoardNetClass {
            net_class_id: net_class_uuid,
            net_class: serde_json::to_value(&net_class)
                .expect("native board net class serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_net_class_report(
        "edit_board_net_class",
        &project,
        net_class,
    ))
}

pub(crate) fn delete_native_project_board_net_class(
    root: &Path,
    net_class_uuid: Uuid,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .net_classes
        .get(&net_class_uuid.to_string())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board net class not found in native project: {net_class_uuid}")
        })?;
    let net_class: NetClass = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board net class in {}",
            project.board_path.display()
        )
    })?;
    commit_board_operation(
        root,
        "delete board net class",
        Operation::DeleteBoardNetClass {
            net_class_id: net_class_uuid,
            net_class: value,
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_net_class_report(
        "delete_board_net_class",
        &project,
        net_class,
    ))
}

fn commit_board_operation(root: &Path, reason: &str, operation: Operation) -> Result<()> {
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
                operations: guarded_existing_object_operation(&model, operation)?,
            },
        )
        .with_context(|| format!("failed to commit {reason}"))?;
    Ok(())
}

pub(crate) fn place_native_project_board_dimension(
    root: &Path,
    from: Point,
    to: Point,
    layer: i32,
    text: Option<String>,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let dimension_uuid = Uuid::new_v4();
    let dimension = Dimension {
        uuid: dimension_uuid,
        from,
        to,
        layer,
        text,
    };
    commit_board_operation(
        root,
        "place board dimension",
        Operation::CreateBoardDimension {
            dimension_id: dimension_uuid,
            dimension: serde_json::to_value(&dimension)
                .expect("native board dimension serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "place_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}

pub(crate) fn edit_native_project_board_dimension(
    root: &Path,
    dimension_uuid: Uuid,
    from_x_nm: Option<i64>,
    from_y_nm: Option<i64>,
    to_x_nm: Option<i64>,
    to_y_nm: Option<i64>,
    layer: Option<i32>,
    text: Option<String>,
    clear_text: bool,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let index = project
        .board
        .dimensions
        .iter()
        .position(|entry| {
            serde_json::from_value::<Dimension>(entry.clone())
                .map(|dimension| dimension.uuid == dimension_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board dimension not found in native project: {dimension_uuid}")
        })?;
    let mut dimension: Dimension = serde_json::from_value(project.board.dimensions[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board dimension in {}",
                project.board_path.display()
            )
        })?;
    if let Some(value) = from_x_nm {
        dimension.from.x = value;
    }
    if let Some(value) = from_y_nm {
        dimension.from.y = value;
    }
    if let Some(value) = to_x_nm {
        dimension.to.x = value;
    }
    if let Some(value) = to_y_nm {
        dimension.to.y = value;
    }
    if let Some(layer) = layer {
        dimension.layer = layer;
    }
    if let Some(text) = text {
        dimension.text = Some(text);
    }
    if clear_text {
        dimension.text = None;
    }
    commit_board_operation(
        root,
        "edit board dimension",
        Operation::SetBoardDimension {
            dimension_id: dimension_uuid,
            dimension: serde_json::to_value(&dimension)
                .expect("native board dimension serialization must succeed"),
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "edit_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}

pub(crate) fn delete_native_project_board_dimension(
    root: &Path,
    dimension_uuid: Uuid,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let index = project
        .board
        .dimensions
        .iter()
        .position(|entry| {
            serde_json::from_value::<Dimension>(entry.clone())
                .map(|dimension| dimension.uuid == dimension_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board dimension not found in native project: {dimension_uuid}")
        })?;
    let value = project.board.dimensions[index].clone();
    let dimension: Dimension = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board dimension in {}",
            project.board_path.display()
        )
    })?;
    commit_board_operation(
        root,
        "delete board dimension",
        Operation::DeleteBoardDimension {
            dimension_id: dimension_uuid,
            dimension: value,
        },
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "delete_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}
