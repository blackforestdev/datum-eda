//! Datum-owned AT-SPI object model and method dispatcher.

use std::env;

use crate::terminal_accessibility::{TerminalAccessibilityBounds, TerminalAccessibilitySnapshot};

use super::body::BodyWriter;
use super::connection::object_reference_body;
use super::dbus::{Message, MessageType};

mod dispatch_error;
mod introspection;
mod text_ranges;
use dispatch_error::{error, invalid_args, unknown_object};
use introspection::introspection_body;
use text_ranges::{char_range, text_at_offset};

pub(super) const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";
pub(super) const TERMINAL_PATH: &str = "/org/a11y/atspi/accessible/terminal";
pub(super) const NULL_PATH: &str = "/org/a11y/atspi/null";
pub(super) const REGISTRY_NAME: &str = "org.a11y.atspi.Registry";
pub(super) const REGISTRY_PATH: &str = "/org/a11y/atspi/accessible/root";

const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const APPLICATION: &str = "org.a11y.atspi.Application";
const COMPONENT: &str = "org.a11y.atspi.Component";
const TEXT: &str = "org.a11y.atspi.Text";
const HYPERTEXT: &str = "org.a11y.atspi.Hypertext";
const HYPERLINK: &str = "org.a11y.atspi.Hyperlink";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const INTROSPECTABLE: &str = "org.freedesktop.DBus.Introspectable";

const ROLE_TERMINAL: u32 = 60;
const ROLE_APPLICATION: u32 = 75;
const ROLE_LINK: u32 = 88;
const STATE_EDITABLE: u32 = 7;
const STATE_ENABLED: u32 = 8;
const STATE_FOCUSABLE: u32 = 11;
const STATE_FOCUSED: u32 = 12;
const STATE_MULTI_LINE: u32 = 17;
const STATE_SENSITIVE: u32 = 24;
const STATE_SHOWING: u32 = 25;
const STATE_VISIBLE: u32 = 30;
const STATE_SELECTABLE_TEXT: u32 = 38;

#[derive(Clone)]
pub(super) struct ServiceState {
    pub(super) snapshot: TerminalAccessibilitySnapshot,
    pub(super) application_id: i32,
    pub(super) registry_parent: (String, String),
    pub(super) bus_name: String,
}

impl ServiceState {
    pub(super) fn new(snapshot: TerminalAccessibilitySnapshot) -> Self {
        Self {
            snapshot,
            application_id: 0,
            registry_parent: (String::new(), NULL_PATH.into()),
            bus_name: String::new(),
        }
    }

    pub(super) fn set_bus_name(&mut self, bus_name: String) {
        self.bus_name = bus_name;
    }

    pub(super) fn dispatch(&mut self, serial: u32, call: &Message) -> Message {
        if call.kind != MessageType::MethodCall {
            return error(
                serial,
                call,
                "org.freedesktop.DBus.Error.InvalidArgs",
                "method call required",
            );
        }
        let path = call.header.path.as_deref().unwrap_or_default();
        let interface = call.header.interface.as_deref().unwrap_or_default();
        let member = call.header.member.as_deref().unwrap_or_default();
        let result = match interface {
            PROPERTIES => self.properties(path, member, call),
            INTROSPECTABLE if member == "Introspect" => Ok(("s", introspection_body(path))),
            ACCESSIBLE => self.accessible(path, member, call),
            APPLICATION => self.application(path, member, call),
            COMPONENT => self.component(path, member, call),
            TEXT => self.text(path, member, call),
            HYPERTEXT => self.hypertext(path, member, call),
            HYPERLINK => self.hyperlink(path, member, call),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownInterface",
                "unsupported interface",
            )),
        };
        match result {
            Ok((signature, body)) => Message::method_return(
                serial,
                call.serial,
                call.header.sender.as_deref(),
                signature,
                body,
            ),
            Err((name, description)) => error(serial, call, name, description),
        }
    }

    fn properties(&mut self, path: &str, member: &str, call: &Message) -> DispatchResult {
        let mut body = call.body_reader();
        let interface = body.string().map_err(|_| invalid_args())?;
        match member {
            "Get" => {
                let property = body.string().map_err(|_| invalid_args())?;
                property_value(self, path, &interface, &property)
                    .map(|(signature, body)| ("v", variant_body(signature, body)))
            }
            "GetAll" => Ok(("a{sv}", properties_body(self, path, &interface)?)),
            "Set" if path == ROOT_PATH && interface == APPLICATION => {
                let property = body.string().map_err(|_| invalid_args())?;
                if property != "Id" {
                    return Err((
                        "org.freedesktop.DBus.Error.PropertyReadOnly",
                        "property is read-only",
                    ));
                }
                self.application_id = body.variant_i32().map_err(|_| invalid_args())?;
                Ok(("", Vec::new()))
            }
            "Set" => Err((
                "org.freedesktop.DBus.Error.PropertyReadOnly",
                "property is read-only",
            )),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported properties method",
            )),
        }
    }

    fn accessible(&self, path: &str, member: &str, call: &Message) -> DispatchResult {
        let terminal = path == TERMINAL_PATH;
        let link = link_index(path).and_then(|index| self.snapshot.links.get(index));
        if !terminal && path != ROOT_PATH && link.is_none() {
            return Err(unknown_object());
        }
        match member {
            "GetChildAtIndex" if path == ROOT_PATH => {
                let mut reader = call.body_reader();
                if reader.i32().map_err(|_| invalid_args())? != 0 {
                    return Err((
                        "org.freedesktop.DBus.Error.InvalidArgs",
                        "child index out of range",
                    ));
                }
                Ok(("(so)", object_reference_body(&self.bus_name, TERMINAL_PATH)))
            }
            "GetChildren" => Ok((
                "a(so)",
                object_array_body(
                    (path == ROOT_PATH).then_some((self.bus_name.as_str(), TERMINAL_PATH)),
                ),
            )),
            "GetIndexInParent" => Ok(("i", i32_body(if terminal { 0 } else { -1 }))),
            "GetRelationSet" => Ok(("a(ua(so))", empty_array_body(8))),
            "GetRole" => Ok((
                "u",
                u32_body(if terminal {
                    ROLE_TERMINAL
                } else if link.is_some() {
                    ROLE_LINK
                } else {
                    ROLE_APPLICATION
                }),
            )),
            "GetRoleName" | "GetLocalizedRoleName" => Ok((
                "s",
                string_body(if terminal {
                    "terminal"
                } else if link.is_some() {
                    "link"
                } else {
                    "application"
                }),
            )),
            "GetState" => Ok((
                "au",
                state_body(terminal, terminal && self.snapshot.focused),
            )),
            "GetApplication" => Ok(("(so)", object_reference_body(&self.bus_name, ROOT_PATH))),
            "GetAttributes" => Ok(("a{ss}", empty_array_body(8))),
            "GetInterfaces" => Ok((
                "as",
                string_array_body(if terminal {
                    &[ACCESSIBLE, COMPONENT, TEXT, HYPERTEXT]
                } else if link.is_some() {
                    &[ACCESSIBLE, HYPERLINK]
                } else {
                    &[ACCESSIBLE, APPLICATION]
                }),
            )),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported accessible method",
            )),
        }
    }

    fn application(&self, path: &str, member: &str, _call: &Message) -> DispatchResult {
        if path != ROOT_PATH {
            return Err(unknown_object());
        }
        match member {
            "GetLocale" => Ok(("s", string_body(&locale()))),
            "GetApplicationBusAddress" => Ok(("s", string_body(""))),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported application method",
            )),
        }
    }

    fn component(&self, path: &str, member: &str, call: &Message) -> DispatchResult {
        if path != TERMINAL_PATH {
            return Err(unknown_object());
        }
        let bounds = self.snapshot.bounds;
        match member {
            "Contains" => {
                let mut reader = call.body_reader();
                let x = reader.i32().map_err(|_| invalid_args())?;
                let y = reader.i32().map_err(|_| invalid_args())?;
                Ok(("b", bool_body(contains(bounds, x, y))))
            }
            "GetAccessibleAtPoint" => {
                Ok(("(so)", object_reference_body(&self.bus_name, TERMINAL_PATH)))
            }
            "GetExtents" => Ok(("(iiii)", rect_body(bounds))),
            "GetPosition" => Ok(("ii", two_i32_body(bounds.x, bounds.y))),
            "GetSize" => Ok(("ii", two_i32_body(bounds.width, bounds.height))),
            "GetLayer" => Ok(("u", u32_body(2))),
            "GetMDIZOrder" => Ok(("n", i16_body(0))),
            "GrabFocus" => Ok(("b", bool_body(false))),
            "GetAlpha" => Ok(("d", f64_body(1.0))),
            "SetExtents" | "SetPosition" | "SetSize" | "ScrollTo" | "ScrollToPoint" => {
                Ok(("b", bool_body(false)))
            }
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported component method",
            )),
        }
    }

    fn text(&self, path: &str, member: &str, call: &Message) -> DispatchResult {
        if path != TERMINAL_PATH {
            return Err(unknown_object());
        }
        let chars = self.snapshot.text.chars().collect::<Vec<_>>();
        match member {
            "GetText" => {
                let mut reader = call.body_reader();
                let start = reader.i32().map_err(|_| invalid_args())?;
                let end = reader.i32().map_err(|_| invalid_args())?;
                Ok(("s", string_body(&char_range(&chars, start, end)?)))
            }
            "GetStringAtOffset" => {
                let mut reader = call.body_reader();
                let offset = reader.i32().map_err(|_| invalid_args())?;
                let granularity = reader.u32().map_err(|_| invalid_args())?;
                let (text, start, end) = text_at_offset(&chars, offset, granularity)?;
                let mut body = BodyWriter::new();
                body.string(&text);
                body.i32(start);
                body.i32(end);
                Ok(("sii", body.finish()))
            }
            "GetCharacterAtOffset" => {
                let mut reader = call.body_reader();
                let offset = usize::try_from(reader.i32().map_err(|_| invalid_args())?)
                    .map_err(|_| invalid_args())?;
                let value = chars.get(offset).copied().map_or(-1, |ch| ch as i32);
                Ok(("i", i32_body(value)))
            }
            "GetNSelections" => Ok(("i", i32_body(i32::from(self.snapshot.selection.is_some())))),
            "GetSelection" => {
                let mut reader = call.body_reader();
                if reader.i32().map_err(|_| invalid_args())? != 0 {
                    return Err(invalid_args());
                }
                let Some((start, end)) = self.snapshot.selection else {
                    return Err(invalid_args());
                };
                Ok(("ii", two_i32_body(clamp_i32(start), clamp_i32(end))))
            }
            "GetAttributes" | "GetAttributeRun" => {
                let mut body = BodyWriter::new();
                body.array(8, |_| {});
                body.i32(0);
                body.i32(clamp_i32(chars.len()));
                if member == "GetAttributeRun" {
                    body.bool(false);
                }
                Ok((
                    if member == "GetAttributes" {
                        "a{ss}ii"
                    } else {
                        "a{ss}iib"
                    },
                    body.finish(),
                ))
            }
            "GetDefaultAttributes" => Ok(("a{ss}", empty_array_body(8))),
            "GetCharacterExtents" | "GetRangeExtents" => {
                Ok(("iiii", rect_values_body(self.snapshot.bounds)))
            }
            "GetOffsetAtPoint" => Ok(("i", i32_body(-1))),
            "SetCaretOffset"
            | "AddSelection"
            | "SetSelection"
            | "RemoveSelection"
            | "ScrollSubstringTo"
            | "ScrollSubstringToPoint" => Ok(("b", bool_body(false))),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported text method",
            )),
        }
    }

    fn hypertext(&self, path: &str, member: &str, call: &Message) -> DispatchResult {
        if path != TERMINAL_PATH {
            return Err(unknown_object());
        }
        match member {
            "GetNLinks" => Ok(("i", i32_body(clamp_i32(self.snapshot.links.len())))),
            "GetLink" => {
                let mut reader = call.body_reader();
                let index = usize::try_from(reader.i32().map_err(|_| invalid_args())?)
                    .map_err(|_| invalid_args())?;
                if index >= self.snapshot.links.len() {
                    return Err(invalid_args());
                }
                Ok((
                    "(so)",
                    object_reference_body(&self.bus_name, &link_path(index)),
                ))
            }
            "GetLinkIndex" => {
                let mut reader = call.body_reader();
                let offset = usize::try_from(reader.i32().map_err(|_| invalid_args())?)
                    .map_err(|_| invalid_args())?;
                let index = self
                    .snapshot
                    .links
                    .iter()
                    .position(|link| link.start <= offset && offset < link.end)
                    .map_or(-1, clamp_i32);
                Ok(("i", i32_body(index)))
            }
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported hypertext method",
            )),
        }
    }

    fn hyperlink(&self, path: &str, member: &str, call: &Message) -> DispatchResult {
        let index = link_index(path).ok_or_else(unknown_object)?;
        let link = self.snapshot.links.get(index).ok_or_else(unknown_object)?;
        match member {
            "GetURI" | "GetObject" => {
                let mut reader = call.body_reader();
                if reader.i32().map_err(|_| invalid_args())? != 0 {
                    return Err(invalid_args());
                }
                if member == "GetURI" {
                    Ok(("s", string_body(&link.uri)))
                } else {
                    Ok(("(so)", object_reference_body(&self.bus_name, path)))
                }
            }
            "IsValid" => Ok(("b", bool_body(true))),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported hyperlink method",
            )),
        }
    }
}

type DispatchResult = Result<(&'static str, Vec<u8>), (&'static str, &'static str)>;

fn property_value(
    state: &ServiceState,
    path: &str,
    interface: &str,
    property: &str,
) -> DispatchResult {
    let terminal = path == TERMINAL_PATH;
    match (path, interface, property) {
        (ROOT_PATH, APPLICATION, "ToolkitName") => Ok(("s", string_body("Datum EDA"))),
        (ROOT_PATH, APPLICATION, "Version" | "ToolkitVersion") => {
            Ok(("s", string_body(env!("CARGO_PKG_VERSION"))))
        }
        (ROOT_PATH, APPLICATION, "AtspiVersion") => Ok(("s", string_body("2.1"))),
        (ROOT_PATH, APPLICATION, "InterfaceVersion") => Ok(("u", u32_body(1))),
        (ROOT_PATH, APPLICATION, "Id") => Ok(("i", i32_body(state.application_id))),
        (_, ACCESSIBLE, "Name") if terminal || path == ROOT_PATH => Ok((
            "s",
            string_body(if terminal {
                &state.snapshot.title
            } else {
                "Datum EDA"
            }),
        )),
        (_, ACCESSIBLE, "Description") if terminal || path == ROOT_PATH => Ok((
            "s",
            string_body(if terminal {
                "Native terminal session"
            } else {
                "Datum EDA application"
            }),
        )),
        (ROOT_PATH, ACCESSIBLE, "Parent") => Ok((
            "(so)",
            object_reference_body(&state.registry_parent.0, &state.registry_parent.1),
        )),
        (TERMINAL_PATH, ACCESSIBLE, "Parent") => {
            Ok(("(so)", object_reference_body(&state.bus_name, ROOT_PATH)))
        }
        (ROOT_PATH, ACCESSIBLE, "ChildCount") => Ok(("i", i32_body(1))),
        (TERMINAL_PATH, ACCESSIBLE, "ChildCount") => Ok(("i", i32_body(0))),
        (_, ACCESSIBLE, "Locale") if terminal || path == ROOT_PATH => {
            Ok(("s", string_body(&locale())))
        }
        (ROOT_PATH, ACCESSIBLE, "AccessibleId") => Ok(("s", string_body("datum-eda"))),
        (TERMINAL_PATH, ACCESSIBLE, "AccessibleId") => {
            Ok(("s", string_body(&state.snapshot.session_id)))
        }
        (_, ACCESSIBLE, "Name") if link_index(path).is_some() => {
            let index = link_index(path).ok_or_else(unknown_object)?;
            let link = state.snapshot.links.get(index).ok_or_else(unknown_object)?;
            Ok(("s", string_body(&link.uri)))
        }
        (_, ACCESSIBLE, "Description" | "HelpText") if link_index(path).is_some() => {
            Ok(("s", string_body("Terminal hyperlink")))
        }
        (_, ACCESSIBLE, "Parent") if link_index(path).is_some() => Ok((
            "(so)",
            object_reference_body(&state.bus_name, TERMINAL_PATH),
        )),
        (_, ACCESSIBLE, "ChildCount") if link_index(path).is_some() => Ok(("i", i32_body(0))),
        (_, ACCESSIBLE, "Locale") if link_index(path).is_some() => {
            Ok(("s", string_body(&locale())))
        }
        (_, ACCESSIBLE, "AccessibleId") if link_index(path).is_some() => {
            Ok(("s", string_body(path)))
        }
        (_, ACCESSIBLE, "HelpText") if terminal || path == ROOT_PATH => Ok(("s", string_body(""))),
        (TERMINAL_PATH, TEXT, "CharacterCount") => Ok((
            "i",
            i32_body(clamp_i32(state.snapshot.text.chars().count())),
        )),
        (TERMINAL_PATH, TEXT, "CaretOffset") => {
            Ok(("i", i32_body(clamp_i32(state.snapshot.caret))))
        }
        (_, HYPERLINK, "NAnchors") if link_index(path).is_some() => Ok(("n", i16_body(1))),
        (_, HYPERLINK, "StartIndex") if link_index(path).is_some() => {
            let link = &state.snapshot.links[link_index(path).ok_or_else(unknown_object)?];
            Ok(("i", i32_body(clamp_i32(link.start))))
        }
        (_, HYPERLINK, "EndIndex") if link_index(path).is_some() => {
            let link = &state.snapshot.links[link_index(path).ok_or_else(unknown_object)?];
            Ok(("i", i32_body(clamp_i32(link.end))))
        }
        (_, ACCESSIBLE | COMPONENT | TEXT | HYPERTEXT | HYPERLINK, "version") => {
            Ok(("u", u32_body(1)))
        }
        _ => Err((
            "org.freedesktop.DBus.Error.UnknownProperty",
            "unsupported property",
        )),
    }
}

fn properties_body(
    state: &ServiceState,
    path: &str,
    interface: &str,
) -> Result<Vec<u8>, (&'static str, &'static str)> {
    let names: &[&str] = match (path, interface) {
        (ROOT_PATH, APPLICATION) => &[
            "ToolkitName",
            "Version",
            "ToolkitVersion",
            "AtspiVersion",
            "InterfaceVersion",
            "Id",
        ],
        (ROOT_PATH | TERMINAL_PATH, ACCESSIBLE) => &[
            "Name",
            "Description",
            "Parent",
            "ChildCount",
            "Locale",
            "AccessibleId",
            "HelpText",
        ],
        (TERMINAL_PATH, TEXT) => &["CharacterCount", "CaretOffset"],
        _ => {
            return Err((
                "org.freedesktop.DBus.Error.UnknownInterface",
                "unsupported interface",
            ));
        }
    };
    let mut body = BodyWriter::new();
    body.array(8, |body| {
        for name in names {
            if let Ok((signature, value)) = property_value(state, path, interface, name) {
                body.structure(|body| {
                    body.string(name);
                    body.variant(signature, |body| append_encoded(body, &value));
                });
            }
        }
    });
    Ok(body.finish())
}

fn append_encoded(writer: &mut BodyWriter, encoded: &[u8]) {
    // Values passed here already begin at their natural alignment. The
    // variant writer has aligned its destination to that same boundary.
    for byte in encoded {
        writer.byte(*byte);
    }
}

fn variant_body(signature: &str, value: Vec<u8>) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.variant(signature, |body| append_encoded(body, &value));
    body.finish()
}

fn state_body(terminal: bool, focused: bool) -> Vec<u8> {
    let mut words = [0_u32; 2];
    let states = if terminal {
        &[
            STATE_EDITABLE,
            STATE_ENABLED,
            STATE_FOCUSABLE,
            STATE_MULTI_LINE,
            STATE_SENSITIVE,
            STATE_SHOWING,
            STATE_VISIBLE,
            STATE_SELECTABLE_TEXT,
        ][..]
    } else {
        &[STATE_ENABLED, STATE_SENSITIVE, STATE_SHOWING, STATE_VISIBLE][..]
    };
    for state in states
        .iter()
        .copied()
        .chain(focused.then_some(STATE_FOCUSED))
    {
        words[(state / 32) as usize] |= 1 << (state % 32);
    }
    let mut body = BodyWriter::new();
    body.array(4, |body| {
        body.u32(words[0]);
        body.u32(words[1]);
    });
    body.finish()
}

fn locale() -> String {
    env::var("LC_ALL")
        .or_else(|_| env::var("LC_CTYPE"))
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "C".into())
}

fn contains(bounds: TerminalAccessibilityBounds, x: i32, y: i32) -> bool {
    x >= bounds.x
        && y >= bounds.y
        && x < bounds.x.saturating_add(bounds.width)
        && y < bounds.y.saturating_add(bounds.height)
}

fn link_path(index: usize) -> String {
    format!("{TERMINAL_PATH}/link/{index}")
}
fn link_index(path: &str) -> Option<usize> {
    path.strip_prefix(&format!("{TERMINAL_PATH}/link/"))?
        .parse()
        .ok()
}
fn clamp_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn object_array_body(reference: Option<(&str, &str)>) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(8, |body| {
        if let Some((name, path)) = reference {
            body.structure(|body| {
                body.string(name);
                body.object_path(path);
            });
        }
    });
    body.finish()
}

fn string_array_body(values: &[&str]) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(4, |body| {
        for value in values {
            body.string(value);
        }
    });
    body.finish()
}

fn empty_array_body(alignment: usize) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(alignment, |_| {});
    body.finish()
}
fn string_body(value: &str) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.string(value);
    body.finish()
}
fn bool_body(value: bool) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.bool(value);
    body.finish()
}
fn i16_body(value: i16) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i16(value);
    body.finish()
}
fn i32_body(value: i32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(value);
    body.finish()
}
fn u32_body(value: u32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.u32(value);
    body.finish()
}
fn f64_body(value: f64) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.f64(value);
    body.finish()
}
fn two_i32_body(a: i32, b: i32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(a);
    body.i32(b);
    body.finish()
}
fn rect_values_body(value: TerminalAccessibilityBounds) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(value.x);
    body.i32(value.y);
    body.i32(value.width);
    body.i32(value.height);
    body.finish()
}
fn rect_body(value: TerminalAccessibilityBounds) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.structure(|body| {
        body.i32(value.x);
        body.i32(value.y);
        body.i32(value.width);
        body.i32(value.height);
    });
    body.finish()
}
