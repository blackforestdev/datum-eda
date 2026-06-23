use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use uuid::Uuid;

use super::{
    NativeProjectBoardComponentMutationReportView, load_native_project_with_resolved_board,
    load_native_project_with_resolved_board_and_model, materialize_supported_pool_package_graphics,
    native_project_board_component_report,
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
    let package = serde_json::to_value(&component)
        .expect("native board component serialization must succeed");
    let materialized = board_package_materialization_payload_for_component(root, &component)?;
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
                reason: "place board component".to_string(),
            },
            operations: vec![Operation::CreateBoardPackage {
                package_id: component_uuid,
                package,
                materialized,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
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
                reason: "move board component".to_string(),
            },
            operations: vec![Operation::SetBoardPackagePosition {
                package_id: component_uuid,
                x: position.x,
                y: position.y,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
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
                reason: "set board component part".to_string(),
            },
            operations: vec![Operation::SetBoardPackagePart {
                package_id: component_uuid,
                part_id: part_uuid,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
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
    let (mut project, _model) = load_native_project_with_resolved_board_and_model(root)?;
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
    let previous_materialized = component_materialization_payload(&project, &key);
    component.package = package_uuid;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    clear_loaded_component_materialization(&mut project, &key);
    materialize_supported_pool_package_graphics(&mut project, &component)?;
    let materialized = component_materialization_payload(&project, &key);
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
                reason: "set board component package".to_string(),
            },
            operations: vec![Operation::SetBoardPackagePackage {
                package_id: component_uuid,
                package_ref_id: package_uuid,
                previous_materialized,
                materialized,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    Ok(native_project_board_component_report(
        "set_board_component_package",
        &project,
        component,
    ))
}

fn board_package_materialization_payload_for_component(
    root: &Path,
    component: &PlacedPackage,
) -> Result<serde_json::Value> {
    let mut project = load_native_project_with_resolved_board(root)?;
    let key = component.uuid.to_string();
    clear_loaded_component_materialization(&mut project, &key);
    materialize_supported_pool_package_graphics(&mut project, component)?;
    Ok(component_materialization_payload(&project, &key))
}

fn clear_loaded_component_materialization(project: &mut super::LoadedNativeProject, key: &str) {
    project.board.component_silkscreen.remove(key);
    project.board.component_silkscreen_texts.remove(key);
    project.board.component_silkscreen_arcs.remove(key);
    project.board.component_silkscreen_circles.remove(key);
    project.board.component_silkscreen_polygons.remove(key);
    project.board.component_silkscreen_polylines.remove(key);
    project.board.component_mechanical_lines.remove(key);
    project.board.component_mechanical_texts.remove(key);
    project.board.component_mechanical_polygons.remove(key);
    project.board.component_mechanical_polylines.remove(key);
    project.board.component_mechanical_circles.remove(key);
    project.board.component_mechanical_arcs.remove(key);
    project.board.component_pads.remove(key);
    project.board.component_models_3d.remove(key);
}

fn component_materialization_payload(
    project: &super::LoadedNativeProject,
    key: &str,
) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen",
        &project.board.component_silkscreen,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_texts",
        &project.board.component_silkscreen_texts,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_arcs",
        &project.board.component_silkscreen_arcs,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_circles",
        &project.board.component_silkscreen_circles,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_polygons",
        &project.board.component_silkscreen_polygons,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_polylines",
        &project.board.component_silkscreen_polylines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_lines",
        &project.board.component_mechanical_lines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_texts",
        &project.board.component_mechanical_texts,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_polygons",
        &project.board.component_mechanical_polygons,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_polylines",
        &project.board.component_mechanical_polylines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_circles",
        &project.board.component_mechanical_circles,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_arcs",
        &project.board.component_mechanical_arcs,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_pads",
        &project.board.component_pads,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_models_3d",
        &project.board.component_models_3d,
        key,
    );
    serde_json::Value::Object(payload)
}

fn insert_component_materialization_map<T: serde::Serialize>(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    map: &std::collections::BTreeMap<String, Vec<T>>,
    key: &str,
) {
    if let Some(value) = map.get(key) {
        payload.insert(
            field.to_string(),
            serde_json::to_value(value)
                .expect("component materialization payload serialization must succeed"),
        );
    }
}

pub(crate) fn rotate_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
    rotation_deg: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
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
                reason: "rotate board component".to_string(),
            },
            operations: vec![Operation::SetBoardPackageRotation {
                package_id: component_uuid,
                rotation: rotation_deg,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
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
                reason: if locked {
                    "set board component locked".to_string()
                } else {
                    "clear board component locked".to_string()
                },
            },
            operations: vec![Operation::SetBoardPackageLocked {
                package_id: component_uuid,
                locked,
            }],
        },
    )?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .packages
        .get(&component_uuid.to_string())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board component not found in native project: {component_uuid}")
        })?;
    let component: PlacedPackage = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    let report =
        native_project_board_component_report("delete_board_component", &project, component);
    let materialized = component_materialization_payload(&project, &component_uuid.to_string());
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
                reason: "delete board component".to_string(),
            },
            operations: vec![Operation::DeleteBoardPackage {
                package_id: component_uuid,
                package: value,
                materialized,
            }],
        },
    )?;
    Ok(report)
}
