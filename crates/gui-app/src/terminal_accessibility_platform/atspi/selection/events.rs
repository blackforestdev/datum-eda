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
        if previous.is_none_or(|previous| {
            previous.view.project != next.view.project
                || previous.view.subject != next.view.subject
                || previous.view.members != next.view.members
                || previous.view.scope != next.view.scope
        }) {
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
        if previous.is_none_or(|previous| previous.focused != next.focused) {
            messages.push(event(
                serial(),
                PATH,
                "StateChanged",
                "focused",
                i32::from(next.focused),
                |body| body.variant("b", |body| body.bool(next.focused)),
            ));
        }
    }
    messages
}
