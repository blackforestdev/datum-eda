//! Menu-only AT-SPI projection. Existing Terminal/Preferences dispatch is unchanged.

use super::*;
use datum_gui_protocol::gui_menu_model::accessibility::{MenuAccessibleNode, MenuAccessibleRole};

mod events;
pub(in crate::terminal_accessibility_platform) use events::messages;
#[cfg(test)]
mod tests;

pub(super) const MENU_PATH: &str = "/org/a11y/atspi/accessible/menus/";

pub(super) fn path(id: &str) -> String {
    // Encode full UTF-8 identity; never depend on the current row index and
    // never place punctuation from an action key into a D-Bus path segment.
    let encoded: String = id
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("{MENU_PATH}n{encoded}")
}

fn children<'a>(nodes: &'a [MenuAccessibleNode], id: &str) -> Vec<&'a MenuAccessibleNode> {
    nodes
        .iter()
        .filter(|node| node.parent.as_deref() == Some(id))
        .collect()
}

fn index_in_parent(state: &ServiceState, node: &MenuAccessibleNode) -> usize {
    if let Some(parent) = &node.parent {
        children(&state.menus, parent)
            .iter()
            .position(|child| child.id == node.id)
            .unwrap_or(0)
    } else {
        state
            .root_child_paths()
            .iter()
            .position(|candidate| *candidate == path(&node.id))
            .unwrap_or(0)
    }
}

pub(super) fn dispatch(
    state: &ServiceState,
    object_path: &str,
    interface: &str,
    member: &str,
    call: &Message,
) -> DispatchResult {
    let node = state
        .menus
        .iter()
        .find(|node| path(&node.id) == object_path)
        .ok_or_else(unknown_object)?;
    if interface == INTROSPECTABLE && member == "Introspect" {
        return Ok((
            "s",
            string_body(
                "<node><interface name=\"org.a11y.atspi.Accessible\"/><interface name=\"org.freedesktop.DBus.Properties\"/><interface name=\"org.freedesktop.DBus.Introspectable\"/></node>",
            ),
        ));
    }
    if interface == PROPERTIES {
        let mut reader = call.body_reader();
        if reader.string().map_err(|_| invalid_args())? != ACCESSIBLE {
            return Err((
                "org.freedesktop.DBus.Error.UnknownInterface",
                "unsupported menu interface",
            ));
        }
        return match member {
            "Get" => {
                let name = reader.string().map_err(|_| invalid_args())?;
                let (signature, value) = property(state, node, &name)?;
                Ok(("v", variant_body(signature, value)))
            }
            "GetAll" => {
                let mut body = BodyWriter::new();
                body.array(8, |body| {
                    for name in [
                        "Name",
                        "Description",
                        "HelpText",
                        "Parent",
                        "ChildCount",
                        "Locale",
                        "AccessibleId",
                    ] {
                        let (signature, value) =
                            property(state, node, name).expect("known menu property");
                        body.structure(|body| {
                            body.string(name);
                            body.variant(signature, |body| {
                                for byte in value {
                                    body.byte(byte);
                                }
                            });
                        });
                    }
                });
                Ok(("a{sv}", body.finish()))
            }
            "Set" => Err((
                "org.freedesktop.DBus.Error.PropertyReadOnly",
                "menu projection is read-only",
            )),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported menu properties method",
            )),
        };
    }
    if interface != ACCESSIBLE {
        return Err((
            "org.freedesktop.DBus.Error.UnknownInterface",
            "unsupported menu interface",
        ));
    }
    let menu = node.role == MenuAccessibleRole::Menu;
    match member {
        "GetChildren" => Ok((
            "a(so)",
            object_array_body(
                &state.bus_name,
                &children(&state.menus, &node.id)
                    .iter()
                    .map(|node| path(&node.id))
                    .collect::<Vec<_>>(),
            ),
        )),
        "GetChildAtIndex" => {
            let index = usize::try_from(call.body_reader().i32().map_err(|_| invalid_args())?)
                .map_err(|_| invalid_args())?;
            let rows = children(&state.menus, &node.id);
            let child = rows.get(index).ok_or_else(invalid_args)?;
            Ok((
                "(so)",
                object_reference_body(&state.bus_name, &path(&child.id)),
            ))
        }
        "GetIndexInParent" => Ok(("i", i32_body(clamp_i32(index_in_parent(state, node))))),
        "GetRole" => Ok(("u", u32_body(if menu { 33 } else { 35 }))),
        "GetRoleName" | "GetLocalizedRoleName" => {
            Ok(("s", string_body(if menu { "menu" } else { "menu item" })))
        }
        "GetState" => Ok((
            "au",
            state_body(false, false, Some((node.available, node.focused))),
        )),
        "GetApplication" => Ok(("(so)", object_reference_body(&state.bus_name, ROOT_PATH))),
        "GetInterfaces" => Ok(("as", string_array_body(&[ACCESSIBLE]))),
        "GetRelationSet" => Ok(("a(ua(so))", empty_array_body(8))),
        "GetAttributes" => {
            let mut body = BodyWriter::new();
            body.array(8, |body| {
                for (key, value) in [
                    ("dispatch-key", node.key.as_deref().unwrap_or("")),
                    ("description", node.description.as_str()),
                    ("available", if node.available { "true" } else { "false" }),
                ] {
                    body.structure(|body| {
                        body.string(key);
                        body.string(value);
                    });
                }
            });
            Ok(("a{ss}", body.finish()))
        }
        _ => Err((
            "org.freedesktop.DBus.Error.UnknownMethod",
            "unsupported menu method",
        )),
    }
}

fn property(state: &ServiceState, node: &MenuAccessibleNode, name: &str) -> DispatchResult {
    match name {
        "Name" => Ok(("s", string_body(&node.name))),
        "Description" | "HelpText" => Ok(("s", string_body(&node.description))),
        "AccessibleId" => Ok(("s", string_body(&node.id))),
        "Locale" => Ok(("s", string_body(&locale()))),
        "Parent" => Ok((
            "(so)",
            object_reference_body(
                &state.bus_name,
                &node
                    .parent
                    .as_deref()
                    .map(path)
                    .unwrap_or_else(|| ROOT_PATH.to_owned()),
            ),
        )),
        "ChildCount" => Ok((
            "i",
            i32_body(clamp_i32(children(&state.menus, &node.id).len())),
        )),
        "version" => Ok(("u", u32_body(1))),
        _ => Err((
            "org.freedesktop.DBus.Error.UnknownProperty",
            "unsupported menu property",
        )),
    }
}
