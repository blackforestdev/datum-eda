//! Fixed-size uniform ownership with exact last-value upload suppression.
use wgpu::util::DeviceExt;

pub(crate) struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    value: Option<T>,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub(crate) fn new(device: &wgpu::Device, label: &str, value: T) -> Self {
        Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::bytes_of(&value),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            value: Some(value),
            #[cfg(test)]
            last_upload_bytes: std::mem::size_of::<T>(),
        }
    }

    pub(crate) fn empty(device: &wgpu::Device, label: &str) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: std::mem::size_of::<T>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            value: None,
            #[cfg(test)]
            last_upload_bytes: 0,
        }
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn sync(&mut self, queue: &wgpu::Queue, value: T) -> usize {
        let bytes = bytemuck::bytes_of(&value);
        let uploaded = if self
            .value
            .as_ref()
            .is_some_and(|old| bytemuck::bytes_of(old) == bytes)
        {
            0
        } else {
            queue.write_buffer(&self.buffer, 0, bytes);
            self.value = Some(value);
            bytes.len()
        };
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        uploaded
    }
}
