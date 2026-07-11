//! Datum universal viewport toolkit — consumer-side shared mechanism.
//!
//! Governed by decision `PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING` and
//! `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`. This crate holds the
//! shared viewport mechanism (grid, camera, stroke, hit-test, snap, …) that
//! every `EditorViewport` surface consumes. It lives on the consumer side of
//! the decision-014 compile-time fence: the engine, daemon, protocol, and
//! persisted formats never depend on it (UVT-002).
//!
//! ## Landed slices
//!
//! S0 landed the [`WeightClass`] stroke model (spec §4) plus the
//! [`ViewportProfile`] / [`GridConfig`] scaffold (spec §1.3, §5). S1a lands the
//! first real mechanism and first consumer wiring: the shared
//! [`grid::GridEngine`] (spec §5), onto which gui-render's board grid repoints
//! byte-identically. S4 adds [`InteractionEngine`] plus typed per-surface hover
//! and cursor policy. Its only non-std boundary dependency is consumer-side GUI
//! protocol state (UVT-002).

pub mod editor;
pub mod grid;
pub mod hit;
pub mod interaction;
pub mod profile;
pub mod stroke;

pub use editor::{EditorViewport, ScreenRectPx};
pub use grid::{
    AxisProjection, GRID_COARSEN_PX, GRID_HIDE_FLOOR_PX, GRID_REFINE_PX, GridEngine, GridLine,
    GridLodState, GridViewport, MAX_GRID_PRIMITIVES,
};
pub use hit::{DEFAULT_HIT_QUERY_BUDGET, HitQuery, HitRegion, HitShape, SpatialHitIndex};
pub use interaction::InteractionEngine;
pub use profile::{
    CursorConfig, GridConfig, GridMark, GridMode, GridTier, HoverConfig, ViewportProfile,
};
pub use stroke::WeightClass;
