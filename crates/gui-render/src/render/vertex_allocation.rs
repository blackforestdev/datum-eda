//! GPU allocation lifetime shared by retained-world and screen vertex streams.
use wgpu::util::DeviceExt;

#[derive(Default)]
pub(crate) struct VertexAllocation {
    buffer: Option<wgpu::Buffer>,
}

impl VertexAllocation {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }

    /// A replacement is initialized with the complete payload; otherwise the
    /// caller uploads its changed ranges. Keep nearby capacity to avoid churn,
    /// but release historical peaks above four times the latest live payload.
    /// Device/renderer teardown drops this owner and its handle.
    pub(crate) fn replace_if_needed(
        &mut self,
        device: &wgpu::Device,
        label: &str,
        bytes: &[u8],
    ) -> bool {
        debug_assert!(
            !bytes.is_empty(),
            "empty streams drop their allocation owner"
        );
        let live = bytes.len() as u64;
        if self
            .buffer
            .as_ref()
            .is_some_and(|buffer| buffer.size() >= live && buffer.size() <= live.saturating_mul(4))
        {
            return false;
        }
        let mut usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        if cfg!(test) {
            usage |= wgpu::BufferUsages::COPY_SRC;
        }
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage,
            }),
        );
        true
    }
}
