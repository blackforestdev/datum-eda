use std::path::Path;

use anyhow::Result;

use super::{
    NativeProjectBoardSummaryView, NativeProjectRulesSummaryView,
    NativeProjectSchematicSummaryView, NativeProjectSummaryView,
    collect_native_project_pool_ref_views, collect_schematic_counts,
    component_has_persisted_mechanical, component_has_persisted_silkscreen, component_model_count,
    component_package_pad_count, load_native_project,
};

pub(crate) fn query_native_project_summary(root: &Path) -> Result<NativeProjectSummaryView> {
    let project = load_native_project(root)?;
    let schematic_counts = collect_schematic_counts(&project.root, &project.schematic)?;
    let components_with_persisted_silkscreen = project
        .board
        .packages
        .keys()
        .filter(|key| component_has_persisted_silkscreen(&project, key))
        .count();
    let components_with_persisted_mechanical = project
        .board
        .packages
        .keys()
        .filter(|key| component_has_persisted_mechanical(&project, key))
        .count();
    let components_with_persisted_models_3d = project
        .board
        .packages
        .keys()
        .filter(|key| component_model_count(&project, key) > 0)
        .count();
    let components_with_persisted_pads = project
        .board
        .packages
        .keys()
        .filter(|key| component_package_pad_count(&project, key) > 0)
        .count();
    let pool_refs = collect_native_project_pool_ref_views(&project);
    Ok(NativeProjectSummaryView {
        domain: "native_project",
        project_name: project.manifest.name,
        schema_version: project.manifest.schema_version,
        pools: project.manifest.pools.len(),
        pool_refs,
        schematic: NativeProjectSchematicSummaryView {
            sheets: project.schematic.sheets.len(),
            sheet_definitions: project.schematic.definitions.len(),
            sheet_instances: project.schematic.instances.len(),
            variants: project.schematic.variants.len(),
            symbols: schematic_counts.symbols,
            wires: schematic_counts.wires,
            junctions: schematic_counts.junctions,
            labels: schematic_counts.labels,
            ports: schematic_counts.ports,
            buses: schematic_counts.buses,
            bus_entries: schematic_counts.bus_entries,
            noconnects: schematic_counts.noconnects,
            texts: schematic_counts.texts,
            drawings: schematic_counts.drawings,
        },
        board: NativeProjectBoardSummaryView {
            name: project.board.name,
            layers: project.board.stackup.layers.len(),
            components: project.board.packages.len(),
            components_with_persisted_silkscreen,
            components_with_persisted_mechanical,
            components_with_persisted_pads,
            components_with_persisted_models_3d,
            pads: project.board.pads.len(),
            nets: project.board.nets.len(),
            net_classes: project.board.net_classes.len(),
            tracks: project.board.tracks.len(),
            vias: project.board.vias.len(),
            zones: project.board.zones.len(),
            keepouts: project.board.keepouts.len(),
            dimensions: project.board.dimensions.len(),
            texts: project.board.texts.len(),
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
        },
        rules: NativeProjectRulesSummaryView {
            count: project.rules.rules.len(),
        },
    })
}
