//! Datum-owned text texture pages, glyph instances and submission lifetimes.
mod atlas;
pub(crate) mod budget;
mod draw;
pub(crate) mod lifetime;
pub(crate) mod upload;
pub(crate) mod upload_totals;

pub(crate) use atlas::{Atlas, UploadRequired};
pub(crate) use draw::{Area, Draw};
pub(crate) use lifetime::hold_until_done;
pub use lifetime::{Kind, Observer, Record};

#[cfg(all(test, feature = "visual"))]
mod parity;

pub(crate) mod sparse_upload;
