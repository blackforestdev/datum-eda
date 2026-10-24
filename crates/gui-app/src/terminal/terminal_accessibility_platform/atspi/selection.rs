//! Lazy read-only AT-SPI selection tree over the complete immutable projection.
use super::*;
use datum_gui_protocol::{AuthoredSelectionIdentity as Id, selection_view::InspectionValue};
#[path = "selection/children.rs"]
mod children;
#[path = "selection/workspace.rs"]
mod workspace;
pub(super) use workspace::NAVIGATOR;
#[path = "selection/presentation.rs"]
mod presentation;
#[path = "selection/state.rs"]
mod state;
use presentation::{description, name};
#[path = "selection/events.rs"]
mod events;
pub(in crate::terminal_accessibility_platform) use events::messages;
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
    Navigator,
    Document(&'a crate::selection_accessibility::workspace::Document),
    Section(&'a crate::selection_accessibility::workspace::Section),
    SectionField(
        &'a crate::selection_accessibility::workspace::Section,
        usize,
    ),
    RunErc,
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
fn snapshot_node<'a>(
    snapshot: &'a crate::selection_accessibility::SelectionAccessibilitySnapshot,
    path: &str,
) -> Option<Node<'a>> {
    if let Some(ui) = snapshot.inspection.as_ref() {
        if path == PATH {
            return Some(Node::Root);
        }
        if let Some(node) = workspace::node(ui, path) {
            return Some(node);
        }
        if path == FIELDS || path.starts_with(&format!("{FIELDS}/")) {
            return None;
        }
    }
    snapshot.view.as_deref().and_then(|view| node(view, path))
}

fn resolved_name(
    snapshot: &crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'_>,
) -> String {
    workspace::name(snapshot, node).unwrap_or_else(|| {
        name(
            snapshot.view.as_deref().expect("native selection node"),
            node,
        )
    })
}
fn parent_path(node: Node<'_>) -> String {
    workspace::parent(node).unwrap_or_else(|| {
        match node {
            Node::Root => ROOT_PATH,
            Node::Members | Node::Fields => PATH,
            Node::Member(_) => MEMBERS,
            Node::Field(_) => FIELDS,
            _ => unreachable!("workspace parent resolved above"),
        }
        .into()
    })
}

fn property(
    state: &ServiceState,
    snapshot: &crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'_>,
    path: &str,
    key: &str,
) -> DispatchResult {
    match key {
        "Name" => Ok(("s", string_body(&resolved_name(snapshot, node)))),
        "Description" | "HelpText" => Ok((
            "s",
            string_body(&description(
                state.selection.as_ref().expect("selection dispatch"),
                node,
            )),
        )),
        "AccessibleId" => Ok(("s", string_body(path))),
        "Locale" => Ok(("s", string_body(&locale()))),
        "ChildCount" => Ok((
            "i",
            i32_body(clamp_i32(children::paths(snapshot, node).count())),
        )),
        "Parent" => Ok((
            "(so)",
            object_reference_body(&state.bus_name, &parent_path(node)),
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
    let view = snapshot.view.as_deref();
    let node = snapshot_node(snapshot, path).ok_or_else(unknown_object)?;
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
                let (signature, value) = property(state, snapshot, node, path, &key)?;
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
                            property(state, snapshot, node, path, key).expect("known property");
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
        "GetChildren" => children::reply(state, snapshot, node, call),
        "GetChildAtIndex" => {
            let index = usize::try_from(call.body_reader().i32().map_err(|_| invalid_args())?)
                .map_err(|_| invalid_args())?;
            let child = children::paths(snapshot, node)
                .nth(index)
                .ok_or_else(invalid_args)?;
            Ok(("(so)", object_reference_body(&state.bus_name, &child)))
        }
        "GetIndexInParent" => {
            let parent = parent_path(node);
            let index = if parent == ROOT_PATH {
                state
                    .root_child_paths()
                    .iter()
                    .position(|p| p == path)
                    .unwrap_or(0)
            } else {
                snapshot_node(snapshot, &parent)
                    .and_then(|parent| {
                        children::paths(snapshot, parent).position(|child| child == path)
                    })
                    .unwrap_or(0)
            };
            Ok(("i", i32_body(clamp_i32(index))))
        }
        "GetRole" => Ok((
            "u",
            u32_body(
                workspace::role(node)
                    .map(preference_role)
                    .unwrap_or(ROLE_PANEL),
            ),
        )),
        "GetRoleName" | "GetLocalizedRoleName" => Ok((
            "s",
            string_body(
                workspace::role(node)
                    .map(preference_role_name)
                    .unwrap_or("panel"),
            ),
        )),
        "GetState" => Ok(("au", state::body(snapshot, node))),
        "GetApplication" => Ok(("(so)", object_reference_body(&state.bus_name, ROOT_PATH))),
        "GetInterfaces" => Ok(("as", string_array_body(&[ACCESSIBLE]))),
        "GetRelationSet" => Ok(("a(ua(so))", empty_array_body(8))),
        "GetAttributes" => {
            let mut attributes = vec![
                ("read-only", "true".to_owned()),
                (
                    "subject-kind",
                    view.map(|view| view.subject.kind())
                        .unwrap_or("document")
                        .to_owned(),
                ),
                (
                    "project-id",
                    view.map(|view| view.project.to_string())
                        .or_else(|| snapshot.inspection.as_ref().map(|ui| ui.project.clone()))
                        .unwrap_or_default(),
                ),
                (
                    "active-pane",
                    snapshot
                        .pane
                        .map(|pane| pane.0.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "model-revision",
                    view.map(|view| view.revision.0.clone())
                        .or_else(|| snapshot.inspection.as_ref().map(|ui| ui.revision.clone()))
                        .unwrap_or_default(),
                ),
                (
                    "description",
                    description(state.selection.as_ref().expect("selection dispatch"), node),
                ),
            ];
            if let (Some(view), Node::Member(id)) = (view, node) {
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
            let native_field = match node {
                Node::Field(field) => Some(field),
                Node::SectionField(section, index) => section.fields[index].native_field.as_deref(),
                _ => None,
            };
            if let (Some(view), Some(field)) = (view, native_field) {
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
            attributes.extend(workspace::attributes(snapshot, node));
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

#[cfg(test)]
#[path = "selection/state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "selection/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "selection/workspace_tests.rs"]
mod workspace_tests;
