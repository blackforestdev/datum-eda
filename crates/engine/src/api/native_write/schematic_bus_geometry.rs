//! Exact visible entry geometry; electrical membership remains separately qualified.
use super::*;
use crate::schematic::{Bus, SchematicWire};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WireEnd {
    From,
    To,
}

pub fn entry_geometry(
    bus: &Bus,
    wire: &SchematicWire,
    at: Point,
    end: WireEnd,
) -> Result<Point, EngineError> {
    if !bus.segments.windows(2).any(|pair| {
        crate::connectivity::exact_segment_intersection(at, at, pair[0], pair[1]) == Ok(Some(at))
    }) {
        return Err(invalid(
            "Bus entry contact must lie exactly on its bus spine",
        ));
    }
    let endpoint = match end {
        WireEnd::From => wire.from,
        WireEnd::To => wire.to,
    };
    let x = endpoint
        .x
        .checked_sub(at.x)
        .ok_or_else(|| invalid("Bus entry geometry overflows"))?;
    let y = endpoint
        .y
        .checked_sub(at.y)
        .ok_or_else(|| invalid("Bus entry geometry overflows"))?;
    if x == 0 && y == 0 {
        return Err(invalid("Bus entry needs distinct bus and wire endpoints"));
    }
    Ok(Point::new(x, y))
}

pub(super) fn validate_bus(bus: &crate::schematic::Bus) -> Result<(), EngineError> {
    if bus.name.trim().is_empty() || bus.name.chars().any(char::is_control) {
        return Err(invalid("Bus name must be nonempty and single-line"));
    }
    if bus.segments.len() < 2 || bus.segments.windows(2).any(|p| p[0] == p[1]) {
        return Err(invalid(
            "Bus needs at least two distinct consecutive points",
        ));
    }
    if bus.members.is_empty()
        || bus
            .members
            .iter()
            .any(|n| n.trim().is_empty() || n.chars().any(char::is_control))
        || bus.members.iter().collect::<BTreeSet<_>>().len() != bus.members.len()
    {
        return Err(invalid("Declare distinct nonempty scalar member names"));
    }
    Ok(())
}
