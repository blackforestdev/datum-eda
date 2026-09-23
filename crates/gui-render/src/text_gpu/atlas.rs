//! Datum-owned glyph texture pages. Rasterization remains an existing public API.
//!
//! Shared by workspace and overlay drawing. Full PM045 accounting also requires
//! CPU/font scratch, aggregate staging and cross-host qualification.
use std::collections::HashMap;

use glyphon::{CacheKey, FontSystem, SwashCache, SwashContent};

use super::lifetime::{Kind, Owner, SubmissionRef, Tracked};
const RETAINED_LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub(super) struct GlyphLocation {
    pub page: usize,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub bearing: [i32; 2],
    pub color: bool,
}

/// Append-only shelves preserve every prepared glyph address until reset.
#[derive(Default)]
struct Shelves {
    x: u32,
    y: u32,
    row_height: u32,
}

impl Shelves {
    fn reserve(&mut self, size: [u32; 2], extent: u32) -> Option<[u32; 2]> {
        if size.contains(&0) || size[0] > extent || size[1] > extent {
            return None;
        }
        let (x, y, row_height) = if self.x + size[0] > extent {
            (0, self.y + self.row_height, 0)
        } else {
            (self.x, self.y, self.row_height)
        };
        if y + size[1] > extent {
            return None;
        }
        self.x = x + size[0];
        self.y = y;
        self.row_height = row_height.max(size[1]);
        Some([x, y])
    }
}

pub(super) struct Page {
    // Release the page's binding reference before its charged texture owner.
    pub bind_group: wgpu::BindGroup,
    pub texture: Tracked<wgpu::Texture>,
    extent: u32,
    color: bool,
    shelves: Shelves,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Uploads {
    pub bytes: u64,
    pub writes: u64,
    pub rasterizations: u64,
}

struct PendingUpload {
    page: usize,
    origin: [u32; 2],
    size: [u32; 2],
    stride: u32,
    pixels: Vec<u8>,
}

pub(crate) struct Atlas {
    pub owner: Owner,
    pub layout: wgpu::BindGroupLayout,
    pub(super) pages: Vec<Page>,
    pub generation: u64,
    pub(super) uploads: Uploads,
    glyphs: HashMap<CacheKey, Option<GlyphLocation>>,
    pending_uploads: Vec<PendingUpload>,
    local_budget: std::sync::Arc<super::budget::Budget>,
    texture_budget: std::sync::Arc<super::budget::Budget>,
}

impl Atlas {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            owner: Owner::new(),
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("datum-glyph-page-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            }),
            pages: Vec::new(),
            generation: 1,
            uploads: Uploads::default(),
            glyphs: HashMap::new(),
            pending_uploads: Vec::new(),
            local_budget: super::budget::Budget::new(RETAINED_LIMIT),
            texture_budget: super::budget::process(),
        }
    }

    /// Flush immediately before the frame submission. Failed preparation and
    /// close can discard pending CPU images without leaving unsubmitted writes.
    pub fn has_pending_uploads(&self) -> bool {
        !self.pending_uploads.is_empty()
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        for upload in self.pending_uploads.drain(..) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.pages[upload.page].texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.origin[0],
                        y: upload.origin[1],
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.stride),
                    rows_per_image: Some(upload.size[1]),
                },
                wgpu::Extent3d {
                    width: upload.size[0],
                    height: upload.size[1],
                    depth_or_array_layers: 1,
                },
            );
            self.uploads.writes += 1;
            self.uploads.bytes += upload.pixels.len() as u64;
        }
    }

    pub fn submission_refs(&self) -> Vec<SubmissionRef> {
        self.pages
            .iter()
            .map(|page| page.texture.submission_ref())
            .collect()
    }

    /// Reuse physical pages in queue order; all prepared address references
    /// become stale. Pending CPU uploads have not reached the queue and cancel.
    pub fn repack(&mut self) {
        self.glyphs.clear();
        self.pending_uploads.clear();
        self.generation = self
            .generation
            .checked_add(1)
            .expect("atlas epoch exhausted");
        for page in &mut self.pages {
            page.shelves = Shelves::default();
        }
    }

    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn set_test_limit(&mut self, bytes: u64) {
        assert_eq!(
            self.local_budget.used(),
            0,
            "test limit requires an empty atlas"
        );
        self.local_budget = super::budget::Budget::new(bytes);
    }

    pub fn reserved_texture_bytes(&self) -> u64 {
        self.local_budget.used()
    }

    #[cfg(all(test, feature = "visual"))]
    fn retained_texture_bytes(&self) -> u64 {
        self.pages
            .iter()
            .map(|page| {
                page.texture.width() as u64
                    * page.texture.height() as u64
                    * page.texture.format().block_copy_size(None).unwrap() as u64
            })
            .sum()
    }

    /// Invalidate ALL prepared references. The caller owns retirement of returned
    /// pages through completion of their last submission (including queued writes).
    /// A CPU reset alone is not GPU release, and these bytes must remain charged.
    #[cfg(all(test, feature = "visual"))]
    pub(super) fn reset(&mut self) -> Vec<Page> {
        self.glyphs = HashMap::new();
        self.pending_uploads.clear();
        self.generation = self
            .generation
            .checked_add(1)
            .expect("atlas epoch exhausted");
        std::mem::take(&mut self.pages)
    }

    pub(super) fn glyph(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        fonts: &mut FontSystem,
        raster: &mut SwashCache,
        key: CacheKey,
    ) -> anyhow::Result<Option<GlyphLocation>> {
        if let Some(location) = self.glyphs.get(&key) {
            return Ok(*location);
        }
        self.uploads.rasterizations += 1;
        let Some(image) = raster.get_image_uncached(fonts, key) else {
            self.glyphs.insert(key, None);
            return Ok(None);
        };
        let size = [image.placement.width, image.placement.height];
        if size.contains(&0) {
            self.glyphs.insert(key, None);
            return Ok(None);
        }
        let color = match image.content {
            SwashContent::Mask => false,
            SwashContent::Color => true,
            SwashContent::SubpixelMask => anyhow::bail!("unsupported subpixel glyph image"),
        };
        let bytes_per_pixel = if color { 4 } else { 1 };
        anyhow::ensure!(
            image.data.len() as u64 == size[0] as u64 * size[1] as u64 * bytes_per_pixel,
            "glyph raster payload does not match its extent"
        );
        let mut slot = None;
        for (index, page) in self.pages.iter_mut().enumerate() {
            if page.color == color
                && let Some(origin) = page.shelves.reserve(size, page.extent)
            {
                slot = Some((index, origin));
                break;
            }
        }
        let (page_index, origin) = match slot {
            Some(slot) => slot,
            None => {
                let limit = device.limits().max_texture_dimension_2d;
                let extent = 1024.min(limit).max(size[0]).max(size[1]);
                anyhow::ensure!(extent <= limit, "glyph exceeds device texture extent");
                let bytes = extent as u64 * extent as u64 * bytes_per_pixel;
                let local_permit = self.local_budget.reserve(bytes)?;
                let permit = self.texture_budget.reserve(bytes)?;
                let gpu_permit = super::budget::gpu_process().reserve(bytes)?;
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("datum-glyph-page"),
                    size: wgpu::Extent3d {
                        width: extent,
                        height: extent,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: if color {
                        wgpu::TextureFormat::Rgba8UnormSrgb
                    } else {
                        wgpu::TextureFormat::R8Unorm
                    },
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("datum-glyph-page-binding"),
                    layout: &self.layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    }],
                });
                let mut shelves = Shelves::default();
                let origin = shelves
                    .reserve(size, extent)
                    .expect("validated glyph extent");
                let index = self.pages.len();
                self.pages.push(Page {
                    texture: self.owner.track_with_permits(
                        texture,
                        bytes,
                        self.generation,
                        Kind::Texture,
                        vec![local_permit, permit, gpu_permit],
                    ),
                    bind_group,
                    extent,
                    color,
                    shelves,
                });
                (index, origin)
            }
        };
        self.pending_uploads.push(PendingUpload {
            page: page_index,
            origin,
            size,
            stride: size[0] * bytes_per_pixel as u32,
            pixels: image.data,
        });
        let location = GlyphLocation {
            page: page_index,
            origin,
            size,
            bearing: [image.placement.left, image.placement.top],
            color,
        };
        self.glyphs.insert(key, Some(location));
        Ok(Some(location))
    }
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    #[test]
    fn shelf_rejection_preserves_previous_and_remaining_space() {
        let mut shelves = Shelves::default();
        assert_eq!(shelves.reserve([7, 3], 10), Some([0, 0]));
        assert_eq!(shelves.reserve([4, 8], 10), None);
        assert_eq!(shelves.reserve([3, 2], 10), Some([7, 0]));
        assert_eq!(shelves.reserve([10, 7], 10), Some([0, 3]));
        assert_eq!(shelves.reserve([1, 1], 10), None);
    }

    #[test]
    #[ignore = "requires local GPU; local atlas retirement admission"]
    fn local_atlas_limit_counts_retired_pages_before_replacement() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut fonts = crate::load_datum_fonts();
        let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
        buffer.set_text(
            &mut fonts,
            "A",
            &crate::text_attrs(crate::TextFace::Ui),
            glyphon::Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let key = buffer.layout_runs().next().unwrap().glyphs[0]
            .physical((0.0, 0.0), 1.0)
            .cache_key;
        let mut raster = SwashCache::new();
        let mut atlas = Atlas::new(&device);
        atlas.set_test_limit(1024 * 1024);
        let local = atlas.local_budget.clone();
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap();
        let held = atlas.submission_refs();
        drop(atlas.reset());
        assert_eq!(atlas.retained_texture_bytes(), 0);
        assert_eq!(local.used(), 1024 * 1024);
        assert!(
            atlas
                .glyph(&device, &queue, &mut fonts, &mut raster, key)
                .is_err()
        );
        assert!(atlas.pages.is_empty());
        drop(held);
        assert_eq!(local.used(), 0);
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap();
        assert_eq!(local.used(), 1024 * 1024);
        drop(atlas);
        assert_eq!(local.used(), 0);
    }

    #[test]
    #[ignore = "requires local GPU; run explicitly with the visual feature"]
    fn owned_atlas_process_admission_precedes_allocation_and_waits_for_retirement() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut fonts = crate::load_datum_fonts();
        let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
        buffer.set_text(
            &mut fonts,
            "A",
            &crate::text_attrs(crate::TextFace::Ui),
            glyphon::Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let key = buffer.layout_runs().next().unwrap().glyphs[0]
            .physical((0.0, 0.0), 1.0)
            .cache_key;
        let budget = super::super::budget::Budget::new(2 * 1024 * 1024);
        let mut raster = SwashCache::new();
        let mut atlases: Vec<_> = (0..3)
            .map(|_| {
                let mut atlas = Atlas::new(&device);
                atlas.texture_budget = budget.clone();
                atlas
            })
            .collect();
        for atlas in &mut atlases[..2] {
            atlas
                .glyph(&device, &queue, &mut fonts, &mut raster, key)
                .unwrap();
        }
        assert_eq!(budget.used(), 2 * 1024 * 1024);
        assert!(
            atlases[2]
                .glyph(&device, &queue, &mut fonts, &mut raster, key)
                .is_err()
        );
        assert!(
            atlases[2].pages.is_empty(),
            "reject before texture allocation"
        );
        let held = atlases[0].submission_refs();
        let retired = atlases.remove(0);
        drop(retired);
        assert_eq!(budget.used(), 2 * 1024 * 1024);
        assert!(
            atlases[1]
                .glyph(&device, &queue, &mut fonts, &mut raster, key)
                .is_err()
        );
        drop(held);
        assert_eq!(budget.used(), 1024 * 1024);
        atlases[1]
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap();
        assert_eq!(budget.used(), 2 * 1024 * 1024);
        drop(atlases);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    #[ignore = "requires local GPU; run explicitly with the visual feature"]
    fn owned_atlas_upload_reuse_reset_and_retirement_handoff() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut fonts = crate::load_datum_fonts();
        let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
        buffer.set_text(
            &mut fonts,
            "A",
            &crate::text_attrs(crate::TextFace::Ui),
            glyphon::Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let key = buffer.layout_runs().next().unwrap().glyphs[0]
            .physical((0.0, 0.0), 1.0)
            .cache_key;
        let mut raster = SwashCache::new();
        let reference = raster.get_image_uncached(&mut fonts, key).unwrap();
        let mut atlas = Atlas::new(&device);
        let budget = super::super::budget::Budget::new(2 * 1024 * 1024);
        atlas.texture_budget = budget.clone();
        let first = atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap()
            .unwrap();
        assert_eq!(
            first.size,
            [reference.placement.width, reference.placement.height]
        );
        assert_eq!(
            first.bearing,
            [reference.placement.left, reference.placement.top]
        );
        atlas.flush_uploads(&queue);
        assert_eq!(atlas.uploads.bytes, reference.data.len() as u64);
        assert_eq!(atlas.uploads.writes, 1);
        let uploads = atlas.uploads;
        let allocation = atlas.pages[first.page].texture.id();
        let retained = atlas.retained_texture_bytes();
        let warm = atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap()
            .unwrap();
        assert_eq!(warm.origin, first.origin);
        assert_eq!(
            atlas.uploads, uploads,
            "warm glyph must not rasterize or upload"
        );
        let epoch = atlas.generation;
        let retiring = atlas.reset();
        assert_eq!(atlas.generation, epoch + 1);
        assert_eq!(atlas.retained_texture_bytes(), 0);
        assert_eq!(
            retiring
                .iter()
                .map(|page| page.texture.width() as u64
                    * page.texture.height() as u64
                    * page.texture.format().block_copy_size(None).unwrap() as u64)
                .sum::<u64>(),
            retained
        );
        let replacement = atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap()
            .unwrap();
        assert_ne!(atlas.pages[replacement.page].texture.id(), allocation);
        atlas.flush_uploads(&queue);
        assert_eq!(atlas.uploads.writes, 2);
        assert_eq!(atlas.uploads.bytes, 2 * reference.data.len() as u64);
        assert_eq!(atlas.retained_texture_bytes(), retained);
        // Both sets remain alive until all queued writes complete. Reset does
        // not silently turn a retired allocation into released accounting.
        let submitted = retiring
            .iter()
            .map(|page| page.texture.submission_ref())
            .collect();
        drop(retiring);
        let records = atlas.owner.records();
        assert!(
            records
                .iter()
                .find(|record| record.id == allocation)
                .unwrap()
                .retiring
        );
        assert_eq!(
            records.iter().map(|record| record.bytes).sum::<u64>(),
            2 * retained
        );
        assert_eq!(budget.used(), 2 * retained);
        let submission = queue.submit([]);
        super::super::lifetime::hold_until_done(&queue, submitted);
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .unwrap();
        let records = atlas.owner.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bytes, retained);
        assert_ne!(records[0].id, allocation);
        assert_eq!(budget.used(), retained);
        drop(atlas);
        assert_eq!(budget.used(), 0);
    }
}
