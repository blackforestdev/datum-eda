//! Damage-driven retained row plans for immutable TerminalCore snapshots.

use datum_gui_protocol::ReviewWorkspaceState;
use datum_gui_viewport::{TERMINAL_CELL_HEIGHT_PX, TerminalScreenGeometry};
use datum_terminal_core::{
    Color, Damage, RenderPalette, RenderRowSource, RenderSnapshot, Selection,
};

use super::{HitRegion, HitTarget, Quad, RectPx, TextRun};

#[derive(Default)]
pub struct TerminalRenderCache {
    rows: Vec<CachedRow>,
    screen: Option<RectPx>,
    columns: u16,
    palette: Option<RenderPalette>,
    selection: Option<Selection>,
    session_id: Option<String>,
    scroll_offset: usize,
    #[cfg(test)]
    rebuilt_rows: usize,
}

struct CachedRow {
    source: RenderRowSource,
    quads: Vec<Quad>,
    text: Vec<TextRun>,
}

impl TerminalRenderCache {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) const fn rebuilt_rows(&self) -> usize {
        self.rebuilt_rows
    }

    pub(crate) fn render(
        &mut self,
        state: &ReviewWorkspaceState,
        snapshot: &RenderSnapshot,
        damage: &[Damage],
        geometry: &TerminalScreenGeometry,
        sinks: (&mut Vec<Quad>, &mut Vec<TextRun>, &mut Vec<HitRegion>),
    ) {
        let (panel_quads, text_runs, hit_regions) = sinks;
        let screen: RectPx = geometry.screen.into();
        hit_regions.push(HitRegion {
            target: HitTarget::TerminalScreen,
            rect: screen,
        });
        panel_quads.push(Quad::from_rect(
            screen,
            super::terminal_core_render::resolve_background(Color::Default, snapshot.palette()),
        ));
        let rows = snapshot.rows().collect::<Vec<_>>();
        let max_rows = usize::from(geometry.rows);
        let scroll = state
            .ui
            .terminal
            .scroll_offset
            .min(rows.len().saturating_sub(max_rows));
        let first = rows.len().saturating_sub(max_rows + scroll);
        let visible = rows
            .into_iter()
            .skip(first)
            .take(max_rows)
            .collect::<Vec<_>>();
        let global_dirty = self.screen != Some(screen)
            || self.columns != geometry.columns
            || self.palette.as_ref() != Some(snapshot.palette())
            || self.selection != snapshot.selection()
            || self.session_id != state.ui.terminal.active_session_id
            || self.scroll_offset != scroll
            || damage.iter().any(|entry| {
                matches!(
                    entry,
                    Damage::Full
                        | Damage::History
                        | Damage::Palette(_)
                        | Damage::Graphics
                        | Damage::Scroll { .. }
                )
            });
        if self.rows.len() != visible.len() {
            self.rows.clear();
            self.rows.resize_with(visible.len(), || CachedRow {
                source: RenderRowSource::History { index: usize::MAX },
                quads: Vec::new(),
                text: Vec::new(),
            });
        }
        for (visible_index, row) in visible.iter().enumerate() {
            let cached = &mut self.rows[visible_index];
            let dirty = global_dirty
                || cached.source != row.source()
                || row_is_damaged(row.source(), damage);
            if dirty {
                cached.source = row.source();
                cached.quads.clear();
                cached.text.clear();
                super::terminal_core_render::render_row(
                    row,
                    snapshot,
                    RectPx {
                        x: screen.x,
                        y: 0.0,
                        width: screen.width,
                        height: screen.height,
                    },
                    0.0,
                    usize::from(geometry.columns),
                    &mut cached.quads,
                    &mut cached.text,
                );
                #[cfg(test)]
                {
                    self.rebuilt_rows += 1;
                }
            }
            let y = screen.y + visible_index as f32 * TERMINAL_CELL_HEIGHT_PX;
            panel_quads.extend(cached.quads.iter().copied().map(|mut quad| {
                for point in &mut quad.points {
                    point.1 += y;
                }
                quad
            }));
            text_runs.extend(cached.text.iter().cloned().map(|mut run| {
                run.y += y;
                if let Some(bounds) = &mut run.clip_bounds {
                    bounds.y += y;
                    bounds.height = bounds.height.min(screen.y + screen.height - bounds.y);
                }
                run
            }));
            if matches!(row.source(), RenderRowSource::Screen { row } if row == snapshot.cursor().position.row.get())
                && snapshot.cursor().visible
            {
                super::terminal_core_render::render_cursor(
                    snapshot,
                    state.ui.focus.is_terminal(),
                    screen.x,
                    y,
                    panel_quads,
                );
            }
        }
        self.screen = Some(screen);
        self.columns = geometry.columns;
        self.palette = Some(snapshot.palette().clone());
        self.selection = snapshot.selection();
        self.session_id = state.ui.terminal.active_session_id.clone();
        self.scroll_offset = scroll;
    }
}

fn row_is_damaged(source: RenderRowSource, damage: &[Damage]) -> bool {
    let RenderRowSource::Screen { row } = source else {
        return damage.contains(&Damage::History);
    };
    damage.iter().any(|entry| match *entry {
        Damage::Cell(point) => point.row.get() == row,
        Damage::Row(damaged) => damaged.get() == row,
        Damage::Rows { first, last } => (first.get()..=last.get()).contains(&row),
        Damage::Cursor | Damage::Title => false,
        Damage::Scroll { .. }
        | Damage::Palette(_)
        | Damage::History
        | Damage::Graphics
        | Damage::Full => true,
    })
}
