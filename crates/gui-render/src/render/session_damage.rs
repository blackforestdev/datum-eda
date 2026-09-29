//! Bounded pixel support of the actual immediate interaction triangles.
//! A union is kept as separate strips, never a viewport-sized crosshair box.
use crate::{PreparedScene, SceneSurface, Vertex};

pub(crate) const MAX_RECTS: usize = 32;

/// Exclusive right/bottom coordinates are shared by GPU scissors and predicates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pixels {
    rects: [[u32; 4]; MAX_RECTS],
    count: usize,
}
impl Pixels {
    pub(crate) fn rectangles(&self) -> &[[u32; 4]] {
        &self.rects[..self.count]
    }

    fn push(&mut self, rect: [u32; 4]) -> Option<()> {
        if rect[0] >= rect[2] || rect[1] >= rect[3] || self.rectangles().contains(&rect) {
            return Some(());
        }
        *self.rects.get_mut(self.count)? = rect;
        self.count += 1;
        Some(())
    }

    pub(crate) fn union(mut self, other: Self) -> Option<Self> {
        for rect in other.rectangles() {
            self.push(*rect)?;
        }
        Some(self)
    }

    fn triangles(&mut self, vertices: &[Vertex], clip: [u32; 4]) -> Option<()> {
        let (triangles, remainder) = vertices.as_chunks::<3>();
        if !remainder.is_empty() {
            return None;
        }
        for triangle in triangles {
            let mut low = [f32::INFINITY; 2];
            let mut high = [f32::NEG_INFINITY; 2];
            for vertex in triangle {
                for axis in 0..2 {
                    let value = vertex.pos[axis];
                    if !value.is_finite() {
                        return None;
                    }
                    low[axis] = low[axis].min(value);
                    high[axis] = high[axis].max(value);
                }
            }
            // Bounds follow emitted geometry, including fractional strokes. The
            // guard is rounded outward before intersecting the actual painter clip.
            self.push([
                ((low[0].floor() - 1.0).max(0.0) as u32).max(clip[0]),
                ((low[1].floor() - 1.0).max(0.0) as u32).max(clip[1]),
                ((high[0].ceil() + 1.0).max(0.0) as u32).min(clip[2]),
                ((high[1].ceil() + 1.0).max(0.0) as u32).min(clip[3]),
            ])?;
        }
        Some(())
    }

    /// Called on an immutable prepared frame; no editor supplies pixel damage.
    pub(crate) fn capture(scene: &PreparedScene, extent: [u32; 2]) -> Option<Self> {
        let mut pixels = Self::default();
        for (surface, vertices) in [
            (SceneSurface::Board, scene.board_interaction_vertices()),
            (SceneSurface::Schematic, scene.schematic_overlay_vertices()),
        ] {
            if vertices.is_empty() {
                continue;
            }
            let viewport = scene.interaction_viewport(surface)?;
            if ![viewport.x, viewport.y, viewport.width, viewport.height]
                .iter()
                .all(|v| v.is_finite())
            {
                return None;
            }
            let [x, y, width, height] =
                crate::immediate_admission::screen_admission::scissor(viewport);
            let right = x.checked_add(width)?.min(extent[0]);
            let bottom = y.checked_add(height)?.min(extent[1]);
            pixels.triangles(vertices, [x, y, right, bottom])?;
        }
        Some(pixels)
    }
}

#[cfg(test)]
#[path = "session_damage_tests.rs"]
mod tests;
