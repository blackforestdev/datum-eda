//! Shared pointer interaction state construction for drawing surfaces.

use datum_gui_protocol::{HoverTarget, PaneContent, ScreenPointPx, ViewportInteraction};

use crate::profile::{CursorConfig, HoverConfig};

/// Stateless shared mechanism that converts surface-owned hit-test output into
/// typed cursor and hover state.
#[derive(Debug, Clone, Copy, Default)]
pub struct InteractionEngine;

impl InteractionEngine {
    /// Resolve one pointer sample over a drawing surface.
    pub fn resolve(
        surface: PaneContent,
        cursor: ScreenPointPx,
        object_id: Option<String>,
        hover: HoverConfig,
        cursor_config: CursorConfig,
    ) -> ViewportInteraction {
        ViewportInteraction {
            cursor: cursor_config.enabled.then_some(cursor),
            hover: if hover.enabled {
                object_id.map(|object_id| HoverTarget { object_id, surface })
            } else {
                None
            },
        }
    }

    /// Clear all pointer state on surface or window exit.
    pub fn clear() -> ViewportInteraction {
        ViewportInteraction::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_preserves_typed_surface_and_screen_point() {
        let point = ScreenPointPx { x: 12.5, y: 41.0 };
        let state = InteractionEngine::resolve(
            PaneContent::Schematic,
            point,
            Some("symbol-7".into()),
            HoverConfig::default(),
            CursorConfig::default(),
        );
        assert_eq!(state.cursor, Some(point));
        assert_eq!(state.hover.unwrap().surface, PaneContent::Schematic);
    }

    #[test]
    fn profile_policy_disables_each_mechanism_independently() {
        let state = InteractionEngine::resolve(
            PaneContent::Board,
            ScreenPointPx { x: 1.0, y: 2.0 },
            Some("pad-1".into()),
            HoverConfig { enabled: false },
            CursorConfig { enabled: false },
        );
        assert_eq!(state, ViewportInteraction::default());
    }

    #[test]
    fn clear_removes_cursor_and_hover_together() {
        assert_eq!(InteractionEngine::clear(), ViewportInteraction::default());
    }
}
