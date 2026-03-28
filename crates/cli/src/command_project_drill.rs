use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::{StackupLayerType, Via};
use eda_engine::export::render_excellon_drill;
use eda_engine::ir::geometry::Point;
use uuid::Uuid;

use super::{
    NativeProjectDrillExportView, NativeProjectDrillHoleClassBucketView,
    NativeProjectDrillHoleClassReportView, NativeProjectExcellonDrillComparisonView,
    NativeProjectExcellonDrillExportView, NativeProjectExcellonDrillHitDriftView,
    NativeProjectExcellonDrillInspectionView, NativeProjectExcellonDrillToolView,
    NativeProjectExcellonDrillValidationView, classify_via_hole_class, csv_escape,
    load_native_project, query_native_project_board_stackup, query_native_project_board_vias,
    render_mm_6,
};

pub(crate) fn export_native_project_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectDrillExportView> {
    let project = load_native_project(root)?;
    let mut vias = query_native_project_board_vias(root)?;
    vias.sort_by(|a, b| {
        a.position
            .x
            .cmp(&b.position.x)
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    let mut csv =
        String::from("via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer\n");
    for via in &vias {
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
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectDrillExportView {
        action: "export_drill".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: output_path.display().to_string(),
        rows: vias.len(),
    })
}

pub(crate) fn export_native_project_excellon_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectExcellonDrillExportView> {
    let project = load_native_project(root)?;
    let drill_hits = query_native_project_drill_hits(root)?;
    let (via_count, component_pad_count) = drill_hit_counts(&drill_hits);
    let tools = build_excellon_tool_views_for_drill_hits(&drill_hits);
    let tool_count = tools.len();
    let excellon = render_excellon_for_drill_hits(&drill_hits)
        .context("failed to render native board drill hits as Excellon drill")?;
    std::fs::write(output_path, excellon)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectExcellonDrillExportView {
        action: "export_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: output_path.display().to_string(),
        via_count,
        component_pad_count,
        hit_count: drill_hits.len(),
        tool_count,
        tools,
    })
}

pub(crate) fn validate_native_project_excellon_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillValidationView> {
    let project = load_native_project(root)?;
    let drill_hits = query_native_project_drill_hits(root)?;
    let (via_count, component_pad_count) = drill_hit_counts(&drill_hits);
    let tools = build_excellon_tool_views_for_drill_hits(&drill_hits);
    let tool_count = tools.len();
    let expected = render_excellon_for_drill_hits(&drill_hits)
        .context("failed to render expected native board drill hits as Excellon drill")?;
    let actual = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    Ok(NativeProjectExcellonDrillValidationView {
        action: "validate_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: drill_path.display().to_string(),
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        via_count,
        component_pad_count,
        hit_count: drill_hits.len(),
        tool_count,
        tools,
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
    let project = load_native_project(root)?;
    let drill_hits = query_native_project_drill_hits(root)?;
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
    let project = load_native_project(root)?;
    let drill_hits = query_native_project_drill_hits(root)?;
    let copper_layers = query_native_project_board_stackup(root)?
        .into_iter()
        .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
        .map(|layer| layer.id)
        .collect::<Vec<_>>();
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
struct NativeDrillHit {
    kind: NativeDrillHitKind,
    uuid: Uuid,
    net: Option<Uuid>,
    position: Point,
    drill_nm: i64,
    from_layer: i32,
    to_layer: i32,
}

fn query_native_project_drill_hits(root: &Path) -> Result<Vec<NativeDrillHit>> {
    let mut hits = query_native_project_board_vias(root)?
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
    hits.extend(query_native_project_component_drill_hits(root)?);
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

fn query_native_project_component_drill_hits(root: &Path) -> Result<Vec<NativeDrillHit>> {
    let project = load_native_project(root)?;
    let Some((top_copper, bottom_copper)) = native_project_outer_copper_pair(root)? else {
        return Ok(Vec::new());
    };
    let mut hits = Vec::new();
    for component_pads in project.board.component_pads.values() {
        for pad in component_pads {
            let Some(drill_nm) = pad.drill_nm else {
                continue;
            };
            if drill_nm <= 0 {
                continue;
            }
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
    Ok(hits)
}

fn native_project_outer_copper_pair(root: &Path) -> Result<Option<(i32, i32)>> {
    let mut copper_layers = query_native_project_board_stackup(root)?
        .into_iter()
        .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
        .map(|layer| layer.id)
        .collect::<Vec<_>>();
    if copper_layers.is_empty() {
        return Ok(None);
    }
    copper_layers.sort_unstable();
    Ok(Some((
        *copper_layers.first().expect("not empty"),
        *copper_layers.last().expect("not empty"),
    )))
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
