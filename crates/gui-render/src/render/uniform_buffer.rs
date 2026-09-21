//! Fixed-size uniform ownership with exact aligned changed-range uploads.
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
        let uploaded = write_changed_ranges(
            self.value.as_ref().map(bytemuck::bytes_of),
            bytes,
            |offset, data| queue.write_buffer(&self.buffer, offset as u64, data),
        );
        if uploaded != 0 {
            self.value = Some(value);
        }
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        uploaded
    }
}

// Uniforms are small fixed records (16-byte screen and 64-byte camera), unlike
// large vertex streams. Coalesce adjacent dirty words without transferring
// internal clean gaps; queue offsets and sizes obey COPY_BUFFER_ALIGNMENT.
fn write_changed_ranges(
    old: Option<&[u8]>,
    new: &[u8],
    mut write: impl FnMut(usize, &[u8]),
) -> usize {
    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    assert_eq!(new.len() % word, 0);
    let Some(old) = old else {
        write(0, new);
        return new.len();
    };
    assert_eq!(old.len(), new.len());
    if old == new {
        return 0;
    }
    let mut uploaded = 0;
    let mut start = None;
    for offset in (0..new.len()).step_by(word) {
        if old[offset..offset + word] != new[offset..offset + word] {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            write(begin, &new[begin..offset]);
            uploaded += offset - begin;
        }
    }
    if let Some(begin) = start {
        write(begin, &new[begin..]);
        uploaded += new.len() - begin;
    }
    uploaded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_ranges_transfer_only_dirty_aligned_words() {
        for mask in 0_u32..256 {
            let old = [0_u8; 32];
            let mut new = old;
            for index in 0..8 {
                if mask & (1 << index) != 0 {
                    new[4 * index + index % 4] = 1;
                }
            }
            let mut result = old;
            let mut writes = 0;
            let bytes = write_changed_ranges(Some(&old), &new, |offset, data| {
                assert_eq!(offset % 4, 0);
                assert_eq!(data.len() % 4, 0);
                for word in data.chunks_exact(4) {
                    assert_ne!(word, [0; 4]);
                }
                result[offset..offset + data.len()].copy_from_slice(data);
                writes += 1;
            });
            assert_eq!(result, new);
            assert_eq!(bytes, mask.count_ones() as usize * 4);
            assert_eq!(writes, (mask & !(mask << 1)).count_ones());
        }
        let mut writes = 0;
        assert_eq!(
            write_changed_ranges(None, &[0; 64], |offset, data| {
                assert_eq!(offset, 0);
                assert_eq!(data, [0; 64]);
                writes += 1;
            }),
            64
        );
        assert_eq!(writes, 1, "new storage must be initialized in full");
    }
}
