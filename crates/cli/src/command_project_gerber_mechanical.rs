use std::collections::BTreeMap;

#[path = "command_project_gerber_mechanical_support.rs"]
mod support;
use self::support::{
    count_native_board_dimensions, count_native_board_texts,
    count_native_component_mechanical_arcs, count_native_component_mechanical_circles,
    count_native_component_mechanical_lines, count_native_component_mechanical_polygons,
    count_native_component_mechanical_polylines, native_board_dimension_to_stroke,
    native_board_text_to_strokes, native_component_mechanical_arc_to_strokes,
    native_component_mechanical_circle_to_strokes, native_component_mechanical_line_to_stroke,
    native_component_mechanical_polygon_to_polygon,
    native_component_mechanical_polyline_to_strokes, native_component_mechanical_text_to_strokes,
};
use super::{
    NativeBoardRoot, NativePoint, NativeProjectGerberMechanicalComparisonView,
    NativeProjectGerberMechanicalExportView, NativeProjectGerberMechanicalValidationView,
    ParsedGerber, ParsedGerberGeometry, compare_entry_views, load_native_project,
    parse_rs274x_subset, query_native_project_board_dimensions,
    query_native_project_board_keepouts, query_native_project_board_stackup,
    query_native_project_board_texts, render_parsed_flash_geometry, render_region_geometry,
    render_stroke_geometry,
};
use anyhow::{Context, Result, bail};
use eda_engine::board::{PlacedPackage, StackupLayer, StackupLayerType};
use eda_engine::export::{MechanicalStroke, render_rs274x_mechanical_layer};
use eda_engine::ir::geometry::Polygon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalLine {
    pub(crate) from: NativePoint,
    pub(crate) to: NativePoint,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalPolygon {
    pub(crate) vertices: Vec<NativePoint>,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalPolyline {
    pub(crate) vertices: Vec<NativePoint>,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalCircle {
    pub(crate) center: NativePoint,
    pub(crate) radius_nm: i64,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalArc {
    pub(crate) center: NativePoint,
    pub(crate) radius_nm: i64,
    pub(crate) start_angle: i32,
    pub(crate) end_angle: i32,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentMechanicalText {
    pub(crate) text: String,
    pub(crate) position: NativePoint,
    pub(crate) rotation: i32,
    pub(crate) height_nm: i64,
    pub(crate) stroke_width_nm: i64,
    pub(crate) layer: i32,
}

pub(crate) fn resolve_native_project_mechanical_context(
    root: &std::path::Path,
    layer: i32,
) -> Result<(
    StackupLayer,
    Vec<Polygon>,
    Vec<Polygon>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
    Vec<MechanicalStroke>,
)> {
    let project = load_native_project(root)?;
    let stackup = query_native_project_board_stackup(root)?;
    let mechanical_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(mechanical_layer.layer_type, StackupLayerType::Mechanical) {
        bail!("board stackup layer is not a mechanical layer: {layer}");
    }

    let polygons = query_native_project_board_keepouts(root)?
        .into_iter()
        .filter(|keepout| keepout.layers.contains(&layer))
        .map(|keepout| keepout.polygon)
        .collect::<Vec<_>>();
    let dimension_strokes = query_native_project_board_dimensions(root)?
        .into_iter()
        .filter(|dimension| dimension.layer == layer)
        .map(native_board_dimension_to_stroke)
        .collect::<Vec<_>>();
    let board_text_strokes = query_native_project_board_texts(root)?
        .into_iter()
        .filter(|text| text.layer == layer)
        .map(|text| native_board_text_to_strokes(&text))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let component_text_strokes = project
        .board
        .component_mechanical_texts
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .map(|entry| native_component_mechanical_text_to_strokes(&component, entry))
                    .collect::<Result<Vec<_>>>(),
            )
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<_>>();

    let component_polygons = project
        .board
        .component_mechanical_polygons
        .iter()
        .filter_map(|(component_uuid, polygons)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                polygons
                    .iter()
                    .filter(|polygon| polygon.layer == layer)
                    .filter_map(|polygon| {
                        native_component_mechanical_polygon_to_polygon(&component, polygon)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();

    let component_strokes = project
        .board
        .component_mechanical_lines
        .iter()
        .filter_map(|(component_uuid, lines)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                lines
                    .iter()
                    .filter(|line| line.layer == layer)
                    .map(|line| native_component_mechanical_line_to_stroke(&component, line))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();

    let component_polyline_strokes = project
        .board
        .component_mechanical_polylines
        .iter()
        .filter_map(|(component_uuid, polylines)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                polylines
                    .iter()
                    .filter(|polyline| polyline.layer == layer)
                    .flat_map(|polyline| {
                        native_component_mechanical_polyline_to_strokes(&component, polyline)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();

    let component_circle_strokes = project
        .board
        .component_mechanical_circles
        .iter()
        .filter_map(|(component_uuid, circles)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                circles
                    .iter()
                    .filter(|circle| circle.layer == layer)
                    .flat_map(|circle| {
                        native_component_mechanical_circle_to_strokes(&component, circle)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();
    let component_arc_strokes = project
        .board
        .component_mechanical_arcs
        .iter()
        .filter_map(|(component_uuid, arcs)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                arcs.iter()
                    .filter(|arc| arc.layer == layer)
                    .flat_map(|arc| native_component_mechanical_arc_to_strokes(&component, arc))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();

    Ok((
        mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
        component_circle_strokes,
        component_arc_strokes,
        dimension_strokes,
        board_text_strokes,
        component_text_strokes,
    ))
}

pub(crate) fn export_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    output_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalExportView> {
    let project = load_native_project(root)?;
    let dimension_count = count_native_board_dimensions(&project.board, layer);
    let board_text_count = count_native_board_texts(&project.board, layer);
    let component_text_count = project
        .board
        .component_mechanical_texts
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum::<usize>();
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let component_circle_count = count_native_component_mechanical_circles(&project.board, layer);
    let component_arc_count = count_native_component_mechanical_arcs(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
        component_circle_strokes,
        component_arc_strokes,
        dimension_strokes,
        board_text_strokes,
        component_text_strokes,
    ) = resolve_native_project_mechanical_context(root, layer)?;
    let all_polygons = polygons
        .iter()
        .cloned()
        .chain(component_polygons.iter().cloned())
        .collect::<Vec<_>>();
    let all_strokes = component_strokes
        .iter()
        .copied()
        .chain(component_polyline_strokes.iter().copied())
        .chain(component_circle_strokes.iter().copied())
        .chain(component_arc_strokes.iter().copied())
        .chain(dimension_strokes.iter().copied())
        .chain(board_text_strokes.iter().copied())
        .chain(component_text_strokes.iter().copied())
        .collect::<Vec<_>>();
    let gerber = render_rs274x_mechanical_layer(layer, &all_polygons, &all_strokes)
        .context("failed to render native board mechanical layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberMechanicalExportView {
        action: "export_gerber_mechanical_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        keepout_count: polygons.len(),
        dimension_count,
        board_text_count,
        component_text_count,
        component_polygon_count,
        component_stroke_count,
        component_polyline_count,
        component_circle_count,
        component_arc_count,
    })
}

pub(crate) fn validate_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    gerber_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalValidationView> {
    let project = load_native_project(root)?;
    let dimension_count = count_native_board_dimensions(&project.board, layer);
    let board_text_count = count_native_board_texts(&project.board, layer);
    let component_text_count = project
        .board
        .component_mechanical_texts
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum::<usize>();
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let component_circle_count = count_native_component_mechanical_circles(&project.board, layer);
    let component_arc_count = count_native_component_mechanical_arcs(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
        component_circle_strokes,
        component_arc_strokes,
        dimension_strokes,
        board_text_strokes,
        component_text_strokes,
    ) = resolve_native_project_mechanical_context(root, layer)?;
    let all_polygons = polygons
        .iter()
        .cloned()
        .chain(component_polygons.iter().cloned())
        .collect::<Vec<_>>();
    let all_strokes = component_strokes
        .iter()
        .copied()
        .chain(component_polyline_strokes.iter().copied())
        .chain(component_circle_strokes.iter().copied())
        .chain(component_arc_strokes.iter().copied())
        .chain(dimension_strokes.iter().copied())
        .chain(board_text_strokes.iter().copied())
        .chain(component_text_strokes.iter().copied())
        .collect::<Vec<_>>();
    let expected = render_rs274x_mechanical_layer(layer, &all_polygons, &all_strokes)
        .context("failed to render expected native board mechanical layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberMechanicalValidationView {
        action: "validate_gerber_mechanical_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        keepout_count: polygons.len(),
        dimension_count,
        board_text_count,
        component_text_count,
        component_polygon_count,
        component_stroke_count,
        component_polyline_count,
        component_circle_count,
        component_arc_count,
    })
}

pub(crate) fn compare_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    gerber_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalComparisonView> {
    let project = load_native_project(root)?;
    let dimension_count = count_native_board_dimensions(&project.board, layer);
    let board_text_count = count_native_board_texts(&project.board, layer);
    let component_text_count = project
        .board
        .component_mechanical_texts
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum::<usize>();
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let component_circle_count = count_native_component_mechanical_circles(&project.board, layer);
    let component_arc_count = count_native_component_mechanical_arcs(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
        component_circle_strokes,
        component_arc_strokes,
        dimension_strokes,
        board_text_strokes,
        component_text_strokes,
    ) = resolve_native_project_mechanical_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber mechanical layer for semantic comparison")?;

    let expected_component_polygon_signatures = component_polygons
        .iter()
        .map(render_polygon_region_geometry)
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_signatures = component_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_polyline_signatures = component_polyline_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_circle_signatures = component_circle_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_arc_signatures = component_arc_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_dimension_signatures = dimension_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_board_text_signatures = board_text_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_text_signatures = component_text_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = gerber_mechanical_expected_entries(
        &polygons,
        &component_polygons,
        &component_strokes,
        &component_polyline_strokes,
        &component_circle_strokes,
        &component_arc_strokes,
        &dimension_strokes,
        &board_text_strokes,
        &component_text_strokes,
    );
    let actual_entries = gerber_mechanical_actual_entries(
        &parsed,
        &expected_component_polygon_signatures,
        &expected_component_signatures,
        &expected_component_polyline_signatures,
        &expected_component_circle_signatures,
        &expected_component_arc_signatures,
        &expected_dimension_signatures,
        &expected_board_text_signatures,
        &expected_component_text_signatures,
    );
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberMechanicalComparisonView {
        action: "compare_gerber_mechanical_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        expected_keepout_count: polygons.len(),
        expected_dimension_count: dimension_count,
        expected_board_text_count: board_text_count,
        expected_component_text_count: component_text_count,
        expected_component_polygon_count: component_polygon_count,
        expected_component_stroke_count: component_stroke_count,
        expected_component_polyline_count: component_polyline_count,
        expected_component_circle_count: component_circle_count,
        expected_component_arc_count: component_arc_count,
        actual_geometry_count: parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn gerber_mechanical_expected_entries(
    keepout_polygons: &[Polygon],
    component_polygons: &[Polygon],
    component_strokes: &[MechanicalStroke],
    component_polyline_strokes: &[MechanicalStroke],
    component_circle_strokes: &[MechanicalStroke],
    component_arc_strokes: &[MechanicalStroke],
    dimension_strokes: &[MechanicalStroke],
    board_text_strokes: &[MechanicalStroke],
    component_text_strokes: &[MechanicalStroke],
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for polygon in keepout_polygons {
        let (kind, signature) = if polygon.closed {
            (
                "keepout".to_string(),
                render_polygon_region_geometry(polygon),
            )
        } else {
            (
                "outline".to_string(),
                render_stroke_geometry(
                    super::DEFAULT_GERBER_OUTLINE_APERTURE_NM,
                    &polygon.vertices,
                ),
            )
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    for polygon in component_polygons {
        let signature = render_polygon_region_geometry(polygon);
        *entries
            .entry(("component_polygon".to_string(), signature))
            .or_insert(0) += 1;
    }
    for stroke in component_strokes {
        *entries
            .entry((
                "component_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in component_polyline_strokes {
        *entries
            .entry((
                "component_polyline_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in component_circle_strokes {
        *entries
            .entry((
                "component_circle_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in component_arc_strokes {
        *entries
            .entry((
                "component_arc_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in dimension_strokes {
        *entries
            .entry((
                "dimension_span".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in board_text_strokes {
        *entries
            .entry((
                "board_text_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    for stroke in component_text_strokes {
        *entries
            .entry((
                "component_text_stroke".to_string(),
                render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]),
            ))
            .or_insert(0) += 1;
    }
    entries
}

fn render_polygon_region_geometry(polygon: &Polygon) -> String {
    let mut points = polygon.vertices.clone();
    if polygon.closed && !points.is_empty() && points.first() != points.last() {
        points.push(points[0]);
    }
    render_region_geometry(&points)
}

pub(crate) fn gerber_mechanical_actual_entries(
    gerber: &ParsedGerber,
    expected_component_polygons: &std::collections::BTreeSet<String>,
    expected_component_strokes: &std::collections::BTreeSet<String>,
    expected_component_polyline_strokes: &std::collections::BTreeSet<String>,
    expected_component_circle_strokes: &std::collections::BTreeSet<String>,
    expected_component_arc_strokes: &std::collections::BTreeSet<String>,
    expected_dimension_strokes: &std::collections::BTreeSet<String>,
    expected_board_text_strokes: &std::collections::BTreeSet<String>,
    expected_component_text_strokes: &std::collections::BTreeSet<String>,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => {
                let signature = render_stroke_geometry(*aperture_diameter_nm, points);
                let kind = if expected_component_strokes.contains(&signature) {
                    "component_stroke".to_string()
                } else if expected_component_polyline_strokes.contains(&signature) {
                    "component_polyline_stroke".to_string()
                } else if expected_component_circle_strokes.contains(&signature) {
                    "component_circle_stroke".to_string()
                } else if expected_component_arc_strokes.contains(&signature) {
                    "component_arc_stroke".to_string()
                } else if expected_dimension_strokes.contains(&signature) {
                    "dimension_span".to_string()
                } else if expected_board_text_strokes.contains(&signature) {
                    "board_text_stroke".to_string()
                } else if expected_component_text_strokes.contains(&signature) {
                    "component_text_stroke".to_string()
                } else {
                    "outline".to_string()
                };
                (kind, signature)
            }
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Region { points } => {
                let signature = render_region_geometry(points);
                let kind = if expected_component_polygons.contains(&signature) {
                    "component_polygon".to_string()
                } else {
                    "keepout".to_string()
                };
                (kind, signature)
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}
