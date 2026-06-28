use std::collections::HashMap;

use uuid::Uuid;

use crate::ir::geometry::Point;
use crate::schematic::{NetLabel, SchematicWire};

#[derive(Clone)]
pub(super) struct ParsedBusSegment {
    pub(super) uuid: Uuid,
    pub(super) points: Vec<Point>,
}

#[derive(Clone)]
pub(super) struct ParsedBusEntrySkeleton {
    pub(super) uuid: Uuid,
    pub(super) position: Point,
    pub(super) size: Point,
}

pub(super) fn attached_bus_specs<'a>(
    bus: &ParsedBusSegment,
    labels: impl Iterator<Item = &'a NetLabel>,
) -> Vec<(String, Vec<String>)> {
    let mut specs = Vec::new();
    for label in labels {
        if !label_touches_bus(label, bus) {
            continue;
        }
        if let Some(spec) = parse_bus_label_spec(&label.name) {
            if !specs.iter().any(|existing| existing == &spec) {
                specs.push(spec);
            }
        }
    }
    specs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    specs
}

pub(super) fn resolve_bus_entry_attachment(
    entry: &ParsedBusEntrySkeleton,
    buses: &[ParsedBusSegment],
    wires: &HashMap<Uuid, SchematicWire>,
) -> (Option<Uuid>, Option<Uuid>) {
    let endpoints = [
        entry.position,
        Point::new(
            entry.position.x + entry.size.x,
            entry.position.y + entry.size.y,
        ),
    ];

    for (wire_point, bus_point) in [(endpoints[0], endpoints[1]), (endpoints[1], endpoints[0])] {
        let bus = unique_bus_at_point(buses, bus_point);
        let wire = unique_wire_at_point(wires, wire_point);
        if bus.is_some() && wire.is_some() {
            return (bus, wire);
        }
    }

    let bus = endpoints
        .iter()
        .find_map(|point| unique_bus_at_point(buses, *point));
    let wire = endpoints
        .iter()
        .find_map(|point| unique_wire_at_point(wires, *point));
    (bus, wire)
}

fn label_touches_bus(label: &NetLabel, bus: &ParsedBusSegment) -> bool {
    bus.points
        .windows(2)
        .any(|segment| point_on_segment(label.position, segment[0], segment[1]))
}

fn unique_bus_at_point(buses: &[ParsedBusSegment], point: Point) -> Option<Uuid> {
    let mut matches: Vec<_> = buses
        .iter()
        .filter(|bus| {
            bus.points
                .windows(2)
                .any(|segment| point_on_segment(point, segment[0], segment[1]))
        })
        .map(|bus| bus.uuid)
        .collect();
    matches.sort();
    matches.dedup();
    if matches.len() == 1 {
        matches.first().copied()
    } else {
        None
    }
}

fn unique_wire_at_point(wires: &HashMap<Uuid, SchematicWire>, point: Point) -> Option<Uuid> {
    let mut matches: Vec<_> = wires
        .values()
        .filter(|wire| point_on_segment(point, wire.from, wire.to))
        .map(|wire| wire.uuid)
        .collect();
    matches.sort();
    matches.dedup();
    if matches.len() == 1 {
        matches.first().copied()
    } else {
        None
    }
}

fn parse_bus_label_spec(name: &str) -> Option<(String, Vec<String>)> {
    let open = name.rfind('[')?;
    let close = name.rfind(']')?;
    if close <= open + 1 || close != name.len() - 1 {
        return None;
    }
    let base = name[..open].trim();
    if base.is_empty() {
        return None;
    }
    let body = &name[open + 1..close];
    let (start_text, end_text) = body.split_once("..")?;
    let start = start_text.trim().parse::<i32>().ok()?;
    let end = end_text.trim().parse::<i32>().ok()?;
    let step = if start <= end { 1 } else { -1 };
    let mut members = Vec::new();
    let mut index = start;
    loop {
        members.push(format!("{base}{index}"));
        if index == end {
            break;
        }
        index += step;
    }
    Some((base.to_string(), members))
}

fn point_on_segment(point: Point, a: Point, b: Point) -> bool {
    if a == b {
        return point == a;
    }

    let cross = (point.y - a.y) * (b.x - a.x) - (point.x - a.x) * (b.y - a.y);
    if cross != 0 {
        return false;
    }

    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}
