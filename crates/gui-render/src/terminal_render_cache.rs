//! Damage-driven retained row plans for immutable TerminalCore snapshots.

use datum_gui_protocol::{ReviewWorkspaceState, TerminalLaneState, TerminalSearchMatch};
use datum_gui_viewport::TerminalScreenGeometry;
use datum_terminal_core::{
    Color, Damage, RenderPalette, RenderRowSource, RenderSnapshot, Selection,
};

use super::{HitRegion, HitTarget, Quad, RectPx, TextRun};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct TerminalRenderCache {
    sessions: BTreeMap<String, TerminalSessionRenderCache>,
}

#[derive(Default)]
struct TerminalSessionRenderCache {
    rows: Vec<CachedRow>,
    screen: Option<RectPx>,
    columns: u16,
    palette: Option<RenderPalette>,
    theme: Option<datum_gui_protocol::TerminalTheme>,
    selection: Option<Selection>,
    search_matches: Vec<TerminalSearchMatch>,
    search_match: Option<TerminalSearchMatch>,
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
    pub(crate) fn rebuilt_rows(&self) -> usize {
        self.sessions
            .values()
            .map(|session| session.rebuilt_rows)
            .sum()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn render(
        &mut self,
        state: &ReviewWorkspaceState,
        snapshot: &RenderSnapshot,
        damage: &[Damage],
        geometry: &TerminalScreenGeometry,
        sinks: (&mut Vec<Quad>, &mut Vec<TextRun>, &mut Vec<HitRegion>),
    ) {
        let session_id = state
            .ui
            .terminal
            .active_session_id
            .as_deref()
            .unwrap_or("terminal");
        let focused = state.ui.focus.is_terminal();
        self.render_pane(
            session_id,
            &state.ui.terminal,
            focused,
            snapshot,
            damage,
            geometry,
            HitTarget::TerminalScreen,
            sinks,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_pane(
        &mut self,
        session_id: &str,
        lane: &TerminalLaneState,
        focused: bool,
        snapshot: &RenderSnapshot,
        damage: &[Damage],
        geometry: &TerminalScreenGeometry,
        target: HitTarget,
        sinks: (&mut Vec<Quad>, &mut Vec<TextRun>, &mut Vec<HitRegion>),
    ) {
        self.sessions
            .entry(session_id.to_string())
            .or_default()
            .render(
                session_id, lane, focused, snapshot, damage, geometry, target, sinks,
            );
    }

    pub(crate) fn retain_sessions<'a>(&mut self, session_ids: impl IntoIterator<Item = &'a str>) {
        let visible = session_ids.into_iter().collect::<BTreeSet<_>>();
        self.sessions
            .retain(|session_id, _| visible.contains(session_id.as_str()));
    }

    #[cfg(test)]
    pub(crate) fn cached_session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl TerminalSessionRenderCache {
    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        session_id: &str,
        lane: &TerminalLaneState,
        focused: bool,
        snapshot: &RenderSnapshot,
        damage: &[Damage],
        geometry: &TerminalScreenGeometry,
        target: HitTarget,
        sinks: (&mut Vec<Quad>, &mut Vec<TextRun>, &mut Vec<HitRegion>),
    ) {
        let (panel_quads, text_runs, hit_regions) = sinks;
        let screen: RectPx = geometry.screen.into();
        hit_regions.push(HitRegion {
            target,
            rect: screen,
        });
        panel_quads.push(Quad::from_rect(
            screen,
            super::terminal_core_render::resolve_background(
                Color::Default,
                snapshot.palette(),
                lane.theme,
            ),
        ));
        let rows = snapshot.rows().collect::<Vec<_>>();
        let max_rows = usize::from(geometry.rows);
        let scroll = lane.scroll_offset.min(rows.len().saturating_sub(max_rows));
        let first = rows.len().saturating_sub(max_rows + scroll);
        let visible = rows
            .into_iter()
            .skip(first)
            .take(max_rows)
            .collect::<Vec<_>>();
        let global_dirty = self.screen != Some(screen)
            || self.columns != geometry.columns
            || self.palette.as_ref() != Some(snapshot.palette())
            || self.theme != Some(lane.theme)
            || self.selection != snapshot.selection()
            || self.search_matches != lane.search.highlights
            || self.search_match != lane.search.matched
            || self.session_id.as_deref() != Some(session_id)
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
                    super::terminal_core_render::TerminalRowRenderContext {
                        screen: RectPx {
                            x: screen.x,
                            y: 0.0,
                            width: screen.width,
                            height: screen.height,
                        },
                        y: 0.0,
                        max_columns: usize::from(geometry.columns),
                        metrics: geometry.metrics,
                        theme: lane.theme,
                        search_highlights: &lane.search.highlights,
                        active_search_match: lane.search.matched,
                    },
                    &mut cached.quads,
                    &mut cached.text,
                );
                #[cfg(test)]
                {
                    self.rebuilt_rows += 1;
                }
            }
            let y = screen.y + visible_index as f32 * geometry.metrics.height;
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
                    focused,
                    screen.x,
                    y,
                    geometry.metrics,
                    lane.theme,
                    panel_quads,
                );
            }
        }
        self.screen = Some(screen);
        self.columns = geometry.columns;
        self.palette = Some(snapshot.palette().clone());
        self.theme = Some(lane.theme);
        self.selection = snapshot.selection();
        self.search_matches = lane.search.highlights.clone();
        self.search_match = lane.search.matched;
        self.session_id = Some(session_id.to_string());
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
