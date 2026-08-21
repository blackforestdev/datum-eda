//! Transient pointer state for resizing one terminal split divider.

use datum_gui_protocol::{TerminalSplitChild, TerminalSplitDirection};
use datum_gui_viewport::{ScreenRectPx, TERMINAL_SPLIT_GUTTER_PX};

pub(crate) struct TerminalSplitDividerDrag {
    pub(crate) path: Vec<TerminalSplitChild>,
    pub(crate) direction: TerminalSplitDirection,
    pub(crate) split_bounds: ScreenRectPx,
}

impl TerminalSplitDividerDrag {
    pub(crate) fn ratio_millis_at(&self, pointer: (f32, f32)) -> u16 {
        let available = match self.direction {
            TerminalSplitDirection::SideBySide => {
                (self.split_bounds.width - TERMINAL_SPLIT_GUTTER_PX).max(1.0)
            }
            TerminalSplitDirection::Stacked => {
                (self.split_bounds.height - TERMINAL_SPLIT_GUTTER_PX).max(1.0)
            }
        };
        let offset = match self.direction {
            TerminalSplitDirection::SideBySide => pointer.0 - self.split_bounds.x,
            TerminalSplitDirection::Stacked => pointer.1 - self.split_bounds.y,
        };
        ((offset / available * 1000.0).round() as i32).clamp(100, 900) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> ScreenRectPx {
        ScreenRectPx {
            x: 100.0,
            y: 50.0,
            width: 406.0,
            height: 206.0,
        }
    }

    #[test]
    fn terminal_split_drag_tracks_axis_and_clamps_to_ten_ninety() {
        let horizontal = TerminalSplitDividerDrag {
            path: vec![],
            direction: TerminalSplitDirection::SideBySide,
            split_bounds: bounds(),
        };
        assert_eq!(horizontal.ratio_millis_at((300.0, 999.0)), 500);
        assert_eq!(horizontal.ratio_millis_at((-100.0, 0.0)), 100);
        assert_eq!(horizontal.ratio_millis_at((999.0, 0.0)), 900);

        let vertical = TerminalSplitDividerDrag {
            path: vec![TerminalSplitChild::Second],
            direction: TerminalSplitDirection::Stacked,
            split_bounds: bounds(),
        };
        assert_eq!(vertical.ratio_millis_at((0.0, 150.0)), 500);
    }
}
