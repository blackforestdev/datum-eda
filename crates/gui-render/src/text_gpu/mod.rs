//! Datum-owned text texture pages, glyph instances and submission lifetimes.
pub(crate) mod allocation_host;
mod atlas;
pub(crate) mod budget;
mod draw;
pub(crate) mod lifetime;
pub(crate) mod upload;
pub(crate) mod upload_totals;

pub(crate) use atlas::{Atlas, UploadRequired};
pub(crate) use draw::{Area, Draw};
pub(crate) use lifetime::hold_until_done;
pub use lifetime::{Kind, Observer, Record, ReleasedAllocations, RetirementReason};

#[cfg(all(test, feature = "visual"))]
mod parity;

pub(crate) mod sparse_upload;

pub(crate) mod slot_generations;

pub(crate) mod staging_vec;

pub(crate) mod raster;
