//! GPU allocation lifetime shared by retained-world and screen vertex streams.
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};

#[derive(Default)]
pub(crate) struct VertexAllocation {
    buffer: Option<Tracked<wgpu::Buffer>>,
    owner: Option<Owner>,
    generation: u64,
}

impl VertexAllocation {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_deref()
    }

    pub(crate) fn submission_ref(&self) -> Option<SubmissionRef> {
        self.buffer.as_ref().map(Tracked::submission_ref)
    }

    /// A replacement requires a complete upload at the submission boundary. Keep nearby capacity to avoid churn,
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
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: live.next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
            usage,
            mapped_at_creation: false,
        });
        self.generation += 1;
        let bytes = buffer.size();
        self.buffer = Some(self.owner.get_or_insert_with(Owner::new).track(
            buffer,
            bytes,
            self.generation,
            Kind::Vertex,
        ));
        true
    }
}
