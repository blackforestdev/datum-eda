use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;

use super::{
    LoadedNativeProject, NativeProjectBoardComponentQueryPointView,
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
