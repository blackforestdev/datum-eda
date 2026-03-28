use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use eda_engine::board::{PlacedPackage, StackupLayer, StackupLayerType};
use eda_engine::export::{MechanicalStroke, render_rs274x_mechanical_layer};
use eda_engine::ir::geometry::{Point, Polygon};
use serde::{Deserialize, Serialize};

use super::{
    NativeBoardRoot, NativePoint, NativeProjectGerberMechanicalComparisonView,
    NativeProjectGerberMechanicalExportView, NativeProjectGerberMechanicalValidationView,
    ParsedGerber, ParsedGerberGeometry, compare_entry_views, load_native_project,
    parse_rs274x_subset, query_native_project_board_keepouts, query_native_project_board_stackup,
    render_parsed_flash_geometry, render_region_geometry, render_stroke_geometry,
};

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

pub(crate) fn resolve_native_project_mechanical_context(
    root: &std::path::Path,
    layer: i32,
) -> Result<(
    StackupLayer,
    Vec<Polygon>,
    Vec<Polygon>,
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

    Ok((
        mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
    ))
}

pub(crate) fn count_native_component_mechanical_lines(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_mechanical_lines
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_mechanical_polygons(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_mechanical_polygons
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_mechanical_polylines(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_mechanical_polylines
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn export_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    output_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalExportView> {
    let project = load_native_project(root)?;
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
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
        component_polygon_count,
        component_stroke_count,
        component_polyline_count,
    })
}

pub(crate) fn validate_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    gerber_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalValidationView> {
    let project = load_native_project(root)?;
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
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
        component_polygon_count,
        component_stroke_count,
        component_polyline_count,
    })
}

pub(crate) fn compare_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    gerber_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalComparisonView> {
    let project = load_native_project(root)?;
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
    ) = resolve_native_project_mechanical_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber mechanical layer for semantic comparison")?;

    let expected_component_polygon_signatures = component_polygons
        .iter()
        .map(|polygon| render_region_geometry(&polygon.vertices))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_signatures = component_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_polyline_signatures = component_polyline_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = gerber_mechanical_expected_entries(
        &polygons,
        &component_polygons,
        &component_strokes,
        &component_polyline_strokes,
    );
    let actual_entries = gerber_mechanical_actual_entries(
        &parsed,
        &expected_component_polygon_signatures,
        &expected_component_signatures,
        &expected_component_polyline_signatures,
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
        expected_component_polygon_count: component_polygon_count,
        expected_component_stroke_count: component_stroke_count,
        expected_component_polyline_count: component_polyline_count,
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
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for polygon in keepout_polygons {
        let (kind, signature) = if polygon.closed {
            (
                "keepout".to_string(),
                render_region_geometry(&polygon.vertices),
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
        let signature = render_region_geometry(&polygon.vertices);
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
    entries
}

pub(crate) fn gerber_mechanical_actual_entries(
    gerber: &ParsedGerber,
    expected_component_polygons: &std::collections::BTreeSet<String>,
    expected_component_strokes: &std::collections::BTreeSet<String>,
    expected_component_polyline_strokes: &std::collections::BTreeSet<String>,
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

fn native_component_mechanical_line_to_stroke(
    component: &PlacedPackage,
    line: &NativeComponentMechanicalLine,
) -> MechanicalStroke {
    MechanicalStroke {
        from: transform_component_local_point(component, &line.from),
        to: transform_component_local_point(component, &line.to),
        width_nm: line.width_nm,
    }
}

fn native_component_mechanical_polygon_to_polygon(
    component: &PlacedPackage,
    polygon: &NativeComponentMechanicalPolygon,
) -> Option<Polygon> {
    if polygon.vertices.len() < 2 {
        return None;
    }
    Some(Polygon {
        vertices: polygon
            .vertices
            .iter()
            .map(|point| transform_component_local_point(component, point))
            .collect(),
        closed: true,
    })
}

fn native_component_mechanical_polyline_to_strokes(
    component: &PlacedPackage,
    polyline: &NativeComponentMechanicalPolyline,
) -> Vec<MechanicalStroke> {
    let points = polyline
        .vertices
        .iter()
        .map(|point| transform_component_local_point(component, point))
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return Vec::new();
    }
    points
        .windows(2)
        .map(|segment| MechanicalStroke {
            from: segment[0],
            to: segment[1],
            width_nm: polyline.width_nm,
        })
        .collect()
}

fn transform_component_local_point(component: &PlacedPackage, point: &NativePoint) -> Point {
    let radians = f64::from(component.rotation).to_radians();
    let x = point.x as f64;
    let y = point.y as f64;
    let rotated_x = x * radians.cos() - y * radians.sin();
    let rotated_y = x * radians.sin() + y * radians.cos();
    Point {
        x: component.position.x + rotated_x.round() as i64,
        y: component.position.y + rotated_y.round() as i64,
    }
}
