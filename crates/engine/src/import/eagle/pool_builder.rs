use std::collections::{HashMap, HashSet};

use crate::error::EngineError;
use crate::import::ImportObjectCounts;
use crate::ir::geometry::Polygon;
use crate::ir::ids::{import_uuid, namespace_eagle};
use crate::pool::{
    Entity, Gate, Lifecycle, Package, Pad, PadMapEntry, Padstack, Part, Pin, Pool, Symbol, Unit,
};

use super::parser::{RawDeviceset, RawPackage, RawSymbol};

pub(super) fn build_pool(
    library_name: &str,
    raw_symbols: Vec<RawSymbol>,
    raw_packages: Vec<RawPackage>,
    raw_devicesets: Vec<RawDeviceset>,
) -> Result<Pool, EngineError> {
    let ns = namespace_eagle();
    let mut pool = Pool::default();

    let mut unit_by_symbol = HashMap::new();
    let mut pin_by_symbol_and_name = HashMap::new();
    let mut symbol_by_name = HashMap::new();

    for raw in raw_symbols {
        let unit_uuid = import_uuid(&ns, &format!("lbr:{library_name}:unit:{}", raw.name));
        let symbol_uuid = import_uuid(&ns, &format!("lbr:{library_name}:symbol:{}", raw.name));

        let mut pins = HashMap::new();
        for pin in raw.pins {
            let pin_uuid = import_uuid(
                &ns,
                &format!("lbr:{library_name}:symbol:{}:pin:{}", raw.name, pin.name),
            );
            pin_by_symbol_and_name.insert((raw.name.clone(), pin.name.clone()), pin_uuid);
            pins.insert(
                pin_uuid,
                Pin {
                    uuid: pin_uuid,
                    name: pin.name,
                    direction: pin.direction,
                    swap_group: 0,
                    alternates: Vec::new(),
                },
            );
        }

        pool.units.insert(
            unit_uuid,
            Unit {
                uuid: unit_uuid,
                name: raw.name.clone(),
                manufacturer: String::new(),
                pins,
                tags: HashSet::new(),
            },
        );
        pool.symbols.insert(
            symbol_uuid,
            Symbol {
                uuid: symbol_uuid,
                name: raw.name.clone(),
                unit: unit_uuid,
            },
        );

        unit_by_symbol.insert(raw.name.clone(), unit_uuid);
        symbol_by_name.insert(raw.name, symbol_uuid);
    }

    let mut package_pad_by_name = HashMap::new();

    for raw in raw_packages {
        let package_uuid = import_uuid(&ns, &format!("lbr:{library_name}:package:{}", raw.name));
        let mut pads = HashMap::new();

        for pad in raw.pads {
            let padstack_uuid = import_uuid(
                &ns,
                &format!(
                    "lbr:{library_name}:package:{}:padstack:{}",
                    raw.name, pad.name
                ),
            );
            let pad_uuid = import_uuid(
                &ns,
                &format!("lbr:{library_name}:package:{}:pad:{}", raw.name, pad.name),
            );
            package_pad_by_name.insert((raw.name.clone(), pad.name.clone()), pad_uuid);

            pool.padstacks.insert(
                padstack_uuid,
                Padstack {
                    uuid: padstack_uuid,
                    name: pad.padstack_name,
                },
            );

            pads.insert(
                pad_uuid,
                Pad {
                    uuid: pad_uuid,
                    name: pad.name,
                    position: pad.position,
                    padstack: padstack_uuid,
                    layer: pad.layer,
                },
            );
        }

        pool.packages.insert(
            package_uuid,
            Package {
                uuid: package_uuid,
                name: raw.name.clone(),
                pads,
                courtyard: Polygon {
                    vertices: Vec::new(),
                    closed: true,
                },
                silkscreen: raw.silkscreen,
                models_3d: Vec::new(),
                tags: HashSet::new(),
            },
        );
    }

    for raw in raw_devicesets {
        let entity_uuid = import_uuid(&ns, &format!("lbr:{library_name}:deviceset:{}", raw.name));
        let mut gates = HashMap::new();
        let mut gate_by_name = HashMap::new();

        for gate in raw.gates {
            let symbol_uuid = *symbol_by_name.get(&gate.symbol).ok_or_else(|| {
                EngineError::Import(format!(
                    "deviceset {} gate {} references unknown symbol {}",
                    raw.name, gate.name, gate.symbol
                ))
            })?;
            let unit_uuid = *unit_by_symbol.get(&gate.symbol).ok_or_else(|| {
                EngineError::Import(format!(
                    "deviceset {} gate {} references unknown unit for symbol {}",
                    raw.name, gate.name, gate.symbol
                ))
            })?;
            let gate_uuid = import_uuid(
                &ns,
                &format!(
                    "lbr:{library_name}:deviceset:{}:gate:{}",
                    raw.name, gate.name
                ),
            );
            gate_by_name.insert(gate.name.clone(), gate_uuid);
            gates.insert(
                gate_uuid,
                Gate {
                    uuid: gate_uuid,
                    name: gate.name,
                    unit: unit_uuid,
                    symbol: symbol_uuid,
                },
            );
        }

        pool.entities.insert(
            entity_uuid,
            Entity {
                uuid: entity_uuid,
                name: raw.name.clone(),
                prefix: raw.prefix.clone(),
                manufacturer: String::new(),
                gates,
                tags: HashSet::new(),
            },
        );

        let package_by_name: HashMap<_, _> = pool
            .packages
            .values()
            .map(|pkg| (pkg.name.clone(), pkg.uuid))
            .collect();

        for device in raw.devices {
            let part_uuid = import_uuid(
                &ns,
                &format!("lbr:{library_name}:{}:{}", raw.name, device.name),
            );
            let package_uuid = *package_by_name.get(&device.package).ok_or_else(|| {
                EngineError::Import(format!(
                    "device {} in deviceset {} references unknown package {}",
                    device.name, raw.name, device.package
                ))
            })?;

            let mut pad_map = HashMap::new();
            for connect in device.connects {
                let gate_uuid = *gate_by_name.get(&connect.gate).ok_or_else(|| {
                    EngineError::Import(format!(
                        "device {} in deviceset {} references unknown gate {}",
                        device.name, raw.name, connect.gate
                    ))
                })?;
                let gate = pool
                    .entities
                    .get(&entity_uuid)
                    .and_then(|entity| entity.gates.get(&gate_uuid))
                    .ok_or_else(|| {
                        EngineError::Import(format!(
                            "gate {} in deviceset {} was not materialized",
                            connect.gate, raw.name
                        ))
                    })?;
                let symbol = pool.symbols.get(&gate.symbol).ok_or_else(|| {
                    EngineError::Import(format!(
                        "gate {} in deviceset {} references missing symbol",
                        connect.gate, raw.name
                    ))
                })?;
                let pin_uuid = *pin_by_symbol_and_name
                    .get(&(symbol.name.clone(), connect.pin.clone()))
                    .ok_or_else(|| {
                        EngineError::Import(format!(
                            "connect {}.{} references unknown pin {}",
                            raw.name, device.name, connect.pin
                        ))
                    })?;
                let pad_uuid = *package_pad_by_name
                    .get(&(device.package.clone(), connect.pad.clone()))
                    .ok_or_else(|| {
                        EngineError::Import(format!(
                            "connect {}.{} references unknown pad {}",
                            raw.name, device.name, connect.pad
                        ))
                    })?;

                pad_map.insert(
                    pad_uuid,
                    PadMapEntry {
                        gate: gate_uuid,
                        pin: pin_uuid,
                    },
                );
            }

            let value = if device.name.is_empty() {
                raw.name.clone()
            } else {
                format!("{} {}", raw.name, device.name)
            };

            pool.parts.insert(
                part_uuid,
                Part {
                    uuid: part_uuid,
                    entity: entity_uuid,
                    package: package_uuid,
                    pad_map,
                    mpn: String::new(),
                    manufacturer: String::new(),
                    value,
                    description: String::new(),
                    datasheet: String::new(),
                    parametric: HashMap::new(),
                    orderable_mpns: Vec::new(),
                    tags: HashSet::new(),
                    lifecycle: Lifecycle::Unknown,
                    base: None,
                },
            );
        }
    }

    Ok(pool)
}

pub(super) fn pool_counts(pool: &Pool) -> ImportObjectCounts {
    ImportObjectCounts {
        units: pool.units.len(),
        symbols: pool.symbols.len(),
        entities: pool.entities.len(),
        padstacks: pool.padstacks.len(),
        packages: pool.packages.len(),
        parts: pool.parts.len(),
    }
}
