//! Nongraphical scalar source validation and explicit occurrence bindings.
use crate::{error::EngineError, schematic::Schematic, substrate::ElectricalOccurrence};
use std::collections::BTreeSet;

pub(super) fn bindings(
    schematic: &Schematic,
) -> Result<Vec<(ElectricalOccurrence, Option<ElectricalOccurrence>)>, EngineError> {
    let mut result = Vec::new();
    for sheet in schematic.sheets.values() {
        if sheet.net_declarations.is_empty()
            && sheet
                .bus_entries
                .values()
                .all(|e| e.member_declaration.is_none())
        {
            continue;
        }
        if schematic.sheet_instances.values().any(|instance| {
            schematic
                .sheet_definitions
                .get(&instance.definition)
                .is_some_and(|definition| definition.root_sheet == sheet.uuid)
        }) {
            return Err(invalid(
                "scalar declarations require a unique root-sheet occurrence",
            ));
        }
        let mut names = BTreeSet::new();
        for (id, declaration) in &sheet.net_declarations {
            if *id != declaration.uuid
                || id.is_nil()
                || declaration.name.is_empty()
                || declaration.name.trim() != declaration.name
                || declaration.name.chars().any(char::is_control)
                || !names.insert(&declaration.name)
            {
                return Err(invalid("invalid or duplicate scalar declaration"));
            }
            if sheet.labels.values().any(|l| l.name == declaration.name)
                || sheet.ports.values().any(|p| p.name == declaration.name)
            {
                return Err(invalid(
                    "Label or Port name collides with a scalar declaration",
                ));
            }
            result.push((
                ElectricalOccurrence {
                    class: "net_declarations".into(),
                    source_id: *id,
                    instance_path: vec![],
                },
                None,
            ));
        }
        for entry in sheet.bus_entries.values() {
            let Some(member) = &entry.member_declaration else {
                continue;
            };
            let declaration = sheet
                .net_declarations
                .get(&member.source_id)
                .ok_or_else(|| invalid("Bus Entry scalar declaration is missing from its sheet"))?;
            if member.class != "net_declarations" || !member.instance_path.is_empty() {
                return Err(invalid("Bus Entry has an invalid declaration occurrence"));
            }
            let bus = sheet
                .buses
                .get(&entry.bus)
                .ok_or_else(|| invalid("Bus Entry drawing is missing"))?;
            if !bus.members.contains(&declaration.name) {
                return Err(invalid(
                    "remove dependent Bus Entries before removing their member",
                ));
            }
            let wire = entry
                .wire
                .and_then(|id| sheet.wires.get(&id))
                .ok_or_else(|| invalid("Bus Entry member wire is missing"))?;
            let end = crate::ir::geometry::Point::new(
                entry
                    .position
                    .x
                    .checked_add(entry.size.x)
                    .ok_or_else(|| invalid("entry overflow"))?,
                entry
                    .position
                    .y
                    .checked_add(entry.size.y)
                    .ok_or_else(|| invalid("entry overflow"))?,
            );
            if (end != wire.from && end != wire.to)
                || end == entry.position
                || !bus.segments.windows(2).any(|p| {
                    super::exact_segment_intersection(entry.position, entry.position, p[0], p[1])
                        == Ok(Some(entry.position))
                })
            {
                return Err(invalid(
                    "Bus Entry no longer contacts its bus and member wire",
                ));
            }
            result.push((
                member.clone(),
                Some(ElectricalOccurrence {
                    class: "wires".into(),
                    source_id: wire.uuid,
                    instance_path: vec![],
                }),
            ));
        }
    }
    Ok(result)
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid scalar declaration: {reason}"))
}
