use anyhow::{Result, bail};
use eda_engine::board::{BoardText, PlacedPackage, StackupLayer, StackupLayerType};
use eda_engine::export::SilkscreenStroke;
use eda_engine::ir::geometry::Point;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    NativeBoardRoot, NativePoint, load_native_project, query_native_project_board_stackup,
    query_native_project_board_texts,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenLine {
    pub(crate) from: NativePoint,
    pub(crate) to: NativePoint,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenText {
    pub(crate) text: String,
    pub(crate) position: NativePoint,
    pub(crate) rotation: i32,
    pub(crate) height_nm: i64,
    pub(crate) stroke_width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenArc {
    pub(crate) center: NativePoint,
    pub(crate) radius_nm: i64,
    pub(crate) start_angle: i32,
    pub(crate) end_angle: i32,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenCircle {
    pub(crate) center: NativePoint,
    pub(crate) radius_nm: i64,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenPolygon {
    pub(crate) vertices: Vec<NativePoint>,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeComponentSilkscreenPolyline {
    pub(crate) vertices: Vec<NativePoint>,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

pub(crate) fn resolve_native_project_silkscreen_context(
    root: &std::path::Path,
    layer: i32,
) -> Result<(StackupLayer, Vec<BoardText>, Vec<SilkscreenStroke>)> {
    let project = load_native_project(root)?;
    let stackup = query_native_project_board_stackup(root)?;
    let silk_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(silk_layer.layer_type, StackupLayerType::Silkscreen) {
        bail!("board stackup layer is not a silkscreen layer: {layer}");
    }
    let mut texts = query_native_project_board_texts(root)?
        .into_iter()
        .filter(|text| text.layer == layer)
        .collect::<Vec<_>>();
    let component_texts = project
        .board
        .component_silkscreen_texts
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .map(|entry| native_component_silkscreen_text_to_board_text(&component, entry))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();
    texts.extend(component_texts);

    let line_strokes = project
        .board
        .component_silkscreen
        .iter()
        .filter_map(|(component_uuid, lines)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                lines
                    .iter()
                    .filter(|line| line.layer == layer)
                    .map(|line| native_component_silkscreen_line_to_stroke(&component, line))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten();
    let arc_strokes = project
        .board
        .component_silkscreen_arcs
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .flat_map(|entry| native_component_silkscreen_arc_to_strokes(&component, entry))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten();
    let circle_strokes = project
        .board
        .component_silkscreen_circles
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .flat_map(|entry| {
                        native_component_silkscreen_circle_to_strokes(&component, entry)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten();
    let polygon_strokes = project
        .board
        .component_silkscreen_polygons
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .flat_map(|entry| {
                        native_component_silkscreen_polygon_to_strokes(&component, entry)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten();
    let polyline_strokes = project
        .board
        .component_silkscreen_polylines
        .iter()
        .filter_map(|(component_uuid, entries)| {
            let component = project.board.packages.get(component_uuid)?;
            let component: PlacedPackage = serde_json::from_value(component.clone()).ok()?;
            Some(
                entries
                    .iter()
                    .filter(|entry| entry.layer == layer)
                    .flat_map(|entry| {
                        native_component_silkscreen_polyline_to_strokes(&component, entry)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten();

    let strokes = line_strokes
        .chain(arc_strokes)
        .chain(circle_strokes)
        .chain(polygon_strokes)
        .chain(polyline_strokes)
        .collect::<Vec<_>>();
    Ok((silk_layer, texts, strokes))
}

pub(crate) fn count_native_component_silkscreen_texts(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_silkscreen_texts
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_silkscreen_lines(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_silkscreen
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_silkscreen_arcs(board: &NativeBoardRoot, layer: i32) -> usize {
    board
        .component_silkscreen_arcs
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_silkscreen_circles(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_silkscreen_circles
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_silkscreen_polygons(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_silkscreen_polygons
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_silkscreen_polylines(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_silkscreen_polylines
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

fn native_component_silkscreen_line_to_stroke(
    component: &PlacedPackage,
    line: &NativeComponentSilkscreenLine,
) -> SilkscreenStroke {
    SilkscreenStroke {
        from: transform_component_local_point(component, &line.from),
        to: transform_component_local_point(component, &line.to),
        width_nm: line.width_nm,
    }
}

fn native_component_silkscreen_text_to_board_text(
    component: &PlacedPackage,
    text: &NativeComponentSilkscreenText,
) -> BoardText {
    BoardText {
        uuid: Uuid::new_v4(),
        text: text.text.clone(),
        position: transform_component_local_point(component, &text.position),
        rotation: component.rotation + text.rotation,
        layer: text.layer,
        height_nm: text.height_nm,
        stroke_width_nm: text.stroke_width_nm,
    }
}

fn native_component_silkscreen_arc_to_strokes(
    component: &PlacedPackage,
    arc: &NativeComponentSilkscreenArc,
) -> Vec<SilkscreenStroke> {
    const ARC_SEGMENT_ANGLE_TENTHS: i32 = 150;
    let mut sweep = arc.end_angle - arc.start_angle;
    if sweep <= 0 {
        sweep += 3600;
    }
    let segment_count = ((sweep + ARC_SEGMENT_ANGLE_TENTHS - 1) / ARC_SEGMENT_ANGLE_TENTHS).max(1);
    let points = (0..=segment_count)
        .map(|idx| {
            let angle_tenths = arc.start_angle + sweep * idx / segment_count;
            let radians = (f64::from(angle_tenths) / 10.0).to_radians();
            let local_x = arc.center.x as f64 + (arc.radius_nm as f64 * radians.cos());
            let local_y = arc.center.y as f64 + (arc.radius_nm as f64 * radians.sin());
            transform_component_local_xy(component, local_x.round() as i64, local_y.round() as i64)
        })
        .collect::<Vec<_>>();
    points
        .windows(2)
        .map(|segment| SilkscreenStroke {
            from: segment[0],
            to: segment[1],
            width_nm: arc.width_nm,
        })
        .collect()
}

fn native_component_silkscreen_circle_to_strokes(
    component: &PlacedPackage,
    circle: &NativeComponentSilkscreenCircle,
) -> Vec<SilkscreenStroke> {
    const CIRCLE_SEGMENT_ANGLE_TENTHS: i32 = 150;
    let segment_count = 3600 / CIRCLE_SEGMENT_ANGLE_TENTHS;
    let points = (0..=segment_count)
        .map(|idx| {
            let angle_tenths = idx * CIRCLE_SEGMENT_ANGLE_TENTHS;
            let radians = (f64::from(angle_tenths) / 10.0).to_radians();
            let local_x = circle.center.x as f64 + (circle.radius_nm as f64 * radians.cos());
            let local_y = circle.center.y as f64 + (circle.radius_nm as f64 * radians.sin());
            transform_component_local_xy(component, local_x.round() as i64, local_y.round() as i64)
        })
        .collect::<Vec<_>>();
    points
        .windows(2)
        .map(|segment| SilkscreenStroke {
            from: segment[0],
            to: segment[1],
            width_nm: circle.width_nm,
        })
        .collect()
}

fn native_component_silkscreen_polygon_to_strokes(
    component: &PlacedPackage,
    polygon: &NativeComponentSilkscreenPolygon,
) -> Vec<SilkscreenStroke> {
    let mut points = polygon
        .vertices
        .iter()
        .map(|point| transform_component_local_point(component, point))
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return Vec::new();
    }
    points.push(points[0]);
    points
        .windows(2)
        .map(|segment| SilkscreenStroke {
            from: segment[0],
            to: segment[1],
            width_nm: polygon.width_nm,
        })
        .collect()
}

fn native_component_silkscreen_polyline_to_strokes(
    component: &PlacedPackage,
    polyline: &NativeComponentSilkscreenPolyline,
) -> Vec<SilkscreenStroke> {
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
        .map(|segment| SilkscreenStroke {
            from: segment[0],
            to: segment[1],
            width_nm: polyline.width_nm,
        })
        .collect()
}

fn transform_component_local_point(component: &PlacedPackage, point: &NativePoint) -> Point {
    transform_component_local_xy(component, point.x, point.y)
}

fn transform_component_local_xy(component: &PlacedPackage, x_nm: i64, y_nm: i64) -> Point {
    let radians = f64::from(component.rotation).to_radians();
    let x = x_nm as f64;
    let y = y_nm as f64;
    let rotated_x = x * radians.cos() - y * radians.sin();
    let rotated_y = x * radians.sin() + y * radians.cos();
    Point {
        x: component.position.x + rotated_x.round() as i64,
        y: component.position.y + rotated_y.round() as i64,
    }
}
