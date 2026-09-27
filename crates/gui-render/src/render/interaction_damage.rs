//! Bounded physical-pixel damage for repainting preserved interaction samples.
//!
//! Regions are disjoint: replaying translucent geometry twice in an overlap
//! would change the image. Overflow and malformed geometry require a full redraw.
use super::{RectPx, Vertex};

const MAX_REGIONS: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn intersect(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self
            .x
            .saturating_add(self.width)
            .min(other.x.saturating_add(other.width));
        let bottom = self
            .y
            .saturating_add(self.height)
            .min(other.y.saturating_add(other.height));
        (right > x && bottom > y).then_some(Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        })
    }

    /// Preserve the renderer's existing pane scissor rounding, then clip to the
    /// physical attachment. Empty intersections must skip the draw entirely.
    pub fn viewport(viewport: RectPx, extent: Self) -> Result<Option<Self>, ()> {
        if ![viewport.x, viewport.y, viewport.width, viewport.height]
            .into_iter()
            .all(f32::is_finite)
            || viewport.width < 0.0
            || viewport.height < 0.0
        {
            return Err(());
        }
        let region = Self {
            x: viewport.x.max(0.0).floor() as u32,
            y: viewport.y.max(0.0).floor() as u32,
            width: viewport.width.max(1.0).ceil() as u32,
            height: viewport.height.max(1.0).ceil() as u32,
        };
        region.validate().ok_or(())?;
        extent.validate().ok_or(())?;
        Ok(region.intersect(extent))
    }

    fn validate(self) -> Option<()> {
        self.x.checked_add(self.width)?;
        self.y.checked_add(self.height)?;
        Some(())
    }

    pub fn set(self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_scissor_rect(self.x, self.y, self.width, self.height);
    }

    fn subtract(self, cut: Self, output: &mut Regions) -> Option<()> {
        let Some(overlap) = self.intersect(cut) else {
            return output.push(self);
        };
        let right = self.x.saturating_add(self.width);
        let bottom = self.y.saturating_add(self.height);
        let overlap_right = overlap.x + overlap.width;
        let overlap_bottom = overlap.y + overlap.height;
        // Top/bottom span the original width; left/right span only the middle.
        // Those four pieces are disjoint, even when the cut touches an edge.
        for piece in [
            Self {
                x: self.x,
                y: self.y,
                width: self.width,
                height: overlap.y - self.y,
            },
            Self {
                x: self.x,
                y: overlap_bottom,
                width: self.width,
                height: bottom - overlap_bottom,
            },
            Self {
                x: self.x,
                y: overlap.y,
                width: overlap.x - self.x,
                height: overlap.height,
            },
            Self {
                x: overlap_right,
                y: overlap.y,
                width: right - overlap_right,
                height: overlap.height,
            },
        ] {
            output.push(piece)?;
        }
        Some(())
    }
}

/// Fixed storage: no allocation proportional to input geometry or fragmentation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Regions {
    entries: [Region; MAX_REGIONS],
    len: usize,
}

impl Default for Regions {
    fn default() -> Self {
        Self {
            entries: [Region::default(); MAX_REGIONS],
            len: 0,
        }
    }
}

impl Regions {
    pub fn as_slice(&self) -> &[Region] {
        &self.entries[..self.len]
    }

    fn push(&mut self, region: Region) -> Option<()> {
        if region.width == 0 || region.height == 0 {
            return Some(());
        }
        if self.len == MAX_REGIONS {
            return None;
        }
        self.entries[self.len] = region;
        self.len += 1;
        Some(())
    }

    /// Add only uncovered pieces. On failure the caller discards this entire
    /// plan; it must never render a partially constructed damage union.
    pub fn add(&mut self, region: Region) -> Option<()> {
        region.validate()?;
        let mut pending = Self::default();
        pending.push(region)?;
        for old in self.as_slice() {
            let mut next = Self::default();
            for part in pending.as_slice() {
                part.subtract(*old, &mut next)?;
            }
            pending = next;
        }
        for part in pending.as_slice() {
            self.push(*part)?;
        }
        Some(())
    }

    pub fn union(&self, other: &Self) -> Option<Self> {
        let mut result = self.clone();
        for region in other.as_slice() {
            result.add(*region)?;
        }
        Some(result)
    }

    /// Interaction geometry is emitted as independent six-vertex quads. Keep
    /// each arm/ring edge separate so FullViewport does not dirty the whole pane.
    /// One physical pixel of conservative padding includes MSAA edge coverage.
    pub fn include_vertices(&mut self, vertices: &[Vertex], clip: Region) -> Option<()> {
        clip.validate()?;
        if !vertices.len().is_multiple_of(6) {
            return None;
        }
        for quad in vertices.chunks_exact(6) {
            let mut min = [f32::INFINITY; 2];
            let mut max = [f32::NEG_INFINITY; 2];
            for vertex in quad {
                for axis in 0..2 {
                    let value = vertex.pos[axis];
                    if !value.is_finite() {
                        return None;
                    }
                    min[axis] = min[axis].min(value);
                    max[axis] = max[axis].max(value);
                }
            }
            let x = (min[0].floor() - 1.0).max(0.0) as u32;
            let y = (min[1].floor() - 1.0).max(0.0) as u32;
            let right = (max[0].ceil() + 1.0).max(0.0) as u32;
            let bottom = (max[1].ceil() + 1.0).max(0.0) as u32;
            if let Some(region) = (Region {
                x,
                y,
                width: right.saturating_sub(x),
                height: bottom.saturating_sub(y),
            })
            .intersect(clip)
            {
                self.add(region)?;
            }
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_full_crosshair_union_covers_pixels_once_without_full_pane_damage() {
        let input = [
            Region {
                x: 0,
                y: 8,
                width: 40,
                height: 3,
            },
            Region {
                x: 12,
                y: 0,
                width: 3,
                height: 30,
            },
            Region {
                x: 0,
                y: 15,
                width: 40,
                height: 3,
            },
            Region {
                x: 23,
                y: 0,
                width: 3,
                height: 30,
            },
        ];
        let mut old = Regions::default();
        let mut new = Regions::default();
        for rect in &input[..2] {
            old.add(*rect).unwrap();
        }
        for rect in &input[2..] {
            new.add(*rect).unwrap();
        }
        let union = old.union(&new).unwrap();
        let mut area = 0;
        for y in 0..30 {
            for x in 0..40 {
                let point = Region {
                    x,
                    y,
                    width: 1,
                    height: 1,
                };
                let expected = input.iter().any(|rect| rect.intersect(point).is_some());
                let count = union
                    .as_slice()
                    .iter()
                    .filter(|rect| rect.intersect(point).is_some())
                    .count();
                assert_eq!(count, usize::from(expected), "pixel {x},{y}");
                area += count;
            }
        }
        assert!(area < 40 * 30 / 2);
        assert_eq!(union.union(&union).unwrap().as_slice(), union.as_slice());
    }

    #[test]
    fn fragmentation_refuses_instead_of_returning_a_truncated_plan() {
        let mut regions = Regions::default();
        for x in 0..MAX_REGIONS as u32 {
            regions
                .add(Region {
                    x: x * 2,
                    y: 0,
                    width: 1,
                    height: 1,
                })
                .unwrap();
        }
        assert!(
            regions
                .add(Region {
                    x: 100,
                    y: 0,
                    width: 1,
                    height: 1
                })
                .is_none()
        );
        assert!(
            Region::default()
                .intersect(Region {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 10
                })
                .is_none()
        );
    }
    #[test]
    fn invalid_inputs_refuse_and_empty_viewport_intersections_stay_distinct() {
        let extent = Region {
            x: 0,
            y: 0,
            width: 40,
            height: 30,
        };
        let mut regions = Regions::default();
        assert!(
            regions
                .add(Region {
                    x: u32::MAX,
                    y: 0,
                    width: 2,
                    height: 1
                })
                .is_none()
        );
        assert!(regions.as_slice().is_empty());
        let outside = RectPx {
            x: 50.0,
            y: 50.0,
            width: 2.0,
            height: 2.0,
        };
        assert_eq!(Region::viewport(outside, extent), Ok(None));
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                Region::viewport(
                    RectPx {
                        x: value,
                        ..outside
                    },
                    extent
                ),
                Err(())
            );
            let vertices = [Vertex {
                pos: [value, 2.0],
                color: [1.0; 3],
            }; 6];
            assert!(regions.include_vertices(&vertices, extent).is_none());
        }
    }
}
