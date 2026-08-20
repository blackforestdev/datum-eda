use crate::kitty_pixels::{copy_rectangle, rectangles_overlap};
use crate::kitty_protocol::{KittyAction, KittyControl, KittyGraphicsError};
use crate::kitty_store::{KittyAnimationState, KittyFrameData, StoreLimits};
use crate::{
    CoreError, CoreUpdate, Damage, KittyImageId, KittyPlacementId, ReplyKind, TerminalCore,
};

impl TerminalCore {
    pub(crate) fn kitty_animate(
        &mut self,
        control: &KittyControl,
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let id = self.resolve_kitty_image(control)?;
        let state = match control.width {
            0 => None,
            1 => Some(KittyAnimationState::Stopped),
            2 => Some(KittyAnimationState::Loading),
            3 => Some(KittyAnimationState::Looping),
            _ => return Err(invalid("unknown animation state")),
        };
        let current = (control.columns != 0).then(|| control.columns.saturating_sub(1) as usize);
        let loops = (control.height != 0).then(|| {
            if control.height == 1 {
                None
            } else {
                Some(control.height - 1)
            }
        });
        let frame_gap =
            (control.rows != 0).then(|| (control.rows.saturating_sub(1) as usize, control.z_index));
        let image = self.state.graphics.kitty.image(id).ok_or_else(not_found)?;
        if current.is_some_and(|index| index >= image.frames().len())
            || frame_gap.is_some_and(|(index, _)| index >= image.frames().len())
        {
            return Err(not_found());
        }
        let pixels = self
            .state
            .graphics
            .kitty
            .control_animation(id, state, current, loops, frame_gap)
            .ok_or_else(not_found)?;
        let image = self.state.graphics.kitty.image(id).expect("resolved image");
        self.state
            .graphics
            .sync_kitty_image(id, pixels, image.width(), image.height());
        self.push_damage(Damage::Graphics, update)
            .map_err(protocol_core)?;
        Ok(())
    }

    pub(crate) fn kitty_compose(
        &mut self,
        control: &KittyControl,
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let id = self.resolve_kitty_image(control)?;
        if control.rows == 0 || control.columns == 0 {
            return Err(invalid(
                "composition requires source and destination frames",
            ));
        }
        let source_index = control.rows.saturating_sub(1) as usize;
        let destination_index = control.columns.saturating_sub(1) as usize;
        let image = self.state.graphics.kitty.image(id).ok_or_else(not_found)?;
        let source = image
            .frames()
            .get(source_index)
            .ok_or_else(not_found)?
            .pixels()
            .to_vec();
        let mut destination = image
            .frames()
            .get(destination_index)
            .ok_or_else(not_found)?
            .pixels()
            .to_vec();
        let width = if control.crop_width == 0 {
            image.width()
        } else {
            control.crop_width
        };
        let height = if control.crop_height == 0 {
            image.height()
        } else {
            control.crop_height
        };
        if source_index == destination_index
            && rectangles_overlap(
                control.x,
                control.y,
                control.offset_x,
                control.offset_y,
                width,
                height,
            )
        {
            return Err(invalid("overlapping self-composition is forbidden"));
        }
        copy_rectangle(
            &source,
            &mut destination,
            image.width(),
            image.height(),
            control.x,
            control.y,
            control.offset_x,
            control.offset_y,
            width,
            height,
            control.composition == 1,
        )?;
        let gap = image.frames()[destination_index].gap_milliseconds();
        let transient = image.frames()[source_index].is_transient()
            || image.frames()[destination_index].is_transient();
        let destination_is_current = image.current_frame() == destination_index;
        self.state.graphics.kitty.add_frame(
            id,
            KittyFrameData {
                pixels: destination.into(),
                gap_milliseconds: gap,
                transient,
            },
            Some(destination_index),
            self.kitty_store_limits(),
            self.sixel_pixel_count(),
        )?;
        if destination_is_current {
            let image = self.state.graphics.kitty.image(id).expect("image retained");
            self.state
                .graphics
                .sync_kitty_image(id, image.pixels(), image.width(), image.height());
        }
        self.push_damage(Damage::Graphics, update)
            .map_err(protocol_core)?;
        Ok(())
    }

    pub(crate) fn kitty_delete(
        &mut self,
        control: &KittyControl,
        update: &mut CoreUpdate,
    ) -> Result<(), KittyGraphicsError> {
        let hard = control.delete.is_ascii_uppercase();
        let mode = control.delete.to_ascii_lowercase();
        let resolved = self
            .state
            .graphics
            .kitty
            .resolve_id(control.image_id, control.image_number);
        let cursor_anchor = self.state.logical_point_at(
            self.state.cursor.position.row.get(),
            self.state.cursor.position.column.get(),
        );
        let target_anchor = self.state.logical_point_at(
            control.y.saturating_sub(1).min(u32::from(u16::MAX)) as u16,
            control.x.saturating_sub(1).min(u32::from(u16::MAX)) as u16,
        );
        let remove_ids: Vec<_> = self
            .state
            .graphics
            .placements
            .iter()
            .filter(|placement| match mode {
                b'a' => !placement.is_virtual(),
                b'i' | b'n' => {
                    placement.kitty_image_id() == resolved
                        && (control.placement_id == 0
                            || placement.kitty_placement_id().map(KittyPlacementId::get)
                                == Some(control.placement_id))
                }
                b'c' => Some(placement.anchor()) == cursor_anchor,
                b'p' | b'q' => {
                    Some(placement.anchor()) == target_anchor
                        && (mode != b'q' || placement.z_index() == control.z_index)
                }
                b'z' => placement.z_index() == control.z_index,
                b'x' => {
                    self.state
                        .resolve_logical_point(placement.anchor())
                        .column()
                        == Some(control.x.saturating_sub(1) as u16)
                }
                b'y' => {
                    self.state.resolve_logical_point(placement.anchor()).row()
                        == Some(control.y.saturating_sub(1) as u16)
                }
                b'r' => placement
                    .kitty_image_id()
                    .is_some_and(|id| id.get() >= control.x && id.get() <= control.y),
                b'f' => false,
                _ => false,
            })
            .map(|placement| placement.id())
            .collect();
        let affected_images: Vec<_> = self
            .state
            .graphics
            .placements
            .iter()
            .filter(|placement| remove_ids.contains(&placement.id()))
            .filter_map(|placement| placement.kitty_image_id())
            .collect();
        self.state
            .graphics
            .remove_kitty(|placement| remove_ids.contains(&placement.id()));
        if mode == b'f'
            && let Some(id) = resolved
            && let Some(image) = self.state.graphics.kitty.image_mut(id)
        {
            image.frames.truncate(1);
        }
        if hard {
            let referenced: Vec<_> = self
                .state
                .graphics
                .placements
                .iter()
                .filter_map(|placement| placement.kitty_image_id())
                .collect();
            self.state.graphics.kitty.remove_images(|image| {
                !referenced.contains(&image.id())
                    && (affected_images.contains(&image.id())
                        || match mode {
                            b'a' => true,
                            b'i' | b'n' => Some(image.id()) == resolved,
                            b'r' => image.id().get() >= control.x && image.id().get() <= control.y,
                            _ => false,
                        })
            });
        }
        self.push_damage(Damage::Graphics, update)
            .map_err(protocol_core)?;
        Ok(())
    }

    pub(crate) fn resolve_kitty_image(
        &self,
        control: &KittyControl,
    ) -> Result<KittyImageId, KittyGraphicsError> {
        self.state
            .graphics
            .kitty
            .resolve_id(control.image_id, control.image_number)
            .ok_or_else(not_found)
    }

    pub(crate) fn kitty_success(
        &mut self,
        control: &KittyControl,
        image_id: u32,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if control.quiet == 1
            || (control.image_id == 0
                && control.image_number == 0
                && !matches!(control.action, KittyAction::Query))
        {
            return Ok(());
        }
        self.kitty_reply(control, image_id, "OK", update)
    }

    pub(crate) fn kitty_failure(
        &mut self,
        control: Option<&KittyControl>,
        image_id: u32,
        error: &KittyGraphicsError,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if control.is_some_and(|control| control.quiet == 2) {
            return Ok(());
        }
        let fallback = KittyControl::default();
        let control = control.unwrap_or(&fallback);
        let code = match error {
            KittyGraphicsError::UnsupportedMedium => "ENOTSUP:external transfer disabled",
            KittyGraphicsError::Limit(_) => "ENOSPC:graphics limit reached",
            KittyGraphicsError::Malformed { reason: "ENOENT" } => "ENOENT:image not found",
            KittyGraphicsError::Malformed {
                reason: "ENOPARENT",
            } => "ENOPARENT:placement not found",
            KittyGraphicsError::Malformed { reason: "ECYCLE" } => "ECYCLE:relative placement cycle",
            KittyGraphicsError::Malformed { reason: "ETOODEEP" } => {
                "ETOODEEP:relative placement depth exceeded"
            }
            KittyGraphicsError::Malformed { .. } | KittyGraphicsError::Codec(_) => {
                "EINVAL:invalid graphics command"
            }
        };
        self.kitty_reply(control, image_id, code, update)
    }

    fn kitty_reply(
        &mut self,
        control: &KittyControl,
        image_id: u32,
        message: &str,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mut keys = format!("i={image_id}");
        if control.image_number != 0 {
            keys.push_str(&format!(",I={}", control.image_number));
        }
        if control.placement_id != 0 {
            keys.push_str(&format!(",p={}", control.placement_id));
        }
        self.push_reply(
            ReplyKind::Graphics,
            format!("\x1b_G{keys};{message}\x1b\\").into_bytes(),
            update,
        )
    }

    pub(crate) fn kitty_store_limits(&self) -> StoreLimits {
        StoreLimits {
            objects: self.limits.graphic_objects,
            other_objects: self
                .state
                .graphics
                .placements
                .len()
                .saturating_add(self.state.graphics.kitty.images().len()),
            pixels: self.limits.graphic_pixels,
            decoded_bytes: self.limits.graphic_decoded_bytes,
            frames: self.limits.graphic_frames,
            other_frames: self
                .state
                .graphics
                .placements
                .iter()
                .filter(|placement| placement.protocol() == crate::GraphicProtocol::Sixel)
                .count(),
        }
    }

    pub(crate) fn sixel_pixel_count(&self) -> usize {
        self.state
            .graphics
            .placements
            .iter()
            .filter(|placement| placement.protocol() == crate::GraphicProtocol::Sixel)
            .map(|placement| placement.pixels().len())
            .fold(0usize, usize::saturating_add)
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

trait AnchorResolutionExt {
    fn row(self) -> Option<u16>;
    fn column(self) -> Option<u16>;
}

impl AnchorResolutionExt for crate::AnchorResolution {
    fn row(self) -> Option<u16> {
        match self {
            crate::AnchorResolution::Screen { row, .. } => Some(row),
            _ => None,
        }
    }

    fn column(self) -> Option<u16> {
        match self {
            crate::AnchorResolution::Screen { column, .. } => Some(column),
            _ => None,
        }
    }
}
