//! Visible-viewport ownership for scene-produced pointer targets.
//!
//! Editor geometry may project beyond its scissored viewport at extreme camera
//! positions. Pointer targets must be clipped by the same boundary so invisible
//! editor content cannot shadow terminal or shell chrome layered below it.

use super::{HitRegion, RectPx};

pub(super) fn clip_new_hit_regions(
    hit_regions: &mut Vec<HitRegion>,
    first_new_region: usize,
    viewport: RectPx,
) {
    let clipped = hit_regions
        .drain(first_new_region..)
        .filter_map(|mut region| {
            region.rect = region.rect.intersect(viewport)?;
            Some(region)
        })
        .collect::<Vec<_>>();
    hit_regions.extend(clipped);
}
