use eda_engine::schematic::{Bus, BusEntry};

use super::common::*;
use crate::*;

/// P2.2e: a bus as a GOLD thick polyline (`Schematic.Bus` -> `--bus`). Geometry
/// comes from the engine `Bus.segments` (KiCad `(bus (pts ...))`); a bus authored
/// through the write path with no segments yet is skipped (nothing to draw).
pub(super) fn push_bus_graphic(graphics: &mut Vec<BoardGraphicPrimitive>, points: &mut Vec<PointNm>, bus: &Bus) {
    if bus.segments.len() < 2 {
        return;
    }
    let path: Vec<PointNm> = bus.segments.iter().map(|p| point_nm(*p)).collect();
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-bus:{}", bus.uuid),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polyline".to_string(),
        source_object_uuid: bus.uuid.to_string(),
        layer_id: SCHEMATIC_BUS_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(BUS_STROKE_NM),
    });
}

/// P2.2e: a bus entry as the diagonal GREEN stub the prototype shows. Runs from
/// `position` to `position + size` (KiCad `(size dx dy)`); entries with no imported
/// size fall back to a 2.54mm 45° stub. Green so it reads as the member wire meeting
/// the bus, so it sits on the shared wire layer.
pub(super) fn push_bus_entry_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    entry: &BusEntry,
) {
    let start = point_nm(entry.position);
    let (dx, dy) = if entry.size.x != 0 || entry.size.y != 0 {
        (entry.size.x, entry.size.y)
    } else {
        (BUS_ENTRY_DEFAULT_STUB_NM, -BUS_ENTRY_DEFAULT_STUB_NM)
    };
    let end = PointNm {
        x: start.x + dx,
        y: start.y + dy,
    };
    let path = vec![start, end];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-bus-entry:{}", entry.uuid),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: entry.uuid.to_string(),
        layer_id: SCHEMATIC_WIRE_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(BUS_ENTRY_STROKE_NM),
    });
}
