// commands/ — home of the command implementations, one subdirectory per
// family (commands/<family>/mod.rs + its split files), alongside the
// command_query/ and command_modify/ directory modules at the crate root.
//
// Phase 5 complete: the command_exec_* forwarding layer is dissolved. Each
// clap args struct carries an inherent `run(self, format)` method in its
// owning family, and commands/dispatch.rs is the single router — the
// exhaustive ProjectCommands match plus the top-level Commands match
// (execute_with_exit_code). Shared core files live here as prelude /
// support / project_core / native_types.
//
// Notes:
//   - command_modify is deliberately NOT glob-re-exported: its items are
//     reached by `command_modify::` paths (main.rs names the two view types
//     it re-exports).
//   - command_plan.rs / command_query/ still live at the crate root and are
//     re-exported below.
//
// `use super::*;` is the scope prelude for the families: their
// `use super::*;` chains resolve here, keeping crate-root names visible
// exactly as the old chains did.

#[allow(unused_imports)] // scope prelude anchor for family files.
use super::*;

mod artifacts;
mod board;
mod check;
// The single router: the exhaustive ProjectCommands match and the top-level
// Commands match (execute_with_exit_code, called by path from main.rs).
pub(crate) mod dispatch;
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
