//! Datum-owned glyph texture pages. Rasterization remains an existing public API.
//!
//! Shared by workspace and overlay drawing. Full PM045 accounting also requires
//! CPU/font scratch, aggregate staging and cross-host qualification.
use std::collections::HashMap;

use glyphon::{CacheKey, FontSystem, SwashCache, SwashContent};

use super::lifetime::{Kind, Owner, SubmissionRef, Tracked};
#[path = "atlas/chunks.rs"]
mod chunks;
#[path = "atlas/cpu_images.rs"]
mod cpu_images;
#[path = "atlas/pending_metadata.rs"]
mod pending_metadata;
pub(crate) use cpu_images::UploadRequired;
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
    uploaded_rows: u32,
    page: usize,
    origin: [u32; 2],
    size: [u32; 2],
    stride: u32,
    pixels: Vec<u8>,
    _cpu_permits: [super::budget::Permit; 2],
}

pub(crate) struct Atlas {
    pub owner: Owner,
    pub layout: wgpu::BindGroupLayout,
    pub(super) pages: Vec<Page>,
    pub generation: u64,
    pub(super) uploads: Uploads,
    scatter: super::sparse_upload::Scatter,
    glyphs: HashMap<CacheKey, Option<GlyphLocation>>,
    pending_uploads: Vec<PendingUpload>,
    pending_copy_bytes: u64,
    pending_metadata_permits: Option<[super::budget::Permit; 2]>,
    local_budget: std::sync::Arc<super::budget::Budget>,
    pub(crate) staging_budget: std::sync::Arc<super::budget::Budget>,
    texture_budget: std::sync::Arc<super::budget::Budget>,
}

impl Atlas {
    #[cfg(all(test, feature = "visual"))]
    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_staging_budget(device, super::budget::Budget::new(16 * 1024 * 1024))
    }

    pub fn with_staging_budget(
        device: &wgpu::Device,
        staging_budget: std::sync::Arc<super::budget::Budget>,
    ) -> Self {
        Self {
            owner: Owner::new(),
            staging_budget,
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
            scatter: super::sparse_upload::Scatter::default(),
            glyphs: HashMap::new(),
            pending_uploads: Vec::new(),
            pending_copy_bytes: 0,
            pending_metadata_permits: None,
            local_budget: super::budget::Budget::new(RETAINED_LIMIT),
            texture_budget: super::budget::process(),
        }
    }

    /// Recreate device objects while old pages remain charged to this host.
    pub fn replacement(&self, device: &wgpu::Device) -> Self {
        let mut replacement = Self::with_staging_budget(device, self.staging_budget.clone());
        replacement.local_budget = self.local_budget.clone();
        replacement.texture_budget = self.texture_budget.clone();
        replacement.owner = self.owner.clone();
        replacement.generation = self.generation.wrapping_add(1);
        replacement
    }

    /// Flush immediately before the frame submission. Failed preparation and
    /// close can discard pending CPU images without leaving unsubmitted writes.
    pub fn has_pending_uploads(&self) -> bool {
        !self.pending_uploads.is_empty()
    }

    pub fn flush_uploads(
        &mut self,
        device: &wgpu::Device,
        buffers: &[super::upload::BufferUpload<'_>],
    ) -> anyhow::Result<Option<super::upload::Batch>> {
        let uploads: Vec<_> = self
            .pending_uploads
            .iter()
            .map(|upload| super::upload::TextureUpload {
                texture: &self.pages[upload.page].texture,
                origin: [upload.origin[0], upload.origin[1] + upload.uploaded_rows],
                size: [upload.size[0], upload.size[1] - upload.uploaded_rows],
                stride: upload.stride,
                pixels: &upload.pixels[(upload.uploaded_rows * upload.stride) as usize..],
            })
            .collect();
        let batch = super::upload::batch_with_scatter(
            device,
            &self.owner,
            self.generation,
            &self.staging_budget,
            &uploads,
            buffers,
            Some(&self.scatter),
        )?;
        self.pending_copy_bytes = 0;
        for upload in self.pending_uploads.drain(..) {
            self.uploads.writes += 1;
            self.uploads.bytes +=
                u64::from(upload.size[1] - upload.uploaded_rows) * u64::from(upload.stride);
        }
        self.release_empty_pending_metadata();
        Ok(batch)
    }

    #[cfg(all(test, feature = "visual"))]
    pub fn flush_for_test(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(mut batch) = self.flush_uploads(device, &[]).unwrap() {
            queue.submit([batch.command()]);
            batch.hold(queue);
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
        self.release_empty_pending_metadata();
        self.pending_copy_bytes = 0;
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
        self.release_empty_pending_metadata();
        self.pending_copy_bytes = 0;
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
        if self.pending_staging_bytes() >= cpu_images::CHUNK_BYTES {
            return Err(UploadRequired.into());
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
        let padded = u64::from(
            (size[0] * bytes_per_pixel as u32).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
        ) * u64::from(size[1]);
        self.reserve_pending_metadata()?;
        let cpu_permits = match self.reserve_cpu_image(
            crate::cpu_alloc::heap::capacity_bytes::<u8>(image.data.capacity()) as u64,
            padded,
        ) {
            Ok(permits) => permits,
            Err(error) => {
                self.release_empty_pending_metadata();
                return Err(error);
            }
        };
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
        self.pending_copy_bytes += padded;
        self.pending_uploads.push(PendingUpload {
            uploaded_rows: 0,
            page: page_index,
            origin,
            size,
            stride: size[0] * bytes_per_pixel as u32,
            pixels: image.data,
            _cpu_permits: cpu_permits,
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
#[path = "atlas/tests.rs"]
mod tests;
