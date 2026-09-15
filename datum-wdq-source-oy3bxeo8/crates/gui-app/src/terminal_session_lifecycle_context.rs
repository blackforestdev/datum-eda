use super::TerminalSession;
use crate::{
    terminal_agent_credential::revoke_session_authority,
    terminal_context::{
        unix_time_ms, update_terminal_lifecycle_file, update_terminal_lifecycle_file_exact,
    },
    terminal_transport::TerminalExitStatus,
};
use anyhow::Result;
use datum_gui_protocol::DatumToolSessionLifecycle;

pub(crate) fn mark_terminal_session_lifecycle(
    session: &TerminalSession,
    lifecycle: DatumToolSessionLifecycle,
    process_exit_code: Option<i32>,
) -> Result<()> {
    if lifecycle != DatumToolSessionLifecycle::Running {
        revoke_session_authority(&session.context_path, &session.session_id, unix_time_ms()?)?;
    }
    for path in [
        &session.context_path,
        &session.latest_context_path,
        &session.session_path,
    ] {
        update_terminal_lifecycle_file(
            path,
            lifecycle,
            process_exit_code,
            Some(session.process_group_id()),
        )?;
    }
    Ok(())
}

pub(crate) fn mark_terminal_session_exit(
    session: &TerminalSession,
    status: TerminalExitStatus,
) -> Result<()> {
    revoke_session_authority(&session.context_path, &session.session_id, unix_time_ms()?)?;
    let (code, signal, core_dumped) = match status {
        TerminalExitStatus::Code(code) => (Some(code), None, None),
        TerminalExitStatus::Signal {
            signal,
            core_dumped,
        } => (None, Some(signal), Some(core_dumped)),
    };
    for path in [
        &session.context_path,
        &session.latest_context_path,
        &session.session_path,
    ] {
        update_terminal_lifecycle_file_exact(
            path,
            DatumToolSessionLifecycle::Exited,
            code,
            signal,
            core_dumped,
            Some(session.process_group_id()),
        )?;
    }
    Ok(())
}
