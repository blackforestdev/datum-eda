//! Exact source-owned electrical contact geometry for a single Symbol move.
//! Logical labels/hidden-power assignments are not an isolation predicate.
use super::{is_bus_container_label, segment_geometry::point_on_wire_segment};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{PlacedSymbol, Sheet},
};
use std::collections::HashSet;
use uuid::Uuid;

/// Shared scalar attachment nodes used by the existing connectivity owner.
pub(super) fn schematic_attachment_points(sheet: &Sheet) -> HashSet<Point> {
    sheet
        .junctions
        .values()
        .map(|j| j.position)
        .chain(
            sheet
                .labels
                .values()
                .filter(|l| !is_bus_container_label(l))
                .map(|l| l.position),
        )
        .chain(sheet.ports.values().map(|p| p.position))
        .chain(
            sheet
                .symbols
                .values()
                .flat_map(|s| s.pins.iter().map(|p| p.position)),
        )
        .collect()
}

/// Immutable complete foreign geometry in the source sheet's coordinate frame.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SymbolContactBasis {
    points: Vec<Point>,
    segments: Vec<(Point, Point)>,
}
impl SymbolContactBasis {
    pub(crate) fn capture(sheet: &Sheet, symbol: Uuid) -> Result<Self, EngineError> {
        let mut points: Vec<_> = sheet
            .symbols
            .values()
            .filter(|s| s.uuid != symbol)
            .flat_map(|s| s.pins.iter().map(|p| p.position))
            .collect();
        // All labels, including bus-container labels, and all markers matter
        // here even when excluded from the scalar logical partition.
        points.extend(sheet.labels.values().map(|l| l.position));
        points.extend(sheet.ports.values().map(|p| p.position));
        points.extend(sheet.junctions.values().map(|j| j.position));
        points.extend(sheet.noconnects.values().map(|n| n.position));
        let mut segments: Vec<_> = sheet.wires.values().map(|w| (w.from, w.to)).collect();
        for bus in sheet.buses.values() {
            if bus.segments.len() < 2 || bus.segments.windows(2).any(|p| p[0] == p[1]) {
                return Err(invalid("Bus contact geometry is incomplete"));
            }
            segments.extend(bus.segments.windows(2).map(|p| (p[0], p[1])));
        }
        for entry in sheet.bus_entries.values() {
            // A zero authored offset uses a renderer fallback, not complete
            // source geometry; never turn unavailable geometry into isolation.
            if entry.size == Point::zero()
                || !sheet.buses.contains_key(&entry.bus)
                || entry.wire.is_some_and(|id| !sheet.wires.contains_key(&id))
            {
                return Err(invalid("Bus-entry contact geometry is incomplete"));
            }
            let end = Point {
                x: entry
                    .position
                    .x
                    .checked_add(entry.size.x)
                    .ok_or_else(|| invalid("Bus-entry coordinate overflow"))?,
                y: entry
                    .position
                    .y
                    .checked_add(entry.size.y)
                    .ok_or_else(|| invalid("Bus-entry coordinate overflow"))?,
            };
            segments.push((entry.position, end));
        }
        Ok(Self { points, segments })
    }

    pub(crate) fn require_detached(&self, symbol: &PlacedSymbol) -> Result<(), EngineError> {
        if symbol.pins.iter().any(|pin| {
            self.points.contains(&pin.position)
                || self
                    .segments
                    .iter()
                    .any(|(a, b)| point_on_wire_segment(pin.position, *a, *b))
        }) {
            return Err(invalid(
                "Symbol move requires no external geometric pin contact in source and destination",
            ));
        }
        Ok(())
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}
