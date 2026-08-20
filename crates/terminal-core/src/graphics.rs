use crate::{
    GraphicDecodedBytesLimit, GraphicFramesLimit, GraphicObjectsLimit, GraphicPixelsLimit,
    LimitError, LimitKind, LogicalPoint, Rgba8, ScreenBuffer,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GraphicId(u64);

impl GraphicId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphicProtocol {
    Sixel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphicAnchorResolution {
    History {
        row: usize,
        column: u16,
    },
    Screen {
        row: u16,
        column: u16,
        visible_pixel_width: u32,
        visible_pixel_height: u32,
    },
    InactiveBuffer,
    Trimmed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelAspect {
    pub numerator: u32,
    pub denominator: u32,
}

impl PixelAspect {
    pub const SQUARE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    pub const fn new(numerator: u32, denominator: u32) -> Option<Self> {
        if numerator == 0 || denominator == 0 {
            None
        } else {
            Some(Self {
                numerator,
                denominator,
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicPlacement {
    id: GraphicId,
    protocol: GraphicProtocol,
    buffer: ScreenBuffer,
    anchor: LogicalPoint,
    width: u32,
    height: u32,
    pixel_aspect: PixelAspect,
    pixels: Vec<Rgba8>,
}

impl GraphicPlacement {
    pub const fn id(&self) -> GraphicId {
        self.id
    }

    pub const fn protocol(&self) -> GraphicProtocol {
        self.protocol
    }

    pub const fn buffer(&self) -> ScreenBuffer {
        self.buffer
    }

    pub const fn anchor(&self) -> LogicalPoint {
        self.anchor
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn pixel_aspect(&self) -> PixelAspect {
        self.pixel_aspect
    }

    pub fn pixels(&self) -> &[Rgba8] {
        &self.pixels
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GraphicLimits {
    pub(crate) objects: GraphicObjectsLimit,
    pub(crate) pixels: GraphicPixelsLimit,
    pub(crate) decoded_bytes: GraphicDecodedBytesLimit,
    pub(crate) frames: GraphicFramesLimit,
}

#[derive(Clone, Debug)]
pub(crate) struct GraphicStore {
    placements: Vec<GraphicPlacement>,
    next_id: u64,
    limits: GraphicLimits,
}

impl GraphicStore {
    pub(crate) fn new(limits: GraphicLimits) -> Self {
        Self {
            placements: Vec::new(),
            next_id: 0,
            limits,
        }
    }

    pub(crate) fn insert_sixel(
        &mut self,
        buffer: ScreenBuffer,
        anchor: LogicalPoint,
        image: crate::SixelImage,
    ) -> Result<GraphicId, LimitError> {
        self.limits
            .objects
            .checked_total(self.placements.len(), 1)?;
        self.limits.frames.checked_total(self.placements.len(), 1)?;
        let existing_pixels = self
            .placements
            .iter()
            .try_fold(0usize, |total, placement| {
                total.checked_add(placement.pixels.len())
            })
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicPixels,
            })?;
        self.limits
            .pixels
            .checked_total(existing_pixels, image.pixels.len())?;
        let existing_bytes =
            existing_pixels
                .checked_mul(4)
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicDecodedBytes,
                })?;
        let image_bytes =
            image
                .pixels
                .len()
                .checked_mul(4)
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicDecodedBytes,
                })?;
        self.limits
            .decoded_bytes
            .checked_total(existing_bytes, image_bytes)?;
        let id = GraphicId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicObjects,
            })?;
        self.placements.push(GraphicPlacement {
            id,
            protocol: GraphicProtocol::Sixel,
            buffer,
            anchor,
            width: image.width,
            height: image.height,
            pixel_aspect: image.pixel_aspect,
            pixels: image.pixels,
        });
        Ok(id)
    }

    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = &GraphicPlacement> {
        self.placements.iter()
    }

    pub(crate) fn get(&self, id: GraphicId) -> Option<&GraphicPlacement> {
        self.placements.iter().find(|placement| placement.id == id)
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&GraphicPlacement) -> bool) {
        self.placements.retain(keep);
    }

    pub(crate) fn clear_buffer(&mut self, buffer: ScreenBuffer) {
        self.placements
            .retain(|placement| placement.buffer != buffer);
    }

    pub(crate) fn clear(&mut self) {
        self.placements.clear();
    }
}
