//! AT-SPI event projection from immutable terminal accessibility changes.

use crate::terminal_accessibility::TerminalAccessibilitySnapshot;
use crate::terminal_accessibility_bridge::TerminalAccessibilityEvent;

use super::atspi::TERMINAL_PATH;
use super::body::BodyWriter;
use super::dbus::Message;

const EVENT_OBJECT: &str = "org.a11y.atspi.Event.Object";

pub(super) fn messages(
    mut serial: impl FnMut() -> u32,
    previous: Option<&TerminalAccessibilitySnapshot>,
    next: &TerminalAccessibilitySnapshot,
    events: &[TerminalAccessibilityEvent],
) -> Vec<Message> {
    let mut messages = Vec::new();
    for event in events {
        match event {
            TerminalAccessibilityEvent::TextChanged => {
                if let Some(previous) = previous
                    && !previous.text.is_empty()
                {
                    messages.push(event_message(
                        serial(),
                        "TextChanged",
                        "delete",
                        0,
                        scalar_len(&previous.text),
                        VariantValue::String(&previous.text),
                    ));
                }
                if !next.text.is_empty() {
                    messages.push(event_message(
                        serial(),
                        "TextChanged",
                        "insert",
                        0,
                        scalar_len(&next.text),
                        VariantValue::String(&next.text),
                    ));
                }
            }
            TerminalAccessibilityEvent::CaretMoved => messages.push(event_message(
                serial(),
                "TextCaretMoved",
                "",
                clamp_i32(next.caret),
                0,
                VariantValue::String(""),
            )),
            TerminalAccessibilityEvent::SelectionChanged => messages.push(event_message(
                serial(),
                "TextSelectionChanged",
                "",
                0,
                0,
                VariantValue::String(""),
            )),
            TerminalAccessibilityEvent::FocusChanged => messages.push(event_message(
                serial(),
                "StateChanged",
                "focused",
                i32::from(next.focused),
                0,
                VariantValue::Bool(next.focused),
            )),
            TerminalAccessibilityEvent::TitleChanged => messages.push(event_message(
                serial(),
                "PropertyChange",
                "accessible-name",
                0,
                0,
                VariantValue::String(&next.title),
            )),
            TerminalAccessibilityEvent::BoundsChanged => messages.push(event_message(
                serial(),
                "BoundsChanged",
                "",
                0,
                0,
                VariantValue::Bounds(next.bounds),
            )),
            TerminalAccessibilityEvent::Bell => messages.push(event_message(
                serial(),
                "Announcement",
                "Terminal bell",
                0,
                0,
                VariantValue::String("Terminal bell"),
            )),
        }
    }
    messages
}

enum VariantValue<'a> {
    String(&'a str),
    Bool(bool),
    Bounds(crate::terminal_accessibility::TerminalAccessibilityBounds),
}

fn event_message(
    serial: u32,
    member: &str,
    detail: &str,
    detail1: i32,
    detail2: i32,
    value: VariantValue<'_>,
) -> Message {
    let mut body = BodyWriter::new();
    body.string(detail);
    body.i32(detail1);
    body.i32(detail2);
    match value {
        VariantValue::String(value) => body.variant("s", |body| body.string(value)),
        VariantValue::Bool(value) => body.variant("b", |body| body.bool(value)),
        VariantValue::Bounds(value) => body.variant("(iiii)", |body| {
            body.structure(|body| {
                body.i32(value.x);
                body.i32(value.y);
                body.i32(value.width);
                body.i32(value.height);
            });
        }),
    }
    body.array(8, |_| {});
    Message::signal(
        serial,
        TERMINAL_PATH,
        EVENT_OBJECT,
        member,
        "siiva{sv}",
        body.finish(),
    )
}

fn scalar_len(text: &str) -> i32 {
    clamp_i32(text.chars().count())
}

fn clamp_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_accessibility::{
        TerminalAccessibilityBounds, TerminalAccessibilitySnapshot,
    };

    fn snapshot(text: &str, caret: usize) -> TerminalAccessibilitySnapshot {
        TerminalAccessibilitySnapshot {
            session_id: "s".into(),
            title: "Terminal".into(),
            text: text.into(),
            caret,
            selection: None,
            links: Vec::new(),
            focused: true,
            bell_count: 0,
            bounds: TerminalAccessibilityBounds::default(),
        }
    }

    #[test]
    fn text_replacement_and_caret_events_have_standard_object_shape() {
        let previous = snapshot("old", 3);
        let next = snapshot("νέο", 3);
        let mut serial = 1;
        let events = messages(
            || {
                let value = serial;
                serial += 1;
                value
            },
            Some(&previous),
            &next,
            &[
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::CaretMoved,
            ],
        );
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].header.member.as_deref(), Some("TextChanged"));
        assert_eq!(events[1].header.signature, "siiva{sv}");
        assert_eq!(events[2].header.member.as_deref(), Some("TextCaretMoved"));
    }

    #[test]
    fn initial_text_and_focus_events_preserve_scalar_counts_and_state() {
        let next = snapshot("aβ", 2);
        let mut serial = 20;
        let events = messages(
            || {
                let value = serial;
                serial += 1;
                value
            },
            None,
            &next,
            &[
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::FocusChanged,
            ],
        );
        assert_eq!(
            events.len(),
            2,
            "initial publication inserts without deletion"
        );
        let mut text = events[0].body_reader();
        assert_eq!(text.string().unwrap(), "insert");
        assert_eq!(text.i32().unwrap(), 0);
        assert_eq!(text.i32().unwrap(), 2);
        assert_eq!(text.variant_string().unwrap(), "aβ");
        let mut focus = events[1].body_reader();
        assert_eq!(focus.string().unwrap(), "focused");
        assert_eq!(focus.i32().unwrap(), 1);
        assert_eq!(focus.i32().unwrap(), 0);
        assert!(focus.variant_bool().unwrap());
    }
}
