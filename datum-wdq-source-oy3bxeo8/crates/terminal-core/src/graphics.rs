use crate::{
    GraphicDecodedBytesLimit, GraphicFramesLimit, GraphicObjectsLimit, GraphicPixelsLimit,
    LimitError, LimitKind, LogicalPoint, Rgba8, ScreenBuffer,
};
use std::sync::Arc;

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
    Kitty,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KittyImageId(u32);

impl KittyImageId {
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KittyPlacementId(u32);

impl KittyPlacementId {
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GraphicSourceRect {
    pub x: u32,
    pub y: u32,
    /// Zero means the remainder of the source image.
    pub width: u32,
    /// Zero means the remainder of the source image.
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GraphicCellExtent {
    pub columns: u32,
    pub rows: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GraphicPixelOffset {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KittyParentPlacement {
    pub image_id: KittyImageId,
    pub placement_id: KittyPlacementId,
    pub horizontal_cells: i32,
    pub vertical_cells: i32,
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
    pixels: Arc<[Rgba8]>,
    kitty_image_id: Option<KittyImageId>,
    kitty_image_number: Option<u32>,
    kitty_placement_id: Option<KittyPlacementId>,
    source: GraphicSourceRect,
    cells: GraphicCellExtent,
    offset: GraphicPixelOffset,
    z_index: i32,
    virtual_placement: bool,
    parent: Option<KittyParentPlacement>,
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

    pub const fn kitty_image_id(&self) -> Option<KittyImageId> {
        self.kitty_image_id
    }

    pub const fn kitty_image_number(&self) -> Option<u32> {
        self.kitty_image_number
    }

    pub const fn kitty_placement_id(&self) -> Option<KittyPlacementId> {
        self.kitty_placement_id
    }

    pub const fn source(&self) -> GraphicSourceRect {
        self.source
    }

    pub const fn cell_extent(&self) -> GraphicCellExtent {
        self.cells
    }

    pub const fn pixel_offset(&self) -> GraphicPixelOffset {
        self.offset
    }

    pub const fn z_index(&self) -> i32 {
        self.z_index
    }

    pub const fn is_virtual(&self) -> bool {
        self.virtual_placement
    }

    pub const fn parent(&self) -> Option<KittyParentPlacement> {
        self.parent
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
    pub(crate) placements: Vec<GraphicPlacement>,
    pub(crate) kitty: crate::kitty_store::KittyStore,
    next_id: u64,
    limits: GraphicLimits,
}

impl GraphicStore {
    pub(crate) fn new(limits: GraphicLimits) -> Self {
        Self {
            placements: Vec::new(),
            kitty: crate::kitty_store::KittyStore::new(),
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
        let objects = self
            .placements
            .len()
            .checked_add(self.kitty.images().len())
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicObjects,
            })?;
        self.limits.objects.checked_total(objects, 1)?;
        let frames = self
            .placements
            .len()
            .checked_add(self.kitty.total_frames())
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicFrames,
            })?;
        self.limits.frames.checked_total(frames, 1)?;
        let placement_pixels = self
            .placements
            .iter()
            .try_fold(0usize, |total, placement| {
                total.checked_add(placement.pixels.len())
            })
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicPixels,
            })?;
        let existing_pixels = placement_pixels
            .checked_add(self.kitty.total_pixels())
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
            pixels: image.pixels.into(),
            kitty_image_id: None,
            kitty_image_number: None,
            kitty_placement_id: None,
            source: GraphicSourceRect::default(),
            cells: GraphicCellExtent::default(),
            offset: GraphicPixelOffset::default(),
            z_index: 0,
            virtual_placement: false,
            parent: None,
        });
        Ok(id)
    }

    pub(crate) fn insert_kitty(
        &mut self,
        placement: crate::kitty_graphics::KittyPlacement,
    ) -> Result<GraphicId, LimitError> {
        let replacing = usize::from(self.placements.iter().any(|existing| {
            existing.kitty_image_id == placement.image_id
                && existing.kitty_placement_id == placement.placement_id
                && placement.image_id.is_some()
                && placement.placement_id.is_some()
        }));
        let objects = self
            .placements
            .len()
            .checked_add(self.kitty.images().len())
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicObjects,
            })?;
        self.limits
            .objects
            .checked_total(objects.saturating_sub(replacing), 1)?;
        let id = GraphicId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicObjects,
            })?;
        if let (Some(image_id), Some(placement_id)) = (placement.image_id, placement.placement_id) {
            self.placements.retain(|existing| {
                existing.kitty_image_id != Some(image_id)
                    || existing.kitty_placement_id != Some(placement_id)
            });
        }
        self.placements.push(GraphicPlacement {
            id,
            protocol: GraphicProtocol::Kitty,
            buffer: placement.buffer,
            anchor: placement.anchor,
            width: placement.width,
            height: placement.height,
            pixel_aspect: PixelAspect::SQUARE,
            pixels: placement.pixels,
            kitty_image_id: placement.image_id,
            kitty_image_number: placement.image_number,
            kitty_placement_id: placement.placement_id,
            source: placement.source,
            cells: placement.cells,
            offset: placement.offset,
            z_index: placement.z_index,
            virtual_placement: placement.virtual_placement,
            parent: placement.parent,
        });
        Ok(id)
    }

    pub(crate) fn sync_kitty_image(
        &mut self,
        image_id: KittyImageId,
        pixels: Arc<[Rgba8]>,
        width: u32,
        height: u32,
    ) {
        for placement in &mut self.placements {
            if placement.kitty_image_id == Some(image_id) {
                placement.pixels = Arc::clone(&pixels);
                placement.width = width;
                placement.height = height;
            }
        }
    }

    pub(crate) fn remove_kitty(&mut self, mut remove: impl FnMut(&GraphicPlacement) -> bool) {
        self.placements
            .retain(|placement| placement.protocol != GraphicProtocol::Kitty || !remove(placement));
        self.prune_missing_kitty_parents();
    }

    fn prune_missing_kitty_parents(&mut self) {
        loop {
            let missing_parent = self.placements.iter().find_map(|placement| {
                let parent = placement.parent?;
                (!self.contains_kitty_parent(parent.image_id, parent.placement_id))
                    .then_some(placement.id)
            });
            let Some(missing_parent) = missing_parent else {
                break;
            };
            self.placements
                .retain(|placement| placement.id != missing_parent);
        }
    }

    pub(crate) fn contains_kitty_parent(
        &self,
        image_id: KittyImageId,
        placement_id: KittyPlacementId,
    ) -> bool {
        self.placements.iter().any(|placement| {
            placement.kitty_image_id == Some(image_id)
                && placement.kitty_placement_id == Some(placement_id)
        })
    }

    pub(crate) fn kitty_parent_anchor(
        &self,
        image_id: KittyImageId,
        placement_id: KittyPlacementId,
    ) -> Option<LogicalPoint> {
        self.placements
            .iter()
            .find(|placement| {
                placement.kitty_image_id == Some(image_id)
                    && placement.kitty_placement_id == Some(placement_id)
            })
            .map(|placement| placement.anchor)
    }

    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = &GraphicPlacement> {
        self.placements.iter()
    }

    pub(crate) fn get(&self, id: GraphicId) -> Option<&GraphicPlacement> {
        self.placements.iter().find(|placement| placement.id == id)
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&GraphicPlacement) -> bool) {
        self.placements.retain(keep);
        self.prune_missing_kitty_parents();
    }

    pub(crate) fn clear_buffer(&mut self, buffer: ScreenBuffer) {
        self.placements
            .retain(|placement| placement.buffer != buffer);
        self.prune_missing_kitty_parents();
    }

    pub(crate) fn clear(&mut self) {
        self.placements.clear();
        self.kitty.clear();
    }
}
