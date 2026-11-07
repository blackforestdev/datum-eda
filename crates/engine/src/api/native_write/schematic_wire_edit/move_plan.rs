use super::{basis::nodes, geometry::*, types::*};
use crate::{
    api::native_write::schematic_junction_move::JunctionMoveBasis,
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, SchematicWire},
};
use uuid::Uuid;

impl WireEditBasis {
    pub fn plan_move(
        &self,
        handle: &WireHandle,
        destination: Point,
        routing: WireRouting,
    ) -> Result<WireEditPlan, EngineError> {
        let anchor = self.anchor(handle)?;
        if anchor == destination {
            return Ok(WireEditPlan::default());
        }
        let mut plan = WireEditPlan::default();
        if let WireHandle::Vertex { point } = handle {
            if self
                .sheet
                .symbols
                .values()
                .flat_map(|s| &s.pins)
                .any(|p| p.position == *point)
                || self.sheet.labels.values().any(|l| l.position == *point)
                || self.sheet.ports.values().any(|p| p.position == *point)
            {
                return Err(invalid(
                    "A fixed pin or attached identifier cannot move with a wire vertex",
                ));
            }
            let junctions: Vec<_> = self
                .sheet
                .junctions
                .values()
                .filter(|j| j.position == *point)
                .collect();
            if junctions.len() > 1 {
                return Err(invalid("Several Junctions share this wire vertex"));
            }
            if let Some(junction) = junctions.first() {
                // Retain the existing exact along-through-wire junction slide.
                if let Ok(edit) = JunctionMoveBasis::capture(&self.sheet, junction.uuid)
                    .and_then(|b| b.destination_edit(destination))
                {
                    if edit.wires.iter().any(|w| {
                        let dx = (i128::from(w.to.x) - i128::from(w.from.x)).abs();
                        let dy = (i128::from(w.to.y) - i128::from(w.from.y)).abs();
                        match routing {
                            WireRouting::Free => false,
                            WireRouting::Orthogonal => dx != 0 && dy != 0,
                            WireRouting::Diagonal45 => dx != 0 && dy != 0 && dx != dy,
                        }
                    }) {
                        // Continue with configured span planning below.
                    } else {
                        plan.junctions.insert(edit.junction.uuid, edit.junction);
                        for wire in edit.wires {
                            if self.sheet.wires.get(&wire.uuid) != Some(&wire) {
                                plan.wires.insert(wire.uuid, wire);
                            }
                        }
                        validate(self, &plan, None)?;
                        return Ok(plan);
                    }
                }
                let mut moved = (**junction).clone();
                moved.position = destination;
                plan.junctions.insert(moved.uuid, moved);
            }
        }
        let mut source_wires: Vec<_> = self.sheet.wires.values().collect();
        source_wires.sort_by_key(|w| w.uuid);
        for wire in source_wires {
            let points = nodes(&self.sheet, wire);
            let mut edges = Vec::new();
            let mut changed = false;
            for (index, pair) in points.windows(2).enumerate() {
                let (a, b) = (pair[0], pair[1]);
                let mut paths = Vec::new();
                let edit = match handle {
                    WireHandle::Endpoint { end } if wire.uuid == self.wire_id => {
                        if (*end == WireEndpoint::From && index == 0)
                            || (*end == WireEndpoint::To && index + 2 == points.len())
                        {
                            let (from, to) = if *end == WireEndpoint::From {
                                (destination, b)
                            } else {
                                (a, destination)
                            };
                            paths.push(route(from, to, a, b, routing)?);
                            true
                        } else {
                            false
                        }
                    }
                    WireHandle::Vertex { point } if a == *point || b == *point => {
                        paths.push(route(
                            if a == *point { destination } else { a },
                            if b == *point { destination } else { b },
                            a,
                            b,
                            routing,
                        )?);
                        true
                    }
                    WireHandle::Segment { from, to }
                        if wire.uuid == self.wire_id && a == *from && b == *to =>
                    {
                        let target = if a.x == b.x {
                            Point::new(destination.x, anchor.y)
                        } else if a.y == b.y {
                            Point::new(anchor.x, destination.y)
                        } else {
                            destination
                        };
                        if target == anchor {
                            return Ok(WireEditPlan::default());
                        }
                        let shifted_a = translate(a, anchor, target)?;
                        let shifted_b = translate(b, anchor, target)?;
                        if shifted_a != a {
                            paths.push(route(a, shifted_a, a, b, routing)?);
                        }
                        paths.push(route(shifted_a, shifted_b, a, b, routing)?);
                        if shifted_b != b {
                            paths.push(route(shifted_b, b, a, b, routing)?);
                        }
                        true
                    }
                    WireHandle::InsertVertex { from, to }
                        if wire.uuid == self.wire_id && a == *from && b == *to =>
                    {
                        paths.push(route(a, destination, a, b, routing)?);
                        paths.push(route(destination, b, a, b, routing)?);
                        true
                    }
                    _ => false,
                };
                if edit {
                    changed = true;
                    for path in paths {
                        for edge in path.windows(2) {
                            edges.push((edge[0], edge[1], true));
                        }
                    }
                } else {
                    edges.push((a, b, false));
                }
            }
            if changed {
                emit(self, &mut plan, wire, &edges)?;
            }
        }
        if plan.wires.is_empty() && plan.junctions.is_empty() {
            return Err(invalid("Wire handle has no editable source span"));
        }
        if matches!(handle, WireHandle::Endpoint { .. })
            && !self
                .sheet
                .junctions
                .values()
                .any(|j| j.position == destination)
            && self.sheet.wires.values().any(|wire| {
                wire.uuid != self.wire_id
                    && destination != wire.from
                    && destination != wire.to
                    && on(destination, wire.from, wire.to)
            })
        {
            let name = format!("wire-edit-junction:{}:endpoint-contact", self.wire_id);
            let id = Uuid::new_v5(&self.edit_seed, name.as_bytes());
            if self.sheet.wires.contains_key(&id)
                || self.sheet.junctions.contains_key(&id)
                || plan.wires.contains_key(&id)
            {
                return Err(invalid("Wire edit physical Junction identity collision"));
            }
            plan.junctions.insert(
                id,
                Junction {
                    uuid: id,
                    position: destination,
                },
            );
        }
        validate(
            self,
            &plan,
            matches!(handle, WireHandle::Endpoint { .. }).then_some(destination),
        )?;
        Ok(plan)
    }
}

pub(super) fn emit(
    basis: &WireEditBasis,
    plan: &mut WireEditPlan,
    wire: &SchematicWire,
    edges: &[(Point, Point, bool)],
) -> Result<(), EngineError> {
    // Unmodified consecutive portions remain one authored tail; virtual contact
    // vertices alone never force normalization of that tail into physical pieces.
    let mut pieces: Vec<(Point, Point, bool)> = Vec::new();
    for &(from, to, modified) in edges {
        if from == to {
            return Err(invalid("Wire edit produced a collapsed span"));
        }
        if let Some(last) = pieces.last_mut()
            && !modified
            && !last.2
            && last.1 == from
        {
            last.1 = to;
        } else {
            pieces.push((from, to, modified));
        }
    }
    for (slot, (from, to, _)) in pieces.into_iter().enumerate() {
        let id = if slot == 0 {
            wire.uuid
        } else {
            let name = format!("wire-edit:{}:{slot}", wire.uuid);
            let id = Uuid::new_v5(&basis.edit_seed, name.as_bytes());
            if basis.sheet.wires.contains_key(&id) || plan.wires.contains_key(&id) {
                return Err(invalid("Wire edit physical edge identity collision"));
            }
            plan.new_wire_origins.insert(id, wire.uuid);
            id
        };
        plan.wires.insert(id, SchematicWire { uuid: id, from, to });
    }
    Ok(())
}
