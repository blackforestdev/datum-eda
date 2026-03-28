use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use uuid::Uuid;

use super::{
    NativeProjectBoardComponentMutationReportView, load_native_project,
    materialize_supported_pool_package_graphics, native_project_board_component_report,
    write_canonical_json,
};

pub(crate) fn place_native_project_board_component(
    root: &Path,
    part_uuid: Uuid,
    package_uuid: Uuid,
    reference: String,
    value: String,
    position: Point,
    layer: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let component_uuid = Uuid::new_v4();
    let component = PlacedPackage {
        uuid: component_uuid,
        part: part_uuid,
        package: package_uuid,
        reference,
        value,
        position,
        rotation: 0,
        layer,
        locked: false,
    };
    project.board.packages.insert(
        component_uuid.to_string(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    materialize_supported_pool_package_graphics(&mut project, &component)?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "place_board_component",
        &project,
        component,
    ))
}

pub(crate) fn move_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.position = position;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "move_board_component",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_part(
    root: &Path,
    component_uuid: Uuid,
    part_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.part = part_uuid;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_part",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_package(
    root: &Path,
    component_uuid: Uuid,
    package_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.package = package_uuid;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    project.board.component_silkscreen.remove(&key);
    project.board.component_silkscreen_texts.remove(&key);
    project.board.component_silkscreen_arcs.remove(&key);
    project.board.component_silkscreen_circles.remove(&key);
    project.board.component_silkscreen_polygons.remove(&key);
    project.board.component_silkscreen_polylines.remove(&key);
    project.board.component_mechanical_lines.remove(&key);
    project.board.component_mechanical_texts.remove(&key);
    project.board.component_mechanical_polygons.remove(&key);
    project.board.component_mechanical_polylines.remove(&key);
    project.board.component_mechanical_circles.remove(&key);
    project.board.component_mechanical_arcs.remove(&key);
    project.board.component_pads.remove(&key);
    project.board.component_models_3d.remove(&key);
    materialize_supported_pool_package_graphics(&mut project, &component)?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_package",
        &project,
        component,
    ))
}

pub(crate) fn rotate_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
    rotation_deg: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.rotation = rotation_deg;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "rotate_board_component",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_locked(
    root: &Path,
    component_uuid: Uuid,
    locked: bool,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.locked = locked;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        if locked {
            "set_board_component_locked"
        } else {
            "clear_board_component_locked"
        },
        &project,
        component,
    ))
}

pub(crate) fn delete_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .packages
        .remove(&component_uuid.to_string())
        .ok_or_else(|| {
            anyhow::anyhow!("board component not found in native project: {component_uuid}")
        })?;
    let component: PlacedPackage = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    let report =
        native_project_board_component_report("delete_board_component", &project, component);
    project
        .board
        .component_silkscreen
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_texts
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_arcs
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_circles
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_polygons
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_polylines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_lines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_texts
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_polygons
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_polylines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_circles
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_arcs
        .remove(&component_uuid.to_string());
    project
        .board
        .component_pads
        .remove(&component_uuid.to_string());
    project
        .board
        .component_models_3d
        .remove(&component_uuid.to_string());
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(report)
}
