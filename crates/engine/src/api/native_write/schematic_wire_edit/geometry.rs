use super::types::*;
use crate::{
    connectivity::{exact_segment_intersection, physical_partitions, schematic_attachment_points},
    error::EngineError,
    ir::geometry::Point,
    schematic::{Schematic, Sheet},
    substrate::ElectricalOccurrence,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use uuid::Uuid;

pub(super) fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
pub(super) fn on(point: Point, from: Point, to: Point) -> bool {
    exact_segment_intersection(point, point, from, to).is_ok_and(|p| p.is_some())
}
pub(super) fn translate(point: Point, from: Point, to: Point) -> Result<Point, EngineError> {
    let x = i128::from(point.x) + i128::from(to.x) - i128::from(from.x);
    let y = i128::from(point.y) + i128::from(to.y) - i128::from(from.y);
    Ok(Point::new(
        i64::try_from(x).map_err(|_| invalid("Wire edit coordinate overflow"))?,
        i64::try_from(y).map_err(|_| invalid("Wire edit coordinate overflow"))?,
    ))
}
pub(super) fn route(
    from: Point,
    to: Point,
    old_from: Point,
    old_to: Point,
    routing: WireRouting,
) -> Result<Vec<Point>, EngineError> {
    if from == to {
        return Err(invalid("Wire edit would collapse an adjacent span"));
    }
    let corner = if routing == WireRouting::Orthogonal && from.x != to.x && from.y != to.y {
        if old_from.x == old_to.x {
            WireCornerOrder::End
        } else if old_from.y == old_to.y {
            WireCornerOrder::Start
        } else {
            return Err(invalid(
                "Choose free routing for a nonorthogonal source span",
            ));
        }
    } else {
        WireCornerOrder::Start
    };
    super::path::routed_wire_points(from, to, WireRouteConfig { routing, corner })
}

fn groups(
    sheet: &Sheet,
    origins: &BTreeMap<Uuid, Uuid>,
) -> Result<Vec<BTreeSet<ElectricalOccurrence>>, EngineError> {
    let schematic = Schematic {
        uuid: sheet.uuid,
        sheets: HashMap::from([(sheet.uuid, sheet.clone())]),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: vec![],
    };
    let mut groups = physical_partitions(&schematic)?;
    for group in &mut groups {
        *group = group
            .iter()
            .cloned()
            .map(|mut member| {
                if member.class == "wires" {
                    member.source_id = origins
                        .get(&member.source_id)
                        .copied()
                        .unwrap_or(member.source_id);
                }
                member
            })
            .collect();
    }
    groups.sort();
    Ok(groups)
}
fn member_at(sheet: &Sheet, member: &ElectricalOccurrence, point: Point) -> bool {
    match member.class.as_str() {
        "wires" => sheet
            .wires
            .get(&member.source_id)
            .is_some_and(|w| on(point, w.from, w.to)),
        "junctions" => sheet
            .junctions
            .get(&member.source_id)
            .is_some_and(|j| j.position == point),
        "labels" => sheet
            .labels
            .get(&member.source_id)
            .is_some_and(|l| l.position == point),
        "ports" => sheet
            .ports
            .get(&member.source_id)
            .is_some_and(|p| p.position == point),
        "pins" => sheet
            .symbols
            .values()
            .flat_map(|s| &s.pins)
            .any(|p| p.uuid == member.source_id && p.position == point),
        _ => false,
    }
}
pub(super) fn validate(
    basis: &WireEditBasis,
    plan: &WireEditPlan,
    endpoint: Option<Point>,
) -> Result<(), EngineError> {
    if endpoint.is_none() {
        return validate_preserving(&basis.sheet, plan);
    }
    let after = admit_plan(&basis.sheet, plan)?;
    let before_groups = groups(&basis.sheet, &BTreeMap::new())?;
    let after_groups = groups(&after, &plan.new_wire_origins)?;
    if let Some(destination) = endpoint {
        let wire = basis.wire();
        for point in schematic_attachment_points(&basis.sheet) {
            if point != wire.from
                && point != wire.to
                && on(point, wire.from, wire.to)
                && !after
                    .wires
                    .values()
                    .filter(|w| {
                        w.uuid == wire.uuid
                            || plan.new_wire_origins.get(&w.uuid) == Some(&wire.uuid)
                    })
                    .any(|w| on(point, w.from, w.to))
            {
                return Err(invalid("Wire edit would detach a fixed interior contact"));
            }
        }
        // Only the explicit endpoint may acquire a foreign electrical group.
        let index: BTreeMap<_, _> = before_groups
            .iter()
            .enumerate()
            .flat_map(|(i, g)| g.iter().cloned().map(move |member| (member, i)))
            .collect();
        let destination_groups: BTreeSet<_> = before_groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.iter().any(|m| member_at(&basis.sheet, m, destination)))
            .map(|(i, _)| i)
            .collect();
        let origin_group = before_groups.iter().position(|g| {
            g.iter()
                .any(|m| m.class == "wires" && m.source_id == wire.uuid)
        });
        for group in after_groups {
            let merged: BTreeSet<_> = group.iter().filter_map(|m| index.get(m).copied()).collect();
            if merged.len() > 1
                && merged
                    .iter()
                    .any(|i| Some(*i) != origin_group && !destination_groups.contains(i))
            {
                return Err(invalid(
                    "Wire route would short a foreign contact away from the edited endpoint",
                ));
            }
        }
    }
    Ok(())
}

/// The same fixed-connectivity admission for complete existing-object transforms.
pub(super) fn validate_preserving(sheet: &Sheet, plan: &WireEditPlan) -> Result<(), EngineError> {
    let after = admit_plan(sheet, plan)?;
    let before_groups = groups(sheet, &BTreeMap::new())?;
    if before_groups != groups(&after, &plan.new_wire_origins)? {
        return Err(invalid(
            "Wire drag would change fixed electrical connectivity",
        ));
    }
    require_no_foreign_endpoint_contacts(sheet, plan, &after, &before_groups)
}

/// A new edge endpoint over a foreign wire interior looks like a T contact even
/// when the explicit-junction graph correctly leaves it disconnected. Preserving
/// drag refuses that misleading topology; deliberate endpoint edits create a dot.
fn require_no_foreign_endpoint_contacts(
    sheet: &Sheet,
    plan: &WireEditPlan,
    after: &Sheet,
    groups: &[BTreeSet<ElectricalOccurrence>],
) -> Result<(), EngineError> {
    let source_group: BTreeMap<_, _> = groups
        .iter()
        .enumerate()
        .flat_map(|(index, members)| {
            members
                .iter()
                .filter(|member| member.class == "wires")
                .map(move |member| (member.source_id, index))
        })
        .collect();
    let origin = |id: Uuid| plan.new_wire_origins.get(&id).copied().unwrap_or(id);
    for wire in plan.wires.values() {
        let source_id = origin(wire.uuid);
        let source = sheet
            .wires
            .get(&source_id)
            .ok_or_else(|| invalid("Wire drag source origin unavailable"))?;
        for point in [wire.from, wire.to] {
            // A preexisting endpoint at this coordinate is not a new contact.
            if point == source.from || point == source.to {
                continue;
            }
            for foreign in after.wires.values() {
                let foreign_id = origin(foreign.uuid);
                if foreign_id == source_id
                    || source_group.get(&foreign_id) == source_group.get(&source_id)
                {
                    continue;
                }
                if point != foreign.from
                    && point != foreign.to
                    && on(point, foreign.from, foreign.to)
                {
                    return Err(invalid(
                        "Wire drag would place a new endpoint on a foreign wire interior; use an explicit endpoint connection",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn admit_plan(sheet: &Sheet, plan: &WireEditPlan) -> Result<Sheet, EngineError> {
    let touched: BTreeSet<_> = plan
        .wires
        .keys()
        .map(|id| plan.new_wire_origins.get(id).copied().unwrap_or(*id))
        .chain(plan.removed_wire_ids.iter().copied())
        .chain(
            plan.coalesced_wire_origins
                .values()
                .flat_map(|ids| ids.iter().copied()),
        )
        .collect();
    if sheet
        .bus_entries
        .values()
        .any(|entry| entry.wire.is_some_and(|id| touched.contains(&id)))
    {
        return Err(invalid(
            "Wire is referenced by a bus entry; remove the entry first",
        ));
    }

    let after = plan.apply_to_sheet(sheet);
    for wire in plan.wires.values() {
        if wire.from == wire.to {
            return Err(invalid("Wire edit would create a collapsed edge"));
        }
        for other in after.wires.values().filter(|w| w.uuid != wire.uuid) {
            if ((wire.from == other.from && wire.to == other.to)
                || (wire.from == other.to && wire.to == other.from))
                && !(sheet.wires.get(&wire.uuid) == Some(wire)
                    && sheet.wires.get(&other.uuid) == Some(other))
            {
                return Err(invalid("Wire edit would create a duplicate physical edge"));
            }
        }
    }
    for source_id in touched {
        let Some(source) = sheet.wires.get(&source_id) else {
            return Err(invalid("Wire edit origin unavailable"));
        };
        for junction in sheet.junctions.values() {
            // Explicit degree-two lifecycle cleanup removes the marker itself;
            // all retained markers must keep their original deletion gate.
            if plan.removed_junction_ids.contains(&junction.uuid) {
                continue;
            }
            if junction.position == source.from
                || junction.position == source.to
                || !on(junction.position, source.from, source.to)
            {
                continue;
            }
            let contact = after
                .junctions
                .get(&junction.uuid)
                .map_or(junction.position, |j| j.position);
            if after
                .wires
                .values()
                .filter(|w| {
                    w.uuid == source_id || plan.new_wire_origins.get(&w.uuid) == Some(&source_id)
                })
                .any(|w| w.from == contact || w.to == contact)
            {
                return Err(invalid(
                    "Wire edit would replace a marker-gated interior contact with physical endpoints",
                ));
            }
        }
    }
    Ok(after)
}

/// Topology edits may remove the selected span, but may never acquire a foreign
/// group. Split/coalescence additionally retain all remaining physical groups.
pub(super) fn validate_topology(
    basis: &WireEditBasis,
    plan: &WireEditPlan,
    preserving: bool,
) -> Result<(), EngineError> {
    let after = admit_plan(&basis.sheet, plan)?;
    let before_groups = groups(&basis.sheet, &BTreeMap::new())?;
    let after_groups = groups(&after, &plan.new_wire_origins)?;
    require_no_foreign_endpoint_contacts(&basis.sheet, plan, &after, &before_groups)?;
    let index: BTreeMap<_, _> = before_groups
        .iter()
        .enumerate()
        .flat_map(|(i, g)| g.iter().cloned().map(move |m| (m, i)))
        .collect();
    if after_groups.iter().any(|g| {
        g.iter()
            .filter_map(|m| index.get(m))
            .collect::<BTreeSet<_>>()
            .len()
            > 1
    }) {
        return Err(invalid(
            "Wire topology edit would join a foreign electrical group",
        ));
    }
    if preserving {
        let mut expected: Vec<BTreeSet<_>> = before_groups
            .into_iter()
            .map(|g| {
                g.into_iter()
                    .filter(|m| {
                        !(m.class == "wires" && plan.removed_wire_ids.contains(&m.source_id))
                            && !(m.class == "junctions"
                                && plan.removed_junction_ids.contains(&m.source_id))
                    })
                    .collect()
            })
            .filter(|g: &BTreeSet<_>| !g.is_empty())
            .collect();
        expected.sort();
        if expected != after_groups {
            return Err(invalid("Wire topology edit would detach a fixed contact"));
        }
    }
    Ok(())
}
