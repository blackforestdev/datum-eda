//! Datum-owned AT-SPI object model and method dispatcher.

use std::env;

use crate::terminal_accessibility::{TerminalAccessibilityBounds, TerminalAccessibilitySnapshot};
use datum_gui_protocol::{GlobalPreferencesAccessibleNode, GlobalPreferencesAccessibleRole};

use super::body::BodyWriter;
use super::connection::object_reference_body;
use super::dbus::{Message, MessageType};

mod dispatch_error;
mod introspection;
mod properties;
mod text_ranges;
use dispatch_error::{error, invalid_args, unknown_object};
use introspection::introspection_body;
use properties::{properties_body, property_value, variant_body};
use text_ranges::{char_range, text_at_offset};

pub(super) const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";
pub(super) const TERMINAL_PATH: &str = "/org/a11y/atspi/accessible/terminal";
pub(super) const PREFERENCES_PATH: &str = "/org/a11y/atspi/accessible/preferences";
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
const ROLE_DIALOG: u32 = 16;
const ROLE_COMBO_BOX: u32 = 11;
const ROLE_PANEL: u32 = 38;
const ROLE_PUSH_BUTTON: u32 = 43;
const ROLE_STATUS_BAR: u32 = 53;
const ROLE_TOGGLE_BUTTON: u32 = 62;
const ROLE_TEXT: u32 = 79;
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
    pub(super) terminal_available: bool,
    pub(super) preferences: Vec<GlobalPreferencesAccessibleNode>,
}

impl ServiceState {
    pub(super) fn new(
        snapshot: TerminalAccessibilitySnapshot,
        terminal_available: bool,
        preferences: Vec<GlobalPreferencesAccessibleNode>,
    ) -> Self {
        Self {
            snapshot,
            application_id: 0,
            registry_parent: (String::new(), NULL_PATH.into()),
            bus_name: String::new(),
            terminal_available,
            preferences,
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
        let terminal = path == TERMINAL_PATH && self.terminal_available;
        let link = link_index(path).and_then(|index| self.snapshot.links.get(index));
        let preference = preference_index(path).and_then(|index| self.preferences.get(index));
        if !terminal && path != ROOT_PATH && link.is_none() && preference.is_none() {
            return Err(unknown_object());
        }
        match member {
            "GetChildAtIndex" if path == ROOT_PATH => {
                let mut reader = call.body_reader();
                let index = usize::try_from(reader.i32().map_err(|_| invalid_args())?)
                    .map_err(|_| invalid_args())?;
                let paths = self.root_child_paths();
                let path = paths.get(index).ok_or_else(invalid_args)?;
                Ok(("(so)", object_reference_body(&self.bus_name, path)))
            }
            "GetChildAtIndex" if preference_index(path) == Some(0) => {
                let mut reader = call.body_reader();
                let index = usize::try_from(reader.i32().map_err(|_| invalid_args())?)
                    .map_err(|_| invalid_args())?;
                if index + 1 >= self.preferences.len() {
                    return Err(invalid_args());
                }
                Ok((
                    "(so)",
                    object_reference_body(&self.bus_name, &preference_path(index + 1)),
                ))
            }
            "GetChildren" if path == ROOT_PATH => {
                let paths = self.root_child_paths();
                Ok(("a(so)", object_array_body(&self.bus_name, &paths)))
            }
            "GetChildren" if preference_index(path) == Some(0) => {
                let paths: Vec<_> = (1..self.preferences.len()).map(preference_path).collect();
                Ok(("a(so)", object_array_body(&self.bus_name, &paths)))
            }
            "GetChildren" => Ok(("a(so)", object_array_body(&self.bus_name, &[]))),
            "GetIndexInParent" => Ok((
                "i",
                i32_body(if terminal {
                    0
                } else if let Some(index) = preference_index(path) {
                    if index == 0 {
                        i32::from(self.terminal_available)
                    } else {
                        clamp_i32(index - 1)
                    }
                } else {
                    -1
                }),
            )),
            "GetRelationSet" => Ok(("a(ua(so))", empty_array_body(8))),
            "GetRole" => Ok((
                "u",
                u32_body(if terminal {
                    ROLE_TERMINAL
                } else if link.is_some() {
                    ROLE_LINK
                } else if let Some(node) = preference {
                    preference_role(node.role)
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
                } else if let Some(node) = preference {
                    preference_role_name(node.role)
                } else {
                    "application"
                }),
            )),
            "GetState" => Ok((
                "au",
                state_body(
                    terminal,
                    terminal && self.snapshot.focused,
                    preference.map(|node| (node.available, node.focused)),
                ),
            )),
            "GetApplication" => Ok(("(so)", object_reference_body(&self.bus_name, ROOT_PATH))),
            "GetAttributes" if preference.is_some() => {
                let node = preference.expect("guarded above");
                Ok(("a{ss}", preference_attributes_body(node)))
            }
            "GetAttributes" => Ok(("a{ss}", empty_array_body(8))),
            "GetInterfaces" => Ok((
                "as",
                string_array_body(if terminal {
                    &[ACCESSIBLE, COMPONENT, TEXT, HYPERTEXT]
                } else if link.is_some() {
                    &[ACCESSIBLE, HYPERLINK]
                } else if preference.is_some() {
                    &[ACCESSIBLE]
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

    fn root_child_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        if self.terminal_available {
            paths.push(TERMINAL_PATH.to_owned());
        }
        if !self.preferences.is_empty() {
            paths.push(preference_path(0));
        }
        paths
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

fn state_body(terminal: bool, focused: bool, preference: Option<(bool, bool)>) -> Vec<u8> {
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
    } else if preference.is_some() {
        &[STATE_FOCUSABLE, STATE_SHOWING, STATE_VISIBLE][..]
    } else {
        &[STATE_ENABLED, STATE_SENSITIVE, STATE_SHOWING, STATE_VISIBLE][..]
    };
    let available = preference.map(|(available, _)| available).unwrap_or(true);
    let focused = focused || preference.map(|(_, focused)| focused).unwrap_or(false);
    for state in states
        .iter()
        .copied()
        .chain(available.then_some(STATE_ENABLED))
        .chain(available.then_some(STATE_SENSITIVE))
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
fn preference_path(index: usize) -> String {
    format!("{PREFERENCES_PATH}/{index}")
}
fn preference_index(path: &str) -> Option<usize> {
    path.strip_prefix(&format!("{PREFERENCES_PATH}/"))?
        .parse()
        .ok()
}
fn preference_role(role: GlobalPreferencesAccessibleRole) -> u32 {
    match role {
        GlobalPreferencesAccessibleRole::Dialog => ROLE_DIALOG,
        GlobalPreferencesAccessibleRole::Navigation => ROLE_PANEL,
        GlobalPreferencesAccessibleRole::SearchBox => ROLE_TEXT,
        GlobalPreferencesAccessibleRole::Button => ROLE_PUSH_BUTTON,
        GlobalPreferencesAccessibleRole::Switch => ROLE_TOGGLE_BUTTON,
        GlobalPreferencesAccessibleRole::ComboBox => ROLE_COMBO_BOX,
        GlobalPreferencesAccessibleRole::SpinButton | GlobalPreferencesAccessibleRole::TextBox => {
            ROLE_TEXT
        }
        GlobalPreferencesAccessibleRole::Status => ROLE_STATUS_BAR,
    }
}
fn preference_role_name(role: GlobalPreferencesAccessibleRole) -> &'static str {
    match role {
        GlobalPreferencesAccessibleRole::Dialog => "dialog",
        GlobalPreferencesAccessibleRole::Navigation => "panel",
        GlobalPreferencesAccessibleRole::SearchBox => "text",
        GlobalPreferencesAccessibleRole::Button => "push button",
        GlobalPreferencesAccessibleRole::Switch => "toggle button",
        GlobalPreferencesAccessibleRole::ComboBox => "combo box",
        GlobalPreferencesAccessibleRole::SpinButton => "spin button",
        GlobalPreferencesAccessibleRole::TextBox => "text box",
        GlobalPreferencesAccessibleRole::Status => "status bar",
    }
}
fn clamp_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn object_array_body(bus_name: &str, paths: &[String]) -> Vec<u8> {
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

fn preference_attributes_body(node: &GlobalPreferencesAccessibleNode) -> Vec<u8> {
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
