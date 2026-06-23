use super::*;

pub(crate) fn execute_journal_command(
    format: &OutputFormat,
    action: JournalCommands,
) -> Result<(String, i32)> {
    match action {
        JournalCommands::List(JournalListArgs { path }) => Ok((
            render_output(format, &query_native_project_journal_list(&path)?),
            0,
        )),
        JournalCommands::Show(JournalShowArgs { path, transaction }) => Ok((
            render_output(
                format,
                &query_native_project_journal_show(&path, transaction)?,
            ),
            0,
        )),
        JournalCommands::Undo(ProjectUndoArgs {
            path,
            expected_model_revision,
            expected_tip_transaction,
        }) => execute_native_project_journal_undo(
            format,
            &path,
            expected_model_revision.as_deref(),
            expected_tip_transaction,
        ),
        JournalCommands::Redo(ProjectRedoArgs {
            path,
            expected_model_revision,
            expected_tip_transaction,
        }) => execute_native_project_journal_redo(
            format,
            &path,
            expected_model_revision.as_deref(),
            expected_tip_transaction,
        ),
    }
}
