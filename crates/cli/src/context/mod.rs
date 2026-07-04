// context/ — target home for the terminal/session context family
// (command_context.rs + command_context_*.rs). Those files are currently
// declared inside the exec layer's private subtree: command_exec_surface.rs
// declares command_context, which declares its command_context_* children via
// #[path].
//
// Wave 1 skeleton: intentionally empty. The legacy context modules are sealed
// inside command_exec's private module subtree, so there is nothing nameable
// to re-export from here yet. main.rs already routes `use context::*;`, so
// the Wave 2 lane's pub(crate) re-exports reach crate scope without touching
// main.rs.
//
// Wave 2 lane protocol (context lane):
//   1. `git mv crates/cli/src/command_context*.rs crates/cli/src/context/`
//      (drop the `command_` prefix in the new file names).
//   2. In THIS file only: add plain `mod x;` lines plus pub(crate)
//      re-exports for the entry points the exec dispatch calls
//      (query_context_envelope, refresh_context_envelope,
//      query_context_session_events, query_context_session_activity, ...).
//   3. Delete `#[path = "command_context.rs"] mod command_context;` from
//      command_exec_surface.rs; the exec callers then resolve the entry
//      points through crate scope via main.rs's `use context::*;`.
//   4. Do NOT touch main.rs.
//
// `use super::*;` is the scope prelude for moved files: their
// `use super::*;` resolves here, keeping crate-root names (engine types,
// serde, cli_args types, ...) visible exactly as the old exec-surface chain
// did.

#[allow(unused_imports)] // Wave 2 anchor: scope prelude for moved family files.
use super::*;
