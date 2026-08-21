//! Bounded introspection descriptions for Datum's two AT-SPI objects.

use super::{HYPERLINK, ROOT_PATH};
use crate::terminal_accessibility_platform::body::BodyWriter;

pub(super) fn introspection_body(path: &str) -> Vec<u8> {
    let interfaces = if path == ROOT_PATH {
        &["org.a11y.atspi.Accessible", "org.a11y.atspi.Application"][..]
    } else if path.contains("/link/") {
        &["org.a11y.atspi.Accessible", HYPERLINK][..]
    } else {
        &[
            "org.a11y.atspi.Accessible",
            "org.a11y.atspi.Component",
            "org.a11y.atspi.Text",
            "org.a11y.atspi.Hypertext",
        ][..]
    };
    let mut xml = String::from(
        "<node><interface name=\"org.freedesktop.DBus.Properties\"/>\
         <interface name=\"org.freedesktop.DBus.Introspectable\"/>",
    );
    for interface in interfaces {
        xml.push_str("<interface name=\"");
        xml.push_str(interface);
        xml.push_str("\"/>");
    }
    xml.push_str("</node>");
    let mut body = BodyWriter::new();
    body.string(&xml);
    body.finish()
}
