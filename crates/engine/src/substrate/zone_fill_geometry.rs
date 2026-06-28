use crate::board::{PlacedPad, Track, Via, Zone};
use crate::ir::geometry::{LayerId, Point, Polygon, Rect};

use super::ZoneFillCopperContext;

pub(super) fn polygon_has_area(polygon: &Polygon) -> bool {
    if !polygon.closed || polygon.vertices.len() < 3 {
        return false;
    }
    let Some(bounds) = polygon.bounding_box() else {
        return false;
    };
    if bounds.min.x == bounds.max.x || bounds.min.y == bounds.max.y {
        return false;
    }
    let mut unique = Vec::new();
    for point in &polygon.vertices {
        if !unique.contains(point) {
            unique.push(*point);
        }
    }
    unique.len() >= 3 && !polygon_self_intersects(polygon)
}

pub(super) fn collect_keepout_obstacles(
    context: &ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) -> usize {
    let mut count = 0;
    for keepout in &context.keepouts {
        if !keepout.layers.contains(&zone.layer)
            || !polygons_may_intersect(&keepout.polygon, &zone.polygon)
        {
            continue;
        }
        let Some(bounds) = keepout.polygon.bounding_box() else {
            continue;
        };
        obstacles.push(bounds);
        count += 1;
    }
    count
}

pub(super) fn collect_foreign_pad_obstacles<'a>(
    context: &'a ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) -> Option<&'a str> {
    for pad in &context.pads {
        if !pad_applies_to_layer(pad, zone.layer) {
            continue;
        }
        let Some(bounds) = pad_bounds(pad) else {
            continue;
        };
        if !rect_intersects_polygon(&bounds, &zone.polygon) {
            continue;
        }
        match pad.net {
            Some(net) if net == zone.net => {}
            Some(_) => obstacles.push(bounds),
            None => {
                return Some(
                    "datum-eda fill-zones: unsupported because an unresolved pad intersects the zone",
                );
            }
        }
    }
    None
}

pub(super) fn collect_foreign_track_obstacles<'a>(
    context: &'a ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) -> Result<bool, &'a str> {
    let mut has_non_orthogonal_track = false;
    for track in &context.tracks {
        if track.layer != zone.layer || track.net == zone.net {
            continue;
        }
        let bounds = track_bounds(track);
        if !rect_intersects_polygon(&bounds, &zone.polygon) {
            continue;
        }
        if track.width <= 0 {
            return Err(
                "datum-eda fill-zones: unsupported because a different-net track has nonpositive width",
            );
        }
        if track.from.x != track.to.x && track.from.y != track.to.y {
            has_non_orthogonal_track = true;
        }
        obstacles.push(bounds);
    }
    Ok(has_non_orthogonal_track)
}

pub(super) fn collect_foreign_via_obstacles(
    context: &ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) {
    for via in &context.vias {
        if via_applies_to_layer(via, zone.layer) && via.net != zone.net {
            let bounds = via_bounds(via);
            if rect_intersects_polygon(&bounds, &zone.polygon) {
                obstacles.push(bounds);
            }
        }
    }
}

pub(super) fn has_same_net_thermal_anchor(context: &ZoneFillCopperContext, zone: &Zone) -> bool {
    context.pads.iter().any(|pad| {
        pad.net == Some(zone.net)
            && pad_applies_to_layer(pad, zone.layer)
            && pad_bounds(pad).is_some_and(|bounds| rect_intersects_polygon(&bounds, &zone.polygon))
    }) || context.vias.iter().any(|via| {
        via.net == zone.net
            && via_applies_to_layer(via, zone.layer)
            && rect_intersects_polygon(&via_bounds(via), &zone.polygon)
    })
}

fn pad_applies_to_layer(pad: &PlacedPad, layer: LayerId) -> bool {
    pad.layer == layer || pad.copper_layers.contains(&layer)
}

fn via_applies_to_layer(via: &Via, layer: LayerId) -> bool {
    layer >= via.from_layer.min(via.to_layer) && layer <= via.from_layer.max(via.to_layer)
}

fn pad_bounds(pad: &PlacedPad) -> Option<Rect> {
    let half_width = match pad.shape {
        crate::board::PadShape::Circle => pad.diameter / 2,
        crate::board::PadShape::Rect
        | crate::board::PadShape::Oval
        | crate::board::PadShape::RoundRect => pad.width / 2,
    };
    let half_height = match pad.shape {
        crate::board::PadShape::Circle => pad.diameter / 2,
        crate::board::PadShape::Rect
        | crate::board::PadShape::Oval
        | crate::board::PadShape::RoundRect => pad.height / 2,
    };
    if half_width <= 0 || half_height <= 0 {
        return None;
    }
    Some(Rect::new(
        Point::new(pad.position.x - half_width, pad.position.y - half_height),
        Point::new(pad.position.x + half_width, pad.position.y + half_height),
    ))
}

fn track_bounds(track: &Track) -> Rect {
    let half = (track.width / 2).max(0);
    Rect::new(
        Point::new(
            track.from.x.min(track.to.x) - half,
            track.from.y.min(track.to.y) - half,
        ),
        Point::new(
            track.from.x.max(track.to.x) + half,
            track.from.y.max(track.to.y) + half,
        ),
    )
}

fn via_bounds(via: &Via) -> Rect {
    let half = (via.diameter / 2).max(0);
    Rect::new(
        Point::new(via.position.x - half, via.position.y - half),
        Point::new(via.position.x + half, via.position.y + half),
    )
}

pub(super) fn rectangular_cutout_islands(
    zone: &Zone,
    obstacle: Rect,
    clearance_nm: i64,
) -> Option<(Vec<Polygon>, bool)> {
    let zone_bounds = rectangular_polygon_bounds(&zone.polygon)?;
    let inflated = inflate_rect(obstacle, clearance_nm);
    let clipped = clip_rect_to_bounds(inflated, zone_bounds)?;
    let clipped_obstacle = clipped != inflated;
    let mut islands = Vec::new();
    push_rect_island(
        &mut islands,
        Rect::new(
            zone_bounds.min,
            Point::new(clipped.min.x, zone_bounds.max.y),
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(clipped.max.x, zone_bounds.min.y),
            zone_bounds.max,
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(clipped.min.x, zone_bounds.min.y),
            Point::new(clipped.max.x, clipped.min.y),
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(clipped.min.x, clipped.max.y),
            Point::new(clipped.max.x, zone_bounds.max.y),
        ),
    );
    (!islands.is_empty()).then_some((islands, clipped_obstacle))
}

pub(super) fn rectangular_multi_cutout_islands(
    zone: &Zone,
    obstacles: &[Rect],
    clearance_nm: i64,
) -> Option<(Vec<Polygon>, bool, bool)> {
    let zone_bounds = rectangular_polygon_bounds(&zone.polygon)?;
    let mut inflated_obstacles = Vec::new();
    let mut clipped_obstacles = false;
    for obstacle in obstacles {
        let inflated = inflate_rect(*obstacle, clearance_nm);
        let clipped = clip_rect_to_bounds(inflated, zone_bounds)?;
        clipped_obstacles |= clipped != inflated;
        inflated_obstacles.push(clipped);
    }
    let original_obstacle_count = inflated_obstacles.len();
    let inflated_obstacles = merge_overlapping_or_touching_rects(inflated_obstacles);
    let merged_obstacles = inflated_obstacles.len() != original_obstacle_count;

    let mut x_edges = vec![zone_bounds.min.x, zone_bounds.max.x];
    let mut y_edges = vec![zone_bounds.min.y, zone_bounds.max.y];
    for obstacle in &inflated_obstacles {
        x_edges.push(obstacle.min.x);
        x_edges.push(obstacle.max.x);
        y_edges.push(obstacle.min.y);
        y_edges.push(obstacle.max.y);
    }
    x_edges.sort_unstable();
    x_edges.dedup();
    y_edges.sort_unstable();
    y_edges.dedup();

    let mut islands = Vec::new();
    for x_pair in x_edges.windows(2) {
        for y_pair in y_edges.windows(2) {
            let cell = Rect::new(
                Point::new(x_pair[0], y_pair[0]),
                Point::new(x_pair[1], y_pair[1]),
            );
            if rect_is_covered_by_any(&cell, &inflated_obstacles) {
                continue;
            }
            push_rect_island(&mut islands, cell);
        }
    }
    (!islands.is_empty()).then_some((islands, merged_obstacles, clipped_obstacles))
}

fn merge_overlapping_or_touching_rects(mut rects: Vec<Rect>) -> Vec<Rect> {
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for index in 0..rects.len() {
            for other_index in (index + 1)..rects.len() {
                if rects[index].intersects(&rects[other_index]) {
                    let merged = rect_union(rects[index], rects[other_index]);
                    rects[index] = merged;
                    rects.remove(other_index);
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    rects
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    Rect::new(
        Point::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y)),
        Point::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y)),
    )
}

fn rect_is_covered_by_any(cell: &Rect, obstacles: &[Rect]) -> bool {
    obstacles.iter().any(|obstacle| {
        cell.min.x >= obstacle.min.x
            && cell.max.x <= obstacle.max.x
            && cell.min.y >= obstacle.min.y
            && cell.max.y <= obstacle.max.y
    })
}

fn rectangular_polygon_bounds(polygon: &Polygon) -> Option<Rect> {
    if !polygon.closed || polygon.vertices.len() != 4 {
        return None;
    }
    let bounds = polygon.bounding_box()?;
    let corners = [
        bounds.min,
        Point::new(bounds.max.x, bounds.min.y),
        bounds.max,
        Point::new(bounds.min.x, bounds.max.y),
    ];
    corners
        .iter()
        .all(|corner| polygon.vertices.contains(corner))
        .then_some(bounds)
}

fn inflate_rect(rect: Rect, amount: i64) -> Rect {
    Rect::new(
        Point::new(rect.min.x - amount, rect.min.y - amount),
        Point::new(rect.max.x + amount, rect.max.y + amount),
    )
}

fn clip_rect_to_bounds(rect: Rect, bounds: Rect) -> Option<Rect> {
    let clipped = Rect::new(
        Point::new(rect.min.x.max(bounds.min.x), rect.min.y.max(bounds.min.y)),
        Point::new(rect.max.x.min(bounds.max.x), rect.max.y.min(bounds.max.y)),
    );
    (clipped.min.x < clipped.max.x && clipped.min.y < clipped.max.y).then_some(clipped)
}

fn push_rect_island(islands: &mut Vec<Polygon>, rect: Rect) {
    if rect.min.x >= rect.max.x || rect.min.y >= rect.max.y {
        return;
    }
    islands.push(Polygon {
        vertices: vec![
            rect.min,
            Point::new(rect.max.x, rect.min.y),
            rect.max,
            Point::new(rect.min.x, rect.max.y),
        ],
        closed: true,
    });
}

fn rect_intersects_polygon(rect: &Rect, polygon: &Polygon) -> bool {
    polygon
        .bounding_box()
        .is_some_and(|bounds| rect.intersects(&bounds))
}

fn polygons_may_intersect(a: &Polygon, b: &Polygon) -> bool {
    match (a.bounding_box(), b.bounding_box()) {
        (Some(a_bounds), Some(b_bounds)) => a_bounds.intersects(&b_bounds),
        _ => false,
    }
}

pub(super) fn polygon_self_intersects(polygon: &Polygon) -> bool {
    let edge_count = polygon.vertices.len();
    if edge_count < 4 {
        return false;
    }
    for edge_a in 0..edge_count {
        let a0 = polygon.vertices[edge_a];
        let a1 = polygon.vertices[(edge_a + 1) % edge_count];
        for edge_b in (edge_a + 1)..edge_count {
            if edges_are_adjacent(edge_a, edge_b, edge_count) {
                continue;
            }
            let b0 = polygon.vertices[edge_b];
            let b1 = polygon.vertices[(edge_b + 1) % edge_count];
            if segments_intersect(a0, a1, b0, b1) {
                return true;
            }
        }
    }
    false
}

fn edges_are_adjacent(a: usize, b: usize, edge_count: usize) -> bool {
    a == b || a + 1 == b || (a == 0 && b + 1 == edge_count)
}

fn segments_intersect(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
    let o1 = orientation(a0, a1, b0);
    let o2 = orientation(a0, a1, b1);
    let o3 = orientation(b0, b1, a0);
    let o4 = orientation(b0, b1, a1);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    (o1 == 0 && point_on_segment(b0, a0, a1))
        || (o2 == 0 && point_on_segment(b1, a0, a1))
        || (o3 == 0 && point_on_segment(a0, b0, b1))
        || (o4 == 0 && point_on_segment(a1, b0, b1))
}

fn point_on_segment(point: Point, from: Point, to: Point) -> bool {
    point.x >= from.x.min(to.x)
        && point.x <= from.x.max(to.x)
        && point.y >= from.y.min(to.y)
        && point.y <= from.y.max(to.y)
}

fn orientation(a: Point, b: Point, c: Point) -> i32 {
    let cross =
        (b.y - a.y) as i128 * (c.x - b.x) as i128 - (b.x - a.x) as i128 * (c.y - b.y) as i128;
    if cross == 0 {
        0
    } else if cross > 0 {
        1
    } else {
        2
    }
}
