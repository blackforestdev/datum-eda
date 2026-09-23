//! Datum-owned text texture pages, glyph instances and submission lifetimes.
mod atlas;
mod draw;
mod lifetime;

pub(crate) use atlas::Atlas;
pub(crate) use draw::Draw;
pub(crate) use lifetime::hold_until_done;
pub use lifetime::{Kind, Observer, Record};

#[cfg(all(test, feature = "visual"))]
mod parity;
