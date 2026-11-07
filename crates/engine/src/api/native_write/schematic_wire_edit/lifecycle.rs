//! Explicit topology edits over the same immutable contact basis and raw origins.
use super::{basis::nodes, geometry::*, move_plan::emit, types::*};
use crate::{error::EngineError, ir::geometry::Point, schematic::Sheet};
use std::collections::BTreeSet;
use uuid::Uuid;

impl WireEditBasis {
    pub fn plan_split(&self, point: Point) -> Result<WireEditPlan, EngineError> {
        let wire = self.wire();
        if point == wire.from || point == wire.to || !on(point, wire.from, wire.to) {
            return Err(invalid(
                "Split requires a strict interior point on the captured wire",
            ));
        }
        let mut plan = WireEditPlan::default();
        emit(
            self,
            &mut plan,
            wire,
            &[(wire.from, point, true), (point, wire.to, true)],
        )?;
        validate_topology(self, &plan, true)?;
        Ok(plan)
    }

    pub fn plan_remove_span(&self, from: Point, to: Point) -> Result<WireEditPlan, EngineError> {
        let wire = self.wire();
        let points = nodes(&self.sheet, wire);
        if !points.windows(2).any(|p| p == [from, to]) {
            return Err(invalid(
                "Remove span requires one captured contact-to-contact segment",
            ));
        }
        let edges: Vec<_> = points
            .windows(2)
            .filter(|p| *p != [from, to])
            .map(|p| (p[0], p[1], false))
            .collect();
        let mut plan = WireEditPlan::default();
        if edges.is_empty() {
            plan.removed_wire_ids.insert(wire.uuid);
        } else {
            emit(self, &mut plan, wire, &edges)?;
        }
        cleanup(self, &mut plan, &[from, to]);
        validate_topology(self, &plan, false)?;
        Ok(plan)
    }

    pub fn plan_remove_vertex(&self, point: Point) -> Result<WireEditPlan, EngineError> {
        self.anchor(&WireHandle::Vertex { point })?;
        if fixed_identifier(&self.sheet, point) {
            return Err(invalid(
                "A fixed pin or identifier cannot be removed with a wire vertex",
            ));
        }
        let incident = incident(&self.sheet, point);
        if incident.len() != 2 {
            return Err(invalid(
                "Remove vertex requires exactly two incident wire spans",
            ));
        }
        let mut plan = WireEditPlan::default();
        for junction in self
            .sheet
            .junctions
            .values()
            .filter(|j| j.position == point)
        {
            plan.removed_junction_ids.insert(junction.uuid);
        }
        if incident[0].0 == incident[1].0 {
            // The vertex is virtual on one straight source member; only its
            // redundant degree-two marker needs removal, never a read-time split.
            if plan.removed_junction_ids.is_empty() {
                return Err(invalid(
                    "This point is not an authored bend or removable marker",
                ));
            }
        } else {
            let keep = self.wire_id;
            let retained = incident
                .iter()
                .find(|s| s.0 == keep)
                .ok_or_else(|| invalid("Vertex source changed"))?;
            let other = incident.iter().find(|s| s.0 != keep).unwrap();
            let source = &self.sheet.wires[&keep];
            let other_source = &self.sheet.wires[&other.0];
            let opposite = |wire: &crate::schematic::SchematicWire| {
                if wire.from == point {
                    Some(wire.to)
                } else if wire.to == point {
                    Some(wire.from)
                } else {
                    None
                }
            };
            if let (Some(a), Some(b)) = (opposite(source), opposite(other_source)) {
                if a != b && on(point, a, b) {
                    // Undo a physical collinear split across the full members.
                    // Virtual contacts inside either member must stay interior;
                    // bridging only their nearest spans would hardwire markers.
                    let mut merged = source.clone();
                    if merged.from == point {
                        merged.from = b;
                    } else {
                        merged.to = b;
                    }
                    plan.wires.insert(keep, merged);
                    plan.removed_wire_ids.insert(other.0);
                    plan.coalesced_wire_origins
                        .insert(keep, BTreeSet::from([keep, other.0]));
                    validate_topology(self, &plan, true)?;
                    return Ok(plan);
                }
            }
            let retained_outer = if retained.1 == point {
                retained.2
            } else {
                retained.1
            };
            let other_outer = if other.1 == point { other.2 } else { other.1 };
            if retained_outer == other_outer {
                return Err(invalid("Vertex removal would collapse a loop"));
            }
            for source_id in [keep, other.0] {
                let source = &self.sheet.wires[&source_id];
                let pairs = nodes(&self.sheet, source);
                let mut edges = Vec::new();
                let mut bridge = None;
                for pair in pairs.windows(2) {
                    if pair[0] == point || pair[1] == point {
                        if source_id == keep {
                            let edge = if pair[0] == point {
                                (other_outer, retained_outer, true)
                            } else {
                                (retained_outer, other_outer, true)
                            };
                            bridge = Some((edge.0, edge.1));
                            edges.push(edge);
                        }
                    } else {
                        edges.push((pair[0], pair[1], false));
                    }
                }
                if edges.is_empty() {
                    plan.removed_wire_ids.insert(source_id);
                } else {
                    emit(self, &mut plan, source, &edges)?;
                    if let Some((from, to)) = bridge {
                        let output = plan
                            .wires
                            .values()
                            .find(|w| {
                                w.from == from
                                    && w.to == to
                                    && (w.uuid == source_id
                                        || plan.new_wire_origins.get(&w.uuid) == Some(&source_id))
                            })
                            .ok_or_else(|| invalid("Coalesced wire output unavailable"))?
                            .uuid;
                        plan.coalesced_wire_origins
                            .insert(output, BTreeSet::from([keep, other.0]));
                    }
                }
            }
        }
        validate_topology(self, &plan, true)?;
        Ok(plan)
    }
}

fn fixed_identifier(sheet: &Sheet, point: Point) -> bool {
    sheet
        .symbols
        .values()
        .flat_map(|s| &s.pins)
        .any(|p| p.position == point)
        || sheet.labels.values().any(|l| l.position == point)
        || sheet.ports.values().any(|p| p.position == point)
}
fn incident(sheet: &Sheet, point: Point) -> Vec<(Uuid, Point, Point)> {
    let mut result: Vec<_> = sheet
        .wires
        .values()
        .flat_map(|wire| {
            nodes(sheet, wire)
                .windows(2)
                .filter(|pair| pair[0] == point || pair[1] == point)
                .map(|pair| (wire.uuid, pair[0], pair[1]))
                .collect::<Vec<_>>()
        })
        .collect();
    result.sort_by_key(|span| span.0);
    result
}
fn cleanup(basis: &WireEditBasis, plan: &mut WireEditPlan, points: &[Point]) {
    for point in points {
        let after = plan.apply_to_sheet(&basis.sheet);
        if fixed_identifier(&after, *point) {
            continue;
        }
        let joints: Vec<_> = after
            .junctions
            .values()
            .filter(|j| j.position == *point)
            .map(|j| j.uuid)
            .collect();
        if joints.is_empty() {
            continue;
        }
        let spans = incident(&after, *point);
        if spans.len() != 2 {
            continue;
        }
        if spans[0].0 == spans[1].0 {
            plan.removed_junction_ids.extend(joints);
            continue;
        }
        let a = &after.wires[&spans[0].0];
        let b = &after.wires[&spans[1].0];
        let outer_a = if a.from == *point {
            a.to
        } else if a.to == *point {
            a.from
        } else {
            continue;
        };
        let outer_b = if b.from == *point {
            b.to
        } else if b.to == *point {
            b.from
        } else {
            continue;
        };
        if outer_a == outer_b || !on(*point, outer_a, outer_b) {
            continue;
        }
        // Retain the selected source identity when it survives, otherwise use
        // deterministic source order. Canonical admission checks raw compatibility.
        let (keep, remove) = if b.uuid == basis.wire_id {
            (b, a)
        } else {
            (a, b)
        };
        let opposite = if remove.from == *point {
            remove.to
        } else {
            remove.from
        };
        let mut merged = keep.clone();
        if merged.from == *point {
            merged.from = opposite;
        } else {
            merged.to = opposite;
        }
        let origins = |wire_id: Uuid| {
            plan.coalesced_wire_origins
                .get(&wire_id)
                .cloned()
                .unwrap_or_else(|| {
                    BTreeSet::from([plan
                        .new_wire_origins
                        .get(&wire_id)
                        .copied()
                        .unwrap_or(wire_id)])
                })
        };
        let mut contributors = origins(keep.uuid);
        contributors.extend(origins(remove.uuid));
        plan.coalesced_wire_origins
            .insert(merged.uuid, contributors);
        plan.coalesced_wire_origins.remove(&remove.uuid);
        plan.wires.insert(merged.uuid, merged);
        plan.wires.remove(&remove.uuid);
        plan.new_wire_origins.remove(&remove.uuid);
        if basis.sheet.wires.contains_key(&remove.uuid) {
            plan.removed_wire_ids.insert(remove.uuid);
        }
        plan.removed_junction_ids.extend(joints);
    }
}
