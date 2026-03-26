use quick_xml::Reader;
use quick_xml::events::BytesStart;
use quick_xml::name::QName;

use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::pool::{PinDirection, Primitive};

use super::parser::{RawPad, RawPin};

pub(super) fn extract_library_name(xml: &str) -> Result<String, EngineError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) if e.name() == QName(b"library") => {
                return required_attr(&e, b"name");
            }
            Ok(quick_xml::events::Event::Eof) => {
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

pub(super) fn parse_pin(start: &BytesStart<'_>) -> Result<RawPin, EngineError> {
    Ok(RawPin {
        name: required_attr(start, b"name")?,
        direction: parse_pin_direction(optional_attr(start, b"direction").as_deref()),
    })
}

pub(super) fn parse_package_pad(start: &BytesStart<'_>, smd: bool) -> Result<RawPad, EngineError> {
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

pub(super) fn parse_silkscreen_wire(start: &BytesStart<'_>) -> Result<Option<Primitive>, EngineError> {
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

pub(super) fn required_attr(start: &BytesStart<'_>, key: &[u8]) -> Result<String, EngineError> {
    optional_attr(start, key).ok_or_else(|| {
        EngineError::Import(format!(
            "missing required attribute {} on <{}>",
            String::from_utf8_lossy(key),
            qname_string(start.name())
        ))
    })
}

pub(super) fn optional_attr(start: &BytesStart<'_>, key: &[u8]) -> Option<String> {
    start
        .attributes()
        .flatten()
        .find(|attr| attr.key == QName(key))
        .map(|attr| String::from_utf8_lossy(attr.value.as_ref()).into_owned())
}

pub(super) fn qname_string(name: QName<'_>) -> String {
    String::from_utf8_lossy(name.as_ref()).into_owned()
}

pub(super) fn unexpected_eof<T>(context: &str) -> Result<T, EngineError> {
    Err(EngineError::Import(format!(
        "unexpected EOF while parsing Eagle {context}"
    )))
}
