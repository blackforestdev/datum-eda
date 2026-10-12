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
    minimum: bool,
    islands: Vec<usize>,
}
struct Inner {
    source: Source,
    high_contrast: bool,
    dim_unrelated: bool,
    choices: Vec<Choice>,
    screen: Vec<Arc<super::cue_plan::PanePlan>>,
    screen_permit: Option<crate::text_gpu::budget::Permit>,
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
        self.0.source == other.0.source
            && self.0.choices == other.0.choices
            && self.0.screen == other.0.screen
            && self.0.high_contrast == other.0.high_contrast
            && self.0.dim_unrelated == other.0.dim_unrelated
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
        if keys.is_empty() {
            return Ok(None);
        }
        let related = match surface {
            datum_gui_protocol::PaneContent::Board => &view.related_board_keys,
            datum_gui_protocol::PaneContent::Schematic => &view.related_schematic_keys,
            datum_gui_protocol::PaneContent::Library
            | datum_gui_protocol::PaneContent::Revision(_) => unreachable!(),
        };
        let count = keys.union(related).count();
        let compound = matches!(
            view.subject,
            datum_gui_protocol::SelectionSubject::Compound(_)
        );
        let boundary = !matches!(view.subject, datum_gui_protocol::SelectionSubject::Run(_));
        let mut bytes = crate::cpu_alloc::heap::capacity_bytes::<Choice>(count);
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
        for key in keys.union(related) {
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
        choices.try_reserve_exact(count)?;
        let member_layout = crate::cpu_alloc::heap::capacity_bytes::<
            &datum_gui_protocol::AuthoredSelectionIdentity,
        >(view.members.len());
        let _scratch = budget.reserve(member_layout as u64)?;
        let mut members = Vec::new();
        members.try_reserve_exact(view.members.len())?;
        members.extend(view.members.iter());
        // Both shared owner map and native members qualify the key. The context
        // envelope, DTO ordinals and spelling supply no membership authority.
        for key in keys.union(related) {
            let owner = keys
                .contains(key)
                .then(|| {
                    view.presentation_owner(scene, surface, key).ok_or_else(|| {
                        anyhow::anyhow!("selected presentation has no authoritative owner: {key}")
                    })
                })
                .transpose()?;
            let label = if let Some(owner) = owner {
                if compound {
                    let ordinal = members.binary_search(&owner).map_err(|_| {
                        anyhow::anyhow!("mask owner absent from complete selection")
                    })?;
                    u32::try_from(
                        ordinal
                            .checked_add(1)
                            .ok_or_else(|| anyhow::anyhow!("mask group overflow"))?,
                    )?
                } else {
                    1
                }
            } else {
                0
            };
            anyhow::ensure!(
                label != u32::MAX,
                "selection visual group exceeds mask representation"
            );
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
                minimum: false,
                islands,
            });
        }
        Ok(Some(Self(Arc::new(Inner {
            source,
            high_contrast: false,
            dim_unrelated: false,
            choices,
            screen: Vec::new(),
            screen_permit: None,
            _permit: Some(permit),
        }))))
    }
    /// Typed artifact intent may have no authored projection keys. Its source
    /// and producing selection remain explicit; no synthetic authored key exists.
    pub(crate) fn capture_screen(
        view: &datum_gui_protocol::selection_view::SelectionView,
        scene: &BoardReviewSceneV1,
        high_contrast: bool,
    ) -> anyhow::Result<Option<Self>> {
        if view.stale || !datum_gui_protocol::selection_artifacts::is_artifact(&view.subject) {
            return Ok(None);
        }
        if !matches!(
            view.field_ref("artifact_payload"),
            Some(datum_gui_protocol::selection_view::InspectionValue::Common(
                _
            ))
        ) {
            return Ok(None);
        }
        let budget = crate::text_gpu::budget::staging_process();
        let bytes = container_bytes()
            .checked_add(
                [&scene.project_uuid, &scene.source_revision, &scene.scene_id]
                    .into_iter()
                    .map(|value| crate::cpu_alloc::heap::capacity_bytes::<u8>(value.len()))
                    .sum::<usize>(),
            )
            .ok_or_else(|| anyhow::anyhow!("mask source overflow"))?;
        let permit = budget.reserve(u64::try_from(bytes)?)?;
        let source = Source::capture(scene, |_| Ok(()))?;
        Ok(Some(Self(Arc::new(Inner {
            source,
            high_contrast,
            dim_unrelated: false,
            choices: Vec::new(),
            screen: Vec::new(),
            screen_permit: None,
            _permit: Some(permit),
        }))))
    }
    pub(crate) fn set_screen(
        &mut self,
        plans: impl Iterator<Item = Arc<super::cue_plan::PanePlan>> + Clone,
    ) -> anyhow::Result<()> {
        let inner = Arc::get_mut(&mut self.0).expect("qualify newly captured mask before sharing");
        let count = plans.clone().count();
        let bytes = crate::cpu_alloc::heap::capacity_bytes::<Arc<super::cue_plan::PanePlan>>(count);
        let permit = crate::text_gpu::budget::staging_process().reserve(u64::try_from(bytes)?)?;
        inner.screen.try_reserve_exact(count)?;
        inner.screen.extend(plans);
        inner.screen_permit = Some(permit);
        Ok(())
    }
    pub(crate) fn screen(&self) -> impl Iterator<Item = &super::cue_plan::ScreenSpan> {
        self.0.screen.iter().flat_map(|plan| plan.screen.iter())
    }
    pub(crate) fn screen_count(&self) -> usize {
        self.0.screen.iter().map(|plan| plan.screen.len()).sum()
    }
    pub(crate) fn detailed_overflow(&self, emitted_glyphs: usize) -> bool {
        self.0
            .screen
            .iter()
            .map(|plan| plan.detail_primitives)
            .fold(emitted_glyphs, usize::saturating_add)
            > super::cue_plan::DETAILED_PRIMITIVES
    }
    pub(crate) fn authored(&self) -> bool {
        !self.0.choices.is_empty()
    }
    pub(crate) fn palette(&self) -> [f32; 3] {
        self.0
            .screen
            .iter()
            .find(|plan| {
                plan.subject
                    .as_ref()
                    .is_some_and(datum_gui_protocol::selection_artifacts::is_artifact)
            })
            .map_or([206.0 / 255.0, 90.0 / 255.0, 146.0 / 255.0], |plan| {
                plan.palette
            })
    }
    pub(crate) fn high_contrast(&self) -> bool {
        self.0.high_contrast
    }
    pub(crate) fn dim_unrelated(&self) -> bool {
        self.0.dim_unrelated
    }
    pub(super) fn configure(
        &mut self,
        lod: &super::lod::Plan,
        view: &datum_gui_protocol::selection_view::SelectionView,
        scene: &BoardReviewSceneV1,
        surface: datum_gui_protocol::PaneContent,
        high_contrast: bool,
        dim_unrelated: bool,
    ) {
        let inner =
            Arc::get_mut(&mut self.0).expect("configure newly captured mask before sharing");
        inner.high_contrast = high_contrast;
        inner.dim_unrelated = dim_unrelated && !view.related.is_empty();
        for choice in &mut inner.choices {
            choice.minimum = !lod.detailed(view, scene, surface, &choice.key);
        }
    }
    pub(crate) fn matches(&self, presentation: &Presentation) -> bool {
        presentation.source.as_ref() == Some(&self.0.source)
    }
    /// 0 unrelated authored, 1 selected material, 2 related baseline,
    /// 3 anonymous structure/void. These are paint roles, never identities.
    pub(crate) fn material_role(&self, span: &Span) -> u32 {
        if span.role == Role::Void {
            return 3;
        }
        let Some(key) = span.key.as_deref() else {
            return 3;
        };
        let Ok(index) = self
            .0
            .choices
            .binary_search_by(|choice| choice.key.as_str().cmp(key))
        else {
            return 0;
        };
        let choice = &self.0.choices[index];
        if choice.label == 0 {
            return 2;
        }
        if Self::selected_part(choice, span.role) {
            1
        } else {
            0
        }
    }
    fn selected_part(choice: &Choice, role: Role) -> bool {
        match role {
            Role::Material => true,
            Role::Void => false,
            Role::ZoneBoundary => choice.boundary,
            Role::ZoneIsland(index) => choice.islands.binary_search(&index).is_ok(),
        }
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
        let selected = !choice.minimum && Self::selected_part(choice, span.role);
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
            high_contrast: false,
            dim_unrelated: false,
            choices: Vec::new(),
            screen: Vec::new(),
            screen_permit: None,
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
    pub glyph: bool,
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
                        let (stroke, glyph, range, spans) = match command {
                            RetainedDrawCommand::Quads { range, .. } => {
                                (false, false, range, &presentation.quads)
                            }
                            RetainedDrawCommand::Glyphs { range, .. } => {
                                (false, true, range, &presentation.glyphs)
                            }
                            RetainedDrawCommand::Strokes { range, .. } => {
                                (true, false, range, &presentation.strokes)
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
                                    glyph,
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
