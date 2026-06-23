use super::*;

pub(super) fn execute_context_command(
    format: &OutputFormat,
    action: ContextCommands,
) -> Result<(String, i32)> {
    match action {
        ContextCommands::Get(args) => Ok((
            render_output(format, &command_context::query_context_envelope(&args)?),
            0,
        )),
        ContextCommands::Refresh(args) => Ok((
            render_output(format, &command_context::refresh_context_envelope(&args)?),
            0,
        )),
        ContextCommands::SessionEvents(args) => Ok((
            render_output(
                format,
                &command_context::query_context_session_events(&args)?,
            ),
            0,
        )),
        ContextCommands::SessionActivity(args) => Ok((
            render_output(
                format,
                &command_context::query_context_session_activity(&args)?,
            ),
            0,
        )),
    }
}
