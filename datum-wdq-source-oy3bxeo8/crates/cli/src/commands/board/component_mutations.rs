use crate::*;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::board_components::{
    BoardPackageAlignMode, BoardPackageEdit, BoardPackagePlacement, build_align_board_packages,
    build_delete_board_package, build_edit_board_package, build_place_board_package,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use crate::{
    NativeProjectBoardComponentAlignReportView, NativeProjectBoardComponentMutationReportView,
    load_native_project_with_resolved_board, load_native_project_with_resolved_board_and_model,
    materialize_supported_pool_package_graphics, native_project_board_component_report,
};

use crate::cli_commit_source;

fn cli_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

fn commit_board_component_edit(
    root: &Path,
    reason: &str,
    component_uuid: Uuid,
    edit: BoardPackageEdit,
) -> Result<()> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_edit_board_package(&model, cli_provenance(reason)?, component_uuid, edit)?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(())
}

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
    let materialized = board_package_materialization_payload_for_component(root, &component)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_place_board_package(
        &model,
        cli_provenance("place board component")?,
        &BoardPackagePlacement {
            package: component.clone(),
            materialized,
        },
    )?;
    commit_prepared(&mut model, root, prepared)?;

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
    commit_board_component_edit(
        root,
        "move board component",
        component_uuid,
        BoardPackageEdit::Position {
            x: position.x,
            y: position.y,
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

pub(crate) fn align_native_project_board_components(
    root: &Path,
    component_uuids: Vec<Uuid>,
    mode: ProjectBoardAlignMode,
) -> Result<NativeProjectBoardComponentAlignReportView> {
    let requested_count = component_uuids.len();
    let mode_label = project_align_mode_label(mode).to_string();
    let engine_mode = board_package_align_mode(mode);
    let mut model = ProjectResolver::new(root).resolve()?;
    let (prepared, plan) = build_align_board_packages(
        &model,
        cli_provenance("align board components")?,
        &component_uuids,
        engine_mode,
    )?;
    commit_prepared(&mut model, root, prepared)?;

    let project = load_native_project_with_resolved_board(root)?;
    Ok(NativeProjectBoardComponentAlignReportView {
        action: "align_board_components".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        mode: mode_label,
        requested_count,
        aligned_count: plan.aligned.len(),
        skipped_locked_count: plan.skipped_locked.len(),
        unchanged_count: plan.unchanged.len(),
        aligned_component_uuids: plan
            .aligned
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        skipped_locked_component_uuids: plan
            .skipped_locked
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        unchanged_component_uuids: plan
            .unchanged
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    })
}

pub(crate) fn set_native_project_board_component_part(
    root: &Path,
    component_uuid: Uuid,
    part_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    commit_board_component_edit(
        root,
        "set board component part",
        component_uuid,
        BoardPackageEdit::Part { part_id: part_uuid },
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
    commit_board_component_edit(
        root,
        "set board component package",
        component_uuid,
        BoardPackageEdit::Package {
            package_ref_id: package_uuid,
            previous_materialized,
            materialized,
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

pub(crate) fn current_board_component_materialization_payload(
    root: &Path,
    component_uuid: Uuid,
) -> Result<serde_json::Value> {
    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    if !project.board.packages.contains_key(&key) {
        anyhow::bail!("board component not found in native project: {component_uuid}");
    }
    Ok(component_materialization_payload(&project, &key))
}

pub(crate) fn board_package_materialization_payload_for_component(
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

fn board_package_align_mode(mode: ProjectBoardAlignMode) -> BoardPackageAlignMode {
    match mode {
        ProjectBoardAlignMode::Left => BoardPackageAlignMode::Left,
        ProjectBoardAlignMode::Right => BoardPackageAlignMode::Right,
        ProjectBoardAlignMode::Top => BoardPackageAlignMode::Top,
        ProjectBoardAlignMode::Bottom => BoardPackageAlignMode::Bottom,
        ProjectBoardAlignMode::HCenter => BoardPackageAlignMode::HCenter,
        ProjectBoardAlignMode::VCenter => BoardPackageAlignMode::VCenter,
        ProjectBoardAlignMode::DistributeH => BoardPackageAlignMode::DistributeH,
        ProjectBoardAlignMode::DistributeV => BoardPackageAlignMode::DistributeV,
    }
}

fn project_align_mode_label(mode: ProjectBoardAlignMode) -> &'static str {
    match mode {
        ProjectBoardAlignMode::Left => "left",
        ProjectBoardAlignMode::Right => "right",
        ProjectBoardAlignMode::Top => "top",
        ProjectBoardAlignMode::Bottom => "bottom",
        ProjectBoardAlignMode::HCenter => "hcenter",
        ProjectBoardAlignMode::VCenter => "vcenter",
        ProjectBoardAlignMode::DistributeH => "distribute-h",
        ProjectBoardAlignMode::DistributeV => "distribute-v",
    }
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
    commit_board_component_edit(
        root,
        "rotate board component",
        component_uuid,
        BoardPackageEdit::Rotation {
            rotation: rotation_deg,
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
    commit_board_component_edit(
        root,
        if locked {
            "set board component locked"
        } else {
            "clear board component locked"
        },
        component_uuid,
        BoardPackageEdit::Locked { locked },
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
    let prepared = build_delete_board_package(
        &model,
        cli_provenance("delete board component")?,
        component_uuid,
        value,
        materialized,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(report)
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectMoveBoardComponentArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            x_nm,
            y_nm,
        } = self;
        let report = move_native_project_board_component(
            &path,
            component_uuid,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectAlignBoardComponentsArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuids,
            mode,
        } = self;
        let report = align_native_project_board_components(&path, component_uuids, mode)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_align_text,
        );
        Ok((output, 0))
    }
}

impl SetBoardComponentPartArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            part_uuid,
        } = self;
        let report = set_native_project_board_component_part(&path, component_uuid, part_uuid)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl SetBoardComponentPackageArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            package_uuid,
        } = self;
        let report =
            set_native_project_board_component_package(&path, component_uuid, package_uuid)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl SetBoardComponentLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            layer,
        } = self;
        let report = set_native_project_board_component_layer(&path, component_uuid, layer)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl SetBoardComponentReferenceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            reference,
        } = self;
        let report =
            set_native_project_board_component_reference(&path, component_uuid, reference)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl SetBoardComponentValueArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            value,
        } = self;
        let report = set_native_project_board_component_value(&path, component_uuid, value)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectRotateBoardComponentArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
            rotation_deg,
        } = self;
        let report = rotate_native_project_board_component(&path, component_uuid, rotation_deg)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectSetBoardComponentLockedArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
        } = self;
        let report = set_native_project_board_component_locked(&path, component_uuid, true)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectClearBoardComponentLockedArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_uuid,
        } = self;
        let report = set_native_project_board_component_locked(&path, component_uuid, false)?;
        let output = render_report(
            format,
            &report,
            render_native_project_board_component_mutation_text,
        );
        Ok((output, 0))
    }
}
