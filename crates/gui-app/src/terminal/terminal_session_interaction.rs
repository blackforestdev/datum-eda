//! Native TerminalCore interaction boundary for the active PTY session.
//!
//! Application chrome decides whether an interaction belongs to the terminal;
//! TerminalCore alone decides which bytes, if any, the child receives.

use anyhow::Result;
use datum_terminal_core::{
    FocusInput, ImeInput, InputDisposition, KeyInput, LogicalPoint, MouseInput, SearchBatch,
    SearchMatch, SearchMatchState, SearchQuery, SelectionScope,
};

use super::TerminalSessionRegistry;

impl TerminalSessionRegistry {
    pub(crate) fn encode_active_key(&self, input: &KeyInput) -> Result<Option<Vec<u8>>> {
        self.ensure_active_ready()?;
        Ok(disposition_bytes(
            self.sessions[self.active_index].core.encode_key(input)?,
        ))
    }

    pub(crate) fn encode_active_focus(&self, input: FocusInput) -> Result<Option<Vec<u8>>> {
        self.ensure_active_ready()?;
        Ok(disposition_bytes(
            self.sessions[self.active_index].core.encode_focus(input)?,
        ))
    }

    pub(crate) fn encode_active_ime(&self, input: &ImeInput) -> Result<Option<Vec<u8>>> {
        self.ensure_active_ready()?;
        Ok(disposition_bytes(
            self.sessions[self.active_index].core.encode_ime(input)?,
        ))
    }

    pub(crate) fn encode_active_paste(&self, text: &str) -> Result<Option<Vec<u8>>> {
        self.ensure_active_ready()?;
        Ok(disposition_bytes(
            self.sessions[self.active_index].core.encode_paste(text)?,
        ))
    }

    pub(crate) fn encode_active_mouse(&self, input: MouseInput) -> Result<Option<Vec<u8>>> {
        self.ensure_active_ready()?;
        Ok(disposition_bytes(
            self.sessions[self.active_index].core.encode_mouse(input)?,
        ))
    }

    pub(crate) fn active_logical_point_at(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        visible_row: usize,
        column: usize,
    ) -> Result<Option<LogicalPoint>> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .logical_point_at_visible_cell(visible_rows, scroll_offset, visible_row, column)?)
    }

    pub(crate) fn set_active_selection(
        &mut self,
        anchor: LogicalPoint,
        focus: LogicalPoint,
        scope: SelectionScope,
    ) -> Result<()> {
        self.ensure_active_ready()?;
        self.sessions[self.active_index]
            .core
            .set_selection(anchor, focus, scope)?;
        Ok(())
    }

    pub(crate) fn clear_active_selection(&mut self) -> Result<()> {
        self.ensure_active_ready()?;
        self.sessions[self.active_index].core.clear_selection();
        Ok(())
    }

    pub(crate) fn copy_active_selection(&self) -> Result<String> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index].core.copy_selection()?)
    }

    pub(crate) fn search_all_active(&self, query: &SearchQuery) -> Result<SearchBatch> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index].core.search_all(query)?)
    }

    pub(crate) fn active_search_match_state(
        &self,
        matched: SearchMatch,
    ) -> Result<SearchMatchState> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .search_match_state(matched))
    }

    pub(crate) fn active_scroll_offset_for_logical_point(
        &self,
        visible_rows: usize,
        point: LogicalPoint,
    ) -> Result<Option<usize>> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .scroll_offset_for_logical_point(visible_rows, point)?)
    }

    #[allow(dead_code)]
    pub(crate) fn active_hyperlink_at(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        visible_row: usize,
        column: usize,
    ) -> Result<Option<(datum_terminal_core::HyperlinkId, String)>> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .hyperlink_at_visible_cell(visible_rows, scroll_offset, visible_row, column)?)
    }

    pub(crate) fn active_link_target_at(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        visible_row: usize,
        column: usize,
        current_working_directory: Option<&str>,
    ) -> Result<Option<datum_gui_protocol::TerminalLinkTarget>> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .link_target_at_visible_cell(
                visible_rows,
                scroll_offset,
                visible_row,
                column,
                current_working_directory,
            )?)
    }

    pub(crate) fn active_accessibility_snapshot(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        focused: bool,
    ) -> Result<crate::terminal_accessibility::TerminalAccessibilitySnapshot> {
        self.ensure_active_ready()?;
        Ok(self.sessions[self.active_index]
            .core
            .accessibility_snapshot(visible_rows, scroll_offset, focused)?)
    }

    fn ensure_active_ready(&self) -> Result<()> {
        if self.active_pending_id.is_some() {
            anyhow::bail!("terminal session is still starting");
        }
        Ok(())
    }
}

fn disposition_bytes(disposition: InputDisposition) -> Option<Vec<u8>> {
    disposition.bytes().map(<[u8]>::to_vec)
}

#[cfg(test)]
mod tests {
    use super::disposition_bytes;
    use datum_terminal_core::InputDisposition;

    #[test]
    fn local_and_ignored_core_input_never_become_pty_bytes() {
        assert_eq!(disposition_bytes(InputDisposition::LocalOnly), None);
        assert_eq!(disposition_bytes(InputDisposition::Ignored), None);
    }
}
