// commands/ — target home for command implementations, one subdirectory per
// family (e.g. commands/<family>/mod.rs + its split files), following the
// existing command_query/ and command_modify/ directory-module pattern.
//
// Wave 1 skeleton: no files have moved yet. This module re-exports the legacy
// command chains that main.rs previously glob-used, so `crate::commands` is
// already the routing surface main.rs consumes (`use commands::*;`).
// Behavior is identical to the pre-skeleton crate.
//
// Wave 2 lane protocol (one command family per lane):
//   1. `git mv` your family's command_project_<family>*.rs (or command_plan /
//      command_query / command_modify) files into commands/<family>/, with a
//      commands/<family>/mod.rs holding plain `mod x;` decls and the family's
//      pub(crate) re-exports.
//   2. In THIS file only: add `mod <family>;` plus
//      `pub(crate) use self::<family>::*;`.
//   3. Delete the family's `#[path = ...] mod ...;` decls from the old host
//      (command_project.rs) and its named re-export block from the old
//      surface file (command_project_surface.rs). Touch no other shared file.
//   4. Do NOT touch main.rs — it already routes `use commands::*;`.
//
// Notes:
//   - command_modify is deliberately NOT glob-re-exported yet: main.rs never
//     glob-used it (its items are reached by `command_modify::` paths). The
//     modify lane adds `pub(crate) use self::modify::*;` (or a named list)
//     when it moves that directory here.
//   - The command_exec_* layer is NOT moved into commands/; Wave 3 deletes it.
//
// Wave 3 deletes the emptied legacy chain files and the shim re-exports below.
//
// `use super::*;` is the scope prelude for moved families: their
// `use super::*;` chains resolve here, keeping crate-root names visible
// exactly as the old chains did.

#[allow(unused_imports)] // Wave 2 anchor: scope prelude for moved family files.
use super::*;

pub(crate) use crate::command_plan::*;
pub(crate) use crate::command_project::*;
pub(crate) use crate::command_query::*;
