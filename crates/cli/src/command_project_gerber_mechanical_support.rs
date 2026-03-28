use anyhow::{Context, Result};
use eda_engine::board::{BoardText, Dimension, PlacedPackage};
use eda_engine::export::{MechanicalStroke, render_silkscreen_text_strokes};
use eda_engine::ir::geometry::{Point, Polygon};
use uuid::Uuid;

use super::{
    NativeBoardRoot, NativeComponentMechanicalArc, NativeComponentMechanicalCircle,
    NativeComponentMechanicalLine, NativeComponentMechanicalPolygon,
    NativeComponentMechanicalPolyline, NativeComponentMechanicalText, NativePoint,
};

pub(crate) const DIMENSION_SPAN_STROKE_WIDTH_NM: i64 = super::super::DEFAULT_GERBER_OUTLINE_APERTURE_NM;

pub(crate) fn count_native_board_dimensions(board: &NativeBoardRoot, layer: i32) -> usize {
    board
        .dimensions
        .iter()
        .filter_map(|value| serde_json::from_value::<Dimension>(value.clone()).ok())
        .filter(|dimension| dimension.layer == layer)
        .count()
}

pub(crate) fn count_native_board_texts(board: &NativeBoardRoot, layer: i32) -> usize {
    board
        .texts
        .iter()
        .filter_map(|value| serde_json::from_value::<BoardText>(value.clone()).ok())
        .filter(|text| text.layer == layer)
        .count()
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

pub(crate) fn count_native_component_mechanical_circles(
    board: &NativeBoardRoot,
    layer: i32,
) -> usize {
    board
        .component_mechanical_circles
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn count_native_component_mechanical_arcs(board: &NativeBoardRoot, layer: i32) -> usize {
    board
        .component_mechanical_arcs
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum()
}

pub(crate) fn native_board_dimension_to_stroke(dimension: Dimension) -> MechanicalStroke {
    MechanicalStroke {
        from: dimension.from,
        to: dimension.to,
        width_nm: DIMENSION_SPAN_STROKE_WIDTH_NM,
    }
}

pub(crate) fn native_board_text_to_strokes(text: &BoardText) -> Result<Vec<MechanicalStroke>> {
    Ok(render_silkscreen_text_strokes(text)
        .with_context(|| format!("failed to render board text '{}' as strokes", text.text))?
        .into_iter()
        .map(|stroke| MechanicalStroke {
            from: stroke.from,
            to: stroke.to,
            width_nm: stroke.width_nm,
        })
        .collect())
}

pub(crate) fn native_component_mechanical_line_to_stroke(
    component: &PlacedPackage,
    line: &NativeComponentMechanicalLine,
) -> MechanicalStroke {
    MechanicalStroke {
        from: transform_component_local_point(component, &line.from),
        to: transform_component_local_point(component, &line.to),
        width_nm: line.width_nm,
    }
}

pub(crate) fn native_component_mechanical_text_to_strokes(
    component: &PlacedPackage,
    text: &NativeComponentMechanicalText,
) -> Result<Vec<MechanicalStroke>> {
    let text = BoardText {
        uuid: Uuid::new_v4(),
        text: text.text.clone(),
        position: transform_component_local_point(component, &text.position),
        rotation: component.rotation + text.rotation,
        layer: text.layer,
        height_nm: text.height_nm,
        stroke_width_nm: text.stroke_width_nm,
    };
    native_board_text_to_strokes(&text)
}

pub(crate) fn native_component_mechanical_polygon_to_polygon(
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

pub(crate) fn native_component_mechanical_polyline_to_strokes(
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

pub(crate) fn native_component_mechanical_circle_to_strokes(
    component: &PlacedPackage,
    circle: &NativeComponentMechanicalCircle,
) -> Vec<MechanicalStroke> {
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
        .map(|segment| MechanicalStroke {
            from: segment[0],
            to: segment[1],
            width_nm: circle.width_nm,
        })
        .collect()
}

pub(crate) fn native_component_mechanical_arc_to_strokes(
    component: &PlacedPackage,
    arc: &NativeComponentMechanicalArc,
) -> Vec<MechanicalStroke> {
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
        .map(|segment| MechanicalStroke {
            from: segment[0],
            to: segment[1],
            width_nm: arc.width_nm,
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
