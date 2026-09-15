use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::board::Via;
use eda_engine::substrate::{PanelBoardInstance, PanelProjection};

use super::{
    LoadedNativeProject, NativeDrillHit, NativeProjectDrillExportView,
    NativeProjectExcellonDrillExportView, panel_instances_for_project_board_base,
    query_native_project_drill_hits, render_native_project_drill_csv,
    render_native_project_excellon_drill_projection_from_hits, sorted_native_project_board_vias,
    write_native_project_drill_csv, write_native_project_excellon_drill,
};

pub(crate) fn export_native_project_panel_drill(
    root: &Path,
    output_path: &Path,
    panel_projection: &PanelProjection,
) -> Result<NativeProjectDrillExportView> {
    let project = super::load_native_project_with_resolved_board(root)?;
    let vias = panel_native_project_board_vias(&project, panel_projection)?;
    write_native_project_drill_csv(output_path, &vias)?;
    Ok(NativeProjectDrillExportView {
        action: "export_drill".to_string(),
        production_classification: "panel_projection_export".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: output_path.display().to_string(),
        rows: vias.len(),
    })
}

pub(crate) fn export_native_project_panel_excellon_drill(
    root: &Path,
    output_path: &Path,
    panel_projection: &PanelProjection,
) -> Result<NativeProjectExcellonDrillExportView> {
    let project = super::load_native_project_with_resolved_board(root)?;
    let drill_hits = panel_native_project_drill_hits(&project, panel_projection)?;
    let projection = render_native_project_excellon_drill_projection_from_hits(
        root,
        &drill_hits,
        "panel_excellon_drill",
        "datum.production_projection.panel_excellon_drill.v1",
    )?;
    write_native_project_excellon_drill(output_path, &projection.excellon)?;
    Ok(NativeProjectExcellonDrillExportView {
        action: "export_excellon_drill".to_string(),
        production_classification: "panel_projection_export".to_string(),
        project_root: projection.project_root,
        board_path: projection.board_path,
        drill_path: output_path.display().to_string(),
        via_count: projection.via_count,
        component_pad_count: projection.component_pad_count,
        hit_count: projection.hit_count,
        tool_count: projection.tool_count,
        tools: projection.tools,
        production_projection: projection.production_projection,
    })
}

pub(crate) fn render_expected_native_project_panel_drill_csv(
    root: &Path,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let project = super::load_native_project_with_resolved_board(root)?;
    let vias = panel_native_project_board_vias(&project, panel_projection)?;
    Ok(render_native_project_drill_csv(&vias))
}

pub(crate) fn render_expected_native_project_panel_excellon_drill(
    root: &Path,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let project = super::load_native_project_with_resolved_board(root)?;
    let drill_hits = panel_native_project_drill_hits(&project, panel_projection)?;
    Ok(render_native_project_excellon_drill_projection_from_hits(
        root,
        &drill_hits,
        "panel_excellon_drill",
        "datum.production_projection.panel_excellon_drill.v1",
    )?
    .excellon)
}

fn panel_native_project_drill_hits(
    project: &LoadedNativeProject,
    panel_projection: &PanelProjection,
) -> Result<Vec<NativeDrillHit>> {
    let base_hits = query_native_project_drill_hits(project)?;
    let mut hits = Vec::new();
    for instance in panel_instances_for_project_board(project, panel_projection)? {
        for hit in &base_hits {
            let mut hit = hit.clone();
            hit.position.x += instance.x_nm;
            hit.position.y += instance.y_nm;
            hits.push(hit);
        }
    }
    Ok(hits)
}

fn panel_native_project_board_vias(
    project: &LoadedNativeProject,
    panel_projection: &PanelProjection,
) -> Result<Vec<Via>> {
    let base_vias = sorted_native_project_board_vias(project)?;
    let mut vias = Vec::new();
    for instance in panel_instances_for_project_board(project, panel_projection)? {
        for via in &base_vias {
            let mut via = via.clone();
            via.position.x += instance.x_nm;
            via.position.y += instance.y_nm;
            vias.push(via);
        }
    }
    Ok(vias)
}

fn panel_instances_for_project_board<'a>(
    project: &LoadedNativeProject,
    panel_projection: &'a PanelProjection,
) -> Result<Vec<&'a PanelBoardInstance>> {
    let instances = panel_instances_for_project_board_base(project, panel_projection)?;
    for instance in &instances {
        if instance.rotation_deg != 0 {
            bail!(
                "panel projection {} has board instance rotation {}; panel drill export currently supports translation-only instances",
                panel_projection.id,
                instance.rotation_deg
            );
        }
    }
    Ok(instances)
}
