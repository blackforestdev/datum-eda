//! Immutable source bounds and all-or-nothing, cancellable native region evaluation.
use datum_gui_protocol::selection_geometry_catalog::{
    QualificationGeometry as Geometry, qualification_catalog,
};
use datum_gui_protocol::{
    AuthoredSelectionIdentity as Id, BoardReviewSceneV1, PaneId, RectNm, WorkspaceFilterState,
};
use datum_gui_viewport::hit::{DEFAULT_REGION_QUERY_BUDGET, RegionQueryCursor};
use datum_gui_viewport::{
    HitRegion, HitShape, SpatialHitIndex, selection_acquisition::AcquisitionMode,
    selection_region::SelectionRegion,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
#[derive(Debug)]
struct Source {
    id: Id,
    geometry: Geometry,
    layers: BTreeSet<String>,
}
#[derive(Debug)]
pub(super) struct RegionBasis {
    scene_id: String,
    source_revision: String,
    layouts: Vec<(
        String,
        Option<datum_gui_protocol::Affine2DFixedPrimitive>,
        Option<datum_gui_protocol::MeshRectEmPrimitive>,
    )>,
    index: SpatialHitIndex<Source>,
    unresolved: Vec<(Id, BTreeSet<String>, String)>,
}
fn layouts(
    scene: &BoardReviewSceneV1,
) -> Vec<(
    String,
    Option<datum_gui_protocol::Affine2DFixedPrimitive>,
    Option<datum_gui_protocol::MeshRectEmPrimitive>,
)> {
    scene
        .board_text_geometries
        .iter()
        .map(|g| {
            (
                g.object_id.clone(),
                g.world_transform_nm,
                g.block_bbox_em_nm,
            )
        })
        .collect()
}
fn geometry_bounds(geometry: &Geometry) -> Result<Option<RectNm>, String> {
    let points: Vec<_> = match geometry {
        Geometry::Anchors(points) => points
            .iter()
            .copied()
            .map(datum_gui_viewport::selection_region::point)
            .collect(),
        Geometry::Layout(points) => points.clone(),
        Geometry::Filled(polygons) => polygons.iter().flatten().copied().collect(),
        Geometry::Disk { center, radius } => {
            if *radius < 0 {
                return Err("Invalid source radius".into());
            }
            return Ok(Some(RectNm {
                min_x: center
                    .x
                    .checked_sub(*radius)
                    .ok_or("Source bound overflow")?,
                max_x: center
                    .x
                    .checked_add(*radius)
                    .ok_or("Source bound overflow")?,
                min_y: center
                    .y
                    .checked_sub(*radius)
                    .ok_or("Source bound overflow")?,
                max_y: center
                    .y
                    .checked_add(*radius)
                    .ok_or("Source bound overflow")?,
            }));
        }
    };
    if points.is_empty() {
        return Ok(None);
    }
    let floor =
        |r: datum_gui_protocol::selection_geometry::Rational| r.numerator.div_euclid(r.denominator);
    let ceil = |r: datum_gui_protocol::selection_geometry::Rational| {
        floor(r) + i128::from(r.numerator.rem_euclid(r.denominator) != 0)
    };
    let bounded = |v: i128| {
        i64::try_from(v).map_err(|_| "Certified source bound exceeds world range".to_owned())
    };
    Ok(Some(RectNm {
        min_x: bounded(points.iter().map(|p| floor(p[0])).min().unwrap())?,
        max_x: bounded(points.iter().map(|p| ceil(p[0])).max().unwrap())?,
        min_y: bounded(points.iter().map(|p| floor(p[1])).min().unwrap())?,
        max_y: bounded(points.iter().map(|p| ceil(p[1])).max().unwrap())?,
    }))
}
impl RegionBasis {
    pub(super) fn matches_source(&self, scene: &BoardReviewSceneV1) -> bool {
        self.scene_id == scene.scene_id && self.source_revision == scene.source_revision
    }
    pub(super) fn matches(&self, scene: &BoardReviewSceneV1) -> bool {
        self.scene_id == scene.scene_id
            && self.source_revision == scene.source_revision
            && self.layouts == layouts(scene)
    }
    pub(super) fn capture(
        model: &eda_engine::substrate::DesignModel,
        authority: &datum_gui_protocol::selection_resolution::NativeSelectionResolution,
        scene: &BoardReviewSceneV1,
    ) -> Self {
        let bindings =
            datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(scene, authority);
        let mut layers: BTreeMap<Id, BTreeSet<String>> = BTreeMap::new();
        for (_, binding) in bindings.iter() {
            layers
                .entry(binding.acquired.clone())
                .or_default()
                .extend(binding.layers.iter().cloned());
        }
        let mut regions = Vec::new();
        let mut unresolved = Vec::new();
        for (id, geometry) in qualification_catalog(model, authority, scene) {
            let layers = layers.remove(&id).unwrap_or_default();
            match geometry {
                Ok(geometry) => match geometry_bounds(&geometry) {
                    Ok(Some(bounds)) => regions.push(HitRegion {
                        target: Source {
                            id,
                            geometry,
                            layers,
                        },
                        layer_id: None,
                        shape: HitShape::Rect(bounds),
                    }),
                    Ok(None) => {} // Empty authored area cannot qualify.
                    Err(reason) => unresolved.push((id, layers, reason)),
                },
                Err(reason) => unresolved.push((id, layers, reason)),
            }
        }
        Self {
            scene_id: scene.scene_id.clone(),
            source_revision: scene.source_revision.clone(),
            layouts: layouts(scene),
            index: SpatialHitIndex::new(regions),
            unresolved,
        }
    }
}
fn eligible(id: &Id, layers: &BTreeSet<String>, filters: &WorkspaceFilterState) -> bool {
    datum_gui_protocol::selection_scene::acquisition_eligible(
        id.class,
        layers.iter().map(String::as_str),
        filters,
    )
}
fn qualifies(region: &SelectionRegion, geometry: &Geometry) -> Result<bool, String> {
    let result = match geometry {
        Geometry::Anchors(a) => region.anchors(a),
        Geometry::Layout(p) => region.majority_layout(p),
        Geometry::Disk { center, radius } => region.encloses_disk(*center, *radius),
        Geometry::Filled(polygons) => {
            let mut all = !polygons.is_empty();
            for polygon in polygons {
                all &= region
                    .encloses(polygon)
                    .map_err(|e| format!("Region refused atomically: {e:?}"))?;
            }
            return Ok(all);
        }
    };
    result.map_err(|e| format!("Region refused atomically: {e:?}"))
}
pub(super) struct RegionEvaluation {
    pub pane: PaneId,
    pub mode: AcquisitionMode,
    pub revision: String,
    pub basis: Arc<RegionBasis>,
    filters: WorkspaceFilterState,
    region: SelectionRegion,
    cursor: RegionQueryCursor,
    candidates: BTreeSet<Id>,
}
impl RegionEvaluation {
    pub(super) fn begin(
        pane: PaneId,
        mode: AcquisitionMode,
        revision: String,
        basis: Arc<RegionBasis>,
        filters: WorkspaceFilterState,
        path: &[datum_gui_protocol::PointNm],
    ) -> Result<Self, String> {
        let region = SelectionRegion::new(path).map_err(|e| format!("Region refused: {e:?}"))?;
        if !region.is_empty()
            && let Some((_, _, reason)) = basis
                .unresolved
                .iter()
                .find(|(id, layers, _)| eligible(id, layers, &filters))
        {
            return Err(reason.clone());
        }
        let Some(bounds) = HitShape::Polygon(path.to_vec()).bounds() else {
            return Err("Region boundary unavailable".into());
        };
        let cursor = basis.index.begin_region(bounds);
        Ok(Self {
            pane,
            mode,
            revision,
            basis,
            filters,
            region,
            cursor,
            candidates: BTreeSet::new(),
        })
    }
    /// No selection is published until the complete exact set is available.
    pub(super) fn advance(&mut self) -> Result<Option<BTreeSet<Id>>, String> {
        self.advance_with_budget(DEFAULT_REGION_QUERY_BUDGET)
    }
    fn advance_with_budget(&mut self, budget: usize) -> Result<Option<BTreeSet<Id>>, String> {
        if self.region.is_empty() {
            return Ok(Some(BTreeSet::new()));
        }
        let mut error = None;
        let step = self
            .basis
            .index
            .advance_region(&mut self.cursor, budget, |source| {
                if error.is_some() || !eligible(&source.id, &source.layers, &self.filters) {
                    return;
                }
                match qualifies(&self.region, &source.geometry) {
                    Ok(true) => {
                        self.candidates.insert(source.id.clone());
                    }
                    Ok(false) => {}
                    Err(reason) => error = Some(reason),
                }
            });
        if let Some(reason) = error {
            return Err(reason);
        }
        Ok(step.complete.then(|| std::mem::take(&mut self.candidates)))
    }
    pub(super) fn filters(&self) -> &WorkspaceFilterState {
        &self.filters
    }
    pub(super) fn examined(&self) -> usize {
        self.cursor.examined
    }
}

#[cfg(test)]
#[path = "selection_region_evaluation/tests.rs"]
mod tests;
