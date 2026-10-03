//! Whole-owner low-zoom cues. Bounds only cull presentation; anchors come from
//! canonical values and never change acquisition or authoritative membership.
use super::*;
use datum_gui_protocol::{
    AuthoredSelectionIdentity as Id, PaneContent, selection_view::SelectionView,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn bounds(points: &[(f32, f32)]) -> RectPx {
    let (mut lo, mut hi) = (
        (f32::INFINITY, f32::INFINITY),
        (f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for &(x, y) in points {
        lo = (lo.0.min(x), lo.1.min(y));
        hi = (hi.0.max(x), hi.1.max(y));
    }
    RectPx {
        x: lo.0,
        y: lo.1,
        width: hi.0 - lo.0,
        height: hi.1 - lo.1,
    }
}
#[derive(Default)]
struct Extent {
    bounds: Option<RectPx>,
    uncertain: bool,
}
impl Extent {
    fn add(&mut self, bounds: RectPx) {
        self.bounds = Some(match self.bounds {
            None => bounds,
            Some(a) => {
                let x = a.x.min(bounds.x);
                let y = a.y.min(bounds.y);
                RectPx {
                    x,
                    y,
                    width: (a.x + a.width).max(bounds.x + bounds.width) - x,
                    height: (a.y + a.height).max(bounds.y + bounds.height) - y,
                }
            }
        });
    }
}
pub(super) struct Plan {
    skip: BTreeSet<Id>,
    cues: Vec<(datum_gui_protocol::AuthoredSelectionClass, (f32, f32))>,
}
impl Plan {
    pub(super) fn minimum_count(&self) -> usize {
        self.cues.len()
    }
    pub(super) fn capture(
        view: &SelectionView,
        scene: &BoardReviewSceneV1,
        surface: PaneContent,
        state: &ReviewWorkspaceState,
        projection: &Projection,
        viewport: RectPx,
        glyphs: Option<&impl glyph_mesh_lookup::GlyphMeshLookup>,
    ) -> Self {
        let mut extents: BTreeMap<Id, Extent> = BTreeMap::new();
        let mut add = |key: &str, points: &[PointNm], width: i64, layer: &str| {
            if !layer_visible(state, layer) {
                return;
            }
            if let Some(owner) = view.presentation_owner(scene, surface, key) {
                let extent = extents.entry(owner.clone()).or_default();
                if points.is_empty() {
                    return;
                }
                let points: Vec<_> = points
                    .iter()
                    .map(|p| projection.project_point(*p))
                    .collect();
                let mut rect = bounds(&points);
                let radius = width.max(0) as f32 * projection.scale * 0.5;
                rect.x -= radius;
                rect.y -= radius;
                rect.width += radius * 2.0;
                rect.height += radius * 2.0;
                extent.add(rect);
            }
        };
        for v in &scene.board_graphics {
            add(&v.object_id, &v.path, v.width_nm.unwrap_or(0), &v.layer_id);
        }
        for v in &scene.tracks {
            add(&v.object_id, &v.path, v.width_nm, &v.layer_id);
        }
        for v in &scene.vias {
            if layer_visible(state, &v.start_layer_id) || layer_visible(state, &v.end_layer_id) {
                let layer = if layer_visible(state, &v.start_layer_id) {
                    &v.start_layer_id
                } else {
                    &v.end_layer_id
                };
                add(&v.object_id, &[v.position], v.diameter_nm, layer);
            }
        }
        for v in &scene.outline {
            add(&v.object_id, &v.path, 0, &v.layer_id);
        }
        for v in &scene.zones {
            for polygon in view.zone_regions.get(&v.object_id).into_iter().flatten() {
                add(&v.object_id, polygon, 0, &v.layer_id);
            }
        }
        for v in &scene.pads {
            if let Some(layer) = pad_copper_layer_ids(v)
                .into_iter()
                .find(|l| layer_visible(state, l))
            {
                let outline = world_primitives::world_pad_outline(v, v.bounds, 0.0, projection);
                add(&v.object_id, &outline, 0, layer);
            }
        }
        for v in &scene.components {
            add(
                &v.object_id,
                &[
                    PointNm {
                        x: v.bounds.min_x,
                        y: v.bounds.min_y,
                    },
                    PointNm {
                        x: v.bounds.max_x,
                        y: v.bounds.max_y,
                    },
                ],
                0,
                &v.placement_layer,
            );
        }
        // Add all owned presentation, not just a symbol/footprint's body record.
        let components: BTreeMap<_, _> = scene
            .components
            .iter()
            .filter_map(|c| {
                view.presentation_owner(scene, surface, &c.object_id)
                    .map(|id| (c.component_uuid.as_str(), id.clone()))
            })
            .collect();
        for v in &scene.component_graphics {
            if let Some(owner) = components.get(v.component_uuid.as_str())
                && v.layer_id.as_ref().is_none_or(|l| layer_visible(state, l))
            {
                let points: Vec<_> = v
                    .path
                    .iter()
                    .map(|p| projection.project_point(*p))
                    .collect();
                if !points.is_empty() {
                    let mut bounds = bounds(&points);
                    let radius = v.width_nm.unwrap_or(0).max(0) as f32 * projection.scale * 0.5;
                    bounds.x -= radius;
                    bounds.y -= radius;
                    bounds.width += radius * 2.0;
                    bounds.height += radius * 2.0;
                    extents.entry(owner.clone()).or_default().add(bounds);
                }
            }
        }
        for v in &scene.component_texts {
            if let Some(owner) = components.get(v.component_uuid.as_str())
                && v.layer_id.as_ref().is_none_or(|l| layer_visible(state, l))
            {
                let extent = extents.entry(owner.clone()).or_default();
                if v.cached_polygons.is_empty() {
                    // The existing shaped text path has no glyph silhouette in
                    // this scene. Never substitute its layout rectangle.
                    extent.uncertain = true;
                } else {
                    for polygon in &v.cached_polygons {
                        let points: Vec<_> = polygon
                            .iter()
                            .map(|p| projection.project_point(*p))
                            .collect();
                        if !points.is_empty() {
                            extent.add(bounds(&points));
                        }
                    }
                }
            }
        }
        for v in &scene.board_text_geometries {
            if !layer_visible(state, &v.layer_id) {
                continue;
            }
            let Some(owner) = view.presentation_owner(scene, surface, &v.object_id) else {
                continue;
            };
            let extent = extents.entry(owner.clone()).or_default();
            let Some(glyphs) = glyphs else {
                extent.uncertain = true;
                continue;
            };
            let mut quads = Vec::new();
            board_text_geometry::push_board_text_geometry_world(
                &mut quads,
                v,
                glyphs,
                TEXT_PRIMARY,
                projection,
            );
            let points: Vec<_> = quads
                .iter()
                .flat_map(|q| q.points)
                .map(|p| {
                    projection.project_point(PointNm {
                        x: p.0.round() as i64,
                        y: p.1.round() as i64,
                    })
                })
                .collect();
            if !points.is_empty() {
                extent.add(bounds(&points));
            }
        }
        let text_geometries: BTreeSet<_> = scene
            .board_text_geometries
            .iter()
            .map(|g| g.object_id.as_str())
            .collect();
        for v in &scene.board_texts {
            if layer_visible(state, &v.layer_id)
                && !text_geometries.contains(v.object_id.as_str())
                && let Some(owner) = view.presentation_owner(scene, surface, &v.object_id)
            {
                extents.entry(owner.clone()).or_default().uncertain = true;
            }
        }
        let mut result = Self {
            skip: BTreeSet::new(),
            cues: Vec::new(),
        };
        for (owner, extent) in extents {
            let Some(bounds) = extent.bounds.filter(|_| !extent.uncertain) else {
                continue;
            };
            if bounds.intersect(viewport).is_none() {
                result.skip.insert(owner);
                continue;
            }
            if bounds.width.max(bounds.height) < 2.0
                && let Some(anchor) = view.authored_anchor(&owner)
            {
                result
                    .cues
                    .push((owner.class, projection.project_point(anchor)));
                result.skip.insert(owner);
            }
        }
        result
    }
    pub(super) fn detailed(
        &self,
        view: &SelectionView,
        scene: &BoardReviewSceneV1,
        surface: PaneContent,
        key: &str,
    ) -> bool {
        view.presentation_owner(scene, surface, key)
            .is_none_or(|owner| !self.skip.contains(owner))
    }
    pub(super) fn push_cues(&self, out: &mut Vec<selection_stroke::Stroke>, viewport: RectPx) {
        for &(class, (x, y)) in &self.cues {
            if class == datum_gui_protocol::AuthoredSelectionClass::NoConnect {
                for (from, to) in [
                    ([x - 3.0, y - 3.0], [x + 3.0, y + 3.0]),
                    ([x - 3.0, y + 3.0], [x + 3.0, y - 3.0]),
                ] {
                    out.push(selection_stroke::Stroke::minimum(from, to, viewport, 2.0));
                }
            } else {
                // One analytic annulus at the canonical anchor, with an open
                // center. Its instance does not enlarge authored geometry.
                out.push(selection_stroke::Stroke::minimum(
                    [x, y],
                    [x, y],
                    viewport,
                    1.0,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimum_no_connect_is_an_x_and_never_a_junction_ring() {
        let plan = Plan {
            skip: BTreeSet::new(),
            cues: vec![(
                datum_gui_protocol::AuthoredSelectionClass::NoConnect,
                (10.0, 20.0),
            )],
        };
        let mut q = Vec::new();
        plan.push_cues(
            &mut q,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
        );
        assert_eq!(q.len(), 2);
        assert!(q.iter().all(|stroke| stroke.high_contrast[1] == 2.0));
        assert_ne!(q[0].from, q[1].from);
    }
    #[test]
    fn dense_minimum_cues_preserve_every_anchor_in_one_bounded_instance_each() {
        let plan = Plan {
            skip: BTreeSet::new(),
            cues: (0..100_000)
                .map(|i| {
                    (
                        datum_gui_protocol::AuthoredSelectionClass::Junction,
                        ((i % 1000) as f32, (i / 1000) as f32),
                    )
                })
                .collect(),
        };
        let mut instances = Vec::new();
        plan.push_cues(
            &mut instances,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 1000.0,
                height: 100.0,
            },
        );
        assert_eq!(instances.len(), 100_000);
        assert_eq!(std::mem::size_of_val(instances.as_slice()), 6_400_000);
        for (i, instance) in instances.iter().enumerate() {
            assert_eq!(instance.from, [(i % 1000) as f32, (i / 1000) as f32]);
            assert_eq!(instance.from, instance.to);
            assert_eq!(instance.high_contrast, [1.0, 1.0, 0.0, 0.0]);
        }
    }
}
