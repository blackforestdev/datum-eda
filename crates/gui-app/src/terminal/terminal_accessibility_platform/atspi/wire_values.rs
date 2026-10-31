//! D-Bus wire-value serialization for AT-SPI method replies.

use crate::terminal_accessibility::TerminalAccessibilityBounds;
use datum_gui_protocol::GlobalPreferencesAccessibleNode;

use super::super::body::BodyWriter;

pub(super) fn object_array_body(bus_name: &str, paths: &[String]) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(8, |body| {
        for path in paths {
            body.structure(|body| {
                body.string(bus_name);
                body.object_path(path);
            });
        }
    });
    body.finish()
}

pub(super) fn preference_attributes_body(node: &GlobalPreferencesAccessibleNode) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(8, |body| {
        for (key, value) in [
            ("value", node.value.as_deref().unwrap_or("")),
            ("available", if node.available { "true" } else { "false" }),
            ("description", node.description.as_str()),
        ] {
            body.structure(|body| {
                body.string(key);
                body.string(value);
            });
        }
    });
    body.finish()
}

pub(super) fn string_array_body(values: &[&str]) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(4, |body| {
        for value in values {
            body.string(value);
        }
    });
    body.finish()
}

pub(super) fn empty_array_body(alignment: usize) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(alignment, |_| {});
    body.finish()
}
pub(super) fn string_body(value: &str) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.string(value);
    body.finish()
}
pub(super) fn bool_body(value: bool) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.bool(value);
    body.finish()
}
pub(super) fn i16_body(value: i16) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i16(value);
    body.finish()
}
pub(super) fn i32_body(value: i32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(value);
    body.finish()
}
pub(super) fn u32_body(value: u32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.u32(value);
    body.finish()
}
pub(super) fn f64_body(value: f64) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.f64(value);
    body.finish()
}
pub(super) fn two_i32_body(a: i32, b: i32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(a);
    body.i32(b);
    body.finish()
}
pub(super) fn rect_values_body(value: TerminalAccessibilityBounds) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(value.x);
    body.i32(value.y);
    body.i32(value.width);
    body.i32(value.height);
    body.finish()
}
pub(super) fn rect_body(value: TerminalAccessibilityBounds) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.structure(|body| {
        body.i32(value.x);
        body.i32(value.y);
        body.i32(value.width);
        body.i32(value.height);
    });
    body.finish()
}
