//! Coalesced selection notifications; never emit one event per dense member.
use super::*;
use crate::selection_accessibility::SelectionAccessibilitySnapshot;
fn event(
    serial: u32,
    path: &str,
    member: &str,
    detail: &str,
    index: i32,
    value: impl FnOnce(&mut BodyWriter),
) -> Message {
    let mut body = BodyWriter::new();
    body.string(detail);
    body.i32(index);
    body.i32(0);
    value(&mut body);
    body.array(8, |_| {});
    Message::signal(
        serial,
        path,
        "org.a11y.atspi.Event.Object",
        member,
        "siiva{sv}",
        body.finish(),
    )
}
pub(in crate::terminal_accessibility_platform) fn messages(
    mut serial: impl FnMut() -> u32,
    bus: &str,
    previous: Option<&SelectionAccessibilitySnapshot>,
    next: Option<&SelectionAccessibilitySnapshot>,
    state: &ServiceState,
) -> Vec<Message> {
    if previous == next {
        return Vec::new();
    }
    let mut messages = Vec::new();
    if previous.is_none() != next.is_none() {
        let index = state
            .root_child_paths()
            .iter()
            .position(|p| p == PATH)
            .unwrap_or(state.root_child_paths().len());
        messages.push(event(
            serial(),
            ROOT_PATH,
            "ChildrenChanged",
            if next.is_some() { "add" } else { "remove" },
            clamp_i32(index),
            |body| {
                body.variant("(so)", |body| {
                    body.structure(|body| {
                        body.string(bus);
                        body.object_path(PATH);
                    })
                })
            },
        ));
    }
    if let Some(next) = next {
        if previous.is_none_or(
            |previous| match (previous.view.as_ref(), next.view.as_ref()) {
                (Some(previous), Some(next)) => {
                    previous.project != next.project
                        || previous.revision != next.revision
                        || previous.subject != next.subject
                        || previous.members != next.members
                        || previous.scope != next.scope
                }
                (None, None) => false,
                _ => true,
            },
        ) {
            messages.push(event(serial(), PATH, "SelectionChanged", "", 0, |body| {
                body.variant("s", |body| body.string(""))
            }));
        }
        messages.push(event(
            serial(),
            PATH,
            "PropertyChange",
            "accessible-description",
            0,
            |body| body.variant("s", |body| body.string(&description(next, Node::Root))),
        ));
        let focused = workspace::focus(next, Node::Root).unwrap_or(next.focused);
        if previous.is_none_or(|previous| {
            workspace::focus(previous, Node::Root).unwrap_or(previous.focused) != focused
        }) {
            messages.push(event(
                serial(),
                PATH,
                "StateChanged",
                "focused",
                i32::from(focused),
                |body| body.variant("b", |body| body.bool(focused)),
            ));
        }
    }
    messages.extend(workspace_messages(&mut serial, bus, previous, next, state));
    messages
}

fn workspace_messages(
    serial: &mut impl FnMut() -> u32,
    bus: &str,
    previous: Option<&SelectionAccessibilitySnapshot>,
    next: Option<&SelectionAccessibilitySnapshot>,
    state: &ServiceState,
) -> Vec<Message> {
    let old_ui = previous.and_then(|snapshot| snapshot.inspection.as_ref());
    let new_ui = next.and_then(|snapshot| snapshot.inspection.as_ref());
    if old_ui.is_none() && new_ui.is_none() {
        return Vec::new();
    }
    let mut messages = Vec::new();
    if old_ui.is_some() != new_ui.is_some() {
        let index = state
            .root_child_paths()
            .iter()
            .position(|path| path == workspace::NAVIGATOR || path == PATH)
            .unwrap_or(0);
        messages.push(child_change(
            serial(),
            bus,
            ROOT_PATH,
            workspace::NAVIGATOR,
            new_ui.is_some(),
            index,
        ));
    }
    let old_paths = previous
        .map(|snapshot| children::paths(snapshot, Node::Root).collect::<Vec<_>>())
        .unwrap_or_default();
    let new_paths = next
        .map(|snapshot| children::paths(snapshot, Node::Root).collect::<Vec<_>>())
        .unwrap_or_default();
    for (index, path) in old_paths
        .iter()
        .enumerate()
        .filter(|(_, path)| !new_paths.contains(path))
    {
        messages.push(child_change(serial(), bus, PATH, path, false, index));
    }
    for (index, path) in new_paths
        .iter()
        .enumerate()
        .filter(|(_, path)| !old_paths.contains(path))
    {
        messages.push(child_change(serial(), bus, PATH, path, true, index));
    }
    messages.extend(document_changes(
        serial,
        bus,
        old_ui.map(AsRef::as_ref),
        new_ui.map(AsRef::as_ref),
    ));
    {
        let (old, new) = (old_ui, new_ui);
        let old_focus = old.is_some_and(|ui| ui.navigator_focused);
        let new_focus = new.is_some_and(|ui| ui.navigator_focused);
        if old_focus != new_focus {
            messages.push(event(
                serial(),
                workspace::NAVIGATOR,
                "StateChanged",
                "focused",
                i32::from(new_focus),
                |body| body.variant("b", |body| body.bool(new_focus)),
            ));
        }
        if let (Some(old), Some(new), Some(snapshot)) = (old, new, next) {
            if old.documents != new.documents {
                messages.push(event(
                    serial(),
                    workspace::NAVIGATOR,
                    "PropertyChange",
                    "accessible-description",
                    0,
                    |body| {
                        body.variant("s", |body| {
                            body.string(&description(snapshot, Node::Navigator))
                        })
                    },
                ));
            }
            for row in &new.documents {
                let prior = old.documents.iter().find(|prior| prior.key == row.key);
                for (detail, changed, value) in [
                    (
                        "accessible-name",
                        prior.is_some_and(|prior| prior.name != row.name),
                        row.name.as_str(),
                    ),
                    (
                        "accessible-description",
                        prior.is_some_and(|prior| prior.description != row.description),
                        row.description.as_str(),
                    ),
                ] {
                    if changed {
                        messages.push(event(
                            serial(),
                            &workspace::document_path(&row.key),
                            "PropertyChange",
                            detail,
                            0,
                            |body| body.variant("s", |body| body.string(value)),
                        ));
                    }
                }
                let focused = new.navigator_focused && row.inspected && row.visible;
                let old_focus =
                    prior.is_some_and(|row| old.navigator_focused && row.inspected && row.visible);
                for (detail, changed, active) in [
                    ("focused", focused != old_focus, focused),
                    (
                        "expanded",
                        prior.is_none_or(|prior| prior.expanded != row.expanded),
                        row.expanded,
                    ),
                    (
                        "showing",
                        prior.is_none_or(|prior| prior.visible != row.visible),
                        row.visible,
                    ),
                ] {
                    if changed {
                        messages.push(event(
                            serial(),
                            &workspace::document_path(&row.key),
                            "StateChanged",
                            detail,
                            i32::from(active),
                            |body| body.variant("b", |body| body.bool(active)),
                        ));
                    }
                }
            }
            if old.rerun_available != new.rerun_available {
                let path = format!(
                    "{}/run_erc",
                    workspace::section_path(
                        datum_gui_protocol::inspector_context::InspectorSectionId::Checks
                    )
                );
                messages.push(event(
                    serial(),
                    &path,
                    "StateChanged",
                    "enabled",
                    i32::from(new.rerun_available),
                    |body| body.variant("b", |body| body.bool(new.rerun_available)),
                ));
            }
            for section in &new.sections {
                let prior = old.sections.iter().find(|prior| prior.id == section.id);
                let parent = workspace::section_path(section.id);
                let prior_count = prior.map_or(0, |prior| prior.fields.len());
                for index in section.fields.len()..prior_count {
                    messages.push(child_change(
                        serial(),
                        bus,
                        &parent,
                        &workspace::field_path(section.id, index),
                        false,
                        index,
                    ));
                }
                for index in prior_count..section.fields.len() {
                    messages.push(child_change(
                        serial(),
                        bus,
                        &parent,
                        &workspace::field_path(section.id, index),
                        true,
                        index,
                    ));
                }
                for (index, field) in section.fields.iter().enumerate() {
                    if prior.and_then(|prior| prior.fields.get(index)) != Some(field)
                        || native_field_changed(previous, snapshot, field)
                    {
                        messages.push(event(
                            serial(),
                            &workspace::field_path(section.id, index),
                            "PropertyChange",
                            "accessible-name",
                            0,
                            |body| {
                                body.variant("s", |body| {
                                    body.string(&resolved_name(
                                        snapshot,
                                        Node::SectionField(section, index),
                                    ))
                                })
                            },
                        ));
                    }
                }
                if prior.is_none_or(|prior| prior.focused != section.focused) {
                    messages.push(event(
                        serial(),
                        &workspace::section_path(section.id),
                        "StateChanged",
                        "focused",
                        i32::from(section.focused),
                        |body| body.variant("b", |body| body.bool(section.focused)),
                    ));
                }
                if prior.is_some_and(|prior| prior.collapsed != section.collapsed) {
                    messages.push(event(
                        serial(),
                        &workspace::section_path(section.id),
                        "StateChanged",
                        "expanded",
                        i32::from(!section.collapsed),
                        |body| body.variant("b", |body| body.bool(!section.collapsed)),
                    ));
                }
            }
        }
    }
    messages
}

fn native_field_changed(
    previous: Option<&SelectionAccessibilitySnapshot>,
    next: &SelectionAccessibilitySnapshot,
    field: &crate::selection_accessibility::workspace::Field,
) -> bool {
    let Some(name) = field.native_field.as_deref() else {
        return false;
    };
    let old = previous.and_then(|snapshot| snapshot.view.as_ref());
    let new = next.view.as_ref();
    if old
        .zip(new)
        .is_some_and(|(old, new)| std::sync::Arc::ptr_eq(old, new))
    {
        return false;
    }
    // Compare borrowed exact values/formatting. No complete membership array
    // or second selection projection is cloned to detect a lazy value change.
    fn label<'a>(
        view: &'a datum_gui_protocol::selection_view::SelectionView,
        name: &str,
    ) -> Option<&'a str> {
        view.inspection
            .iter()
            .find(|scope| scope.scope == view.scope)
            .and_then(|scope| scope.display_labels.get(name))
            .map(String::as_str)
    }
    old.and_then(|view| view.field_ref(name)) != new.and_then(|view| view.field_ref(name))
        || old.and_then(|view| view.field_quantity_presentation(name))
            != new.and_then(|view| view.field_quantity_presentation(name))
        || old.and_then(|view| label(view, name)) != new.and_then(|view| label(view, name))
}

fn document_changes(
    serial: &mut impl FnMut() -> u32,
    bus: &str,
    old: Option<&crate::selection_accessibility::workspace::WorkspaceInspection>,
    new: Option<&crate::selection_accessibility::workspace::WorkspaceInspection>,
) -> Vec<Message> {
    use crate::selection_accessibility::workspace::{Document, WorkspaceInspection};
    fn positions(ui: Option<&WorkspaceInspection>) -> Vec<(&Document, usize)> {
        let mut siblings = std::collections::BTreeMap::new();
        let mut rows = Vec::new();
        for row in ui.into_iter().flat_map(|ui| &ui.documents) {
            let index = siblings.entry(row.parent.as_ref()).or_insert(0);
            rows.push((row, *index));
            *index += 1;
        }
        // Child removals precede parent removals; reverse sibling indexes keep
        // an event-driven client's cached child list valid after each removal.
        rows.sort_by_key(|(row, index)| (row.key.instance_path.len(), *index));
        rows
    }
    let old = positions(old);
    let new = positions(new);
    let mut messages = Vec::new();
    for (row, index) in old.iter().rev().filter(|(row, index)| {
        !new.iter().any(|(next, next_index)| {
            next.key == row.key && next.parent == row.parent && next_index == index
        })
    }) {
        let path = workspace::document_path(&row.key);
        if !new.iter().any(|(next, _)| next.key == row.key) {
            messages.push(event(
                serial(),
                &path,
                "StateChanged",
                "defunct",
                1,
                |body| body.variant("b", |body| body.bool(true)),
            ));
        }
        messages.push(child_change(
            serial(),
            bus,
            &workspace::parent(Node::Document(row)).expect("document parent"),
            &path,
            false,
            *index,
        ));
    }
    for (row, index) in new.iter().filter(|(row, index)| {
        !old.iter().any(|(prior, prior_index)| {
            prior.key == row.key && prior.parent == row.parent && prior_index == index
        })
    }) {
        messages.push(child_change(
            serial(),
            bus,
            &workspace::parent(Node::Document(row)).expect("document parent"),
            &workspace::document_path(&row.key),
            true,
            *index,
        ));
    }
    messages
}

fn child_change(
    serial: u32,
    bus: &str,
    parent: &str,
    path: &str,
    add: bool,
    index: usize,
) -> Message {
    event(
        serial,
        parent,
        "ChildrenChanged",
        if add { "add" } else { "remove" },
        clamp_i32(index),
        |body| {
            body.variant("(so)", |body| {
                body.structure(|body| {
                    body.string(bus);
                    body.object_path(path);
                })
            });
        },
    )
}
