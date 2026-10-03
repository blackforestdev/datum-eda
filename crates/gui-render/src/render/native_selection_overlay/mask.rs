//! Frozen, source-bound mask intent. Labels are disposable visual groups, never
//! authored identities. Complete membership stays in the shared SelectionView.
use super::*;
use crate::presentation_ranges::{Presentation, Role, Source, Span};
use std::sync::Arc;
#[derive(Debug, PartialEq, Eq)]
struct Choice {
    key: String,
    label: u32,
    boundary: bool,
    islands: Vec<usize>,
}
struct Inner {
    source: Source,
    choices: Vec<Choice>,
    _permit: Option<crate::text_gpu::budget::Permit>,
}
#[derive(Clone)]
pub(crate) struct Request(Arc<Inner>);
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaskRequest")
            .field("source", &self.0.source)
            .field("choices", &self.0.choices)
            .finish()
    }
}
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.0.source == other.0.source && self.0.choices == other.0.choices
    }
}
impl Request {
    pub(crate) fn capture(
        view: &datum_gui_protocol::selection_view::SelectionView,
        scene: &BoardReviewSceneV1,
        surface: datum_gui_protocol::PaneContent,
    ) -> anyhow::Result<Option<Self>> {
        Self::capture_with_budget(
            view,
            scene,
            surface,
            &crate::text_gpu::budget::staging_process(),
        )
    }
    fn capture_with_budget(
        view: &datum_gui_protocol::selection_view::SelectionView,
        scene: &BoardReviewSceneV1,
        surface: datum_gui_protocol::PaneContent,
        budget: &Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<Option<Self>> {
        let Some(keys) = view.projection_keys(scene, surface) else {
            return Ok(None);
        };
        let compound = matches!(
            view.subject,
            datum_gui_protocol::SelectionSubject::Compound(_)
        );
        let boundary = !matches!(view.subject, datum_gui_protocol::SelectionSubject::Run(_));
        let mut bytes = crate::cpu_alloc::heap::capacity_bytes::<Choice>(keys.len());
        bytes = bytes
            .checked_add(container_bytes())
            .ok_or_else(|| anyhow::anyhow!("mask source layout overflow"))?;
        for value in [&scene.project_uuid, &scene.source_revision, &scene.scene_id] {
            bytes = bytes
                .checked_add(crate::cpu_alloc::heap::capacity_bytes::<u8>(value.len()))
                .ok_or_else(|| anyhow::anyhow!("mask source overflow"))?;
        }
        let _zone_permit = budget.reserve(crate::cpu_alloc::heap::capacity_bytes::<
            &datum_gui_protocol::ZonePrimitive,
        >(scene.zones.len()) as u64)?;
        let mut zones = Vec::new();
        zones.try_reserve_exact(scene.zones.len())?;
        zones.extend(scene.zones.iter());
        zones.sort_unstable_by(|a, b| a.object_id.cmp(&b.object_id));
        let zone_for = |key: &str| {
            zones
                .binary_search_by(|zone| zone.object_id.as_str().cmp(key))
                .ok()
                .map(|index| zones[index])
        };
        for key in keys {
            bytes = bytes
                .checked_add(crate::cpu_alloc::heap::capacity_bytes::<u8>(key.len()))
                .ok_or_else(|| anyhow::anyhow!("mask key overflow"))?;
            if let Some(zone) = zone_for(key) {
                bytes = bytes
                    .checked_add(crate::cpu_alloc::heap::capacity_bytes::<usize>(
                        zone.occupied_polygons().count(),
                    ))
                    .ok_or_else(|| anyhow::anyhow!("mask island overflow"))?;
            }
        }
        let permit = budget.reserve(bytes as u64)?;
        let source = Source::capture(scene, |_| Ok(()))?;
        let mut choices = Vec::new();
        choices.try_reserve_exact(keys.len())?;
        let member_layout = crate::cpu_alloc::heap::capacity_bytes::<
            &datum_gui_protocol::AuthoredSelectionIdentity,
        >(view.members.len());
        let _scratch = budget.reserve(member_layout as u64)?;
        let mut members = Vec::new();
        members.try_reserve_exact(view.members.len())?;
        members.extend(view.members.iter());
        // Both shared owner map and native members qualify the key. The context
        // envelope, DTO ordinals and spelling supply no membership authority.
        for key in keys {
            let owner = view
                .presentation_owner(scene, surface, key)
                .ok_or_else(|| {
                    anyhow::anyhow!("selected presentation has no authoritative owner: {key}")
                })?;
            let label = if compound {
                let ordinal = members
                    .binary_search(&owner)
                    .map_err(|_| anyhow::anyhow!("mask owner absent from complete selection"))?;
                u32::try_from(
                    ordinal
                        .checked_add(1)
                        .ok_or_else(|| anyhow::anyhow!("mask group overflow"))?,
                )?
            } else {
                1
            };
            let mut copy = String::new();
            copy.try_reserve_exact(key.len())?;
            copy.push_str(key);
            let mut islands = Vec::new();
            if let Some(zone) = zone_for(key) {
                islands.try_reserve_exact(zone.occupied_polygons().count())?;
                if let Some(qualified) = view.zone_regions.get(key) {
                    let _qualified_permit = budget
                        .reserve(crate::cpu_alloc::heap::capacity_bytes::<&[PointNm]>(
                            qualified.len(),
                        ) as u64)?;
                    let mut index = Vec::new();
                    index.try_reserve_exact(qualified.len())?;
                    index.extend(qualified.iter().map(Vec::as_slice));
                    let compare = |a: &[PointNm], b: &[PointNm]| {
                        a.iter()
                            .map(|p| (p.x, p.y))
                            .cmp(b.iter().map(|p| (p.x, p.y)))
                    };
                    index.sort_unstable_by(|a, b| compare(a, b));
                    for (ordinal, polygon) in zone.occupied_polygons().enumerate() {
                        if index
                            .binary_search_by(|region| compare(region, polygon))
                            .is_ok()
                        {
                            islands.push(ordinal);
                        }
                    }
                }
            }
            choices.push(Choice {
                key: copy,
                label,
                boundary,
                islands,
            });
        }
        Ok(Some(Self(Arc::new(Inner {
            source,
            choices,
            _permit: Some(permit),
        }))))
    }
    pub(crate) fn matches(&self, presentation: &Presentation) -> bool {
        presentation.source.as_ref() == Some(&self.0.source)
    }
    pub(crate) fn label(&self, span: &Span) -> u32 {
        let Some(key) = span.key.as_deref() else {
            return 0;
        };
        let Ok(index) = self
            .0
            .choices
            .binary_search_by(|choice| choice.key.as_str().cmp(key))
        else {
            return 0;
        };
        let choice = &self.0.choices[index];
        let selected = match span.role {
            Role::Material => true,
            Role::Void => false,
            Role::ZoneBoundary => choice.boundary,
            Role::ZoneIsland(index) => choice.islands.binary_search(&index).is_ok(),
        };
        if selected { choice.label } else { 0 }
    }
}

fn container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        return std::mem::size_of::<Inner>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        crate::cpu_alloc::heap::arc_bytes(Inner {
            source: Source::default(),
            choices: Vec::new(),
            _permit: None,
        })
    })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintRange {
    pub pane: datum_gui_protocol::PaneId,
    pub surface: SceneSurface,
    pub range: std::ops::Range<usize>,
    pub stroke: bool,
    /// Disposable visual group, never an authored identity or acquisition token.
    pub group: u32,
}
impl PreparedScene {
    /// Exact selected material incidence in the admitted retained painter. Void,
    /// stale/foreign source and disconnected Zone Run islands are excluded.
    /// This observes mask input; it does not establish presentation or acceptance.
    pub fn selected_paint_ranges<'a>(
        &'a self,
        board: &'a RetainedScene,
        schematic: Option<&'a RetainedScene>,
    ) -> impl Iterator<Item = PaintRange> + 'a {
        self.selection_masks
            .iter()
            .filter(|(pane, surface, _)| {
                self.surface_passes
                    .iter()
                    .any(|pass| pass.pane_id == *pane && pass.surface == *surface)
            })
            .flat_map(move |(pane, surface, request)| {
                let retained = match surface {
                    SceneSurface::Board => Some(board),
                    SceneSurface::Schematic => schematic,
                };
                let presentation = retained
                    .map(|retained| &retained.presentation)
                    .filter(|presentation| request.matches(presentation));
                let commands = match surface {
                    SceneSurface::Board => self.visible_draw_commands(),
                    SceneSurface::Schematic => {
                        retained.map_or(&[][..], RetainedScene::all_draw_commands)
                    }
                };
                presentation.into_iter().flat_map(move |presentation| {
                    commands.iter().flat_map(move |command| {
                        let (stroke, range, spans) = match command {
                            RetainedDrawCommand::Quads { range, .. } => {
                                (false, range, &presentation.quads)
                            }
                            RetainedDrawCommand::Strokes { range, .. } => {
                                (true, range, &presentation.strokes)
                            }
                        };
                        let start = range.start as usize;
                        let end = range.end as usize;
                        let index = spans.partition_point(|span| span.range.end <= start);
                        spans[index..]
                            .iter()
                            .take_while(move |span| span.range.start < end)
                            .filter_map(move |span| {
                                let group = request.label(span);
                                (group != 0).then(|| PaintRange {
                                    pane: *pane,
                                    surface: *surface,
                                    range: span.range.start.max(start)..span.range.end.min(end),
                                    stroke,
                                    group,
                                })
                            })
                    })
                })
            })
    }
}

#[cfg(test)]
#[path = "mask_tests.rs"]
mod tests;
