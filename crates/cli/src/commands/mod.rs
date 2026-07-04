// commands/ — target home for command implementations, one subdirectory per
// family (e.g. commands/<family>/mod.rs + its split files), following the
// existing command_query/ and command_modify/ directory-module pattern.
//
// Wave 2 complete: every command_project_* family has moved into a
// commands/<family>/ directory module, and the legacy command_project.rs /
// command_project_surface.rs hosts are dissolved (their scope prelude and
// shared core files live here as prelude / support / project_core /
// native_types). `crate::commands` is the routing surface main.rs consumes
// (`use commands::*;`).
//
// Notes:
//   - command_modify is deliberately NOT glob-re-exported: main.rs never
//     glob-used it (its items are reached by `command_modify::` paths). The
//     modify lane adds `pub(crate) use self::modify::*;` (or a named list)
//     when it moves that directory here.
//   - The command_exec_* layer is NOT moved into commands/; Wave 3 deletes it.
//   - command_plan.rs / command_query/ still live at the crate root and are
//     re-exported below; Wave 3 (or a follow-up lane) folds them in.
//
// Wave 3 deletes the emptied legacy chain files and the shim re-exports below.
//
// `use super::*;` is the scope prelude for moved families: their
// `use super::*;` chains resolve here, keeping crate-root names visible
// exactly as the old chains did.

#[allow(unused_imports)] // Wave 2 anchor: scope prelude for moved family files.
use super::*;

mod artifacts;
mod board;
mod check;
mod drill;
mod forward_annotation;
mod gerber;
mod imports;
mod inventory;
mod library;
mod manufacturing;
// Shared core files from the dissolved command_project.rs /
// command_project_surface.rs hosts (Wave 2 endgame): the scope prelude,
// project-core loaders, cross-family support helpers, and native root types.
mod native_types;
mod output_jobs;
mod pool;
mod prelude;
mod project;
mod project_core;
mod route;
mod schematic;
mod standards;
mod support;
// Cross-family CLI view helpers (project create, rules) whose owning
// families have not yet moved into commands/.
mod views;

pub(crate) use self::artifacts::*;
pub(crate) use self::board::*;
pub(crate) use self::check::*;
pub(crate) use self::drill::*;
pub(crate) use self::forward_annotation::*;
pub(crate) use self::gerber::*;
pub(crate) use self::imports::*;
pub(crate) use self::inventory::*;
pub(crate) use self::library::*;
pub(crate) use self::manufacturing::*;
pub(crate) use self::native_types::{
    NativeBoardRoot, NativeComponentPad, NativeOutline, NativePoint,
};
pub(crate) use self::output_jobs::*;
pub(crate) use self::pool::*;
pub(crate) use self::prelude::*;
pub(crate) use self::project::*;
pub(crate) use self::project_core::*;
pub(crate) use self::route::*;
pub(crate) use self::schematic::*;
pub(crate) use self::standards::*;
pub(crate) use self::support::*;
pub(crate) use self::views::*;

pub(crate) use crate::command_plan::*;
pub(crate) use crate::command_query::*;
