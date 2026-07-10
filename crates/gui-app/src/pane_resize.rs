//! Divider-drag resize state for the workspace pane tree (decision 021).
//!
//! Grabbing a split's divider gutter starts a resize: the runtime records which
//! `Split` the gutter controls (its root-to-node `path`), the split's frame, and
//! its orientation, then translates each pointer move into a new ratio for that
//! split via `WorkspaceLayout::set_ratio_at_path`. Consumer/workspace view state —
//! never journaled.

use datum_gui_protocol::{SplitChild, SplitOrientation};
use datum_gui_render::RectPx;

/// An in-progress divider drag: the `Split` being resized plus the geometry needed
/// to turn a cursor position into a ratio.
pub(crate) struct DividerDrag {
    pub(crate) path: Vec<SplitChild>,
    pub(crate) split_frame: RectPx,
    pub(crate) orientation: SplitOrientation,
}

impl DividerDrag {
    /// The `first`-child ratio implied by cursor `(x, y)`: the cursor's position
    /// along the split axis as a fraction of the split frame. Returned UNCLAMPED;
    /// `WorkspaceLayout::set_ratio_at_path` clamps it to
    /// `[PANE_RATIO_MIN, PANE_RATIO_MAX]`, so the panes never collapse to zero.
    pub(crate) fn ratio_at(&self, x: f32, y: f32) -> f32 {
        match self.orientation {
            SplitOrientation::Vertical => {
                (x - self.split_frame.x) / self.split_frame.width.max(1.0)
            }
            SplitOrientation::Horizontal => {
                (y - self.split_frame.y) / self.split_frame.height.max(1.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> RectPx {
        RectPx {
            x: 100.0,
            y: 50.0,
            width: 400.0,
            height: 200.0,
        }
    }

    #[test]
    fn vertical_ratio_tracks_x_within_frame() {
        let d = DividerDrag {
            path: vec![],
            split_frame: frame(),
            orientation: SplitOrientation::Vertical,
        };
        assert!((d.ratio_at(300.0, 999.0) - 0.5).abs() < 1e-6); // midpoint
        assert!(d.ratio_at(100.0, 0.0).abs() < 1e-6); // left edge -> 0
        assert!((d.ratio_at(500.0, 0.0) - 1.0).abs() < 1e-6); // right edge -> 1
    }

    #[test]
    fn horizontal_ratio_tracks_y_within_frame() {
        let d = DividerDrag {
            path: vec![],
            split_frame: frame(),
            orientation: SplitOrientation::Horizontal,
        };
        assert!((d.ratio_at(0.0, 150.0) - 0.5).abs() < 1e-6); // midpoint (50 + 200*0.5)
        assert!(d.ratio_at(0.0, 50.0).abs() < 1e-6); // top edge -> 0
    }
}
