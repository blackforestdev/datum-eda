use super::*;
#[path = "command_exec_entry.rs"]
mod command_exec_entry;
// pub(crate): commands/dispatch.rs (the unified project router) reaches the
// exec layer through this module's re-exports until the exec fns convert to
// run() impls in the next wave.
#[path = "command_exec_surface.rs"]
pub(crate) mod command_exec_surface;

pub(crate) use self::command_exec_entry::execute_with_exit_code;
