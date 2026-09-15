//! Shared per-pane screen/world projection keystone.

use datum_gui_protocol::{PaneContent, PaneId, PointNm, RectNm, ScreenPointPx};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRectPx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ScreenRectPx {
    pub fn contains(self, point: ScreenPointPx) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

/// One drawing pane's complete coordinate authority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorViewport {
    pub pane_id: PaneId,
    pub surface: PaneContent,
    pub screen: ScreenRectPx,
    pub world: RectNm,
    pub scale_px_per_nm: f32,
    pub offset_x_px: f32,
    pub offset_y_px: f32,
}

impl EditorViewport {
    pub fn screen_to_world(self, point: ScreenPointPx) -> Option<PointNm> {
        if !self.screen.contains(point)
            || !self.scale_px_per_nm.is_finite()
            || self.scale_px_per_nm <= 0.0
        {
            return None;
        }
        let x = self.world.min_x as f64
            + (point.x as f64 - self.offset_x_px as f64) / self.scale_px_per_nm as f64;
        let y = self.world.min_y as f64
            + (point.y as f64 - self.offset_y_px as f64) / self.scale_px_per_nm as f64;
        Some(PointNm {
            x: round_clamped_i64(x),
            y: round_clamped_i64(y),
        })
    }

    pub fn world_to_screen(self, point: PointNm) -> ScreenPointPx {
        ScreenPointPx {
            x: self.offset_x_px
                + ((point.x as i128 - self.world.min_x as i128) as f64
                    * self.scale_px_per_nm as f64) as f32,
            y: self.offset_y_px
                + ((point.y as i128 - self.world.min_y as i128) as f64
                    * self.scale_px_per_nm as f64) as f32,
        }
    }
}

fn round_clamped_i64(value: f64) -> i64 {
    value.round().clamp(i64::MIN as f64, i64::MAX as f64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> EditorViewport {
        EditorViewport {
            pane_id: PaneId(7),
            surface: PaneContent::Schematic,
            screen: ScreenRectPx {
                x: 10.0,
                y: 20.0,
                width: 200.0,
                height: 100.0,
            },
            world: RectNm {
                min_x: 1_000,
                min_y: -2_000,
                max_x: 3_000,
                max_y: -1_000,
            },
            scale_px_per_nm: 0.1,
            offset_x_px: 10.0,
            offset_y_px: 20.0,
        }
    }

    #[test]
    fn round_trip_uses_the_panes_own_coordinate_space() {
        let viewport = viewport();
        let world = PointNm {
            x: 1_500,
            y: -1_500,
        };
        assert_eq!(
            viewport.screen_to_world(viewport.world_to_screen(world)),
            Some(world)
        );
    }

    #[test]
    fn points_outside_the_pane_do_not_resolve() {
        assert_eq!(
            viewport().screen_to_world(ScreenPointPx { x: 9.0, y: 20.0 }),
            None
        );
    }
}
