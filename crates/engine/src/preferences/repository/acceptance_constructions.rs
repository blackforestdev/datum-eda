//! Acceptance/test precondition-construction surface (GP-CM05E, PM-authorized).
//!
//! BINDING: the re-exports below exist SOLELY so the engine-surface acceptance
//! harness (`crates/engine/src/preferences/acceptance_support`) can construct the
//! valid-but-unusual stored generations that a DIFFERENT engine version legitimately
//! wrote and THIS version must handle on read/migration (an unregistered key, a stale
//! schema_version, a retired alias). It does this through the engine's OWN
//! generation-write (`write_generation`) exactly as this crate's repository unit tests
//! do (see `repository/tests.rs`), so nothing escapes the journal.
//!
//! This is NOT a product path. ANY product-code call through this surface is a defect
//! of the same class as a private writer bypassing `commit()`. The surface is
//! crate-internal (`pub(crate)`), named, and greppable precisely so that constraint
//! can be enforced by review. Authorized by the PM seed ruling (write_generation with
//! a hand-built `RepositoryState` is the engine's own test construction; crate-internal
//! visibility only, no public API or behavior change).

pub(crate) use super::io::{next_generation_number, promote_head, write_generation};
pub(crate) use super::model::RepositoryState;
