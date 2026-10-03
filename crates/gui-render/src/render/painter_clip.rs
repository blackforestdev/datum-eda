//! Shared exclusive pixel clip. Individual painters cannot widen damage.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Clip {
    source: [u32; 4],
    destination: [u32; 2],
    raster_scale: Option<[u32; 4]>,
    raster_band: Option<[u32; 2]>,
}
impl Clip {
    pub(crate) fn full(width: u32, height: u32) -> Self {
        Self::translated([0, 0, width, height], [0, 0])
    }
    pub(crate) fn resampled(prepared: [u32; 2], raster: [u32; 2]) -> Self {
        let mut clip = Self::full(prepared[0], prepared[1]);
        if prepared != raster {
            assert!(!prepared.contains(&0) && !raster.contains(&0));
            clip.raster_scale = Some([prepared[0], prepared[1], raster[0], raster[1]]);
        }
        clip
    }
    /// Adjacent suffix stripes own disjoint raster rows even during PM052
    /// temporary resampling. Internal painter bounds still round outward.
    pub(crate) fn resampled_band(
        prepared: [u32; 2],
        raster: [u32; 2],
        top: u32,
        bottom: u32,
    ) -> Self {
        let mut clip = Self::resampled(prepared, raster);
        clip.source = [0, top, prepared[0], bottom.min(prepared[1])];
        clip.destination = [0, top];
        let row = |y| (u64::from(y) * u64::from(raster[1]) / u64::from(prepared[1])) as u32;
        clip.raster_band = Some([row(top), row(bottom.min(prepared[1]))]);
        clip
    }
    pub(crate) fn translated(source: [u32; 4], destination: [u32; 2]) -> Self {
        Self {
            source,
            destination,
            raster_scale: None,
            raster_band: None,
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
        let Some(rect) = self.raster_rect(original) else {
            return false;
        };
        pass.set_scissor_rect(rect[0], rect[1], rect[2], rect[3]);
        true
    }
    fn raster_rect(self, original: [u32; 4]) -> Option<[u32; 4]> {
        let [x, y, width, height] = self.intersect(original)?;
        let result = if let Some([vw, vh, rw, rh]) = self.raster_scale {
            let lower = |v, raster, virtual_size| {
                (u64::from(v) * u64::from(raster) / u64::from(virtual_size)) as u32
            };
            let upper = |v, raster, virtual_size| {
                (u64::from(v) * u64::from(raster)).div_ceil(u64::from(virtual_size)) as u32
            };
            let left = lower(x, rw, vw);
            let top = lower(y, rh, vh);
            [
                left,
                top,
                upper(x + width, rw, vw) - left,
                upper(y + height, rh, vh) - top,
            ]
        } else {
            [x, y, width, height]
        };
        if let Some([band_top, band_bottom]) = self.raster_band {
            let top = result[1].max(band_top);
            let bottom = (result[1] + result[3]).min(band_bottom);
            (top < bottom).then_some([result[0], top, result[2], bottom - top])
        } else {
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resampled_scissors_cover_fractional_edges_without_widening_source_clip() {
        let clip = Clip::resampled([1347, 840], [1376, 864]);
        assert_eq!(
            clip.intersect([1300, 800, 100, 100]),
            Some([1300, 800, 47, 40])
        );
        assert_eq!(
            clip.raster_rect([1300, 800, 100, 100]),
            Some([1327, 822, 49, 42])
        );
        assert_eq!(clip.raster_rect([0, 0, 1347, 840]), Some([0, 0, 1376, 864]));
        assert_eq!(clip.raster_rect([1347, 0, 1, 1]), None);
        assert_eq!(
            Clip::resampled([32, 32], [32, 32]).raster_rect([1, 2, 3, 4]),
            Some([1, 2, 3, 4])
        );
    }
    #[test]
    fn stripe_rows_partition_settled_and_resampled_targets_exactly_once() {
        for (prepared, raster) in [
            ([1280, 800], [1280, 800]),
            ([1347, 840], [1376, 864]),
            ([1376, 864], [1347, 840]),
            ([1280, 800], [3840, 2160]),
        ] {
            let mut next = 0;
            for top in (0..prepared[1]).step_by(96) {
                let clip = Clip::resampled_band(prepared, raster, top, (top + 96).min(prepared[1]));
                let rect = clip.raster_rect([0, 0, prepared[0], prepared[1]]).unwrap();
                assert_eq!(rect[0], 0);
                assert_eq!(rect[1], next, "no stripe overlap or uncovered row");
                assert_eq!(rect[2], raster[0]);
                next += rect[3];
            }
            assert_eq!(next, raster[1]);
        }
    }
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
