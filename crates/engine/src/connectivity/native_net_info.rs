//! Native scalar projection from recorded identity and captured source only.
use crate::{
    error::EngineError,
    schematic::{LabelKind, NetPinRef, Schematic, SchematicNetInfo},
    substrate::{DesignModel, ElectricalIdentity},
};
use std::collections::BTreeSet;

pub fn native_schematic_net_info(
    model: &DesignModel,
    schematic: &Schematic,
) -> Result<Vec<SchematicNetInfo>, EngineError> {
    // Declarations are not required for explicit native Net adoption. Once any
    // Net authority is recorded, even retired/partial records must not silently
    // turn back into coordinate-derived identities. Pure legacy stays unchanged.
    if schematic
        .sheets
        .values()
        .all(|s| s.net_declarations.is_empty())
        && !model
            .electrical_identities
            .values()
            .any(|record| matches!(record.identity, ElectricalIdentity::Net { .. }))
    {
        return Ok(super::schematic_net_info(schematic));
    }
    let mut result = Vec::new();
    for group in super::partitions(schematic)? {
        let owners: Vec<_> = model.electrical_identities.values().filter(|record| matches!(
            &record.identity, ElectricalIdentity::Net { anchor, retired: false, .. } if group.contains(anchor)
        )).collect();
        if owners.len() != 1 {
            return Err(EngineError::Validation(
                "Native net query/ERC requires explicit adoption of all scalar groups".into(),
            ));
        }
        let mut names = BTreeSet::new();
        let mut pins = Vec::new();
        let mut ports = BTreeSet::new();
        let mut labels = 0;
        let mut power_label = false;
        let mut sheets = BTreeSet::new();
        for occurrence in &group {
            for sheet in schematic.sheets.values() {
                let mut found = true;
                match occurrence.class.as_str() {
                    "net_declarations" => {
                        if let Some(d) = sheet.net_declarations.get(&occurrence.source_id) {
                            names.insert((2, d.name.clone()));
                        } else {
                            found = false;
                        }
                    }
                    "labels" => {
                        if let Some(l) = sheet.labels.get(&occurrence.source_id) {
                            labels += 1;
                            power_label |= matches!(l.kind, LabelKind::Power);
                            names.insert((
                                if matches!(l.kind, LabelKind::Power | LabelKind::Global) {
                                    0
                                } else {
                                    1
                                },
                                super::canonical_label_name(&l.name),
                            ));
                        } else {
                            found = false;
                        }
                    }
                    "ports" => {
                        if let Some(p) = sheet.ports.get(&occurrence.source_id) {
                            ports.insert(p.uuid);
                            names.insert((3, p.name.clone()));
                        } else {
                            found = false;
                        }
                    }
                    "wires" => found = sheet.wires.contains_key(&occurrence.source_id),
                    "junctions" => found = sheet.junctions.contains_key(&occurrence.source_id),
                    "pins" => {
                        found = false;
                        for symbol in sheet.symbols.values() {
                            if let Some(pin) =
                                symbol.pins.iter().find(|p| p.uuid == occurrence.source_id)
                            {
                                found = true;
                                pins.push(NetPinRef {
                                    uuid: pin.uuid,
                                    instance_path: occurrence.instance_path.clone(),
                                    component: symbol.reference.clone(),
                                    pin: pin.number.clone(),
                                    electrical_type: pin.electrical_type.clone(),
                                });
                            }
                        }
                    }
                    _ => found = false,
                }
                if found {
                    sheets.insert(sheet.name.clone());
                }
            }
        }
        pins.sort_by(|a, b| {
            a.component
                .cmp(&b.component)
                .then_with(|| a.pin.cmp(&b.pin))
                .then_with(|| a.uuid.cmp(&b.uuid))
                .then_with(|| a.instance_path.cmp(&b.instance_path))
        });
        let id = owners[0].id;
        let name = names
            .first()
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| format!("N${}", &id.as_simple().to_string()[..8]));
        result.push(SchematicNetInfo {
            uuid: id,
            occurrences: group.into_iter().collect(),
            semantic_class: super::infer_semantic_class(&name)
                .or_else(|| power_label.then(|| "power".into())),
            name,
            class: None,
            pins,
            labels,
            ports: ports.len(),
            port_uuids: ports.into_iter().collect(),
            sheets: sheets.into_iter().collect(),
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(result)
}

#[cfg(test)]
#[path = "native_net_info_tests.rs"]
mod tests;
