//! Existing wiring poses share checked coordinates and fixed-contact admission.
use super::{geometry::*, types::*};
use crate::{
    api::native_write::design_clipboard::{
        ClipboardClass, ClipboardPlacement, ClipboardTarget, checked_cardinal_point,
    },
    connectivity::schematic_attachment_points,
    error::EngineError,
    ir::geometry::Point,
    schematic::Sheet,
};
use std::collections::BTreeSet;
use uuid::Uuid;

/// Complete explicit ownership; capture never grows a selected Run or network.
#[derive(Clone, Debug)]
pub struct WiringTransformBasis {
    sheet: Sheet,
    wires: BTreeSet<Uuid>,
    junctions: BTreeSet<Uuid>,
}
impl WiringTransformBasis {
    pub fn capture(sheet: &Sheet, targets: &[ClipboardTarget]) -> Result<Self, EngineError> {
        if targets.is_empty() {
            return Err(invalid("Select wiring to transform"));
        }
        let mut wires = BTreeSet::new();
        let mut junctions = BTreeSet::new();
        let mut all = BTreeSet::new();
        for target in targets {
            if !all.insert(target.id) {
                return Err(invalid("Wiring transform contains duplicate targets"));
            }
            match target.class {
                ClipboardClass::Symbol => {
                    return Err(invalid("Use the Symbol Move command for existing Symbols"));
                }
                ClipboardClass::Wire => {
                    let wire = sheet
                        .wires
                        .get(&target.id)
                        .ok_or_else(|| invalid("Selected Wire is unavailable in this source"))?;
                    if wire.uuid != target.id || wire.from == wire.to {
                        return Err(invalid("Selected Wire is degenerate or mismatched"));
                    }
                    wires.insert(target.id);
                }
                ClipboardClass::Junction => {
                    let junction = sheet.junctions.get(&target.id).ok_or_else(|| {
                        invalid("Selected Junction is unavailable in this source")
                    })?;
                    if junction.uuid != target.id {
                        return Err(invalid("Selected Junction identity is mismatched"));
                    }
                    junctions.insert(target.id);
                }
            }
        }
        Ok(Self {
            sheet: sheet.clone(),
            wires,
            junctions,
        })
    }

    pub fn plan(
        &self,
        reference: Point,
        placement: ClipboardPlacement,
    ) -> Result<WireEditPlan, EngineError> {
        // Validate pose even for an identity or empty changed-output frame.
        checked_cardinal_point(reference, reference, placement)?;
        let mut plan = WireEditPlan::default();
        for id in &self.wires {
            let original = &self.sheet.wires[id];
            let mut wire = original.clone();
            wire.from = checked_cardinal_point(wire.from, reference, placement)?;
            wire.to = checked_cardinal_point(wire.to, reference, placement)?;
            if &wire != original {
                plan.wires.insert(*id, wire);
            }
        }
        for id in &self.junctions {
            let original = &self.sheet.junctions[id];
            let mut junction = original.clone();
            junction.position = checked_cardinal_point(junction.position, reference, placement)?;
            if &junction != original {
                plan.junctions.insert(*id, junction);
            }
        }
        let after = plan.apply_to_sheet(&self.sheet);
        // Selected markers move, but any coincident unselected pin/label/port
        // remains a fixed attachment through the canonical census.
        let mut fixed = self.sheet.clone();
        fixed.junctions.retain(|id, _| !self.junctions.contains(id));
        let fixed_points = schematic_attachment_points(&fixed);
        let mut independent = self.sheet.clone();
        independent.junctions.clear();
        let independent_points = schematic_attachment_points(&independent);
        let gate_survives = |point| {
            self.sheet
                .junctions
                .values()
                .filter(|gate| gate.position == point)
                .any(|gate| {
                    after
                        .junctions
                        .get(&gate.uuid)
                        .is_some_and(|junction| junction.position == point)
                })
        };
        for id in &self.wires {
            let before_wire = &self.sheet.wires[id];
            let after_wire = &after.wires[id];
            let wire_contacts = self
                .sheet
                .wires
                .iter()
                .filter(|(id, _)| !self.wires.contains(*id))
                .flat_map(|(_, wire)| [wire.from, wire.to]);
            for point in wire_contacts {
                let direct = point == before_wire.from || point == before_wire.to;
                let gated = strictly_on(point, before_wire.from, before_wire.to)
                    && self
                        .sheet
                        .junctions
                        .values()
                        .any(|gate| gate.position == point);
                if !direct && !gated {
                    continue;
                }
                if !on(point, after_wire.from, after_wire.to) {
                    return Err(invalid("Wiring transform would detach a fixed contact"));
                }
                if independent_points.contains(&point) {
                    continue;
                }
                if direct && point != after_wire.from && point != after_wire.to {
                    return Err(invalid(
                        "Wiring transform would replace a fixed direct endpoint contact",
                    ));
                }
                if gated
                    && (!strictly_on(point, after_wire.from, after_wire.to)
                        || !gate_survives(point))
                {
                    return Err(invalid(
                        "Wiring transform would detach a fixed marker-gated contact",
                    ));
                }
            }
            for point in &fixed_points {
                if on(*point, before_wire.from, before_wire.to)
                    && !on(*point, after_wire.from, after_wire.to)
                {
                    return Err(invalid("Wiring transform would detach a fixed contact"));
                }
            }
        }
        for (id, junction) in &plan.junctions {
            if after
                .junctions
                .iter()
                .any(|(other, j)| other != id && j.position == junction.position)
            {
                return Err(invalid("Wiring transform would overlap Junctions"));
            }
            let old = &self.sheet.junctions[id];
            if independent_points.contains(&old.position) || gate_survives(old.position) {
                continue;
            }
            // Keep each original fixed-wire incidence, even when a redundant
            // route hides its loss in the whole-net partition. A through dot
            // may slide along its one fixed through-wire, never off it or onto
            // its endpoint; fixed physical endpoints retain their coordinate.
            for (wire_id, wire) in &self.sheet.wires {
                if self.wires.contains(wire_id) || !on(old.position, wire.from, wire.to) {
                    continue;
                }
                let retained = if old.position == wire.from || old.position == wire.to {
                    junction.position == old.position
                } else {
                    strictly_on(junction.position, wire.from, wire.to)
                };
                if !retained {
                    return Err(invalid(
                        "Wiring transform would detach a fixed marker-gated contact",
                    ));
                }
            }
        }
        validate_preserving(&self.sheet, &plan)?;
        Ok(plan)
    }
}

fn strictly_on(point: Point, from: Point, to: Point) -> bool {
    point != from && point != to && on(point, from, to)
}

impl WireEditBasis {
    /// Grid/menu displacement configures the same captured handle plan. The
    /// caller supplies checked nanometer steps; this service assigns no keys.
    pub fn plan_displacement(
        &self,
        handle: &WireHandle,
        delta: Point,
        routing: WireRouting,
    ) -> Result<WireEditPlan, EngineError> {
        let anchor = self.anchor(handle)?;
        let destination = translate(anchor, Point::zero(), delta)?;
        self.plan_move(handle, destination, routing)
    }
}
