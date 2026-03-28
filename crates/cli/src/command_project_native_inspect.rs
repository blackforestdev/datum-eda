use std::path::Path;

use anyhow::Result;

use super::{
    NativeProjectInspectPoolRefView, NativeProjectInspectReportView,
    component_has_persisted_mechanical, component_has_persisted_silkscreen, component_model_count,
    component_package_pad_count, load_native_project, resolve_native_project_pool_path,
};

pub(crate) fn inspect_native_project(root: &Path) -> Result<NativeProjectInspectReportView> {
    let project = load_native_project(root)?;
    let pool_refs = project
        .manifest
        .pools
        .iter()
        .map(|pool_ref| {
            let resolved_path = resolve_native_project_pool_path(&project.root, &pool_ref.path);
            NativeProjectInspectPoolRefView {
                manifest_path: pool_ref.path.clone(),
                priority: pool_ref.priority,
                resolved_path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            }
        })
        .collect();

    Ok(NativeProjectInspectReportView {
        project_root: project.root.display().to_string(),
        project_name: project.manifest.name.clone(),
        schema_version: project.manifest.schema_version,
        project_uuid: project.manifest.uuid.to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        board_uuid: project.board.uuid.to_string(),
        pools: project.manifest.pools.len(),
        pool_refs,
        schematic_path: project.schematic_path.display().to_string(),
        board_path: project.board_path.display().to_string(),
        rules_path: project.rules_path.display().to_string(),
        sheet_count: project.schematic.sheets.len(),
        sheet_definition_count: project.schematic.definitions.len(),
        sheet_instance_count: project.schematic.instances.len(),
        variant_count: project.schematic.variants.len(),
        board_package_count: project.board.packages.len(),
        board_components_with_persisted_silkscreen: project
            .board
            .packages
            .keys()
            .filter(|key| component_has_persisted_silkscreen(&project, key))
            .count(),
        board_components_with_persisted_mechanical: project
            .board
            .packages
            .keys()
            .filter(|key| component_has_persisted_mechanical(&project, key))
            .count(),
        board_components_with_persisted_pads: project
            .board
            .packages
            .keys()
            .filter(|key| component_package_pad_count(&project, key) > 0)
            .count(),
        board_components_with_persisted_models_3d: project
            .board
            .packages
            .keys()
            .filter(|key| component_model_count(&project, key) > 0)
            .count(),
        board_pad_count: project.board.pads.len(),
        board_net_count: project.board.nets.len(),
        board_track_count: project.board.tracks.len(),
        board_via_count: project.board.vias.len(),
        board_zone_count: project.board.zones.len(),
        persisted_component_silkscreen_texts: project
            .board
            .component_silkscreen_texts
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_lines: project
            .board
            .component_silkscreen
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_arcs: project
            .board
            .component_silkscreen_arcs
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_circles: project
            .board
            .component_silkscreen_circles
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_polygons: project
            .board
            .component_silkscreen_polygons
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_polylines: project
            .board
            .component_silkscreen_polylines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_texts: project
            .board
            .component_mechanical_texts
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_lines: project
            .board
            .component_mechanical_lines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_arcs: project
            .board
            .component_mechanical_arcs
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_circles: project
            .board
            .component_mechanical_circles
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_polygons: project
            .board
            .component_mechanical_polygons
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_polylines: project
            .board
            .component_mechanical_polylines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_pads: project.board.component_pads.values().map(Vec::len).sum(),
        persisted_component_models_3d: project
            .board
            .component_models_3d
            .values()
            .map(Vec::len)
            .sum(),
        rule_count: project.rules.rules.len(),
    })
}
