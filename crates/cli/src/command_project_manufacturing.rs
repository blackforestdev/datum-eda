use std::path::Path;

use anyhow::Result;

use crate::NativeProjectManufacturingReportView;

use super::{
    NativeProjectGerberPlanArtifactView, load_native_project, plan_native_project_gerber_export,
    query_native_project_board_components, query_native_project_board_vias,
    report_native_project_drill_hole_classes,
};

pub(crate) fn report_native_project_manufacturing(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingReportView> {
    let project = load_native_project(root)?;
    let component_count = query_native_project_board_components(root)?.len();
    let via_count = query_native_project_board_vias(root)?.len();
    let gerber_plan = plan_native_project_gerber_export(root, prefix_override)?;
    let drill_report = report_native_project_drill_hole_classes(root)?;
    let gerber_artifacts = gerber_plan
        .artifacts
        .into_iter()
        .map(|artifact| NativeProjectGerberPlanArtifactView {
            kind: artifact.kind,
            layer_id: artifact.layer_id,
            layer_name: artifact.layer_name,
            filename: artifact.filename,
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectManufacturingReportView {
        action: "report_manufacturing".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        prefix: gerber_plan.prefix,
        bom_component_count: component_count,
        pnp_component_count: component_count,
        drill_csv_row_count: via_count,
        excellon_via_count: drill_report.via_count,
        excellon_component_pad_count: drill_report.component_pad_count,
        excellon_hit_count: drill_report.hit_count,
        drill_hole_class_count: drill_report.class_count,
        gerber_artifact_count: gerber_artifacts.len(),
        gerber_artifacts,
    })
}
