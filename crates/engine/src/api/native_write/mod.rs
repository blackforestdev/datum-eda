//! The single engine-owned authoring surface for native design mutations.
//!
//! Every native write — CLI, daemon, GUI, MCP — is authored here: builders in
//! the per-domain family modules compose typed [`crate::substrate::Operation`]s
//! into a guarded, revision-stamped [`PreparedWrite`]; callers are thin
//! argument-parsers over this facade and never hand-roll batches. Builders
//! are build-only (they return a [`PreparedWrite`] and never touch disk);
//! committing is the separate [`commit_prepared`] step through the one
//! journaled `commit()` path, so proposal and dry-run flows share the exact
//! same builders as direct commits.
//!
//! Module map:
//! - [`context`] — provenance, batch building, build/commit split (the
//!   contract every family builds on)
//! - [`guards`] — object-revision guard insertion
//! - [`ids`] — deterministic v5 object-id derivation conventions
//! - the remaining family modules are the declared migration targets for the
//!   CLI's hand-rolled operation authoring; they are populated family by
//!   family and this module list is final (later migrations edit only their
//!   own family file).

pub mod artifacts;
pub mod board_annotations;
pub mod board_components;
pub mod board_layout;
pub mod board_routing;
pub mod component_instances;
pub mod context;
pub mod forward_annotation;
pub mod genesis;
pub mod guards;
pub mod ids;
pub mod imports;
pub mod library;
pub mod library_footprint;
pub mod library_pin_pad_map;
pub mod manufacturing;
pub mod output_jobs;
pub mod project;
pub mod schematic_connectivity;
pub mod schematic_sheets;
pub mod schematic_symbols;
pub mod waivers;

#[cfg(test)]
mod test_support;

pub use context::{BatchComposer, PreparedWrite, WriteProvenance, build_batch, commit_prepared};
