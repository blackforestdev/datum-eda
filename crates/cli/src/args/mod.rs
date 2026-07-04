// args/ — target home for all CLI argument/parser types (clap structs and
// enums), one file per family.
//
// Wave 1 skeleton: no files have moved yet. This module re-exports the legacy
// cli_args chain (cli_args.rs -> cli_args_root.rs -> cli_args_surface.rs) so
// `crate::args` is already the routing surface main.rs consumes
// (`use args::*;`). Behavior is identical to the pre-skeleton crate.
//
// Wave 2 lane protocol (one arg family per lane):
//   1. `git mv crates/cli/src/cli_args_<family>.rs
//       crates/cli/src/args/<family>.rs` (drop the `cli_args_` prefix).
//   2. In THIS file only: add a plain `mod <family>;` line plus
//      `pub(crate) use self::<family>::*;` (or the family's named list).
//   3. Delete that family's `#[path = "cli_args_<family>.rs"] mod ...;` line
//      from cli_args_root.rs and its `pub(crate) use
//      self::cli_args_<family>::...` block from cli_args_surface.rs. Touch no
//      other shared file.
//   4. Do NOT touch main.rs — it already routes `use args::*;`.
//   If a moved file fails to resolve a name it previously got through the
//   chain, add the needed `use` line inside the moved file itself — never to
//   a shared surface file.
//
// Wave 3 deletes the emptied legacy chain files and the shim re-export below.
//
// `use super::*;` is the scope prelude for moved files: their `use super::*;`
// resolves here, and this glob keeps crate-root names visible exactly as the
// old chains did.

#[allow(unused_imports)] // Wave 2 anchor: scope prelude for moved family files.
use super::*;

pub(crate) use crate::cli_args::*;
