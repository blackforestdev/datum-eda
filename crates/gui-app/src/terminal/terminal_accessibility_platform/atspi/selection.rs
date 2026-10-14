//! Lazy read-only AT-SPI selection tree over the complete immutable projection.
use super::*;
use datum_gui_protocol::{AuthoredSelectionIdentity as Id, selection_view::InspectionValue};
#[path = "selection/children.rs"]
mod children;
#[path = "selection/presentation.rs"]
mod presentation;
#[path = "selection/state.rs"]
mod state;
#[cfg(test)]
#[path = "selection/state_tests.rs"]
mod state_tests;
use presentation::{description, name};
#[path = "selection/events.rs"]
mod events;
pub(in crate::terminal_accessibility_platform) use events::messages;
#[cfg(test)]
#[path = "selection/tests.rs"]
mod tests;
pub(super) const PATH: &str = "/org/a11y/atspi/accessible/selection";
const MEMBERS: &str = "/org/a11y/atspi/accessible/selection/members";
const FIELDS: &str = "/org/a11y/atspi/accessible/selection/fields";
fn encoded(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn decoded(value: &str) -> Option<String> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(text, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
fn member_path(project: datum_gui_protocol::SelectionProjectId, id: &Id) -> String {
    format!(
        "{MEMBERS}/p{}/n{}",
        project.simple(),
        encoded(&serde_json::to_string(id).expect("typed identity serializes"))
    )
}
fn field_path(field: &str) -> String {
    format!("{FIELDS}/n{}", encoded(field))
}
#[derive(Clone, Copy)]
enum Node<'a> {
    Root,
    Members,
    Fields,
    Member(&'a Id),
    Field(&'a str),
}
fn node<'a>(
    view: &'a datum_gui_protocol::selection_view::SelectionView,
    path: &str,
) -> Option<Node<'a>> {
    match path {
        PATH => Some(Node::Root),
        MEMBERS => Some(Node::Members),
        FIELDS => Some(Node::Fields),
        _ => {
            if let Some(key) = path.strip_prefix(&format!("{MEMBERS}/p{}/n", view.project.simple()))
            {
                let id: Id = serde_json::from_str(&decoded(key)?).ok()?;
                return view.members.get(&id).map(Node::Member);
            }
            let key = decoded(path.strip_prefix(&format!("{FIELDS}/n"))?)?;
            view.inspection
                .iter()
                .find(|scope| scope.scope == view.scope)?
                .fields
                .get_key_value(&key)
                .map(|(key, _)| Node::Field(key))
        }
    }
}
fn count(view: &datum_gui_protocol::selection_view::SelectionView, node: Node<'_>) -> usize {
    match node {
        Node::Root => 2,
        Node::Members => view.members.len(),
        Node::Fields => view.fields().len(),
        _ => 0,
    }
}
fn child(
    view: &datum_gui_protocol::selection_view::SelectionView,
    node: Node<'_>,
    index: usize,
) -> Option<String> {
    match node {
        Node::Root => [MEMBERS, FIELDS].get(index).map(|s| (*s).into()),
        Node::Members => view
            .members
            .iter()
            .nth(index)
            .map(|id| member_path(view.project, id)),
        Node::Fields => view.fields().into_iter().nth(index).map(field_path),
        _ => None,
    }
}
fn property(
    state: &ServiceState,
    view: &datum_gui_protocol::selection_view::SelectionView,
    node: Node<'_>,
    path: &str,
    key: &str,
) -> DispatchResult {
    match key {
        "Name" => Ok(("s", string_body(&name(view, node)))),
        "Description" | "HelpText" => Ok((
            "s",
            string_body(&description(
                state.selection.as_ref().expect("selection dispatch"),
                node,
            )),
        )),
        "AccessibleId" => Ok(("s", string_body(path))),
        "Locale" => Ok(("s", string_body(&locale()))),
        "ChildCount" => Ok(("i", i32_body(clamp_i32(count(view, node))))),
        "Parent" => Ok((
            "(so)",
            object_reference_body(
                &state.bus_name,
                match node {
                    Node::Root => ROOT_PATH,
                    Node::Members | Node::Fields => PATH,
                    Node::Member(_) => MEMBERS,
                    Node::Field(_) => FIELDS,
                },
            ),
        )),
        "version" => Ok(("u", u32_body(1))),
        _ => Err((
            "org.freedesktop.DBus.Error.UnknownProperty",
            "unsupported selection property",
        )),
    }
}
pub(super) fn dispatch(
    state: &ServiceState,
    path: &str,
    interface: &str,
    member: &str,
    call: &Message,
) -> DispatchResult {
    let snapshot = state.selection.as_ref().ok_or_else(unknown_object)?;
    let view = snapshot.view.as_ref();
    let node = node(view, path).ok_or_else(unknown_object)?;
    if interface == INTROSPECTABLE && member == "Introspect" {
        return Ok((
            "s",
            string_body(
                "<node><interface name=\"org.a11y.atspi.Accessible\"/><interface name=\"org.freedesktop.DBus.Properties\"/></node>",
            ),
        ));
    }
    if interface == PROPERTIES {
        let mut reader = call.body_reader();
        if reader.string().map_err(|_| invalid_args())? != ACCESSIBLE {
            return Err((
                "org.freedesktop.DBus.Error.UnknownInterface",
                "selection exposes Accessible only",
            ));
        }
        return match member {
            "Get" => {
                let key = reader.string().map_err(|_| invalid_args())?;
                let (signature, value) = property(state, view, node, path, &key)?;
                Ok(("v", variant_body(signature, value)))
            }
            "GetAll" => {
                let mut body = BodyWriter::new();
                body.array(8, |body| {
                    for key in [
                        "Name",
                        "Description",
                        "HelpText",
                        "Parent",
                        "ChildCount",
                        "Locale",
                        "AccessibleId",
                    ] {
                        let (signature, value) =
                            property(state, view, node, path, key).expect("known property");
                        body.structure(|body| {
                            body.string(key);
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
                "selection inspection is read-only",
            )),
            _ => Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                "unsupported selection property method",
            )),
        };
    }
    if interface != ACCESSIBLE {
        return Err((
            "org.freedesktop.DBus.Error.UnknownInterface",
            "selection has no mutation interface",
        ));
    }
    match member {
        "GetChildren" => children::reply(state, view, node, call),
        "GetChildAtIndex" => {
            let index = usize::try_from(call.body_reader().i32().map_err(|_| invalid_args())?)
                .map_err(|_| invalid_args())?;
            let child = child(view, node, index).ok_or_else(invalid_args)?;
            Ok(("(so)", object_reference_body(&state.bus_name, &child)))
        }
        "GetIndexInParent" => {
            let index = match node {
                Node::Root => state
                    .root_child_paths()
                    .iter()
                    .position(|p| p == PATH)
                    .unwrap_or(0),
                Node::Members => 0,
                Node::Fields => 1,
                Node::Member(id) => view.members.iter().position(|v| v == id).unwrap_or(0),
                Node::Field(field) => view
                    .fields()
                    .into_iter()
                    .position(|v| v == field)
                    .unwrap_or(0),
            };
            Ok(("i", i32_body(clamp_i32(index))))
        }
        "GetRole" => Ok(("u", u32_body(ROLE_PANEL))),
        "GetRoleName" | "GetLocalizedRoleName" => Ok(("s", string_body("panel"))),
        "GetState" => Ok(("au", state::body(snapshot, node))),
        "GetApplication" => Ok(("(so)", object_reference_body(&state.bus_name, ROOT_PATH))),
        "GetInterfaces" => Ok(("as", string_array_body(&[ACCESSIBLE]))),
        "GetRelationSet" => Ok(("a(ua(so))", empty_array_body(8))),
        "GetAttributes" => {
            let mut attributes = vec![
                ("read-only", "true".to_owned()),
                ("subject-kind", view.subject.kind().to_owned()),
                ("project-id", view.project.to_string()),
                (
                    "active-pane",
                    snapshot
                        .pane
                        .map(|pane| pane.0.to_string())
                        .unwrap_or_default(),
                ),
                ("model-revision", view.revision.0.clone()),
                (
                    "description",
                    description(state.selection.as_ref().expect("selection dispatch"), node),
                ),
            ];
            if let Node::Member(id) = node {
                attributes.extend([
                    ("selected", "true".into()),
                    (
                        "source-identity",
                        serde_json::to_string(id).expect("typed identity"),
                    ),
                    ("selection-focus", (view.focus() == Some(id)).to_string()),
                    ("hidden", view.hidden.contains(id).to_string()),
                    ("locked", view.locked.contains(id).to_string()),
                ]);
            }
            if let Node::Field(field) = node {
                match view.field_ref(field) {
                    Some(InspectionValue::Common(value)) => attributes.extend([
                        ("value-state", "common".into()),
                        ("canonical-value", value.to_string()),
                    ]),
                    Some(InspectionValue::Mixed) => {
                        attributes.push(("value-state", "mixed".into()))
                    }
                    Some(InspectionValue::Unavailable(reason)) => attributes.extend([
                        ("value-state", "unavailable".into()),
                        ("unavailable-reason", reason.clone()),
                    ]),
                    None => {}
                }
            }
            let mut body = BodyWriter::new();
            body.array(8, |body| {
                for (key, value) in attributes {
                    body.structure(|body| {
                        body.string(key);
                        body.string(&value);
                    });
                }
            });
            Ok(("a{ss}", body.finish()))
        }
        _ => Err((
            "org.freedesktop.DBus.Error.UnknownMethod",
            "unsupported selection method",
        )),
    }
}
