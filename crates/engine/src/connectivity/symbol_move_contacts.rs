//! Exact source-owned electrical contact geometry for a single Move.
//! Logical labels/hidden-power assignments are not an isolation predicate.
use super::{
    is_bus_container_label, physical_partitions,
    segment_geometry::{point_on_wire_segment, segments_touch},
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{NetLabel, PlacedSymbol, Schematic, SchematicWire, Sheet},
};
use std::{collections::HashSet, sync::Arc};
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
    sheet: Arc<Sheet>,
    attached: Vec<SchematicWire>,
    source_points: Vec<Point>,
}
impl SymbolContactBasis {
    pub(crate) fn capture(sheet: &Sheet, symbol: Uuid) -> Result<Self, EngineError> {
        let mut points: Vec<_> = sheet
            .symbols
            .values()
            .filter(|s| s.uuid != symbol)
            .flat_map(|s| s.pins.iter().map(|p| p.position))
            .collect();
        points.extend(sheet.labels.values().map(|l| l.position));
        let symbol = sheet
            .symbols
            .get(&symbol)
            .ok_or_else(|| invalid("Symbol source unavailable"))?;
        Self::capture_points(
            sheet,
            symbol.pins.iter().map(|pin| pin.position).collect(),
            points,
        )
    }

    pub(crate) fn capture_label(sheet: &Sheet, label: Uuid) -> Result<Self, EngineError> {
        let target = sheet
            .labels
            .get(&label)
            .ok_or_else(|| invalid("Label source unavailable"))?;
        if is_bus_container_label(target) {
            return Err(invalid("Move requires a scalar Label"));
        }
        let mut points: Vec<_> = sheet
            .symbols
            .values()
            .flat_map(|s| s.pins.iter().map(|pin| pin.position))
            .collect();
        points.extend(
            sheet
                .labels
                .values()
                .filter(|l| l.uuid != label)
                .map(|l| l.position),
        );
        Self::capture_points(sheet, vec![target.position], points)
    }

    fn capture_points(
        sheet: &Sheet,
        source_points: Vec<Point>,
        mut points: Vec<Point>,
    ) -> Result<Self, EngineError> {
        // All fixed scalar markers, including bus containers, qualify the move.
        points.extend(sheet.ports.values().map(|p| p.position));
        points.extend(sheet.junctions.values().map(|j| j.position));
        points.extend(sheet.noconnects.values().map(|n| n.position));
        let pin_positions: HashSet<_> = source_points.iter().copied().collect();
        if pin_positions.len() != source_points.len() {
            return Err(invalid("Move has ambiguous coincident attachment points"));
        }
        let mut attached = Vec::new();
        let mut segments = Vec::new();
        for wire in sheet.wires.values() {
            if pin_positions
                .iter()
                .any(|pin| point_on_wire_segment(*pin, wire.from, wire.to))
            {
                if wire.from == wire.to
                    || pin_positions.iter().any(|pin| {
                        *pin != wire.from
                            && *pin != wire.to
                            && point_on_wire_segment(*pin, wire.from, wire.to)
                    })
                {
                    return Err(invalid(
                        "Move has ambiguous geometric pin contact inside a wire",
                    ));
                }
                attached.push(wire.clone());
            }
        }
        attached.sort_by_key(|wire| wire.uuid);
        for bus in sheet.buses.values() {
            if bus.segments.len() < 2 || bus.segments.windows(2).any(|p| p[0] == p[1]) {
                return Err(invalid("Bus contact geometry is incomplete"));
            }
            segments.extend(bus.segments.windows(2).map(|p| (p[0], p[1])));
        }
        for entry in sheet.bus_entries.values() {
            if entry
                .wire
                .is_some_and(|id| attached.iter().any(|wire| wire.uuid == id))
            {
                return Err(invalid("Move cannot reshape a wire owned by a bus entry"));
            }
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
        let basis = Self {
            points,
            segments,
            sheet: Arc::new(sheet.clone()),
            attached,
            source_points,
        };
        basis.require_no_foreign_contact(&basis.source_points)?;
        Ok(basis)
    }

    fn require_no_foreign_contact(&self, points: &[Point]) -> Result<(), EngineError> {
        if points.iter().any(|point| {
            self.points.contains(point)
                || self
                    .segments
                    .iter()
                    .any(|(a, b)| point_on_wire_segment(*point, *a, *b))
        }) {
            return Err(invalid("Move has ambiguous external geometric pin contact"));
        }
        Ok(())
    }

    pub(crate) fn moved_wires(
        &self,
        symbol: &PlacedSymbol,
    ) -> Result<Vec<SchematicWire>, EngineError> {
        let mut sheet = self.sheet.as_ref().clone();
        sheet.symbols.insert(symbol.uuid, symbol.clone());
        self.moved_geometry(sheet, symbol.pins.iter().map(|p| p.position).collect())
    }

    pub(crate) fn moved_label_wires(
        &self,
        label: &NetLabel,
    ) -> Result<Vec<SchematicWire>, EngineError> {
        let mut sheet = self.sheet.as_ref().clone();
        sheet.labels.insert(label.uuid, label.clone());
        self.moved_geometry(sheet, vec![label.position])
    }

    fn moved_geometry(
        &self,
        mut sheet: Sheet,
        points: Vec<Point>,
    ) -> Result<Vec<SchematicWire>, EngineError> {
        self.require_no_foreign_contact(&points)?;
        let destinations: std::collections::HashMap<_, _> =
            self.source_points.iter().copied().zip(points).collect();
        let mut wires = Vec::new();
        for wire in &self.attached {
            let mut moved = wire.clone();
            moved.from = destinations.get(&wire.from).copied().unwrap_or(wire.from);
            moved.to = destinations.get(&wire.to).copied().unwrap_or(wire.to);
            if moved.from == moved.to {
                return Err(invalid("Move would collapse an attached wire"));
            }
            if moved != *wire {
                wires.push(moved);
            }
        }
        for wire in &wires {
            sheet.wires.insert(wire.uuid, wire.clone());
        }
        // Equality of complete physical member sets is stronger than equality
        // of names: labels cannot conceal an accidental shortcut or disconnection.
        // Only this uniquely occurring sheet's geometry changes; all external
        // label/hierarchy relationships therefore retain their original members.
        let before = local_source(self.sheet.as_ref().clone());
        let after = local_source(sheet);
        if physical_partitions(&before)? != physical_partitions(&after)? {
            return Err(invalid(
                "Move would change electrical connections through geometric pin contact",
            ));
        }
        // Retain every fixed attachment even if another physical path would
        // conceal its loss. This also covers no-connect and bus-container
        // labels, which are outside scalar partitions.
        for wire in &wires {
            let old = &self.sheet.wires[&wire.uuid];
            for point in &self.points {
                if point_on_wire_segment(*point, wire.from, wire.to)
                    != point_on_wire_segment(*point, old.from, old.to)
                {
                    return Err(invalid("Move would change a fixed attachment contact"));
                }
            }
            if self.segments.iter().any(|(from, to)| {
                segments_touch(wire.from, wire.to, *from, *to)
                    || segments_touch(old.from, old.to, *from, *to)
            }) {
                return Err(invalid(
                    "Move cannot reshape a wire contacting bus geometry",
                ));
            }
        }
        Ok(wires)
    }

    pub(crate) fn original_wire(&self, id: Uuid) -> &SchematicWire {
        &self.sheet.wires[&id]
    }
}

fn local_source(sheet: Sheet) -> Schematic {
    Schematic {
        uuid: Uuid::nil(),
        sheets: std::collections::HashMap::from([(sheet.uuid, sheet)]),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: Vec::new(),
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}
