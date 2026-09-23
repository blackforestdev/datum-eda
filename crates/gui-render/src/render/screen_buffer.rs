//! Bounded last-content ownership for immediate screen-space vertex uploads.
use super::vertex_allocation::VertexAllocation;

// One snapshot per semantic stream, not frame history. Larger streams still
// render normally but bypass CPU retention. Nine fixed screen streams retain
// at most 9 * 256 KiB per renderer. Each visible terminal image quad additionally
// retains its 96-byte vertex snapshot.
const MAX_SNAPSHOT_BYTES: usize = 256 * 1024;

#[derive(Default)]
pub(crate) struct ScreenBuffer {
    snapshot: Box<[u8]>,
    allocation: VertexAllocation,
    pending: Vec<std::ops::Range<usize>>,
    // Only over-cap streams need an extra payload; normal uploads borrow snapshot.
    pending_large: Box<[u8]>,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl ScreenBuffer {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.allocation.buffer()
    }

    pub(crate) fn submission_ref(&self) -> Option<crate::text_gpu::lifetime::SubmissionRef> {
        self.allocation.submission_ref()
    }

    pub(crate) fn cancel_uploads(&mut self) {
        if !self.pending.is_empty() {
            self.pending.clear();
            self.snapshot = Box::default();
            self.pending_large = Box::default();
        }
    }

    pub(crate) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        let bytes = if self.pending_large.is_empty() {
            &self.snapshot
        } else {
            &self.pending_large
        };
        for range in self.pending.drain(..) {
            // Multiple preparations before submission supersede the old tail.
            let end = range.end.min(bytes.len());
            if range.start < end {
                queue.write_buffer(
                    self.allocation.buffer().unwrap(),
                    range.start as u64,
                    &bytes[range.start..end],
                );
            }
        }
        self.pending_large = Box::default();
    }

    /// Exact bytes are the key: equal size, allocator reuse, NaN payloads and
    /// signed zero cannot produce a false hit. Placement is already baked into
    /// these screen vertices; painter order stays in the caller's draw schedule.
    pub(crate) fn sync<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        label: &str,
        vertices: &[T],
    ) -> anyhow::Result<usize> {
        #[cfg(test)]
        {
            self.last_upload_bytes = 0;
        }
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.is_empty() {
            *self = Self::default();
            return Ok(0);
        }
        if self.allocation.buffer().is_some() && self.snapshot.as_ref() == bytes {
            return Ok(0);
        }
        let uploaded = if self.allocation.replace_if_needed(device, label, bytes)? {
            self.pending.clear();
            self.pending.push(0..bytes.len());
            bytes.len()
        } else {
            dirty_ranges(
                &self.snapshot,
                bytes,
                std::mem::size_of::<T>(),
                |offset, bytes| {
                    self.pending
                        .push(offset as usize..offset as usize + bytes.len())
                },
            )
        };
        if bytes.len() <= MAX_SNAPSHOT_BYTES {
            self.pending_large = Box::default();
            if self.snapshot.len() == bytes.len() {
                self.snapshot.copy_from_slice(bytes);
            } else {
                self.snapshot = bytes.into();
            }
        } else {
            self.snapshot = Box::default();
            self.pending_large = bytes.into();
        }
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        Ok(uploaded)
    }
}

// A vertex is the update unit. Join adjacent changed vertices, preserving
// unchanged vertex gaps. This avoids one queue write per coordinate/color word
// during camera changes. Trim clean edge words within each resulting span;
// internal clean words remain coalesced, so this is not byte-minimal transfer.
// Production Vertex has a four-byte-aligned stride.
pub(crate) fn write_dirty_ranges(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    old: &[u8],
    new: &[u8],
    stride: usize,
) -> usize {
    dirty_ranges(old, new, stride, |offset, bytes| {
        queue.write_buffer(buffer, offset, bytes)
    })
}

fn dirty_ranges(old: &[u8], new: &[u8], stride: usize, mut write: impl FnMut(u64, &[u8])) -> usize {
    assert!(stride > 0);
    assert_eq!(stride % wgpu::COPY_BUFFER_ALIGNMENT as usize, 0);
    let mut start = None;
    let mut uploaded = 0;
    for offset in (0..new.len()).step_by(stride) {
        let end = offset + stride;
        if old.get(offset..end) != Some(&new[offset..end]) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            uploaded += write_trimmed_span(&mut write, old, new, begin, offset);
        }
    }
    if let Some(begin) = start {
        uploaded += write_trimmed_span(&mut write, old, new, begin, new.len());
    }
    uploaded
}

// Preserve the existing number of queue writes. Word-by-word queue writes and
// thousands of individual buffer copies both regress moving-stream CPU cost.
fn write_trimmed_span(
    write: &mut impl FnMut(u64, &[u8]),
    old: &[u8],
    new: &[u8],
    mut begin: usize,
    mut end: usize,
) -> usize {
    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    while begin < end && old.get(begin..begin + word) == Some(&new[begin..begin + word]) {
        begin += word;
    }
    while begin < end && old.get(end - word..end) == Some(&new[end - word..end]) {
        end -= word;
    }
    if begin < end {
        write(begin as u64, &new[begin..end]);
    }
    end - begin
}

#[cfg(all(test, feature = "visual"))]
#[path = "screen_buffer/tests.rs"]
mod tests;
