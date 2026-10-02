//! PM055 physical Bus paths within one declared Bus and occurrence.
use super::segment_geometry::{point_on_wire_segment, segments_touch};
use crate::{
    ir::geometry::Point,
    schematic::{Bus, Schematic},
    substrate::{ElectricalOccurrence, ElectricalQueryFailure},
};
use std::collections::BTreeSet;

pub(crate) struct BusPhysicalMembership {
    pub owned: BTreeSet<ElectricalOccurrence>,
    pub foreign: BTreeSet<(ElectricalOccurrence, ElectricalOccurrence)>,
}

pub(crate) fn bus_physical(
    schematic: &Schematic,
    origin: &ElectricalOccurrence,
    declared: &BTreeSet<ElectricalOccurrence>,
) -> Result<BusPhysicalMembership, ElectricalQueryFailure> {
    let sheet = super::occurrence_sheet(schematic, origin).map_err(|_| {
        ElectricalQueryFailure::InvalidOccurrence {
            origin: origin.clone(),
        }
    })?;
    let occurrence = |class: &str, source_id| ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: origin.instance_path.clone(),
    };
    let local = |id| declared.contains(&occurrence("buses", id));
    let mut selected = BTreeSet::from([origin.source_id]);
    for bus in sheet.buses.values().filter(|b| local(b.uuid)) {
        valid(bus, &occurrence("buses", bus.uuid))?;
    }
    loop {
        let mut next = selected.clone();
        for bus in sheet.buses.values().filter(|b| local(b.uuid)) {
            if selected.iter().any(|id| touches(&sheet.buses[id], bus)) {
                next.insert(bus.uuid);
            }
        }
        if next == selected {
            break;
        }
        selected = next;
    }
    let mut owned: BTreeSet<_> = selected.iter().map(|id| occurrence("buses", *id)).collect();
    for entry in sheet
        .bus_entries
        .values()
        .filter(|e| selected.contains(&e.bus))
    {
        let member = occurrence("bus_entries", entry.uuid);
        if !declared.contains(&member) {
            return Err(ElectricalQueryFailure::UnavailableAssignment { origin: member });
        }
        owned.insert(member);
    }
    for (class, id, point) in sheet
        .labels
        .values()
        .map(|l| ("labels", l.uuid, l.position))
        .chain(sheet.ports.values().map(|p| ("ports", p.uuid, p.position)))
    {
        let member = occurrence(class, id);
        if declared.contains(&member) && selected.iter().any(|id| on_path(point, &sheet.buses[id]))
        {
            owned.insert(member);
        }
    }
    let mut foreign = BTreeSet::new();
    for bus in sheet.buses.values().filter(|b| !local(b.uuid)) {
        // An unavailable foreign path cannot silently become no contact.
        valid(bus, &occurrence("buses", bus.uuid))?;
        for id in &selected {
            if touches(&sheet.buses[id], bus) {
                foreign.insert((occurrence("buses", *id), occurrence("buses", bus.uuid)));
            }
        }
    }
    Ok(BusPhysicalMembership { owned, foreign })
}

fn valid(bus: &Bus, origin: &ElectricalOccurrence) -> Result<(), ElectricalQueryFailure> {
    if bus.segments.len() < 2 || bus.segments.windows(2).any(|p| p[0] == p[1]) {
        Err(ElectricalQueryFailure::UnavailableGeometry {
            origin: origin.clone(),
            reason: "Bus spine has no valid authored path".into(),
        })
    } else {
        Ok(())
    }
}
fn touches(a: &Bus, b: &Bus) -> bool {
    a.segments.windows(2).any(|a| {
        b.segments
            .windows(2)
            .any(|b| segments_touch(a[0], a[1], b[0], b[1]))
    })
}
fn on_path(point: Point, bus: &Bus) -> bool {
    bus.segments
        .windows(2)
        .any(|p| point_on_wire_segment(point, p[0], p[1]))
}
