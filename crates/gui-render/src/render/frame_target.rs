//! Explicit full-texture presentation capability; arbitrary views never grant copy access.
#[derive(Clone)]
pub struct FrameTarget {
    view: wgpu::TextureView,
    copy_destination: Option<wgpu::Texture>,
    raster_extent: Option<[u32; 2]>,
    #[cfg(feature = "visual")]
    native_source: Option<wgpu::Texture>,
    #[cfg(feature = "visual")]
    native_readback: Option<NativeReadback>,
}

impl FrameTarget {
    /// The caller supplies the actual acquired/capture texture, not a view from
    /// which the renderer guesses a subresource. Unsupported copy usage retains
    /// ordinary render-attachment fallback.
    pub fn full_texture(texture: &wgpu::Texture) -> anyhow::Result<Self> {
        anyhow::ensure!(
            texture.dimension() == wgpu::TextureDimension::D2
                && texture.depth_or_array_layers() == 1
                && texture.sample_count() == 1
                && texture
                    .usage()
                    .contains(wgpu::TextureUsages::RENDER_ATTACHMENT),
            "frame target must be a single-layer, single-sample 2D render attachment"
        );
        Ok(Self {
            raster_extent: None,
            view: texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                ..Default::default()
            }),
            copy_destination: texture
                .usage()
                .contains(wgpu::TextureUsages::COPY_DST)
                .then(|| texture.clone()),
            #[cfg(feature = "visual")]
            native_source: texture
                .usage()
                .contains(wgpu::TextureUsages::COPY_SRC)
                .then(|| texture.clone()),
            #[cfg(feature = "visual")]
            native_readback: None,
        })
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Explicit diagnostic permission for platform viewporter resampling.
    /// Ordinary callers never infer this permission from a mismatched texture.
    pub fn diagnostic_resampled_texture(texture: &wgpu::Texture) -> anyhow::Result<Self> {
        let mut target = Self::full_texture(texture)?;
        target.raster_extent = Some([texture.width(), texture.height()]);
        Ok(target)
    }

    pub(crate) fn raster_extent(&self, prepared: [u32; 2]) -> [u32; 2] {
        self.raster_extent.unwrap_or(prepared)
    }

    /// Copies must match the prepared target exactly; never scale, reinterpret
    /// its color format, or write outside a caller-selected view.
    pub fn copy_destination(
        &self,
        extent: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> Option<&wgpu::Texture> {
        self.copy_destination.as_ref().filter(|texture| {
            [texture.width(), texture.height()] == extent && texture.format() == format
        })
    }
}

impl From<wgpu::TextureView> for FrameTarget {
    fn from(view: wgpu::TextureView) -> Self {
        Self {
            view,
            copy_destination: None,
            raster_extent: None,
            #[cfg(feature = "visual")]
            native_source: None,
            #[cfg(feature = "visual")]
            native_readback: None,
        }
    }
}

#[cfg(feature = "visual")]
#[derive(Clone)]
pub struct NativeReadback(std::sync::Arc<NativeReadbackInner>);

#[cfg(feature = "visual")]
struct NativeReadbackInner {
    buffer: crate::capture_resource::CaptureReadback,
    extent: [u32; 2],
    format: wgpu::TextureFormat,
    padded_row: u32,
    snapshot: std::sync::Mutex<Option<NativeContentSnapshot>>,
}

#[cfg(feature = "visual")]
#[derive(Clone, Debug)]
pub struct NativeContentSnapshot {
    /// Actual consumed FramePlan receipt: owner/attempt/revision/preparation/host/device/config.
    pub attempt: [u64; 7],
    pub text: Vec<crate::prepared_scene_access::NativeContentText>,
}

#[cfg(feature = "visual")]
impl NativeReadback {
    pub fn new(
        device: &wgpu::Device,
        extent: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            extent.iter().all(|v| *v > 0),
            "native readback extent is empty"
        );
        anyhow::ensure!(
            matches!(
                format,
                wgpu::TextureFormat::Rgba8Unorm
                    | wgpu::TextureFormat::Rgba8UnormSrgb
                    | wgpu::TextureFormat::Bgra8Unorm
                    | wgpu::TextureFormat::Bgra8UnormSrgb
            ),
            "unsupported native readback format"
        );
        let row = extent[0]
            .checked_mul(4)
            .ok_or_else(|| anyhow::anyhow!("native row overflow"))?;
        let padded_row = row
            .checked_add(255)
            .ok_or_else(|| anyhow::anyhow!("native row alignment overflow"))?
            / 256
            * 256;
        let buffer = crate::capture_resource::CaptureReadback::new(
            device,
            u64::from(padded_row) * u64::from(extent[1]),
        )?;
        Ok(Self(std::sync::Arc::new(NativeReadbackInner {
            buffer,
            extent,
            format,
            padded_row,
            snapshot: std::sync::Mutex::new(None),
        })))
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.0.buffer
    }
    pub fn extent(&self) -> [u32; 2] {
        self.0.extent
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.0.format
    }
    pub fn padded_row(&self) -> u32 {
        self.0.padded_row
    }
    pub fn snapshot(&self) -> Option<NativeContentSnapshot> {
        self.0.snapshot.lock().unwrap().clone()
    }
}

#[cfg(feature = "visual")]
impl FrameTarget {
    pub fn with_native_readback(mut self, readback: NativeReadback) -> anyhow::Result<Self> {
        let source = self
            .native_source
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("native target lacks COPY_SRC"))?;
        anyhow::ensure!(
            [source.width(), source.height()] == readback.extent()
                && source.format() == readback.format(),
            "native readback target changed"
        );
        self.native_readback = Some(readback);
        Ok(self)
    }
    pub(crate) fn encode_native_readback(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        prepared: &crate::PreparedScene,
        attempt: [u64; 7],
    ) -> anyhow::Result<()> {
        let Some(readback) = &self.native_readback else {
            return Ok(());
        };
        let mut snapshot = readback.0.snapshot.lock().unwrap();
        anyhow::ensure!(snapshot.is_none(), "native readback encoded twice");
        let source = self
            .native_source
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("native readback source unavailable"))?;
        encoder.copy_texture_to_buffer(
            source.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: readback.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback.padded_row()),
                    rows_per_image: Some(readback.extent()[1]),
                },
            },
            wgpu::Extent3d {
                width: readback.extent()[0],
                height: readback.extent()[1],
                depth_or_array_layers: 1,
            },
        );
        *snapshot = Some(NativeContentSnapshot {
            attempt,
            text: prepared.native_content_text(),
        });
        Ok(())
    }
    pub(crate) fn hold_native_readback(&self, queue: &wgpu::Queue) {
        if let Some(readback) = &self.native_readback {
            readback.0.buffer.hold_submission(queue);
        }
    }
}

impl From<&wgpu::TextureView> for FrameTarget {
    fn from(view: &wgpu::TextureView) -> Self {
        Self::from(view.clone())
    }
}

impl From<&FrameTarget> for FrameTarget {
    fn from(target: &FrameTarget) -> Self {
        target.clone()
    }
}
