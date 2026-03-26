use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;

use crate::error::EngineError;
use crate::import::eagle::pool_builder::build_pool;
use crate::import::eagle::xml_helpers::{
    optional_attr, parse_package_pad, parse_pin, parse_silkscreen_wire, qname_string,
    required_attr, unexpected_eof,
};
use crate::ir::geometry::{LayerId, Point};
use crate::pool::{PinDirection, Pool, Primitive};

pub(super) fn parse_library(xml: &str) -> Result<Pool, EngineError> {
    let mut parser = EagleLibraryParser::new(xml);
    parser.parse()
}

#[derive(Debug, Default)]
pub(super) struct RawSymbol {
    pub(super) name: String,
    pub(super) pins: Vec<RawPin>,
}

#[derive(Debug)]
pub(super) struct RawPin {
    pub(super) name: String,
    pub(super) direction: PinDirection,
}

#[derive(Debug, Default)]
pub(super) struct RawPackage {
    pub(super) name: String,
    pub(super) pads: Vec<RawPad>,
    pub(super) silkscreen: Vec<Primitive>,
}

#[derive(Debug)]
pub(super) struct RawPad {
    pub(super) name: String,
    pub(super) position: Point,
    pub(super) layer: LayerId,
    pub(super) padstack_name: String,
}

#[derive(Debug, Default)]
pub(super) struct RawDeviceset {
    pub(super) name: String,
    pub(super) prefix: String,
    pub(super) gates: Vec<RawGate>,
    pub(super) devices: Vec<RawDevice>,
}

#[derive(Debug)]
pub(super) struct RawGate {
    pub(super) name: String,
    pub(super) symbol: String,
}

#[derive(Debug, Default)]
pub(super) struct RawDevice {
    pub(super) name: String,
    pub(super) package: String,
    pub(super) connects: Vec<RawConnect>,
}

#[derive(Debug)]
pub(super) struct RawConnect {
    pub(super) gate: String,
    pub(super) pin: String,
    pub(super) pad: String,
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
