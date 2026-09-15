//! Standard fail-closed D-Bus errors for AT-SPI object dispatch.

use crate::terminal_accessibility_platform::dbus::Message;

pub(super) fn error(serial: u32, call: &Message, name: &str, description: &str) -> Message {
    Message::error(
        serial,
        call.serial,
        call.header.sender.as_deref(),
        name,
        description,
    )
}

pub(super) fn invalid_args() -> (&'static str, &'static str) {
    (
        "org.freedesktop.DBus.Error.InvalidArgs",
        "invalid arguments",
    )
}

pub(super) fn unknown_object() -> (&'static str, &'static str) {
    (
        "org.freedesktop.DBus.Error.UnknownObject",
        "unknown accessible object",
    )
}
