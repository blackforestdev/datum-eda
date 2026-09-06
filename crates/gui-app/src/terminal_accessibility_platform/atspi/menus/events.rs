//! Native menu child/state events; no test-only accessibility tree.
use super::*;

pub(in crate::terminal_accessibility_platform) fn messages(
    mut serial: impl FnMut() -> u32,
    bus: &str,
    previous: &[MenuAccessibleNode],
    next: &[MenuAccessibleNode],
    previous_root_offset: usize,
    next_root_offset: usize,
) -> Vec<Message> {
    let mut messages = Vec::new();
    for old in previous
        .iter()
        .rev()
        .filter(|old| !next.iter().any(|node| node.id == old.id))
    {
        messages.push(child_event(
            serial(),
            bus,
            previous,
            old,
            "remove",
            previous_root_offset,
        ));
    }
    for node in next {
        let old = previous.iter().find(|old| old.id == node.id);
        if old.is_none() {
            messages.push(child_event(
                serial(),
                bus,
                next,
                node,
                "add",
                next_root_offset,
            ));
        }
        for (name, before, after) in [
            ("focused", old.map(|old| old.focused), node.focused),
            ("enabled", old.map(|old| old.available), node.available),
            ("sensitive", old.map(|old| old.available), node.available),
        ] {
            if before == Some(after) {
                continue;
            }
            messages.push(event(
                serial(),
                &path(&node.id),
                "StateChanged",
                name,
                i32::from(after),
                |body| body.variant("b", |body| body.bool(after)),
            ));
        }
        if old.is_some_and(|old| old.description != node.description) {
            messages.push(event(
                serial(),
                &path(&node.id),
                "PropertyChange",
                "accessible-description",
                0,
                |body| body.variant("s", |body| body.string(&node.description)),
            ));
        }
    }
    messages
}

fn child_event(
    serial: u32,
    bus: &str,
    nodes: &[MenuAccessibleNode],
    node: &MenuAccessibleNode,
    detail: &str,
    root_offset: usize,
) -> Message {
    let parent = node
        .parent
        .as_deref()
        .map(path)
        .unwrap_or_else(|| ROOT_PATH.to_owned());
    let index = nodes
        .iter()
        .filter(|candidate| candidate.parent == node.parent)
        .position(|candidate| candidate.id == node.id)
        .unwrap_or(0)
        + if node.parent.is_none() {
            root_offset
        } else {
            0
        };
    event(
        serial,
        &parent,
        "ChildrenChanged",
        detail,
        clamp_i32(index),
        |body| {
            body.variant("(so)", |body| {
                body.structure(|body| {
                    body.string(bus);
                    body.object_path(&path(&node.id));
                })
            });
        },
    )
}

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
