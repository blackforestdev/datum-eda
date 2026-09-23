//! Datum-owned text texture pages, glyph instances and submission lifetimes.
mod atlas;
mod budget;
mod draw;
mod lifetime;

pub(crate) use atlas::Atlas;
pub(crate) use draw::{Area, Draw};
pub(crate) use lifetime::hold_until_done;
pub use lifetime::{Kind, Observer, Record};

#[cfg(all(test, feature = "visual"))]
mod parity;
