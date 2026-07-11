//! Datum universal viewport toolkit — consumer-side shared mechanism.
//!
//! Governed by decision `PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING` and
//! `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`. This crate holds the
//! shared viewport mechanism (grid, camera, stroke, hit-test, snap, …) that
//! every `EditorViewport` surface consumes. It lives on the consumer side of
//! the decision-014 compile-time fence: the engine, daemon, protocol, and
//! persisted formats never depend on it (UVT-002).
//!
//! ## Slices S0 → S1a
//!
//! S0 landed the [`WeightClass`] stroke model (spec §4) plus the
//! [`ViewportProfile`] / [`GridConfig`] scaffold (spec §1.3, §5). S1a lands the
//! first real mechanism and first consumer wiring: the shared
//! [`grid::GridEngine`] (spec §5), onto which gui-render's board grid repoints
//! byte-identically. The crate stays std-only (UVT-002).

pub mod grid;
pub mod profile;
pub mod stroke;

pub use grid::{AxisProjection, GridBounds, GridEngine, GridLine, GridViewport};
pub use profile::{GridConfig, GridMode, GridTier, ViewportProfile};
pub use stroke::WeightClass;
