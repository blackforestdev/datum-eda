use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use std::collections::{HashMap, HashSet};

use crate::error::EngineError;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::ir::geometry::{LayerId, Point, Polygon};
use crate::ir::ids::{import_uuid, namespace_eagle};
use crate::pool::{
    Entity, Gate, Lifecycle, Package, Pad, PadMapEntry, Padstack, Part, Pin, PinDirection, Pool,
    Primitive, Symbol, Unit,
};

/// Import a standalone Eagle library (`.lbr`) into pool objects.
///
/// M0 scope only:
/// - symbols -> Unit + Symbol
/// - packages -> Package + Padstack + Pad
/// - devicesets/gates -> Entity + Gate
/// - devices/connects -> Part + pad_map
pub fn import_library_str(xml: &str) -> Result<Pool, EngineError> {
    let mut parser = EagleLibraryParser::new(xml);
    parser.parse()
}

pub fn import_library_file(path: &std::path::Path) -> Result<(Pool, ImportReport), EngineError> {
    let xml = std::fs::read_to_string(path)?;
    let library_name = extract_library_name(&xml)?;
    let pool = import_library_str(&xml)?;
    let report = ImportReport::new(ImportKind::EagleLibrary, path, pool_counts(&pool))
        .with_metadata("library_name", library_name);
    Ok((pool, report))
}

pub fn import_board_file(path: &std::path::Path) -> Result<ImportReport, EngineError> {
    Err(EngineError::Import(format!(
        "Eagle board import is not implemented yet; Eagle design import is secondary in M1: {}",
        path.display()
    )))
}

pub fn import_schematic_file(path: &std::path::Path) -> Result<ImportReport, EngineError> {
    Err(EngineError::Import(format!(
        "Eagle schematic import is not implemented yet; Eagle design import is secondary in M1: {}",
        path.display()
    )))
}

#[derive(Debug, Default)]
struct RawSymbol {
    name: String,
    pins: Vec<RawPin>,
}

#[derive(Debug)]
struct RawPin {
    name: String,
    direction: PinDirection,
}

#[derive(Debug, Default)]
struct RawPackage {
    name: String,
    pads: Vec<RawPad>,
    silkscreen: Vec<Primitive>,
}

#[derive(Debug)]
struct RawPad {
    name: String,
    position: Point,
    layer: LayerId,
    padstack_name: String,
}

#[derive(Debug, Default)]
struct RawDeviceset {
    name: String,
    prefix: String,
    gates: Vec<RawGate>,
    devices: Vec<RawDevice>,
}

#[derive(Debug)]
struct RawGate {
    name: String,
    symbol: String,
}

#[derive(Debug, Default)]
struct RawDevice {
    name: String,
    package: String,
    connects: Vec<RawConnect>,
}

#[derive(Debug)]
struct RawConnect {
    gate: String,
    pin: String,
    pad: String,
}

struct EagleLibraryParser<'a> {
    reader: Reader<&'a [u8]>,
    buf: Vec<u8>,
}

impl<'a> EagleLibraryParser<'a> {
    fn new(xml: &'a str) -> Self {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        Self {
            reader,
            buf: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Pool, EngineError> {
        let mut library_name = String::new();
        let mut symbols = Vec::new();
        let mut packages = Vec::new();
        let mut devicesets = Vec::new();

        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"library") => {
                    library_name = required_attr(&e, b"name")?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"symbols") => {
                    symbols = self.parse_symbols()?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"packages") => {
                    packages = self.parse_packages()?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"devicesets") => {
                    devicesets = self.parse_devicesets()?;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse Eagle XML: {err}"
                    )));
                }
            }
            self.buf.clear();
        }

        if library_name.is_empty() {
            return Err(EngineError::Import(
                "Eagle library missing <library name=...>".to_string(),
            ));
        }

        build_pool(&library_name, symbols, packages, devicesets)
    }

    fn parse_symbols(&mut self) -> Result<Vec<RawSymbol>, EngineError> {
        let mut symbols = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"symbol") => {
                    let owned = e.to_owned();
                    symbols.push(self.parse_symbol(&owned)?);
                }
                Ok(Event::End(e)) if e.name() == QName(b"symbols") => break,
                Ok(Event::Eof) => return unexpected_eof("symbols"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse symbols: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(symbols)
    }

    fn parse_symbol(&mut self, start: &BytesStart<'_>) -> Result<RawSymbol, EngineError> {
        let mut symbol = RawSymbol {
            name: required_attr(start, b"name")?,
            pins: Vec::new(),
        };

        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Empty(e)) if e.name() == QName(b"pin") => {
                    symbol.pins.push(parse_pin(&e)?);
                }
                Ok(Event::Start(e)) if e.name() == QName(b"pin") => {
                    symbol.pins.push(parse_pin(&e)?);
                    self.skip_element(QName(b"pin"))?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"symbol") => break,
                Ok(Event::Eof) => return unexpected_eof("symbol"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse symbol: {err}"
                    )));
                }
            }
            self.buf.clear();
        }

        Ok(symbol)
    }

    fn parse_packages(&mut self) -> Result<Vec<RawPackage>, EngineError> {
        let mut packages = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"package") => {
                    let owned = e.to_owned();
                    packages.push(self.parse_package(&owned)?);
                }
                Ok(Event::End(e)) if e.name() == QName(b"packages") => break,
                Ok(Event::Eof) => return unexpected_eof("packages"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse packages: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(packages)
    }

    fn parse_package(&mut self, start: &BytesStart<'_>) -> Result<RawPackage, EngineError> {
        let mut package = RawPackage {
            name: required_attr(start, b"name")?,
            pads: Vec::new(),
            silkscreen: Vec::new(),
        };

        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Empty(e)) if e.name() == QName(b"pad") => {
                    package.pads.push(parse_package_pad(&e, false)?);
                }
                Ok(Event::Empty(e)) if e.name() == QName(b"smd") => {
                    package.pads.push(parse_package_pad(&e, true)?);
                }
                Ok(Event::Empty(e)) if e.name() == QName(b"wire") => {
                    if let Some(line) = parse_silkscreen_wire(&e)? {
                        package.silkscreen.push(line);
                    }
                }
                Ok(Event::Start(e)) if e.name() == QName(b"pad") => {
                    package.pads.push(parse_package_pad(&e, false)?);
                    self.skip_element(QName(b"pad"))?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"smd") => {
                    package.pads.push(parse_package_pad(&e, true)?);
                    self.skip_element(QName(b"smd"))?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"wire") => {
                    if let Some(line) = parse_silkscreen_wire(&e)? {
                        package.silkscreen.push(line);
                    }
                    self.skip_element(QName(b"wire"))?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"package") => break,
                Ok(Event::Eof) => return unexpected_eof("package"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse package: {err}"
                    )));
                }
            }
            self.buf.clear();
        }

        Ok(package)
    }

    fn parse_devicesets(&mut self) -> Result<Vec<RawDeviceset>, EngineError> {
        let mut devicesets = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"deviceset") => {
                    let owned = e.to_owned();
                    devicesets.push(self.parse_deviceset(&owned)?);
                }
                Ok(Event::End(e)) if e.name() == QName(b"devicesets") => break,
                Ok(Event::Eof) => return unexpected_eof("devicesets"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse devicesets: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(devicesets)
    }

    fn parse_deviceset(&mut self, start: &BytesStart<'_>) -> Result<RawDeviceset, EngineError> {
        let mut deviceset = RawDeviceset {
            name: required_attr(start, b"name")?,
            prefix: optional_attr(start, b"prefix").unwrap_or_default(),
            gates: Vec::new(),
            devices: Vec::new(),
        };

        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"gates") => {
                    deviceset.gates = self.parse_gates()?;
                }
                Ok(Event::Start(e)) if e.name() == QName(b"devices") => {
                    deviceset.devices = self.parse_devices()?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"deviceset") => break,
                Ok(Event::Eof) => return unexpected_eof("deviceset"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse deviceset: {err}"
                    )));
                }
            }
            self.buf.clear();
        }

        Ok(deviceset)
    }

    fn parse_gates(&mut self) -> Result<Vec<RawGate>, EngineError> {
        let mut gates = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Empty(e)) if e.name() == QName(b"gate") => {
                    gates.push(RawGate {
                        name: required_attr(&e, b"name")?,
                        symbol: required_attr(&e, b"symbol")?,
                    });
                }
                Ok(Event::Start(e)) if e.name() == QName(b"gate") => {
                    gates.push(RawGate {
                        name: required_attr(&e, b"name")?,
                        symbol: required_attr(&e, b"symbol")?,
                    });
                    self.skip_element(QName(b"gate"))?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"gates") => break,
                Ok(Event::Eof) => return unexpected_eof("gates"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!("failed to parse gates: {err}")));
                }
            }
            self.buf.clear();
        }
        Ok(gates)
    }

    fn parse_devices(&mut self) -> Result<Vec<RawDevice>, EngineError> {
        let mut devices = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"device") => {
                    let owned = e.to_owned();
                    devices.push(self.parse_device(&owned)?);
                }
                Ok(Event::End(e)) if e.name() == QName(b"devices") => break,
                Ok(Event::Eof) => return unexpected_eof("devices"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse devices: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(devices)
    }

    fn parse_device(&mut self, start: &BytesStart<'_>) -> Result<RawDevice, EngineError> {
        let mut device = RawDevice {
            name: optional_attr(start, b"name").unwrap_or_default(),
            package: optional_attr(start, b"package").unwrap_or_default(),
            connects: Vec::new(),
        };

        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"connects") => {
                    device.connects = self.parse_connects()?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"device") => break,
                Ok(Event::Eof) => return unexpected_eof("device"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse device: {err}"
                    )));
                }
            }
            self.buf.clear();
        }

        Ok(device)
    }

    fn parse_connects(&mut self) -> Result<Vec<RawConnect>, EngineError> {
        let mut connects = Vec::new();
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Empty(e)) if e.name() == QName(b"connect") => {
                    connects.push(RawConnect {
                        gate: required_attr(&e, b"gate")?,
                        pin: required_attr(&e, b"pin")?,
                        pad: required_attr(&e, b"pad")?,
                    });
                }
                Ok(Event::Start(e)) if e.name() == QName(b"connect") => {
                    connects.push(RawConnect {
                        gate: required_attr(&e, b"gate")?,
                        pin: required_attr(&e, b"pin")?,
                        pad: required_attr(&e, b"pad")?,
                    });
                    self.skip_element(QName(b"connect"))?;
                }
                Ok(Event::End(e)) if e.name() == QName(b"connects") => break,
                Ok(Event::Eof) => return unexpected_eof("connects"),
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed to parse connects: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(connects)
    }

    fn skip_element(&mut self, end_name: QName<'_>) -> Result<(), EngineError> {
        let mut depth = 1usize;
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) if e.name() == end_name => depth += 1,
                Ok(Event::End(e)) if e.name() == end_name => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Ok(Event::Eof) => {
                    return Err(EngineError::Import(format!(
                        "unexpected EOF while skipping <{}>",
                        qname_string(end_name)
                    )));
                }
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Import(format!(
                        "failed while skipping XML element: {err}"
                    )));
                }
            }
            self.buf.clear();
        }
        Ok(())
    }
}

fn build_pool(
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

fn pool_counts(pool: &Pool) -> ImportObjectCounts {
    ImportObjectCounts {
        units: pool.units.len(),
        symbols: pool.symbols.len(),
        entities: pool.entities.len(),
        padstacks: pool.padstacks.len(),
        packages: pool.packages.len(),
        parts: pool.parts.len(),
    }
}

fn extract_library_name(xml: &str) -> Result<String, EngineError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name() == QName(b"library") => {
                return required_attr(&e, b"name");
            }
            Ok(Event::Eof) => {
                return Err(EngineError::Import(
                    "Eagle library missing <library name=...>".to_string(),
                ));
            }
            Ok(_) => {}
            Err(err) => {
                return Err(EngineError::Import(format!(
                    "failed to parse Eagle XML header: {err}"
                )));
            }
        }
        buf.clear();
    }
}

fn parse_pin(start: &BytesStart<'_>) -> Result<RawPin, EngineError> {
    Ok(RawPin {
        name: required_attr(start, b"name")?,
        direction: parse_pin_direction(optional_attr(start, b"direction").as_deref()),
    })
}

fn parse_package_pad(start: &BytesStart<'_>, smd: bool) -> Result<RawPad, EngineError> {
    let name = required_attr(start, b"name")?;
    let x = parse_eagle_coord(required_attr(start, b"x")?.as_str())?;
    let y = parse_eagle_coord(required_attr(start, b"y")?.as_str())?;
    let layer = if smd {
        parse_i32_attr(start, b"layer")?.unwrap_or(1)
    } else {
        1
    };

    Ok(RawPad {
        name: name.clone(),
        position: Point::new(x, y),
        layer,
        padstack_name: if smd {
            format!("smd:{name}")
        } else {
            format!("th:{name}")
        },
    })
}

fn parse_silkscreen_wire(start: &BytesStart<'_>) -> Result<Option<Primitive>, EngineError> {
    let layer = parse_i32_attr(start, b"layer")?.unwrap_or_default();
    if layer != 21 && layer != 22 {
        return Ok(None);
    }

    let x1 = parse_eagle_coord(required_attr(start, b"x1")?.as_str())?;
    let y1 = parse_eagle_coord(required_attr(start, b"y1")?.as_str())?;
    let x2 = parse_eagle_coord(required_attr(start, b"x2")?.as_str())?;
    let y2 = parse_eagle_coord(required_attr(start, b"y2")?.as_str())?;
    let width = optional_attr(start, b"width")
        .map(|value| parse_eagle_coord(&value))
        .transpose()?
        .unwrap_or(0);

    Ok(Some(Primitive::Line {
        from: Point::new(x1, y1),
        to: Point::new(x2, y2),
        width,
    }))
}

fn parse_pin_direction(direction: Option<&str>) -> PinDirection {
    match direction.unwrap_or("pas") {
        "in" => PinDirection::Input,
        "out" => PinDirection::Output,
        "io" | "bid" => PinDirection::Bidirectional,
        "pas" => PinDirection::Passive,
        "pwr" => PinDirection::PowerIn,
        "sup" => PinDirection::PowerOut,
        "oc" => PinDirection::OpenCollector,
        "oe" => PinDirection::OpenEmitter,
        "hiz" => PinDirection::TriState,
        "nc" => PinDirection::NoConnect,
        _ => PinDirection::Passive,
    }
}

fn parse_eagle_coord(value: &str) -> Result<i64, EngineError> {
    let parsed = value.parse::<f64>().map_err(|_| {
        EngineError::Import(format!("invalid Eagle coordinate or width value: {value}"))
    })?;
    Ok((parsed * 25_400_000.0).round() as i64)
}

fn parse_i32_attr(start: &BytesStart<'_>, key: &[u8]) -> Result<Option<i32>, EngineError> {
    optional_attr(start, key)
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|_| EngineError::Import(format!("invalid integer attribute: {value}")))
        })
        .transpose()
}

fn required_attr(start: &BytesStart<'_>, key: &[u8]) -> Result<String, EngineError> {
    optional_attr(start, key).ok_or_else(|| {
        EngineError::Import(format!(
            "missing required attribute {} on <{}>",
            String::from_utf8_lossy(key),
            qname_string(start.name())
        ))
    })
}

fn optional_attr(start: &BytesStart<'_>, key: &[u8]) -> Option<String> {
    start
        .attributes()
        .flatten()
        .find(|attr| attr.key == QName(key))
        .map(|attr| String::from_utf8_lossy(attr.value.as_ref()).into_owned())
}

fn qname_string(name: QName<'_>) -> String {
    String::from_utf8_lossy(name.as_ref()).into_owned()
}

fn unexpected_eof<T>(context: &str) -> Result<T, EngineError> {
    Err(EngineError::Import(format!(
        "unexpected EOF while parsing Eagle {context}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    use crate::api::Engine;
    use crate::ir::serialization::to_json_deterministic;

    const SIMPLE_EAGLE_LIBRARY: &str =
        include_str!("../../../testdata/import/eagle/simple-opamp.lbr");
    const DUAL_GATE_EAGLE_LIBRARY: &str =
        include_str!("../../../testdata/import/eagle/dual-nand.lbr");

    #[test]
    fn imports_symbol_package_entity_and_part() {
        let pool = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");

        assert_eq!(pool.units.len(), 1);
        assert_eq!(pool.symbols.len(), 1);
        assert_eq!(pool.entities.len(), 1);
        assert_eq!(pool.packages.len(), 1);
        assert_eq!(pool.parts.len(), 1);
        assert_eq!(pool.padstacks.len(), 3);

        let unit = pool.units.values().next().unwrap();
        assert_eq!(unit.name, "OPAMP");
        assert_eq!(unit.pins.len(), 3);

        let entity = pool.entities.values().next().unwrap();
        assert_eq!(entity.name, "LMV321");
        assert_eq!(entity.prefix, "U");
        assert_eq!(entity.gates.len(), 1);

        let package = pool.packages.values().next().unwrap();
        assert_eq!(package.name, "SOT23-5");
        assert_eq!(package.pads.len(), 3);
        assert_eq!(package.silkscreen.len(), 1);

        let part = pool.parts.values().next().unwrap();
        assert_eq!(part.entity, entity.uuid);
        assert_eq!(part.package, package.uuid);
        assert_eq!(part.pad_map.len(), 3);
    }

    #[test]
    fn import_is_deterministic_for_same_library() {
        let a = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");
        let b = import_library_str(SIMPLE_EAGLE_LIBRARY).expect("fixture should import");

        assert_eq!(a, b);
    }

    #[test]
    fn rejects_unknown_connect_symbol_binding() {
        let broken = SIMPLE_EAGLE_LIBRARY.replace("pad=\"3\"", "pad=\"99\"");
        let err = import_library_str(&broken).expect_err("broken fixture must fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown pad 99"), "unexpected error: {msg}");
    }

    #[test]
    fn imports_multi_gate_device_and_through_hole_pad() {
        let pool = import_library_str(DUAL_GATE_EAGLE_LIBRARY).expect("fixture should import");

        assert_eq!(pool.units.len(), 1);
        assert_eq!(pool.symbols.len(), 1);
        assert_eq!(pool.entities.len(), 1);
        assert_eq!(pool.packages.len(), 1);
        assert_eq!(pool.parts.len(), 1);

        let entity = pool.entities.values().next().unwrap();
        assert_eq!(entity.gates.len(), 2);

        let package = pool.packages.values().next().unwrap();
        assert_eq!(package.pads.len(), 4);
        assert!(package.pads.values().any(|pad| {
            package
                .pads
                .values()
                .find(|p| p.uuid == pad.uuid)
                .map(|_| true)
                .unwrap_or(false)
        }));

        let through_hole_count = package
            .pads
            .values()
            .filter(|pad| {
                pool.padstacks
                    .get(&pad.padstack)
                    .map(|stack| stack.name.starts_with("th:"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(through_hole_count, 4);

        let part = pool.parts.values().next().unwrap();
        assert_eq!(part.pad_map.len(), 4);
    }

    #[test]
    fn engine_import_eagle_library_returns_report_and_indexes_parts() {
        let mut engine = Engine::new().expect("engine should initialize");
        let path = fixture_path("simple-opamp.lbr");

        let report = engine
            .import_eagle_library(&path)
            .expect("fixture should import through engine facade");

        assert_eq!(report.kind, ImportKind::EagleLibrary);
        assert_eq!(report.counts.units, 1);
        assert_eq!(report.counts.parts, 1);
        assert!(report.warnings.is_empty());
        assert_eq!(
            report.metadata.get("library_name").map(String::as_str),
            Some("demo-analog")
        );

        let search = engine
            .search_pool("SOT23")
            .expect("pool search should work");
        assert_eq!(search.len(), 1);
    }

    #[test]
    fn eagle_fixture_corpus_imports_and_is_deterministic() {
        let fixtures = corpus_fixture_paths();
        assert!(
            fixtures.len() >= 20,
            "expected at least 20 Eagle library fixtures, found {}",
            fixtures.len()
        );

        for path in fixtures {
            let xml = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            let a = import_library_str(&xml)
                .unwrap_or_else(|err| panic!("failed to import {}: {err}", path.display()));
            let b = import_library_str(&xml)
                .unwrap_or_else(|err| panic!("failed to re-import {}: {err}", path.display()));

            assert_eq!(
                a,
                b,
                "import must be deterministic for fixture {}",
                path.display()
            );
        }
    }

    #[test]
    fn eagle_fixture_corpus_canonicalizes_deterministically() {
        let fixtures = corpus_fixture_paths();
        assert!(
            fixtures.len() >= 20,
            "expected at least 20 Eagle library fixtures, found {}",
            fixtures.len()
        );

        for path in fixtures {
            let xml = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            let pool = import_library_str(&xml)
                .unwrap_or_else(|err| panic!("failed to import {}: {err}", path.display()));
            let json_a = serde_json::to_string(&pool)
                .unwrap_or_else(|err| panic!("failed to serialize {}: {err}", path.display()));
            let json_b = serde_json::to_string(&pool)
                .unwrap_or_else(|err| panic!("failed to reserialize {}: {err}", path.display()));
            assert_eq!(
                json_a,
                json_b,
                "canonical serialization must be stable for {}",
                path.display()
            );
        }
    }

    #[test]
    fn eagle_golden_subset_matches_checked_in_canonical_json() {
        let subset = ["simple-opamp.lbr", "dual-nand.lbr", "regulator-sot223.lbr"];

        for fixture in subset {
            let fixture_path = fixture_path(fixture);
            let xml = fs::read_to_string(&fixture_path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
            let pool = import_library_str(&xml)
                .unwrap_or_else(|err| panic!("failed to import {}: {err}", fixture_path.display()));
            let canonical = to_json_deterministic(&pool).unwrap_or_else(|err| {
                panic!("failed to serialize {}: {err}", fixture_path.display())
            });
            let golden_path = golden_path_for_fixture(fixture);

            if std::env::var_os("UPDATE_GOLDENS").is_some() {
                if let Some(parent) = golden_path.parent() {
                    fs::create_dir_all(parent).unwrap_or_else(|err| {
                        panic!("failed to create golden dir {}: {err}", parent.display())
                    });
                }
                fs::write(&golden_path, &canonical).unwrap_or_else(|err| {
                    panic!("failed to write golden {}: {err}", golden_path.display())
                });
                continue;
            }

            let expected = fs::read_to_string(&golden_path).unwrap_or_else(|err| {
                panic!(
                    "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
                    golden_path.display()
                )
            });
            assert_eq!(
                canonical,
                expected,
                "golden mismatch for fixture {}",
                fixture_path.display()
            );
        }
    }

    fn fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("testdata/import/eagle");
        path.push(name);
        path
    }

    fn corpus_fixture_paths() -> Vec<PathBuf> {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("testdata/import/eagle");

        let mut paths: Vec<_> = std::fs::read_dir(&root)
            .expect("fixture directory should exist")
            .map(|entry| entry.expect("fixture entry should read").path())
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("lbr"))
            .collect();
        paths.sort();
        paths
    }

    fn golden_path_for_fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/golden/eagle")
            .join(format!("{name}.json"))
    }
}
