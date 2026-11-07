use super::{geometry::*, types::*};
use crate::{
    connectivity::schematic_attachment_points,
    error::EngineError,
    ir::geometry::Point,
    schematic::{SchematicWire, Sheet},
};
use uuid::Uuid;

impl WireEditBasis {
    pub fn capture(sheet: &Sheet, wire_id: Uuid, edit_seed: Uuid) -> Result<Self, EngineError> {
        let handles = available_handles(sheet, wire_id)?;
        Ok(Self {
            sheet: sheet.clone(),
            wire_id,
            edit_seed,
            handles,
        })
    }
    pub fn capture_junction(
        sheet: &Sheet,
        junction_id: Uuid,
        edit_seed: Uuid,
    ) -> Result<Self, EngineError> {
        let junction = sheet
            .junctions
            .get(&junction_id)
            .ok_or_else(|| invalid("Junction edit source unavailable"))?;
        let wire_id = sheet
            .wires
            .values()
            .filter(|wire| on(junction.position, wire.from, wire.to))
            .map(|wire| wire.uuid)
            .min()
            .ok_or_else(|| invalid("Junction has no incident wire; use marker Move"))?;
        let mut basis = Self::capture(sheet, wire_id, edit_seed)?;
        let vertex = WireHandle::Vertex {
            point: junction.position,
        };
        if !basis.handles.contains(&vertex) {
            basis.handles.push(vertex);
        }
        Ok(basis)
    }
    pub fn wire(&self) -> &SchematicWire {
        &self.sheet.wires[&self.wire_id]
    }
    pub fn handles(&self) -> &[WireHandle] {
        &self.handles
    }
    pub fn anchor(&self, handle: &WireHandle) -> Result<Point, EngineError> {
        if !self.handles.contains(handle) {
            return Err(invalid("Wire handle does not belong to captured source"));
        }
        Ok(match handle {
            WireHandle::Endpoint {
                end: WireEndpoint::From,
            } => self.wire().from,
            WireHandle::Endpoint {
                end: WireEndpoint::To,
            } => self.wire().to,
            WireHandle::Vertex { point } => *point,
            WireHandle::Segment { from, to } | WireHandle::InsertVertex { from, to } => Point::new(
                ((i128::from(from.x) + i128::from(to.x)) / 2) as i64,
                ((i128::from(from.y) + i128::from(to.y)) / 2) as i64,
            ),
        })
    }
}

pub(super) fn nodes(sheet: &Sheet, wire: &SchematicWire) -> Vec<Point> {
    let mut points: Vec<_> = schematic_attachment_points(sheet)
        .into_iter()
        .filter(|point| on(*point, wire.from, wire.to))
        .collect();
    points.extend([wire.from, wire.to]);
    points.sort_by_key(|p| {
        if wire.from.x != wire.to.x {
            (i128::from(p.x) - i128::from(wire.from.x))
                * i128::from((wire.to.x > wire.from.x) as i8 * 2 - 1)
        } else {
            (i128::from(p.y) - i128::from(wire.from.y))
                * i128::from((wire.to.y > wire.from.y) as i8 * 2 - 1)
        }
    });
    points.dedup();
    points
}

/// Read-only handle discovery never creates a gesture seed or physical members.
pub fn available_handles(sheet: &Sheet, wire_id: Uuid) -> Result<Vec<WireHandle>, EngineError> {
    let wire = sheet
        .wires
        .get(&wire_id)
        .ok_or_else(|| invalid("Wire edit source unavailable"))?;
    if wire.uuid != wire_id || wire.from == wire.to {
        return Err(invalid("Wire edit source is degenerate or mismatched"));
    }
    let nodes = nodes(sheet, wire);
    let all_nodes: Vec<_> = sheet
        .wires
        .values()
        .map(|w| self::nodes(sheet, w))
        .collect();
    let mut handles = vec![
        WireHandle::Endpoint {
            end: WireEndpoint::From,
        },
        WireHandle::Endpoint {
            end: WireEndpoint::To,
        },
    ];
    for point in &nodes {
        let degree: usize = all_nodes
            .iter()
            .map(|points| {
                points
                    .windows(2)
                    .filter(|pair| pair[0] == *point || pair[1] == *point)
                    .count()
            })
            .sum();
        if degree >= 2 {
            handles.push(WireHandle::Vertex { point: *point });
        }
    }
    for pair in nodes.windows(2) {
        handles.push(WireHandle::Segment {
            from: pair[0],
            to: pair[1],
        });
        handles.push(WireHandle::InsertVertex {
            from: pair[0],
            to: pair[1],
        });
    }
    Ok(handles)
}
