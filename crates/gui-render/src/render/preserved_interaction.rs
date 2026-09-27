//! One submitted MSAA composition and its bounded interaction footprint.
//! This owns no attachment: its key refers to the renderer's existing owner.
use super::interaction_damage::{Region, Regions};
use super::{
    PreparedScene, RectPx, SceneSurface, ShellLayout, gpu_surface::SurfaceAttachmentSnapshot,
};

#[derive(Debug)]
pub(crate) struct PreservedInteraction {
    revision: u64,
    layout: ShellLayout,
    viewport: RectPx,
    attachment: SurfaceAttachmentSnapshot,
    schematic_pass: bool,
    footprint: Regions,
}

impl PreservedInteraction {
    pub fn capture(
        prepared: &PreparedScene,
        attachment: SurfaceAttachmentSnapshot,
        schematic_pass: bool,
    ) -> Option<Self> {
        // Legacy/fallback paths remain full-frame until their complete static
        // dependencies and clipping are represented by the composed path.
        if prepared.surface_passes().is_empty() {
            return None;
        }
        let extent = Region {
            x: 0,
            y: 0,
            width: attachment.extent.0,
            height: attachment.extent.1,
        };
        let mut footprint = Regions::default();
        for (surface, vertices) in [
            (SceneSurface::Board, prepared.board_interaction_vertices()),
            (
                SceneSurface::Schematic,
                prepared.schematic_overlay_vertices(),
            ),
        ] {
            if vertices.is_empty() {
                continue;
            }
            let viewport = prepared
                .interaction_viewport(surface)
                .or_else(|| (surface == SceneSurface::Board).then_some(prepared.scene_viewport))?;
            if let Some(clip) = Region::viewport(viewport, extent).ok()? {
                footprint.include_vertices(vertices, clip)?;
            }
        }
        Some(Self {
            revision: prepared.composition_revision.identity(),
            layout: prepared.layout.clone(),
            viewport: prepared.scene_viewport,
            attachment,
            schematic_pass,
            footprint,
        })
    }

    /// Compare before reusing samples. The caller additionally requires unchanged
    /// retained-world upload sources and must have taken the previous record out
    /// of the renderer before any fallible/deferring preparation operation.
    pub fn damage(&self, next: &Self) -> Option<Regions> {
        if self.revision != next.revision
            || self.layout != next.layout
            || self.viewport != next.viewport
            || self.attachment != next.attachment
            || self.schematic_pass != next.schematic_pass
        {
            return None;
        }
        self.footprint.union(&next.footprint)
    }
}

impl super::Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_interaction_damage(
        &mut self,
        device: &wgpu::Device,
        prepared: &PreparedScene,
        schematic_present: bool,
        world_unchanged: bool,
        previous_composition: Option<PreservedInteraction>,
        width: u32,
        height: u32,
    ) -> (Option<PreservedInteraction>, Regions, bool) {
        let next_composition = self.surface_attachment_snapshot().and_then(|attachment| {
            PreservedInteraction::capture(prepared, attachment, schematic_present)
        });
        let damage = world_unchanged
            .then(|| {
                previous_composition
                    .as_ref()
                    .zip(next_composition.as_ref())
                    .and_then(|(old, next)| old.damage(next))
            })
            .flatten()
            .filter(|regions| !regions.as_slice().is_empty());
        let partial = damage.is_some();
        let regions = damage.unwrap_or_else(|| {
            let mut regions = Regions::default();
            regions
                .add(Region {
                    x: 0,
                    y: 0,
                    width,
                    height,
                })
                .expect("attachment extent fits damage coordinates");
            regions
        });
        self.damage_regions = if partial { regions.as_slice().len() } else { 0 };
        if partial {
            self.damage_reset.get_or_init(|| {
                super::damage_reset::pipeline(device, self.msaa_format, self.msaa_samples)
            });
        }
        self.frame_damage_regions = regions;
        (next_composition, regions, partial)
    }
}

impl super::Renderer {
    /// Physical scissors replayed by each active text pass. This describes this
    /// attempt's encoding plan; submission still needs separate confirmation.
    pub fn frame_redraw_scissors(&self) -> impl Iterator<Item = [u32; 4]> + '_ {
        self.frame_damage_regions
            .as_slice()
            .iter()
            .map(|r| [r.x, r.y, r.width, r.height])
    }

    /// Actual region receipts decoded against this attempt's immutable damage
    /// plan. Invalid receipts yield None items so observers fail closed.
    pub fn encoded_region_scissors(
        &self,
        mask: u32,
        clip: [u32; 4],
    ) -> impl Iterator<Item = Option<[u32; 4]>> + '_ {
        let clip = Region {
            x: clip[0],
            y: clip[1],
            width: clip[2],
            height: clip[3],
        };
        (0..32)
            .filter(move |index| mask & (1 << index) != 0)
            .map(move |index| {
                self.frame_damage_regions
                    .as_slice()
                    .get(index)
                    .and_then(|region| region.intersect(clip))
                    .map(|region| [region.x, region.y, region.width, region.height])
            })
    }
}
