//! Shared exclusive pixel clip. Individual painters cannot widen damage.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Clip {
    source: [u32; 4],
    destination: [u32; 2],
}
impl Clip {
    pub(crate) fn full(width: u32, height: u32) -> Self {
        Self::translated([0, 0, width, height], [0, 0])
    }
    pub(crate) fn translated(source: [u32; 4], destination: [u32; 2]) -> Self {
        Self {
            source,
            destination,
        }
    }
    pub(crate) fn intersect(self, [x, y, width, height]: [u32; 4]) -> Option<[u32; 4]> {
        let [left, top, right, bottom] = self.source;
        let x0 = left.max(x);
        let y0 = top.max(y);
        let x1 = right.min(x.checked_add(width)?);
        let y1 = bottom.min(y.checked_add(height)?);
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        Some([
            self.destination[0].checked_add(x0 - left)?,
            self.destination[1].checked_add(y0 - top)?,
            x1 - x0,
            y1 - y0,
        ])
    }
    pub(crate) fn set(self, pass: &mut wgpu::RenderPass<'_>, original: [u32; 4]) -> bool {
        let Some([x, y, w, h]) = self.intersect(original) else {
            return false;
        };
        pass.set_scissor_rect(x, y, w, h);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_intersection_precedes_positive_and_negative_translation() {
        for destination in [[0, 0], [480, 480]] {
            let clip = Clip::translated([64, 96, 96, 128], destination);
            assert_eq!(
                clip.intersect([60, 100, 12, 40]),
                Some([destination[0], destination[1] + 4, 8, 28])
            );
            assert_eq!(clip.intersect([0, 0, 64, 96]), None);
        }
    }
    #[test]
    fn every_local_painter_clip_intersects_instead_of_overwriting_damage() {
        let clip = Clip::translated([7, 9, 12, 16], [7, 9]);
        for original in [[0, 0, 1280, 800], [5, 5, 15, 15], [7, 9, 5, 7]] {
            assert_eq!(clip.intersect(original), Some([7, 9, 5, 7]));
        }
        assert_eq!(clip.intersect([9, 10, 8, 20]), Some([9, 10, 3, 6]));
        assert_eq!(clip.intersect([12, 9, 4, 7]), None);
        assert_eq!(clip.intersect([u32::MAX, 9, 2, 7]), None);
        assert_eq!(clip.intersect([9, 10, 0, 0]), None);
    }
}
