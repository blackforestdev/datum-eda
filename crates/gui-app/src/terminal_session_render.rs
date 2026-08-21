//! Renderer-facing snapshot and surface-size boundary for terminal sessions.

use super::*;

pub(super) fn terminal_tab_label(
    fallback: &str,
    explicit: bool,
    terminal_title: Option<&str>,
) -> String {
    if explicit {
        return fallback.to_string();
    }
    terminal_title
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fallback.to_string())
}

impl TerminalSessionRegistry {
    pub(crate) fn active_render_row_count(&self) -> usize {
        if self.active_pending_id.is_some() {
            return 0;
        }
        self.sessions[self.active_index]
            .core
            .render_row_count()
            .unwrap_or(0)
    }

    pub(crate) fn take_active_render_state(
        &mut self,
    ) -> Result<
        (
            datum_terminal_core::RenderSnapshot,
            Vec<datum_terminal_core::Damage>,
        ),
        crate::terminal_core_adapter::TerminalCoreAdapterError,
    > {
        self.sessions[self.active_index].core.take_render_state()
    }

    #[cfg(test)]
    pub(crate) fn resize_active(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.resize_active_surface(cols, rows, 0, 0)
    }

    pub(crate) fn resize_active_surface(
        &mut self,
        cols: u16,
        rows: u16,
        pixel_width: u32,
        pixel_height: u32,
    ) -> Result<()> {
        if self.active_pending_id.is_some() {
            return Ok(());
        }
        let slot = &mut self.sessions[self.active_index];
        let cols = cols.max(1);
        let rows = rows.max(1);
        if slot.columns != cols || slot.rows != rows {
            slot.session.resize(cols, rows)?;
            slot.columns = cols;
            slot.rows = rows;
        }
        slot.core.resize(cols, rows, pixel_width, pixel_height)?;
        Ok(())
    }
}
