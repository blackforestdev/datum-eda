use crate::kitty_pixels::{composite_block, rgba, valid_continuation_header};
use crate::kitty_protocol::{
    KittyAction, KittyControl, KittyGraphicsError, KittyMedium, decode_pixels, parse_control,
};
use crate::kitty_store::{KittyFrameData, KittyImageData, PendingKittyTransfer};
use crate::{
    Base64Limits, CoreError, CoreEvent, CoreUpdate, Damage, GraphicCellExtent, GraphicPixelOffset,
    GraphicSourceRect, KittyImageId, KittyParentPlacement, KittyPlacementId, LimitError, LimitKind,
    LogicalPoint, Rgba8, ScreenBuffer, TerminalCore,
};
use std::sync::Arc;

const MAX_RELATIVE_DEPTH: usize = 32;

#[derive(Clone, Debug)]
pub(crate) struct KittyPlacement {
    pub(crate) buffer: ScreenBuffer,
    pub(crate) anchor: LogicalPoint,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Arc<[Rgba8]>,
    pub(crate) image_id: Option<KittyImageId>,
    pub(crate) image_number: Option<u32>,
    pub(crate) placement_id: Option<KittyPlacementId>,
    pub(crate) source: GraphicSourceRect,
    pub(crate) cells: GraphicCellExtent,
    pub(crate) offset: GraphicPixelOffset,
    pub(crate) z_index: i32,
    pub(crate) virtual_placement: bool,
    pub(crate) parent: Option<KittyParentPlacement>,
}

impl TerminalCore {
    pub(crate) fn apply_kitty_graphics(
        &mut self,
        bytes: &[u8],
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let Some(body) = bytes.strip_prefix(b"G") else {
            return Ok(());
        };
        update.recognized = true;
        let (header, payload) = body
            .iter()
            .position(|byte| *byte == b';')
            .map_or((body, &[][..]), |offset| {
                (&body[..offset], &body[offset + 1..])
            });
        let control = match parse_control(header) {
            Ok(control) => control,
            Err(error) => return self.kitty_failure(None, 0, &error, update),
        };

        if matches!(control.action, KittyAction::Delete) {
            self.state.graphics.kitty.pending_mut().take();
        }
        match self.collect_kitty_transfer(control, header, payload) {
            Ok(Some((control, encoded))) => self.execute_kitty(control, &encoded, update),
            Ok(None) => Ok(()),
            Err(error) => self.kitty_failure(None, 0, &error, update),
        }
    }

    pub fn advance_kitty_animations(
        &mut self,
        elapsed_milliseconds: u64,
    ) -> Result<CoreUpdate, CoreError> {
        let mut update = CoreUpdate::new(self);
        for (image_id, pixels) in self.state.graphics.kitty.advance(elapsed_milliseconds) {
            let (width, height) = self
                .state
                .graphics
                .kitty
                .image(image_id)
                .map(|image| (image.width(), image.height()))
                .expect("advanced image remains stored");
            self.state
                .graphics
                .sync_kitty_image(image_id, pixels, width, height);
            self.push_damage(Damage::Graphics, &mut update)?;
        }
        Ok(update)
    }

    fn collect_kitty_transfer(
        &mut self,
        mut control: KittyControl,
        raw_header: &[u8],
        payload: &[u8],
    ) -> Result<Option<(KittyControl, Vec<u8>)>, KittyGraphicsError> {
        let transfer = matches!(
            control.action,
            KittyAction::Transmit
                | KittyAction::TransmitAndPut
                | KittyAction::Query
                | KittyAction::Frame
        );
        if !transfer {
            self.state.graphics.kitty.pending_mut().take();
            return Ok(Some((control, payload.to_vec())));
        }

        let encoded_limit = self
            .limits
            .graphic_decoded_bytes
            .get()
            .checked_add(2)
            .and_then(|value| value.checked_div(3))
            .and_then(|value| value.checked_mul(4))
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicDecodedBytes,
            })?;
        if let Some(mut pending) = self.state.graphics.kitty.pending_mut().take() {
            if !valid_continuation_header(raw_header, pending.control.action) {
                return Err(KittyGraphicsError::Malformed {
                    reason: "chunk continuation repeats or changes transfer metadata",
                });
            }
            let combined_length = pending.encoded.len().checked_add(payload.len()).ok_or(
                LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicDecodedBytes,
                },
            )?;
            if combined_length > encoded_limit {
                return Err(LimitError::Exceeded {
                    kind: LimitKind::GraphicDecodedBytes,
                    requested: combined_length,
                    maximum: encoded_limit,
                }
                .into());
            }
            pending.encoded.extend_from_slice(payload);
            pending.control.more = control.more;
            pending.control.quiet = control.quiet;
            control = pending.control.clone();
            if control.more {
                *self.state.graphics.kitty.pending_mut() = Some(pending);
                return Ok(None);
            }
            return Ok(Some((control, pending.encoded)));
        }
        if payload.len() > encoded_limit {
            return Err(LimitError::Exceeded {
                kind: LimitKind::GraphicDecodedBytes,
                requested: payload.len(),
                maximum: encoded_limit,
            }
            .into());
        }
        if control.more {
            *self.state.graphics.kitty.pending_mut() = Some(PendingKittyTransfer {
                control,
                encoded: payload.to_vec(),
            });
            Ok(None)
        } else {
            Ok(Some((control, payload.to_vec())))
        }
    }

    fn execute_kitty(
        &mut self,
        control: KittyControl,
        payload: &[u8],
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let result = match control.action {
            KittyAction::Transmit | KittyAction::TransmitAndPut | KittyAction::Query => {
                self.kitty_transmit(&control, payload, update)
            }
            KittyAction::Put => self.kitty_put(&control, update).map(|_| control.image_id),
            KittyAction::Delete => self
                .kitty_delete(&control, update)
                .map(|_| control.image_id),
            KittyAction::Frame => self
                .kitty_frame(&control, payload, update)
                .map(|_| control.image_id),
            KittyAction::Animate => self
                .kitty_animate(&control, update)
                .map(|_| control.image_id),
            KittyAction::Compose => self
                .kitty_compose(&control, update)
                .map(|_| control.image_id),
        };
        match result {
            Ok(reply_id) => self.kitty_success(&control, reply_id, update),
            Err(error) => self.kitty_failure(Some(&control), control.image_id, &error, update),
        }
    }

    fn kitty_transmit(
        &mut self,
        control: &KittyControl,
        payload: &[u8],
        update: &mut CoreUpdate,
    ) -> Result<u32, KittyGraphicsError> {
        if control.medium != KittyMedium::Direct {
            return Err(KittyGraphicsError::UnsupportedMedium);
        }
        let (width, height, pixels) = decode_pixels(
            payload,
            control,
            Base64Limits::graphics(self.limits.graphic_decoded_bytes, self.limits.parser_work),
            self.limits.into(),
        )?;
        if matches!(control.action, KittyAction::Query) {
            return Ok(control.image_id);
        }
        let other_pixels = self.sixel_pixel_count();
        let id = self.state.graphics.kitty.store_image(
            control.image_id,
            control.image_number,
            KittyImageData {
                width,
                height,
                pixels,
                transient: control.usage & 1 != 0,
            },
            self.kitty_store_limits(),
            other_pixels,
        )?;
        self.state
            .graphics
            .remove_kitty(|placement| placement.kitty_image_id() == Some(id));
        if matches!(control.action, KittyAction::TransmitAndPut) {
            self.kitty_put_resolved(control, id, update)?;
        }
        Ok(id.get())
    }

    fn kitty_put(
        &mut self,
        control: &KittyControl,
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let id = self.resolve_kitty_image(control)?;
        self.kitty_put_resolved(control, id, update)
    }

    fn kitty_put_resolved(
        &mut self,
        control: &KittyControl,
        id: KittyImageId,
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let image = self.state.graphics.kitty.image(id).ok_or_else(not_found)?;
        let pixels = image.pixels();
        let width = image.width();
        let height = image.height();
        let number = image.number();
        let placement_id = KittyPlacementId::new(control.placement_id);
        let parent = match (
            KittyImageId::new(control.parent_image_id),
            KittyPlacementId::new(control.parent_placement_id),
        ) {
            (None, None) => None,
            (Some(parent_image), Some(parent_placement)) => {
                if control.virtual_placement {
                    return Err(invalid("virtual placements cannot be relative"));
                }
                if !self
                    .state
                    .graphics
                    .contains_kitty_parent(parent_image, parent_placement)
                {
                    return Err(KittyGraphicsError::Malformed {
                        reason: "ENOPARENT",
                    });
                }
                self.check_relative_depth(parent_image, parent_placement, id, placement_id)?;
                Some(KittyParentPlacement {
                    image_id: parent_image,
                    placement_id: parent_placement,
                    horizontal_cells: control.horizontal_offset,
                    vertical_cells: control.vertical_offset,
                })
            }
            _ => return Err(invalid("relative parent requires both P and Q")),
        };
        let anchor = if let Some(parent) = parent {
            self.state
                .graphics
                .kitty_parent_anchor(parent.image_id, parent.placement_id)
                .expect("relative parent was validated")
        } else {
            self.state
                .logical_point_at(
                    self.state.cursor.position.row.get(),
                    self.state.cursor.position.column.get(),
                )
                .expect("cursor belongs to active grid")
        };
        let graphic_id = self.state.graphics.insert_kitty(KittyPlacement {
            buffer: self.state.active_buffer,
            anchor,
            width,
            height,
            pixels,
            image_id: Some(id),
            image_number: number,
            placement_id,
            source: GraphicSourceRect {
                x: control.x,
                y: control.y,
                width: control.crop_width,
                height: control.crop_height,
            },
            cells: GraphicCellExtent {
                columns: control.columns,
                rows: control.rows,
            },
            offset: GraphicPixelOffset {
                x: control.offset_x,
                y: control.offset_y,
            },
            z_index: control.z_index,
            virtual_placement: control.virtual_placement,
            parent,
        })?;
        self.push_event(CoreEvent::GraphicAdded(graphic_id), update)
            .map_err(protocol_core)?;
        self.push_damage(Damage::Graphics, update)
            .map_err(protocol_core)?;
        if !control.no_cursor_move && !control.virtual_placement && parent.is_none() {
            let columns = self.placement_columns(control, width, height);
            let rows = self.placement_rows(control, width, height);
            self.apply_screen(
                crate::ScreenAction::MoveCursor {
                    rows: rows.min(i32::MAX as u32) as i32,
                    columns: columns.min(i32::MAX as u32) as i32,
                },
                update,
            )
            .map_err(protocol_core)?;
        }
        Ok(())
    }

    fn kitty_frame(
        &mut self,
        control: &KittyControl,
        payload: &[u8],
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let id = self.resolve_kitty_image(control)?;
        let (block_width, block_height, block) = decode_pixels(
            payload,
            control,
            Base64Limits::graphics(self.limits.graphic_decoded_bytes, self.limits.parser_work),
            self.limits.into(),
        )?;
        let image = self.state.graphics.kitty.image(id).ok_or_else(not_found)?;
        let (width, height) = (image.width(), image.height());
        let replace =
            (control.rows != 0).then(|| usize::try_from(control.rows - 1).unwrap_or(usize::MAX));
        let base = if let Some(index) = replace {
            image
                .frames()
                .get(index)
                .map(|frame| frame.pixels().to_vec())
        } else if control.columns != 0 {
            image
                .frames()
                .get(usize::try_from(control.columns - 1).unwrap_or(usize::MAX))
                .map(|frame| frame.pixels().to_vec())
        } else {
            None
        };
        let mut canvas = base
            .unwrap_or_else(|| vec![rgba(control.background); image.frames()[0].pixels().len()]);
        composite_block(
            &mut canvas,
            width,
            height,
            &block,
            block_width,
            block_height,
            control.x,
            control.y,
            control.composition == 1,
        )?;
        self.state.graphics.kitty.add_frame(
            id,
            KittyFrameData {
                pixels: canvas.into(),
                gap_milliseconds: if control.z_set { control.z_index } else { 40 },
                transient: control.usage & 1 != 0,
            },
            replace,
            self.kitty_store_limits(),
            self.sixel_pixel_count(),
        )?;
        self.push_damage(Damage::Graphics, update)
            .map_err(protocol_core)?;
        Ok(())
    }

    fn placement_columns(&self, control: &KittyControl, width: u32, height: u32) -> u32 {
        if control.columns != 0 {
            return control.columns;
        }
        if control.rows != 0 && height != 0 {
            return width.saturating_mul(control.rows).div_ceil(height).max(1);
        }
        let cell_width = self
            .state
            .size
            .pixels
            .width
            .checked_div(u32::from(self.state.size.columns.get()))
            .unwrap_or(0)
            .max(1);
        width.div_ceil(cell_width).max(1)
    }

    fn placement_rows(&self, control: &KittyControl, width: u32, height: u32) -> u32 {
        if control.rows != 0 {
            return control.rows;
        }
        if control.columns != 0 && width != 0 {
            return height
                .saturating_mul(control.columns)
                .div_ceil(width)
                .max(1);
        }
        let cell_height = self
            .state
            .size
            .pixels
            .height
            .checked_div(u32::from(self.state.size.rows.get()))
            .unwrap_or(0)
            .max(1);
        height.div_ceil(cell_height).max(1)
    }

    fn check_relative_depth(
        &self,
        mut image_id: KittyImageId,
        mut placement_id: KittyPlacementId,
        new_image: KittyImageId,
        new_placement: Option<KittyPlacementId>,
    ) -> Result<(), KittyGraphicsError> {
        for _ in 0..MAX_RELATIVE_DEPTH {
            if image_id == new_image && Some(placement_id) == new_placement {
                return Err(KittyGraphicsError::Malformed { reason: "ECYCLE" });
            }
            let Some(parent) = self
                .state
                .graphics
                .placements
                .iter()
                .find(|placement| {
                    placement.kitty_image_id() == Some(image_id)
                        && placement.kitty_placement_id() == Some(placement_id)
                })
                .and_then(|placement| placement.parent())
            else {
                return Ok(());
            };
            image_id = parent.image_id;
            placement_id = parent.placement_id;
        }
        Err(KittyGraphicsError::Malformed { reason: "ETOODEEP" })
    }
}

fn invalid(reason: &'static str) -> KittyGraphicsError {
    KittyGraphicsError::Malformed { reason }
}

fn not_found() -> KittyGraphicsError {
    KittyGraphicsError::Malformed { reason: "ENOENT" }
}

fn protocol_core(error: CoreError) -> KittyGraphicsError {
    match error {
        CoreError::Limit(error) => KittyGraphicsError::Limit(error),
        _ => KittyGraphicsError::Malformed {
            reason: "terminal state rejected graphics operation",
        },
    }
}
