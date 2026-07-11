//! Datum universal viewport toolkit — consumer-side shared mechanism.
//!
//! Governed by decision `PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING` and
//! `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`. This crate holds the
//! shared viewport mechanism (grid, camera, stroke, hit-test, snap, …) that
//! every `EditorViewport` surface consumes. It lives on the consumer side of
//! the decision-014 compile-time fence: the engine, daemon, protocol, and
//! persisted formats never depend on it (UVT-002).
//!
//! ## Slice S0
//!
//! This is the first slice of the crate: the [`StrokeWeightModel`] (spec §4)
//! plus scaffold skeletons for [`ViewportProfile`] / [`GridConfig`] (spec §1.3,
//! §5) that later slices (S1+) populate. It introduces no consumer wiring and
//! no external dependencies — std only.

pub mod stroke;
pub mod profile;

pub use profile::{GridConfig, GridMode, ViewportProfile};
pub use stroke::WeightClass;
