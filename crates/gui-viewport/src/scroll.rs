//! Continuous consumer-side scrolling. Geometry, wheel input and thumb dragging
//! share one bounded pixel offset in the caller's coordinate space; no timer or redraw loop is owned here.
use crate::ScreenRectPx;

/// Pointer routing and visible damage are separate: grabbing a thumb consumes
/// the press without changing its pixels or document offset.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollPress {
    pub consumed: bool,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub struct ScrollViewport {
    pub viewport: ScreenRectPx,
    pub content_height: f32,
    offset: f32,
    drag_grab: Option<f32>,
}

impl Default for ScrollViewport {
    fn default() -> Self {
        Self {
            viewport: ScreenRectPx {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            content_height: 0.0,
            offset: 0.0,
            drag_grab: None,
        }
    }
}

impl ScrollViewport {
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn maximum(&self) -> f32 {
        (self.content_height - self.viewport.height).max(0.0)
    }
    pub fn layout(&mut self, viewport: ScreenRectPx, content_height: f32) {
        self.viewport = viewport;
        self.content_height = content_height.max(0.0);
        self.set_offset(self.offset);
        if self.maximum() == 0.0 {
            self.drag_grab = None;
        }
    }
    pub fn set_offset(&mut self, offset: f32) -> bool {
        if !offset.is_finite() {
            return false;
        }
        let next = offset.clamp(0.0, self.maximum());
        let changed = next != self.offset;
        self.offset = next;
        changed
    }
    /// Positive deltas move content down, matching native wheel coordinates.
    pub fn wheel(&mut self, delta: f32) -> bool {
        self.set_offset(self.offset - delta)
    }
    pub fn reveal(&mut self, top: f32, bottom: f32) {
        if top < self.offset {
            self.set_offset(top);
        } else if bottom > self.offset + self.viewport.height {
            self.set_offset(if bottom - top > self.viewport.height {
                top
            } else {
                bottom - self.viewport.height
            });
        }
    }
    pub fn track(&self) -> Option<ScreenRectPx> {
        (self.maximum() > 0.0 && self.viewport.height > 0.0).then_some(ScreenRectPx {
            x: self.viewport.x + self.viewport.width - 11.0,
            y: self.viewport.y,
            width: 11.0,
            height: self.viewport.height,
        })
    }
    pub fn thumb(&self) -> Option<ScreenRectPx> {
        let track = self.track()?;
        let height = (track.height * self.viewport.height / self.content_height)
            .max(24.0)
            .min(track.height);
        Some(ScreenRectPx {
            x: track.x + 2.0,
            y: track.y + (track.height - height) * self.offset / self.maximum(),
            width: track.width - 4.0,
            height,
        })
    }
    /// Track clicks page; thumb presses retain the exact pointer grab position.
    pub fn press(&mut self, x: f32, y: f32) -> ScrollPress {
        let Some(track) = self.track() else {
            return ScrollPress::default();
        };
        if x < track.x || x > track.x + track.width || y < track.y || y > track.y + track.height {
            return ScrollPress::default();
        }
        let before = self.offset;
        let thumb = self.thumb().expect("scrollable track has thumb");
        if y >= thumb.y && y <= thumb.y + thumb.height {
            self.drag_grab = Some(y - thumb.y);
        } else {
            self.set_offset(
                self.offset
                    + if y < thumb.y {
                        -self.viewport.height
                    } else {
                        self.viewport.height
                    },
            );
        }
        ScrollPress {
            consumed: true,
            changed: self.offset != before,
        }
    }
    pub fn drag(&mut self, y: f32) -> bool {
        let Some(grab) = self.drag_grab else {
            return false;
        };
        let Some(thumb) = self.thumb() else {
            return false;
        };
        let travel = self.viewport.height - thumb.height;
        if travel <= 0.0 {
            return false;
        }
        self.set_offset((y - self.viewport.y - grab) / travel * self.maximum())
    }
    pub fn release(&mut self) {
        self.drag_grab = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scroll() -> ScrollViewport {
        let mut s = ScrollViewport::default();
        s.layout(
            ScreenRectPx {
                x: 100.0,
                y: 60.0,
                width: 300.0,
                height: 400.0,
            },
            1000.0,
        );
        s
    }
    #[test]
    fn fractional_input_is_never_rounded_away_and_bounds_do_not_accumulate_debt() {
        let mut s = scroll();
        for _ in 0..100 {
            assert!(s.wheel(-0.25));
        }
        assert_eq!(s.offset(), 25.0);
        s.wheel(-10000.0);
        assert_eq!(s.offset(), 600.0);
        assert!(!s.wheel(-10000.0));
        assert!(s.wheel(0.25));
        assert_eq!(s.offset(), 599.75);
    }
    #[test]
    fn thumb_endpoints_drag_page_and_resize_share_content_bounds() {
        let mut s = scroll();
        let t = s.thumb().unwrap();
        assert_eq!(t.height, 160.0);
        assert_eq!(
            s.press(t.x, t.y + 20.0),
            ScrollPress {
                consumed: true,
                changed: false
            }
        );
        assert!(s.drag(320.0));
        assert_eq!(s.offset(), 600.0);
        assert_eq!(s.thumb().unwrap().y + t.height, 460.0);
        s.release();
        assert!(!s.drag(100.0));
        assert_eq!(
            s.press(t.x, 61.0),
            ScrollPress {
                consumed: true,
                changed: true
            }
        );
        assert_eq!(s.offset(), 200.0);
        s.layout(s.viewport, 200.0);
        assert_eq!(s.offset(), 0.0);
        assert!(s.thumb().is_none());
    }
    #[test]
    fn reveal_moves_only_when_required_and_nonfinite_input_is_ignored() {
        let mut s = scroll();
        s.set_offset(100.0);
        s.reveal(120.0, 200.0);
        assert_eq!(s.offset(), 100.0);
        s.reveal(480.0, 600.0);
        assert_eq!(s.offset(), 200.0);
        assert!(!s.wheel(f32::NAN));
        assert_eq!(s.offset(), 200.0);
    }
}
