use std::collections::BTreeMap;
use std::path::Path;

use super::{
    LoadedNativeProject, NativeProjectDrillComparisonView, NativeProjectDrillExportView,
    NativeProjectDrillHoleClassBucketView, NativeProjectDrillHoleClassReportView,
    NativeProjectDrillInspectionRowView, NativeProjectDrillInspectionView,
    NativeProjectDrillValidationView, NativeProjectExcellonDrillComparisonView,
    NativeProjectExcellonDrillExportView, NativeProjectExcellonDrillHitDriftView,
    NativeProjectExcellonDrillInspectionView, NativeProjectExcellonDrillToolView,
    NativeProjectExcellonDrillValidationView, classify_via_hole_class, csv_escape,
    load_native_project_with_resolved_board, render_mm_6,
};
use anyhow::{Context, Result, bail};
use eda_engine::board::{StackupLayer, StackupLayerType, Via};
use eda_engine::export::render_excellon_drill;
use eda_engine::ir::geometry::Point;
use uuid::Uuid;
#[path = "command_project_drill_csv.rs"]
mod command_project_drill_csv;
#[path = "command_project_drill_panel.rs"]
mod command_project_drill_panel;
#[path = "command_project_drill_projection.rs"]
mod command_project_drill_projection;
use command_project_drill_csv::{
    csv_drill_row_from_via, parse_native_project_drill_csv, parse_native_project_drill_csv_rows,
};
pub(crate) use command_project_drill_panel::{
    export_native_project_panel_drill, export_native_project_panel_excellon_drill,
    render_expected_native_project_panel_drill_csv,
    render_expected_native_project_panel_excellon_drill,
};
use command_project_drill_projection::{
    render_native_project_excellon_drill_projection,
    render_native_project_excellon_drill_projection_from_hits,
};

pub(crate) fn export_native_project_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectDrillExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let vias = sorted_native_project_board_vias(&project)?;
    write_native_project_drill_csv(output_path, &vias)?;
    Ok(NativeProjectDrillExportView {
        action: "export_drill".to_string(),
        production_classification: "manual_debug_export".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: output_path.display().to_string(),
        rows: vias.len(),
    })
}

pub(crate) fn validate_native_project_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectDrillValidationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let vias = sorted_native_project_board_vias(&project)?;
    let expected = render_native_project_drill_csv(&vias);
    let actual = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    Ok(NativeProjectDrillValidationView {
        action: "validate_drill".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: drill_path.display().to_string(),
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        rows: vias.len(),
    })
}
pub(crate) fn compare_native_project_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectDrillComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let expected = sorted_native_project_board_vias(&project)?
        .into_iter()
        .map(csv_drill_row_from_via)
        .collect::<BTreeMap<_, _>>();
    let actual = parse_native_project_drill_csv(drill_path)?;

    let matched = expected
        .iter()
        .filter_map(|(via_uuid, expected_row)| {
            actual
                .get(via_uuid)
                .filter(|actual_row| *actual_row == expected_row)
                .map(|_| via_uuid.to_string())
        })
        .collect::<Vec<_>>();
    let missing = expected
        .keys()
        .filter(|via_uuid| !actual.contains_key(*via_uuid))
        .map(Uuid::to_string)
        .collect::<Vec<_>>();
    let extra = actual
        .keys()
        .filter(|via_uuid| !expected.contains_key(*via_uuid))
        .map(Uuid::to_string)
        .collect::<Vec<_>>();
    let drift = expected
        .iter()
        .filter_map(|(via_uuid, expected_row)| {
            actual.get(via_uuid).and_then(|actual_row| {
                if actual_row == expected_row {
                    None
                } else {
                    Some(via_uuid.to_string())
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectDrillComparisonView {
        action: "compare_drill".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: drill_path.display().to_string(),
        expected_row_count: expected.len(),
        actual_row_count: actual.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        drift_count: drift.len(),
        matched,
        missing,
        extra,
        drift,
    })
}

pub(crate) fn inspect_native_project_drill(
    drill_path: &Path,
) -> Result<NativeProjectDrillInspectionView> {
    let rows = parse_native_project_drill_csv_rows(drill_path)?
        .into_iter()
        .map(|(via_uuid, row)| NativeProjectDrillInspectionRowView {
            via_uuid: via_uuid.to_string(),
            net_uuid: row.net_uuid.to_string(),
            x_nm: row.x_nm,
            y_nm: row.y_nm,
            drill_nm: row.drill_nm,
            diameter_nm: row.diameter_nm,
            from_layer: row.from_layer,
            to_layer: row.to_layer,
        })
        .collect::<Vec<_>>();
    Ok(NativeProjectDrillInspectionView {
        action: "inspect_drill".to_string(),
        drill_path: drill_path.display().to_string(),
        row_count: rows.len(),
        rows,
    })
}

pub(crate) fn export_native_project_excellon_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectExcellonDrillExportView> {
    let projection = render_native_project_excellon_drill_projection(root)?;
    write_native_project_excellon_drill(output_path, &projection.excellon)?;
    Ok(NativeProjectExcellonDrillExportView {
        action: "export_excellon_drill".to_string(),
        production_classification: "manual_debug_export".to_string(),
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

pub(crate) fn validate_native_project_excellon_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillValidationView> {
    let projection = render_native_project_excellon_drill_projection(root)?;
    let actual = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    Ok(NativeProjectExcellonDrillValidationView {
        action: "validate_excellon_drill".to_string(),
        project_root: projection.project_root,
        board_path: projection.board_path,
        drill_path: drill_path.display().to_string(),
        matches_expected: actual == projection.excellon,
        expected_bytes: projection.excellon.len(),
        actual_bytes: actual.len(),
        via_count: projection.via_count,
        component_pad_count: projection.component_pad_count,
        hit_count: projection.hit_count,
        tool_count: projection.tool_count,
        tools: projection.tools,
        production_projection: projection.production_projection,
    })
}

pub(crate) fn inspect_excellon_drill(
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillInspectionView> {
    let contents = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    let mut metric = false;
    let mut tools = BTreeMap::<String, NativeProjectExcellonDrillToolView>::new();
    let mut current_tool = None::<String>;
    let mut hit_count = 0usize;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line == "M48" || line == "%" || line == "M30" {
            continue;
        }
        if line == "METRIC,TZ" {
            metric = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix('T') {
            if let Some((tool_digits, diameter)) = rest.split_once('C') {
                let tool = format!("T{tool_digits}");
                tools.insert(
                    tool.clone(),
                    NativeProjectExcellonDrillToolView {
                        tool,
                        diameter_mm: diameter.to_string(),
                        hits: 0,
                    },
                );
                continue;
            }
            current_tool = Some(format!("T{rest}"));
            continue;
        }
        if line.starts_with('X') {
            let tool = current_tool.clone().with_context(|| {
                format!("drill hit without active tool in {}", drill_path.display())
            })?;
            let entry = tools.get_mut(&tool).with_context(|| {
                format!(
                    "drill hit references unknown tool `{tool}` in {}",
                    drill_path.display()
                )
            })?;
            entry.hits += 1;
            hit_count += 1;
        }
    }

    Ok(NativeProjectExcellonDrillInspectionView {
        action: "inspect_excellon_drill".to_string(),
        drill_path: drill_path.display().to_string(),
        metric,
        tool_count: tools.len(),
        hit_count,
        tools: tools.into_values().collect(),
    })
}

pub(crate) fn compare_native_project_excellon_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let drill_hits = query_native_project_drill_hits(&project)?;
    let expected_tools = build_excellon_tool_views_for_drill_hits(&drill_hits);
    let actual = inspect_excellon_drill(drill_path)?;

    let expected_by_diameter = expected_tools
        .iter()
        .map(|tool| (tool.diameter_mm.clone(), tool.hits))
        .collect::<BTreeMap<_, _>>();
    let actual_by_diameter = actual
        .tools
        .iter()
        .map(|tool| (tool.diameter_mm.clone(), tool.hits))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_diameter
        .iter()
        .filter_map(|(diameter, expected_hits)| {
            actual_by_diameter
                .get(diameter)
                .filter(|actual_hits| **actual_hits == *expected_hits)
                .map(|_| diameter.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_diameter
        .keys()
        .filter(|diameter| !actual_by_diameter.contains_key(*diameter))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_diameter
        .keys()
        .filter(|diameter| !expected_by_diameter.contains_key(*diameter))
        .cloned()
        .collect::<Vec<_>>();
    let hit_drift = expected_by_diameter
        .iter()
        .filter_map(|(diameter, expected_hits)| {
            actual_by_diameter.get(diameter).and_then(|actual_hits| {
                if actual_hits == expected_hits {
                    None
                } else {
                    Some(NativeProjectExcellonDrillHitDriftView {
                        diameter_mm: diameter.clone(),
                        expected_hits: *expected_hits,
                        actual_hits: *actual_hits,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectExcellonDrillComparisonView {
        action: "compare_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: drill_path.display().to_string(),
        expected_tool_count: expected_tools.len(),
        actual_tool_count: actual.tools.len(),
        expected_hit_count: drill_hits.len(),
        actual_hit_count: actual.hit_count,
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        hit_drift_count: hit_drift.len(),
        matched,
        missing,
        extra,
        hit_drift,
    })
}

pub(crate) fn report_native_project_drill_hole_classes(
    root: &Path,
) -> Result<NativeProjectDrillHoleClassReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let drill_hits = query_native_project_drill_hits(&project)?;
    let copper_layers = native_project_copper_layers(&project)?;
    let top_copper = copper_layers.iter().min().copied();
    let bottom_copper = copper_layers.iter().max().copied();

    let mut grouped = BTreeMap::<(String, i32, i32), Vec<NativeDrillHit>>::new();
    for hit in drill_hits.iter().cloned() {
        let start = hit.from_layer.min(hit.to_layer);
        let end = hit.from_layer.max(hit.to_layer);
        let class = classify_via_hole_class(start, end, top_copper, bottom_copper);
        grouped.entry((class, start, end)).or_default().push(hit);
    }

    let classes = grouped
        .into_iter()
        .map(|((class, from_layer, to_layer), hits)| {
            let (via_count, component_pad_count) = drill_hit_counts(&hits);
            let tools = build_excellon_tool_views_for_drill_hits(&hits);
            NativeProjectDrillHoleClassBucketView {
                class,
                from_layer,
                to_layer,
                via_count,
                component_pad_count,
                hit_count: hits.len(),
                tool_count: tools.len(),
                tools,
            }
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectDrillHoleClassReportView {
        action: "report_drill_hole_classes".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        copper_layer_count: copper_layers.len(),
        via_count: drill_hits
            .iter()
            .filter(|hit| matches!(hit.kind, NativeDrillHitKind::Via))
            .count(),
        component_pad_count: drill_hits
            .iter()
            .filter(|hit| matches!(hit.kind, NativeDrillHitKind::ComponentPad))
            .count(),
        hit_count: drill_hits.len(),
        class_count: classes.len(),
        classes,
    })
}

#[derive(Debug, Clone)]
enum NativeDrillHitKind {
    Via,
    ComponentPad,
}

#[derive(Debug, Clone)]
pub(super) struct NativeDrillHit {
    kind: NativeDrillHitKind,
    uuid: Uuid,
    net: Option<Uuid>,
    position: Point,
    drill_nm: i64,
    from_layer: i32,
    to_layer: i32,
}

pub(super) fn query_native_project_drill_hits(
    project: &LoadedNativeProject,
) -> Result<Vec<NativeDrillHit>> {
    let mut hits = sorted_native_project_board_vias(project)?
        .into_iter()
        .map(|via| NativeDrillHit {
            kind: NativeDrillHitKind::Via,
            uuid: via.uuid,
            net: Some(via.net),
            position: via.position,
            drill_nm: via.drill,
            from_layer: via.from_layer,
            to_layer: via.to_layer,
        })
        .collect::<Vec<_>>();
    hits.extend(query_native_project_component_drill_hits(project)?);
    hits.sort_by(|a, b| {
        a.drill_nm
            .cmp(&b.drill_nm)
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.from_layer.cmp(&b.from_layer))
            .then_with(|| a.to_layer.cmp(&b.to_layer))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(hits)
}

fn query_native_project_component_drill_hits(
    project: &LoadedNativeProject,
) -> Result<Vec<NativeDrillHit>> {
    let mut copper_layers = native_project_copper_layers(project)?;
    copper_layers.sort_unstable();
    let Some((top_copper, bottom_copper)) = copper_layers
        .first()
        .copied()
        .zip(copper_layers.last().copied())
    else {
        return Ok(Vec::new());
    };
    let mut hits = Vec::new();
    for component_pads in project.board.component_pads.values() {
        for pad in component_pads {
            let Some(drill_nm) = pad.drill_nm else {
                continue;
            };
            if drill_nm > 0 {
                hits.push(NativeDrillHit {
                    kind: NativeDrillHitKind::ComponentPad,
                    uuid: pad.uuid,
                    net: None,
                    position: Point {
                        x: pad.position.x,
                        y: pad.position.y,
                    },
                    drill_nm,
                    from_layer: top_copper,
                    to_layer: bottom_copper,
                });
            }
        }
    }
    Ok(hits)
}

fn native_project_copper_layers(project: &LoadedNativeProject) -> Result<Vec<i32>> {
    Ok(project
        .board
        .stackup
        .layers
        .iter()
        .cloned()
        .map(|value| {
            serde_json::from_value::<StackupLayer>(value)
                .context("failed to parse board stackup layer")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
        .map(|layer| layer.id)
        .collect())
}

pub(crate) fn render_expected_native_project_drill_csv(root: &Path) -> Result<String> {
    let project = load_native_project_with_resolved_board(root)?;
    let vias = sorted_native_project_board_vias(&project)?;
    Ok(render_native_project_drill_csv(&vias))
}

pub(super) fn sorted_native_project_board_vias(project: &LoadedNativeProject) -> Result<Vec<Via>> {
    let mut vias = project
        .board
        .vias
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board via"))
        .collect::<Result<Vec<Via>>>()?;
    vias.sort_by(|a, b| {
        a.position
            .x
            .cmp(&b.position.x)
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(vias)
}

pub(super) fn panel_instances_for_project_board_base<'a>(
    project: &LoadedNativeProject,
    panel_projection: &'a eda_engine::substrate::PanelProjection,
) -> Result<Vec<&'a eda_engine::substrate::PanelBoardInstance>> {
    let instances = panel_projection
        .board_instances
        .iter()
        .filter(|instance| instance.board == project.board.uuid)
        .collect::<Vec<_>>();
    if instances.is_empty() {
        bail!(
            "panel projection {} does not reference board {}",
            panel_projection.id,
            project.board.uuid
        );
    }
    Ok(instances)
}

pub(super) fn render_native_project_drill_csv(vias: &[Via]) -> String {
    let mut csv =
        String::from("via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer\n");
    for via in vias {
        let row = [
            csv_escape(&via.uuid.to_string()),
            csv_escape(&via.net.to_string()),
            via.position.x.to_string(),
            via.position.y.to_string(),
            via.drill.to_string(),
            via.diameter.to_string(),
            via.from_layer.to_string(),
            via.to_layer.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    csv
}

pub(super) fn write_native_project_drill_csv(output_path: &Path, vias: &[Via]) -> Result<()> {
    let csv = render_native_project_drill_csv(vias);
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))
}

pub(super) fn write_native_project_excellon_drill(
    output_path: &Path,
    excellon: &str,
) -> Result<()> {
    std::fs::write(output_path, excellon)
        .with_context(|| format!("failed to write {}", output_path.display()))
}

fn drill_hit_counts(hits: &[NativeDrillHit]) -> (usize, usize) {
    let via_count = hits
        .iter()
        .filter(|hit| matches!(hit.kind, NativeDrillHitKind::Via))
        .count();
    let component_pad_count = hits
        .iter()
        .filter(|hit| matches!(hit.kind, NativeDrillHitKind::ComponentPad))
        .count();
    (via_count, component_pad_count)
}

fn build_excellon_tool_views_for_drill_hits(
    hits: &[NativeDrillHit],
) -> Vec<NativeProjectExcellonDrillToolView> {
    let mut grouped = BTreeMap::<i64, usize>::new();
    for hit in hits {
        *grouped.entry(hit.drill_nm).or_default() += 1;
    }
    grouped
        .into_iter()
        .enumerate()
        .map(
            |(idx, (drill_nm, hits))| NativeProjectExcellonDrillToolView {
                tool: format!("T{:02}", idx + 1),
                diameter_mm: render_mm_6(drill_nm),
                hits,
            },
        )
        .collect()
}

fn render_excellon_for_drill_hits(
    hits: &[NativeDrillHit],
) -> Result<String, eda_engine::export::ExportError> {
    let vias = hits
        .iter()
        .map(|hit| Via {
            uuid: hit.uuid,
            net: hit.net.unwrap_or_else(Uuid::nil),
            position: hit.position,
            drill: hit.drill_nm,
            diameter: hit.drill_nm,
            from_layer: hit.from_layer,
            to_layer: hit.to_layer,
        })
        .collect::<Vec<_>>();
    render_excellon_drill(&vias)
}
