//! Intersect every existing painter scissor with one disjoint damage region.
//! Empty intersections suppress draws; pipeline/buffer setup may still proceed.
use super::interaction_damage::Region;
use std::ops::{Deref, DerefMut, Range};

pub(crate) struct ClippedPass<'borrow, 'pass> {
    inner: &'borrow mut wgpu::RenderPass<'pass>,
    damage: Region,
    region_index: usize,
    clip: Option<Region>,
}

impl<'borrow, 'pass> ClippedPass<'borrow, 'pass> {
    pub fn new(
        inner: &'borrow mut wgpu::RenderPass<'pass>,
        damage: Region,
        region_index: usize,
    ) -> Self {
        damage.set(inner);
        Self {
            inner,
            damage,
            region_index,
            clip: Some(damage),
        }
    }

    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        let clipped = self.damage.intersect(Region {
            x,
            y,
            width,
            height,
        });
        self.clip = clipped;
        if let Some(clipped) = clipped {
            clipped.set(self.inner);
        }
    }

    /// Record the exact planned region only when this painter's clip emits work.
    pub fn record_execution(&self, receipt: &std::cell::Cell<Option<u32>>) {
        if self.clip.is_none() {
            return;
        }
        receipt.set(receipt.get().and_then(|mask| {
            let bit = 1_u32.checked_shl(self.region_index.try_into().ok()?)?;
            (mask & bit == 0).then_some(mask | bit)
        }));
    }

    pub fn current_scissor(&self) -> Option<[u32; 4]> {
        self.clip.map(|r| [r.x, r.y, r.width, r.height])
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        if self.clip.is_some() {
            self.inner.draw(vertices, instances);
        }
    }

    pub fn execute_bundles<'a>(
        &mut self,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle>,
    ) {
        if self.clip.is_some() {
            self.inner.execute_bundles(bundles);
        }
    }
}

impl<'pass> Deref for ClippedPass<'_, 'pass> {
    type Target = wgpu::RenderPass<'pass>;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl DerefMut for ClippedPass<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
