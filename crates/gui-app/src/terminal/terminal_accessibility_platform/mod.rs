//! Datum-owned Linux AT-SPI platform bridge.

#[path = "atspi.rs"]
mod atspi;
#[cfg(test)]
#[path = "atspi_tests.rs"]
mod atspi_tests;
#[path = "body.rs"]
mod body;
#[path = "connection.rs"]
mod connection;
#[path = "dbus.rs"]
mod dbus;
#[path = "events.rs"]
mod events;
#[path = "worker.rs"]
mod worker;

pub(crate) use worker::PlatformBridge;

#[cfg(test)]
#[path = "../../../../gui-protocol/tests/support/native_selection_fixture.rs"]
pub(crate) mod selection_fixture;
