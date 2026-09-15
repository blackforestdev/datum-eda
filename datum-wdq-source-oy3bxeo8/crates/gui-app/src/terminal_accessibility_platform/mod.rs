//! Datum-owned Linux AT-SPI platform bridge.

mod atspi;
#[cfg(test)]
mod atspi_tests;
mod body;
mod connection;
mod dbus;
mod events;
mod worker;

pub(crate) use worker::PlatformBridge;
