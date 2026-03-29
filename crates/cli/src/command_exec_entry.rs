use super::*;

pub(crate) fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    super::command_exec_surface::execute_with_exit_code(cli)
}
