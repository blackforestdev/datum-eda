//! AT-SPI property projection for application, terminal, link, and preferences nodes.

use super::*;

pub(super) fn property_value(
    state: &ServiceState,
    path: &str,
    interface: &str,
    property: &str,
) -> DispatchResult {
    let terminal = path == TERMINAL_PATH;
    let preference = preference_index(path).and_then(|index| state.preferences.get(index));
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
        (ROOT_PATH, ACCESSIBLE, "ChildCount") => Ok((
            "i",
            i32_body(
                i32::from(state.terminal_available) + i32::from(!state.preferences.is_empty()),
            ),
        )),
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
        (_, ACCESSIBLE, "Name") if preference.is_some() => {
            Ok(("s", string_body(&preference.expect("guarded").name)))
        }
        (_, ACCESSIBLE, "Description" | "HelpText") if preference.is_some() => {
            let node = preference.expect("guarded");
            let value = node.value.as_deref().unwrap_or("No current value");
            Ok((
                "s",
                string_body(&format!("{}. Current value: {value}", node.description)),
            ))
        }
        (_, ACCESSIBLE, "Parent") if preference.is_some() => {
            let index = preference_index(path).expect("guarded");
            let parent = if index == 0 {
                ROOT_PATH.to_owned()
            } else {
                preference_path(0)
            };
            Ok(("(so)", object_reference_body(&state.bus_name, &parent)))
        }
        (_, ACCESSIBLE, "ChildCount") if preference.is_some() => {
            let index = preference_index(path).expect("guarded");
            Ok((
                "i",
                i32_body(if index == 0 {
                    clamp_i32(state.preferences.len().saturating_sub(1))
                } else {
                    0
                }),
            ))
        }
        (_, ACCESSIBLE, "Locale") if preference.is_some() => Ok(("s", string_body(&locale()))),
        (_, ACCESSIBLE, "AccessibleId") if preference.is_some() => {
            Ok(("s", string_body(&preference.expect("guarded").id)))
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

pub(super) fn properties_body(
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
        (_, ACCESSIBLE) if preference_index(path).is_some() => &[
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
    for byte in encoded {
        writer.byte(*byte);
    }
}

pub(super) fn variant_body(signature: &str, value: Vec<u8>) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.variant(signature, |body| append_encoded(body, &value));
    body.finish()
}
