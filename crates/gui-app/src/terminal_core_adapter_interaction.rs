//! TerminalCore-owned input, selection, search, link, and accessibility projection.

use datum_terminal_core::{
    CellContent, Damage, FocusInput, HyperlinkId, ImeInput, InputDisposition, KeyInput,
    LogicalPoint, MouseInput, RenderRow, RenderRowSource, RenderSnapshot, SearchCursor,
    SearchQuery, SearchResult, Selection, SelectionScope,
};

use super::{TerminalCoreAdapterError, TerminalCoreSessionAdapter};
use crate::terminal_accessibility::{TerminalAccessibilityLink, TerminalAccessibilitySnapshot};

impl TerminalCoreSessionAdapter {
    pub(crate) fn encode_key(
        &self,
        input: &KeyInput,
    ) -> Result<InputDisposition, TerminalCoreAdapterError> {
        self.core
            .encode_key(input)
            .map_err(TerminalCoreAdapterError::Input)
    }

    pub(crate) fn encode_focus(
        &self,
        input: FocusInput,
    ) -> Result<InputDisposition, TerminalCoreAdapterError> {
        self.core
            .encode_focus(input)
            .map_err(TerminalCoreAdapterError::Input)
    }

    pub(crate) fn encode_ime(
        &self,
        input: &ImeInput,
    ) -> Result<InputDisposition, TerminalCoreAdapterError> {
        self.core
            .encode_ime(input)
            .map_err(TerminalCoreAdapterError::Input)
    }

    pub(crate) fn encode_paste(
        &self,
        text: &str,
    ) -> Result<InputDisposition, TerminalCoreAdapterError> {
        self.core
            .encode_paste(text)
            .map_err(TerminalCoreAdapterError::Input)
    }

    pub(crate) fn encode_mouse(
        &self,
        input: MouseInput,
    ) -> Result<InputDisposition, TerminalCoreAdapterError> {
        self.core
            .encode_mouse(input)
            .map_err(TerminalCoreAdapterError::Input)
    }

    pub(crate) fn logical_point_at_visible_cell(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        visible_row: usize,
        column: usize,
    ) -> Result<Option<LogicalPoint>, TerminalCoreAdapterError> {
        let snapshot = self
            .core
            .render_snapshot()
            .map_err(TerminalCoreAdapterError::Snapshot)?;
        Ok(
            visible_row_from_snapshot(&snapshot, visible_rows, scroll_offset, visible_row)
                .map(|row| logical_point_in_row(row, column)),
        )
    }

    pub(crate) fn set_selection(
        &mut self,
        anchor: LogicalPoint,
        focus: LogicalPoint,
        scope: SelectionScope,
    ) -> Result<(), TerminalCoreAdapterError> {
        self.core
            .set_selection(Selection::new(anchor, focus, scope))
            .map_err(TerminalCoreAdapterError::Selection)?;
        self.merge_render_damage(&[Damage::Full]);
        Ok(())
    }

    pub(crate) fn clear_selection(&mut self) {
        if self.core.selection().is_some() {
            self.core.clear_selection();
            self.merge_render_damage(&[Damage::Full]);
        }
    }

    pub(crate) fn copy_selection(&self) -> Result<String, TerminalCoreAdapterError> {
        self.core
            .copy_selection()
            .map(|text| text.into_string())
            .map_err(TerminalCoreAdapterError::Selection)
    }

    #[allow(dead_code)]
    pub(crate) fn search(
        &self,
        query: &SearchQuery,
        cursor: SearchCursor,
    ) -> Result<SearchResult, TerminalCoreAdapterError> {
        self.core
            .search(query, cursor)
            .map_err(TerminalCoreAdapterError::Search)
    }

    #[allow(dead_code)]
    pub(crate) fn hyperlink_at_visible_cell(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        visible_row: usize,
        column: usize,
    ) -> Result<Option<(HyperlinkId, String)>, TerminalCoreAdapterError> {
        let snapshot = self
            .core
            .render_snapshot()
            .map_err(TerminalCoreAdapterError::Snapshot)?;
        let Some(row) =
            visible_row_from_snapshot(&snapshot, visible_rows, scroll_offset, visible_row)
        else {
            return Ok(None);
        };
        let Some(id) = row.cells().get(column).and_then(|cell| cell.hyperlink) else {
            return Ok(None);
        };
        Ok(self
            .core
            .state()
            .hyperlink(id)
            .map(|link| (id, link.uri().to_owned())))
    }

    pub(crate) fn accessibility_snapshot(
        &self,
        visible_rows: usize,
        scroll_offset: usize,
        focused: bool,
    ) -> Result<TerminalAccessibilitySnapshot, TerminalCoreAdapterError> {
        let snapshot = self
            .core
            .render_snapshot()
            .map_err(TerminalCoreAdapterError::Snapshot)?;
        let rows = visible_rows_from_snapshot(&snapshot, visible_rows, scroll_offset);
        let selection = snapshot.selection();
        let cursor = snapshot.cursor().position;
        let mut text = String::new();
        let mut caret = None;
        let mut selection_offsets = [None, None];
        let mut links = Vec::new();
        let mut active_link: Option<(HyperlinkId, usize)> = None;

        for (row_index, row) in rows.iter().enumerate() {
            if row_index != 0 {
                finish_accessibility_link(&mut active_link, text.chars().count(), &mut links, self);
                text.push('\n');
            }
            let screen_cursor_row = matches!(
                row.source(),
                RenderRowSource::Screen { row } if row == cursor.row.get()
            );
            let mut logical_cluster = row.logical_start().cluster;
            for (column, cell) in row.cells().iter().enumerate() {
                let offset = text.chars().count();
                if screen_cursor_row && column == usize::from(cursor.column.get()) {
                    caret = Some(offset);
                }
                if let Some(selection) = selection {
                    let point = LogicalPoint {
                        line: row.logical_start().line,
                        cluster: logical_cluster,
                    };
                    if point == selection.anchor() {
                        selection_offsets[0] = Some(offset);
                    }
                    if point == selection.focus() {
                        selection_offsets[1] = Some(offset);
                    }
                }
                if active_link.map(|(id, _)| id) != cell.hyperlink {
                    finish_accessibility_link(&mut active_link, offset, &mut links, self);
                    active_link = cell.hyperlink.map(|id| (id, offset));
                }
                match &cell.content {
                    CellContent::Cluster(cluster) => text.push_str(cluster.text()),
                    CellContent::Empty => text.push(' '),
                    CellContent::Continuation { .. } => continue,
                }
                logical_cluster = logical_cluster.saturating_add(1);
            }
        }
        finish_accessibility_link(&mut active_link, text.chars().count(), &mut links, self);
        let selection = match selection_offsets {
            [Some(anchor), Some(focus)] => Some(if anchor <= focus {
                (anchor, focus.saturating_add(1))
            } else {
                (focus, anchor.saturating_add(1))
            }),
            _ => None,
        };
        Ok(TerminalAccessibilitySnapshot {
            session_id: self.session_id.clone(),
            title: self
                .core
                .state()
                .title()
                .map_or_else(|| "Terminal".into(), |title| title.as_str().to_owned()),
            caret: caret.unwrap_or_else(|| text.chars().count()),
            text,
            selection,
            links,
            focused,
            bell_count: self.bell_count,
        })
    }

    pub(crate) fn render_row_count(&self) -> Result<usize, TerminalCoreAdapterError> {
        Ok(self
            .core
            .render_snapshot()
            .map_err(TerminalCoreAdapterError::Snapshot)?
            .rows()
            .len())
    }
}

fn visible_row_from_snapshot(
    snapshot: &RenderSnapshot,
    visible_rows: usize,
    scroll_offset: usize,
    visible_row: usize,
) -> Option<&RenderRow> {
    visible_rows_from_snapshot(snapshot, visible_rows, scroll_offset)
        .get(visible_row.min(visible_rows.max(1).saturating_sub(1)))
        .copied()
}

fn visible_rows_from_snapshot(
    snapshot: &RenderSnapshot,
    visible_rows: usize,
    scroll_offset: usize,
) -> Vec<&RenderRow> {
    let rows = snapshot.rows().collect::<Vec<_>>();
    let shown = visible_rows.max(1);
    let scroll = scroll_offset.min(rows.len().saturating_sub(shown));
    let first = rows.len().saturating_sub(shown + scroll);
    rows.into_iter().skip(first).take(shown).collect()
}

fn logical_point_in_row(row: &RenderRow, column: usize) -> LogicalPoint {
    let mut cluster = row.logical_start().cluster;
    for cell in row.cells().iter().take(column) {
        if !matches!(cell.content, CellContent::Continuation { .. }) {
            cluster = cluster.saturating_add(1);
        }
    }
    LogicalPoint {
        line: row.logical_start().line,
        cluster,
    }
}

fn finish_accessibility_link(
    active: &mut Option<(HyperlinkId, usize)>,
    end: usize,
    links: &mut Vec<TerminalAccessibilityLink>,
    adapter: &TerminalCoreSessionAdapter,
) {
    let Some((id, start)) = active.take() else {
        return;
    };
    if start == end {
        return;
    }
    if let Some(link) = adapter.core.state().hyperlink(id) {
        links.push(TerminalAccessibilityLink {
            start,
            end,
            uri: link.uri().to_owned(),
        });
    }
}
