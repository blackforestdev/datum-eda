//! Keyboard-owned search chrome over TerminalCore's bounded search authority.

use datum_gui_protocol::{TerminalSearchMatch, TerminalSearchPoint};
use datum_terminal_core::{
    LogicalLineId, LogicalPoint, SearchCase, SearchDirection, SearchMatchState, SearchQuery,
    grapheme_indices,
};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};

use crate::Runtime;

impl Runtime {
    pub(super) fn begin_terminal_search(&mut self) {
        let search = &mut self.session.workspace_mut().ui.terminal.search;
        search.active = true;
        search.escape_release_pending = false;
        search.status = "type to search · Enter next · Shift+Enter previous · Esc close".into();
        self.invalidate_frame();
    }

    pub(super) fn handle_terminal_search_key(&mut self, event: &KeyEvent) -> bool {
        if !self.workspace().ui.terminal.search.active {
            let escape_release = search_owns_escape_release(
                self.workspace().ui.terminal.search.escape_release_pending,
                event.state == ElementState::Released,
                matches!(event.logical_key, Key::Named(NamedKey::Escape)),
            );
            if escape_release {
                self.session
                    .workspace_mut()
                    .ui
                    .terminal
                    .search
                    .escape_release_pending = false;
                return true;
            }
            return false;
        }
        if event.state == ElementState::Released {
            return true;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                let search = &mut self.session.workspace_mut().ui.terminal.search;
                search.active = false;
                search.escape_release_pending = true;
                search.matches.clear();
                search.highlights.clear();
                search.active_match = None;
                search.matched = None;
                self.invalidate_frame();
            }
            Key::Named(NamedKey::Backspace) => {
                let query = &mut self.session.workspace_mut().ui.terminal.search.query;
                if let Some((offset, _)) = grapheme_indices(query).last() {
                    query.truncate(offset);
                }
                self.refresh_terminal_search(SearchDirection::Forward, false);
            }
            Key::Named(NamedKey::Enter) => {
                let direction = if self.modifiers.shift_key() {
                    SearchDirection::Backward
                } else {
                    SearchDirection::Forward
                };
                self.refresh_terminal_search(direction, true);
            }
            Key::Named(NamedKey::F3) => {
                let direction = if self.modifiers.shift_key() {
                    SearchDirection::Backward
                } else {
                    SearchDirection::Forward
                };
                self.refresh_terminal_search(direction, true);
            }
            Key::Character(_)
                if self.modifiers.alt_key()
                    && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyC)) =>
            {
                let search = &mut self.session.workspace_mut().ui.terminal.search;
                search.case_sensitive = !search.case_sensitive;
                self.refresh_terminal_search(SearchDirection::Forward, false);
            }
            Key::Character(_)
                if self.modifiers.alt_key()
                    && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyR)) =>
            {
                let search = &mut self.session.workspace_mut().ui.terminal.search;
                search.regex = !search.regex;
                self.refresh_terminal_search(SearchDirection::Forward, false);
            }
            Key::Character(text)
                if !text.is_empty()
                    && !self.modifiers.control_key()
                    && !self.modifiers.super_key()
                    && !self.modifiers.alt_key() =>
            {
                let maximum =
                    crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES.input_bytes;
                let current = self.workspace().ui.terminal.search.query.len();
                if current.saturating_add(text.len()) <= maximum {
                    self.session
                        .workspace_mut()
                        .ui
                        .terminal
                        .search
                        .query
                        .push_str(text);
                    self.refresh_terminal_search(SearchDirection::Forward, false);
                } else {
                    self.session.workspace_mut().ui.terminal.search.status =
                        "search query reached the approved input-byte limit".into();
                    self.invalidate_frame();
                }
            }
            _ => {}
        }
        true
    }

    pub(super) fn maintain_terminal_search_after_output(&mut self) {
        let search = &self.workspace().ui.terminal.search;
        if !search.active || search.query.is_empty() {
            return;
        }
        let Some(current) = search.matched else {
            self.refresh_terminal_search(SearchDirection::Forward, false);
            return;
        };
        let state = self
            .terminal_sessions
            .active_search_match_state(core_match(current));
        if !matches!(state, Ok(SearchMatchState::Active)) {
            self.refresh_terminal_search(SearchDirection::Forward, false);
        }
    }

    fn refresh_terminal_search(&mut self, direction: SearchDirection, advance: bool) {
        let state = self.workspace().ui.terminal.search.clone();
        if state.query.is_empty() {
            let search = &mut self.session.workspace_mut().ui.terminal.search;
            search.matches.clear();
            search.highlights.clear();
            search.active_match = None;
            search.matched = None;
            search.status = "type to search · Enter next · Shift+Enter previous · Esc close".into();
            self.invalidate_frame();
            return;
        }
        let case = if state.case_sensitive {
            SearchCase::Sensitive
        } else {
            SearchCase::Insensitive
        };
        let query = if state.regex {
            SearchQuery::regex(state.query.clone(), case)
        } else {
            SearchQuery::literal(state.query.clone(), case)
        };
        let result = self.terminal_sessions.search_all_active(&query);
        match result {
            Ok(result) => {
                let matches = result
                    .matches()
                    .iter()
                    .copied()
                    .map(protocol_match)
                    .collect::<Vec<_>>();
                let prior = state
                    .matched
                    .and_then(|current| matches.iter().position(|found| *found == current));
                let active_match = choose_active_match(matches.len(), prior, direction, advance);
                let matched = active_match.map(|index| matches[index]);
                let visible_rows = usize::from(self.terminal_screen_geometry().rows);
                let scroll = matched.and_then(|found| {
                    self.terminal_sessions
                        .active_scroll_offset_for_logical_point(
                            visible_rows,
                            core_point(found.start),
                        )
                        .ok()
                        .flatten()
                });
                let search = &mut self.session.workspace_mut().ui.terminal.search;
                search.highlights = merge_highlights(&matches);
                search.matches = matches;
                search.active_match = active_match;
                search.matched = matched;
                search.status = if let Some(index) = active_match {
                    format!(
                        "{}/{} · {} · {} · Enter next · Shift+Enter previous · Esc close",
                        index + 1,
                        search.matches.len(),
                        if search.case_sensitive {
                            "case"
                        } else {
                            "ignore case"
                        },
                        if search.regex { "regex" } else { "literal" }
                    )
                } else {
                    "no match · Alt+C case · Alt+R regex · Esc close".into()
                };
                if let Some(scroll) = scroll {
                    self.session.workspace_mut().ui.terminal.scroll_offset = scroll;
                }
            }
            Err(error) => {
                let search = &mut self.session.workspace_mut().ui.terminal.search;
                search.matches.clear();
                search.highlights.clear();
                search.active_match = None;
                search.matched = None;
                search.status = format!("search error: {error}");
            }
        }
        self.invalidate_frame();
    }
}

fn core_match(matched: TerminalSearchMatch) -> datum_terminal_core::SearchMatch {
    datum_terminal_core::SearchMatch::new(core_point(matched.start), core_point(matched.end))
}

fn protocol_point(point: LogicalPoint) -> TerminalSearchPoint {
    TerminalSearchPoint {
        line: point.line.get(),
        cluster: point.cluster,
    }
}

fn core_point(point: TerminalSearchPoint) -> LogicalPoint {
    LogicalPoint {
        line: LogicalLineId::new(point.line),
        cluster: point.cluster,
    }
}

fn protocol_match(matched: datum_terminal_core::SearchMatch) -> TerminalSearchMatch {
    TerminalSearchMatch {
        start: protocol_point(matched.start()),
        end: protocol_point(matched.end()),
    }
}

fn choose_active_match(
    count: usize,
    prior: Option<usize>,
    direction: SearchDirection,
    advance: bool,
) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match (prior, advance, direction) {
        (Some(index), true, SearchDirection::Forward) => (index + 1) % count,
        (Some(index), true, SearchDirection::Backward) => index.checked_sub(1).unwrap_or(count - 1),
        (Some(index), false, _) if index < count => index,
        (_, _, SearchDirection::Forward) => 0,
        (_, _, SearchDirection::Backward) => count - 1,
    })
}

fn merge_highlights(matches: &[TerminalSearchMatch]) -> Vec<TerminalSearchMatch> {
    let mut merged: Vec<TerminalSearchMatch> = Vec::new();
    for matched in matches.iter().copied() {
        let overlaps_or_touches = merged.last().is_some_and(|prior| {
            point_key(matched.start) <= point_key(prior.end)
                || (matched.start.line == prior.end.line
                    && matched.start.cluster == prior.end.cluster.saturating_add(1))
        });
        if overlaps_or_touches {
            if let Some(prior) = merged.last_mut()
                && point_key(matched.end) > point_key(prior.end)
            {
                prior.end = matched.end;
            }
        } else {
            merged.push(matched);
        }
    }
    merged
}

fn point_key(point: TerminalSearchPoint) -> (u64, u32) {
    (point.line, point.cluster)
}

fn search_owns_escape_release(pending: bool, released: bool, escape: bool) -> bool {
    pending && released && escape
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_navigation_wraps_and_stable_refresh_preserves_current_match() {
        assert_eq!(
            choose_active_match(3, Some(2), SearchDirection::Forward, true),
            Some(0)
        );
        assert!(search_owns_escape_release(true, true, true));
        assert!(!search_owns_escape_release(false, true, true));
        assert!(!search_owns_escape_release(true, false, true));
        assert_eq!(
            choose_active_match(3, Some(0), SearchDirection::Backward, true),
            Some(2)
        );
        assert_eq!(
            choose_active_match(4, Some(1), SearchDirection::Forward, false),
            Some(1),
            "new output may add results without moving the active logical match"
        );
        assert_eq!(
            choose_active_match(4, None, SearchDirection::Backward, false),
            Some(3)
        );
        assert_eq!(
            choose_active_match(0, Some(0), SearchDirection::Forward, true),
            None
        );

        let point = |cluster| TerminalSearchPoint { line: 7, cluster };
        assert_eq!(
            merge_highlights(&[
                TerminalSearchMatch {
                    start: point(1),
                    end: point(3),
                },
                TerminalSearchMatch {
                    start: point(2),
                    end: point(5),
                },
                TerminalSearchMatch {
                    start: point(6),
                    end: point(7),
                },
                TerminalSearchMatch {
                    start: point(10),
                    end: point(11),
                },
            ]),
            vec![
                TerminalSearchMatch {
                    start: point(1),
                    end: point(7),
                },
                TerminalSearchMatch {
                    start: point(10),
                    end: point(11),
                },
            ]
        );
    }
}
