//! Borrowed exact upload producers; fragment descriptors exist only while visited.
use super::BufferUpload;
use crate::text_gpu::lifetime::UploadTarget;
use std::ops::Range;

#[derive(Clone, Copy)]
pub(crate) enum Ranges<'a> {
    Prepared(&'a [Range<usize>]),
    Dirty { old: &'a [u8], stride: usize },
}
#[derive(Clone, Copy)]
pub(crate) struct Source<'a> {
    pub target: Option<UploadTarget<'a>>,
    pub buffer: &'a wgpu::Buffer,
    pub bytes: &'a [u8],
    pub ranges: Ranges<'a>,
}
#[derive(Default)]
pub(crate) struct SourceCount(pub usize);
impl<'a> Extend<Source<'a>> for SourceCount {
    fn extend<I: IntoIterator<Item = Source<'a>>>(&mut self, sources: I) {
        self.0 += sources.into_iter().count();
    }
}

// This is the same exact-word law used by ScreenBuffer, with no widened copies.
#[derive(Clone)]
pub(crate) struct Dirty<'o, 'a> {
    old: &'o [u8],
    new: &'a [u8],
    cursor: usize,
}
impl<'o, 'a> Dirty<'o, 'a> {
    pub(crate) fn new(old: &'o [u8], new: &'a [u8], stride: usize) -> Self {
        let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
        assert!(stride > 0 && stride % word == 0 && new.len() % stride == 0);
        Self {
            old,
            new,
            cursor: 0,
        }
    }
}
impl<'o, 'a> Iterator for Dirty<'o, 'a> {
    type Item = (u64, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
        let dirty = |offset: usize| {
            self.old.get(offset..offset + word) != Some(&self.new[offset..offset + word])
        };
        while self.cursor < self.new.len() && !dirty(self.cursor) {
            self.cursor += word;
        }
        let start = self.cursor;
        while self.cursor < self.new.len() && dirty(self.cursor) {
            self.cursor += word;
        }
        (start < self.cursor).then(|| (start as u64, &self.new[start..self.cursor]))
    }
}
#[derive(Clone)]
enum SourceRanges<'a> {
    Prepared(std::slice::Iter<'a, Range<usize>>),
    Dirty(Dirty<'a, 'a>),
}
#[derive(Clone)]
struct SourceIter<'a> {
    source: Source<'a>,
    ranges: SourceRanges<'a>,
}
impl<'a> SourceIter<'a> {
    fn new(source: Source<'a>) -> Self {
        let ranges = match source.ranges {
            Ranges::Prepared(ranges) => SourceRanges::Prepared(ranges.iter()),
            Ranges::Dirty { old, stride } => {
                SourceRanges::Dirty(Dirty::new(old, source.bytes, stride))
            }
        };
        Self { source, ranges }
    }
}
impl<'a> Iterator for SourceIter<'a> {
    type Item = BufferUpload<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (offset, bytes) = match &mut self.ranges {
            SourceRanges::Dirty(ranges) => ranges.next()?,
            SourceRanges::Prepared(ranges) => {
                let range = ranges.next()?;
                // Keep malformed ranges visible to the shared validation owner.
                (
                    range.start as u64,
                    self.source.bytes.get(range.clone()).unwrap_or(&[]),
                )
            }
        };
        Some(BufferUpload {
            target: self.source.target,
            buffer: self.source.buffer,
            offset,
            bytes,
        })
    }
}
#[derive(Clone)]
pub(crate) struct Uploads<'s, 'a> {
    flat: std::slice::Iter<'s, BufferUpload<'a>>,
    sources: std::slice::Iter<'s, Source<'a>>,
    current: Option<SourceIter<'a>>,
}
impl<'s, 'a> Uploads<'s, 'a> {
    pub(crate) fn sources(sources: &'s [Source<'a>]) -> Self {
        Self {
            flat: [].iter(),
            sources: sources.iter(),
            current: None,
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.clone().next().is_none()
    }
}
impl<'s, 'a> From<&'s [BufferUpload<'a>]> for Uploads<'s, 'a> {
    fn from(flat: &'s [BufferUpload<'a>]) -> Self {
        Self {
            flat: flat.iter(),
            sources: [].iter(),
            current: None,
        }
    }
}
impl<'s, 'a, const N: usize> From<&'s [BufferUpload<'a>; N]> for Uploads<'s, 'a> {
    fn from(flat: &'s [BufferUpload<'a>; N]) -> Self {
        flat.as_slice().into()
    }
}
impl<'s, 'a> From<&'s Vec<BufferUpload<'a>>> for Uploads<'s, 'a> {
    fn from(flat: &'s Vec<BufferUpload<'a>>) -> Self {
        flat.as_slice().into()
    }
}
impl<'s, 'a> From<&'s crate::text_gpu::staging_vec::StagingVec<BufferUpload<'a>>>
    for Uploads<'s, 'a>
{
    fn from(flat: &'s crate::text_gpu::staging_vec::StagingVec<BufferUpload<'a>>) -> Self {
        (&flat[..]).into()
    }
}
impl<'s, 'a> Iterator for Uploads<'s, 'a> {
    type Item = BufferUpload<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(upload) = self.flat.next() {
            return Some(*upload);
        }
        loop {
            if let Some(upload) = self.current.as_mut().and_then(Iterator::next) {
                return Some(upload);
            }
            self.current = Some(SourceIter::new(*self.sources.next()?));
        }
    }
}

#[cfg(test)]
#[path = "upload_plan/tests.rs"]
mod tests;
