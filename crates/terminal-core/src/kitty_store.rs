use crate::{
    GraphicDecodedBytesLimit, GraphicFramesLimit, GraphicObjectsLimit, GraphicPixelsLimit,
    KittyImageId, LimitError, LimitKind, Rgba8,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KittyAnimationState {
    Stopped,
    Loading,
    Looping,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KittyFrame {
    pub(crate) pixels: Arc<[Rgba8]>,
    pub(crate) gap_milliseconds: i32,
    pub(crate) transient: bool,
}

impl KittyFrame {
    pub fn pixels(&self) -> &[Rgba8] {
        &self.pixels
    }

    pub const fn gap_milliseconds(&self) -> i32 {
        self.gap_milliseconds
    }

    pub const fn is_transient(&self) -> bool {
        self.transient
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KittyImage {
    id: KittyImageId,
    number: Option<u32>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frames: Vec<KittyFrame>,
    current_frame: usize,
    animation: KittyAnimationState,
    loops_remaining: Option<u32>,
    elapsed_milliseconds: u64,
    transient: bool,
}

impl KittyImage {
    pub const fn id(&self) -> KittyImageId {
        self.id
    }

    pub const fn number(&self) -> Option<u32> {
        self.number
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn frames(&self) -> &[KittyFrame] {
        &self.frames
    }

    pub const fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub const fn animation_state(&self) -> KittyAnimationState {
        self.animation
    }

    pub(crate) fn pixels(&self) -> Arc<[Rgba8]> {
        Arc::clone(&self.frames[self.current_frame].pixels)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingKittyTransfer {
    pub(crate) control: crate::kitty_protocol::KittyControl,
    pub(crate) encoded: Vec<u8>,
}

pub(crate) struct KittyImageData {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<Rgba8>,
    pub(crate) transient: bool,
}

pub(crate) struct KittyFrameData {
    pub(crate) pixels: Arc<[Rgba8]>,
    pub(crate) gap_milliseconds: i32,
    pub(crate) transient: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct KittyStore {
    images: Vec<KittyImage>,
    pending: Option<PendingKittyTransfer>,
    next_image_id: u32,
}

impl KittyStore {
    pub(crate) fn new() -> Self {
        Self {
            images: Vec::new(),
            pending: None,
            next_image_id: 1,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.images.clear();
        self.pending = None;
        self.next_image_id = 1;
    }

    pub(crate) fn images(&self) -> impl ExactSizeIterator<Item = &KittyImage> {
        self.images.iter()
    }

    pub(crate) fn total_pixels(&self) -> usize {
        self.images
            .iter()
            .flat_map(|image| &image.frames)
            .map(|frame| frame.pixels.len())
            .fold(0usize, usize::saturating_add)
    }

    pub(crate) fn total_frames(&self) -> usize {
        self.images
            .iter()
            .map(|image| image.frames.len())
            .fold(0usize, usize::saturating_add)
    }

    pub(crate) fn pending_mut(&mut self) -> &mut Option<PendingKittyTransfer> {
        &mut self.pending
    }

    pub(crate) fn resolve_id(&self, id: u32, number: u32) -> Option<KittyImageId> {
        if id != 0 {
            return KittyImageId::new(id).filter(|id| self.images.iter().any(|v| v.id == *id));
        }
        (number != 0)
            .then(|| {
                self.images
                    .iter()
                    .rev()
                    .find(|image| image.number == Some(number))
                    .map(|image| image.id)
            })
            .flatten()
    }

    pub(crate) fn image(&self, id: KittyImageId) -> Option<&KittyImage> {
        self.images.iter().find(|image| image.id == id)
    }

    pub(crate) fn image_mut(&mut self, id: KittyImageId) -> Option<&mut KittyImage> {
        self.images.iter_mut().find(|image| image.id == id)
    }

    pub(crate) fn store_image(
        &mut self,
        requested_id: u32,
        number: u32,
        data: KittyImageData,
        limits: StoreLimits,
        other_pixels: usize,
    ) -> Result<KittyImageId, LimitError> {
        let id = if requested_id != 0 {
            KittyImageId::new(requested_id).expect("nonzero checked")
        } else {
            self.allocate_id()?
        };
        let removed_pixels = self
            .image(id)
            .map(|image| {
                image
                    .frames
                    .iter()
                    .map(|frame| frame.pixels.len())
                    .fold(0usize, usize::saturating_add)
            })
            .unwrap_or(0usize);
        let removed_frames = self.image(id).map_or(0, |image| image.frames.len());
        let replacing = usize::from(self.image(id).is_some());
        limits
            .objects
            .checked_total(limits.other_objects.saturating_sub(replacing), 1)?;
        let retained_pixels = self.total_pixels().saturating_sub(removed_pixels);
        let retained_frames = self.total_frames().saturating_sub(removed_frames);
        limits.pixels.checked_total(other_pixels, retained_pixels)?;
        let retained_total =
            other_pixels
                .checked_add(retained_pixels)
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicPixels,
                })?;
        limits
            .pixels
            .checked_total(retained_total, data.pixels.len())?;
        let retained_frames = limits.other_frames.checked_add(retained_frames).ok_or(
            LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicFrames,
            },
        )?;
        limits.frames.checked_total(retained_frames, 1)?;
        let total_pixels = other_pixels
            .checked_add(retained_pixels)
            .and_then(|value| value.checked_add(data.pixels.len()))
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicPixels,
            })?;
        let total_bytes = total_pixels
            .checked_mul(4)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicDecodedBytes,
            })?;
        limits.decoded_bytes.check(total_bytes)?;

        self.images.retain(|image| image.id != id);
        self.images.push(KittyImage {
            id,
            number: (number != 0).then_some(number),
            width: data.width,
            height: data.height,
            frames: vec![KittyFrame {
                pixels: data.pixels.into(),
                gap_milliseconds: 0,
                transient: data.transient,
            }],
            current_frame: 0,
            animation: KittyAnimationState::Stopped,
            loops_remaining: None,
            elapsed_milliseconds: 0,
            transient: data.transient,
        });
        Ok(id)
    }

    pub(crate) fn add_frame(
        &mut self,
        id: KittyImageId,
        data: KittyFrameData,
        replace: Option<usize>,
        limits: StoreLimits,
        other_pixels: usize,
    ) -> Result<usize, LimitError> {
        let old = replace
            .and_then(|index| self.image(id)?.frames.get(index))
            .map_or(0, |frame| frame.pixels.len());
        let retained_pixels = self.total_pixels().saturating_sub(old);
        let retained_total =
            other_pixels
                .checked_add(retained_pixels)
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicPixels,
                })?;
        limits
            .pixels
            .checked_total(retained_total, data.pixels.len())?;
        let total_pixels = retained_total.checked_add(data.pixels.len()).ok_or(
            LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicPixels,
            },
        )?;
        let total_bytes = total_pixels
            .checked_mul(4)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicDecodedBytes,
            })?;
        limits.decoded_bytes.check(total_bytes)?;
        if replace.is_none() {
            let frames = limits.other_frames.checked_add(self.total_frames()).ok_or(
                LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicFrames,
                },
            )?;
            limits.frames.checked_total(frames, 1)?;
        }
        let image = self
            .image_mut(id)
            .expect("image resolved before frame store");
        let frame = KittyFrame {
            pixels: data.pixels,
            gap_milliseconds: data.gap_milliseconds,
            transient: data.transient,
        };
        if let Some(index) = replace {
            image.frames[index] = frame;
            Ok(index)
        } else {
            image.frames.push(frame);
            Ok(image.frames.len() - 1)
        }
    }

    pub(crate) fn control_animation(
        &mut self,
        id: KittyImageId,
        state: Option<KittyAnimationState>,
        current: Option<usize>,
        loops: Option<Option<u32>>,
        frame_gap: Option<(usize, i32)>,
    ) -> Option<Arc<[Rgba8]>> {
        let image = self.image_mut(id)?;
        if let Some(index) = current.filter(|index| *index < image.frames.len()) {
            image.current_frame = index;
        }
        if let Some((index, gap)) = frame_gap
            && let Some(frame) = image.frames.get_mut(index)
        {
            frame.gap_milliseconds = gap;
        }
        if let Some(state) = state {
            image.animation = state;
            if state == KittyAnimationState::Stopped {
                image.loops_remaining = None;
                image.elapsed_milliseconds = 0;
            }
        }
        if let Some(loops) = loops {
            image.loops_remaining = loops;
        }
        Some(image.pixels())
    }

    pub(crate) fn advance(&mut self, elapsed_ms: u64) -> Vec<(KittyImageId, Arc<[Rgba8]>)> {
        let mut changed = Vec::new();
        for image in &mut self.images {
            if image.animation == KittyAnimationState::Stopped || image.frames.len() < 2 {
                continue;
            }
            image.elapsed_milliseconds = image.elapsed_milliseconds.saturating_add(elapsed_ms);
            loop {
                let gap = image.frames[image.current_frame].gap_milliseconds;
                if gap < 0 {
                    image.current_frame = (image.current_frame + 1) % image.frames.len();
                } else if image.elapsed_milliseconds < gap.max(1) as u64 {
                    break;
                } else {
                    image.elapsed_milliseconds -= gap.max(1) as u64;
                    let next = image.current_frame + 1;
                    if next == image.frames.len() {
                        if image.animation == KittyAnimationState::Loading {
                            break;
                        }
                        if let Some(remaining) = image.loops_remaining.as_mut() {
                            if *remaining == 0 {
                                image.animation = KittyAnimationState::Stopped;
                                break;
                            }
                            *remaining -= 1;
                        }
                        image.current_frame = 0;
                    } else {
                        image.current_frame = next;
                    }
                }
                if image.frames[image.current_frame].gap_milliseconds >= 0 {
                    changed.push((image.id, image.pixels()));
                    break;
                }
            }
        }
        changed
    }

    pub(crate) fn remove_images(&mut self, mut remove: impl FnMut(&KittyImage) -> bool) {
        self.images.retain(|image| !remove(image));
    }

    fn allocate_id(&mut self) -> Result<KittyImageId, LimitError> {
        for _ in 0..u32::MAX {
            let value = self.next_image_id.max(1);
            self.next_image_id = value.wrapping_add(1).max(1);
            let id = KittyImageId::new(value).expect("allocated id is nonzero");
            if self.images.iter().all(|image| image.id != id) {
                return Ok(id);
            }
        }
        Err(LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicObjects,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StoreLimits {
    pub(crate) objects: GraphicObjectsLimit,
    pub(crate) other_objects: usize,
    pub(crate) pixels: GraphicPixelsLimit,
    pub(crate) decoded_bytes: GraphicDecodedBytesLimit,
    pub(crate) frames: GraphicFramesLimit,
    pub(crate) other_frames: usize,
}
