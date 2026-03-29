use std::collections::BTreeMap;
use std::path::Path;

use crate::{NativeProjectBoardComponentMechanicalView, NativeProjectBoardComponentSilkscreenView};
use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;
use uuid::Uuid;

use super::{
    LoadedNativeProject, NativeProjectBoardComponentModels3dView,
    NativeProjectBoardComponentPadsView, NativeProjectBoardComponentQueryPointView,
    NativeProjectBoardComponentQueryView, load_native_project,
};

pub(crate) fn query_native_project_board_component_views(
    root: &Path,
) -> Result<Vec<NativeProjectBoardComponentQueryView>> {
    let project = load_native_project(root)?;
    let mut components = project
        .board
        .packages
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board component"))
        .collect::<Result<Vec<PlacedPackage>>>()?;
    components.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(components
        .into_iter()
        .map(|component| native_project_board_component_query_view(&project, component))
        .collect())
}

pub(crate) fn query_native_project_board_components(root: &Path) -> Result<Vec<PlacedPackage>> {
    let project = load_native_project(root)?;
    let mut components = project
        .board
        .packages
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board component"))
        .collect::<Result<Vec<PlacedPackage>>>()?;
    components.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(components)
}

pub(crate) fn query_native_project_board_component_view(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentQueryView> {
    let project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let component = project
        .board
        .packages
        .get(&key)
        .cloned()
        .with_context(|| format!("board component not found: {key}"))
        .and_then(|value| {
            serde_json::from_value(value).context("failed to parse board component")
        })?;
    Ok(native_project_board_component_query_view(&project, component))
}

pub(crate) fn query_native_project_board_component_models_3d(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentModels3dView> {
    let project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let models = project
        .board
        .component_models_3d
        .get(&key)
        .cloned()
        .unwrap_or_default();
    Ok(NativeProjectBoardComponentModels3dView {
        component_uuid: key,
        model_count: models.len(),
        models,
    })
}

pub(crate) fn query_native_project_board_component_pads(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentPadsView> {
    let project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let pads = project
        .board
        .component_pads
        .get(&key)
        .cloned()
        .unwrap_or_default();
    Ok(NativeProjectBoardComponentPadsView {
        component_uuid: key,
        pad_count: pads.len(),
        pads,
    })
}

pub(crate) fn query_native_project_board_component_silkscreen(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentSilkscreenView> {
    let project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let texts = project
        .board
        .component_silkscreen_texts
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let lines = project
        .board
        .component_silkscreen
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let arcs = project
        .board
        .component_silkscreen_arcs
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let circles = project
        .board
        .component_silkscreen_circles
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let polygons = project
        .board
        .component_silkscreen_polygons
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let polylines = project
        .board
        .component_silkscreen_polylines
        .get(&key)
        .cloned()
        .unwrap_or_default();
    Ok(NativeProjectBoardComponentSilkscreenView {
        component_uuid: key,
        text_count: texts.len(),
        line_count: lines.len(),
        arc_count: arcs.len(),
        circle_count: circles.len(),
        polygon_count: polygons.len(),
        polyline_count: polylines.len(),
        texts,
        lines,
        arcs,
        circles,
        polygons,
        polylines,
    })
}

pub(crate) fn query_native_project_board_component_mechanical(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMechanicalView> {
    let project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let texts = project
        .board
        .component_mechanical_texts
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let lines = project
        .board
        .component_mechanical_lines
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let arcs = project
        .board
        .component_mechanical_arcs
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let circles = project
        .board
        .component_mechanical_circles
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let polygons = project
        .board
        .component_mechanical_polygons
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let polylines = project
        .board
        .component_mechanical_polylines
        .get(&key)
        .cloned()
        .unwrap_or_default();
    Ok(NativeProjectBoardComponentMechanicalView {
        component_uuid: key,
        text_count: texts.len(),
        line_count: lines.len(),
        arc_count: arcs.len(),
        circle_count: circles.len(),
        polygon_count: polygons.len(),
        polyline_count: polylines.len(),
        texts,
        lines,
        arcs,
        circles,
        polygons,
        polylines,
    })
}

pub(crate) fn native_project_board_component_query_view(
    project: &LoadedNativeProject,
    component: PlacedPackage,
) -> NativeProjectBoardComponentQueryView {
    let key = component.uuid.to_string();
    NativeProjectBoardComponentQueryView {
        uuid: key.clone(),
        part: component.part.to_string(),
        package: component.package.to_string(),
        reference: component.reference,
        value: component.value,
        position: NativeProjectBoardComponentQueryPointView {
            x: component.position.x,
            y: component.position.y,
        },
        rotation: component.rotation,
        layer: component.layer,
        locked: component.locked,
        has_persisted_component_silkscreen: component_has_persisted_silkscreen(project, &key),
        persisted_component_silkscreen_text_count: component_graphic_count(
            &project.board.component_silkscreen_texts,
            &key,
        ),
        persisted_component_silkscreen_line_count: component_graphic_count(
            &project.board.component_silkscreen,
            &key,
        ),
        persisted_component_silkscreen_arc_count: component_graphic_count(
            &project.board.component_silkscreen_arcs,
            &key,
        ),
        persisted_component_silkscreen_circle_count: component_graphic_count(
            &project.board.component_silkscreen_circles,
            &key,
        ),
        persisted_component_silkscreen_polygon_count: component_graphic_count(
            &project.board.component_silkscreen_polygons,
            &key,
        ),
        persisted_component_silkscreen_polyline_count: component_graphic_count(
            &project.board.component_silkscreen_polylines,
            &key,
        ),
        has_persisted_component_mechanical: component_has_persisted_mechanical(project, &key),
        persisted_component_mechanical_text_count: component_graphic_count(
            &project.board.component_mechanical_texts,
            &key,
        ),
        persisted_component_mechanical_line_count: component_graphic_count(
            &project.board.component_mechanical_lines,
            &key,
        ),
        persisted_component_mechanical_arc_count: component_graphic_count(
            &project.board.component_mechanical_arcs,
            &key,
        ),
        persisted_component_mechanical_circle_count: component_graphic_count(
            &project.board.component_mechanical_circles,
            &key,
        ),
        persisted_component_mechanical_polygon_count: component_graphic_count(
            &project.board.component_mechanical_polygons,
            &key,
        ),
        persisted_component_mechanical_polyline_count: component_graphic_count(
            &project.board.component_mechanical_polylines,
            &key,
        ),
        has_persisted_component_pads: component_package_pad_count(project, &key) > 0,
        persisted_component_pad_count: component_package_pad_count(project, &key),
        has_persisted_component_models_3d: component_model_count(project, &key) > 0,
        persisted_component_model_3d_count: component_model_count(project, &key),
    }
}

pub(crate) fn component_graphic_count<T>(
    map: &BTreeMap<String, Vec<T>>,
    component_key: &str,
) -> usize {
    map.get(component_key).map_or(0, Vec::len)
}

pub(super) fn component_has_persisted_silkscreen(
    project: &LoadedNativeProject,
    component_key: &str,
) -> bool {
    component_graphic_count(&project.board.component_silkscreen_texts, component_key) > 0
        || component_graphic_count(&project.board.component_silkscreen, component_key) > 0
        || component_graphic_count(&project.board.component_silkscreen_arcs, component_key) > 0
        || component_graphic_count(&project.board.component_silkscreen_circles, component_key) > 0
        || component_graphic_count(&project.board.component_silkscreen_polygons, component_key) > 0
        || component_graphic_count(&project.board.component_silkscreen_polylines, component_key) > 0
}

pub(super) fn component_has_persisted_mechanical(
    project: &LoadedNativeProject,
    component_key: &str,
) -> bool {
    component_graphic_count(&project.board.component_mechanical_texts, component_key) > 0
        || component_graphic_count(&project.board.component_mechanical_lines, component_key) > 0
        || component_graphic_count(&project.board.component_mechanical_arcs, component_key) > 0
        || component_graphic_count(&project.board.component_mechanical_circles, component_key) > 0
        || component_graphic_count(&project.board.component_mechanical_polygons, component_key) > 0
        || component_graphic_count(&project.board.component_mechanical_polylines, component_key) > 0
}

pub(super) fn component_model_count(project: &LoadedNativeProject, component_key: &str) -> usize {
    project
        .board
        .component_models_3d
        .get(component_key)
        .map_or(0, Vec::len)
}

pub(super) fn component_package_pad_count(
    project: &LoadedNativeProject,
    component_key: &str,
) -> usize {
    project
        .board
        .component_pads
        .get(component_key)
        .map_or(0, Vec::len)
}
