// Per-pane coordinate + hit resolution (UVT-004, the CoordinateHit keystone).
//
// This is the include-module that generalizes the two board-only chokepoints so
// they resolve for whichever pane a screen point lands in, in THAT pane's own
// camera/space:
//
//   * `PreparedScene::world_point_at_screen` — screen -> world for the containing
//     pane (board or schematic), reporting the `SceneSurface` so the caller routes
//     the follow-up world hit-test to the matching retained scene.
//   * `RetainedScene::hit_test_authored_world` (board, filtered) and
//     `hit_test_world` (schematic, unfiltered) — one scan core, so the board path
//     stays byte-identical while the schematic surface gets a filter-free twin.
//   * `schematic_hit_regions` — typed retained regions for the schematic
//     primitives that participate in editor interaction.
//
// It is a real `#[path] mod` child of the crate root (declared in `scene.rs`), so
// as a DESCENDANT of the module that defines them it can still reach the private
// `PreparedScene`/`RetainedScene` fields and the private `WorldHitShape`/
// `WorldHitRegion` types exactly as the code did when it lived in `scene.rs`. Split
// out to keep `scene.rs` under its ceiling with real (non-`include!`) extraction.

use super::*;

pub(crate) fn surface_pane_ids(
    shell: &ShellLayout,
    layout: &datum_gui_protocol::WorkspaceLayout,
) -> (
    datum_gui_protocol::PaneId,
    Option<datum_gui_protocol::PaneId>,
) {
    let panes = shell.viewport_panes(layout);
    let pane_for = |content| {
        panes
            .panes
            .iter()
            .find(|pane| pane.content == content)
            .map(|pane| pane.id)
    };
    (
        pane_for(datum_gui_protocol::PaneContent::Board).unwrap_or(layout.focused),
        pane_for(datum_gui_protocol::PaneContent::Schematic),
    )
}

pub(crate) fn build_surface_passes(
    shell: &ShellLayout,
    state: &ReviewWorkspaceState,
    board_camera: CameraState,
    schematic_camera: CameraState,
) -> Vec<PreparedSurfacePass> {
    shell
        .viewport_panes(&state.ui.layout)
        .panes
        .iter()
        .filter_map(|pane| {
            let (surface, bounds, camera) = match pane.content {
                datum_gui_protocol::PaneContent::Board => (
                    SceneSurface::Board,
                    state.scene.bounds.clone(),
                    board_camera,
                ),
                datum_gui_protocol::PaneContent::Schematic => (
                    SceneSurface::Schematic,
                    state.schematic_scene.as_ref()?.bounds.clone(),
                    schematic_camera,
                ),
                datum_gui_protocol::PaneContent::Revision(_) => return None,
            };
            Some(PreparedSurfacePass {
                pane_id: pane.id,
                surface,
                scene_viewport: pane.rect.scene,
                bounds,
                camera,
                grid_lod_previous: datum_gui_viewport::GridLodState::default(),
                grid_lod_resolved: datum_gui_viewport::GridLodState::default(),
            })
        })
        .collect()
}

impl PreparedScene {
    /// Resolve a screen point to a world point in the pane that contains it, and
    /// report which surface that is (UVT-004). The board branch is byte-identical
    /// to the pre-S3 board-only resolve — same field inset, same `Projection`,
    /// same camera; the schematic branch is new and only fires when a Schematic
    /// pane exists and contains the point, projecting with the schematic pane's
    /// OWN camera into its own inset field. Board and schematic scene rects are
    /// disjoint tiled panes, so the board-first order only decides points that
    /// land in neither (both miss). A future pane is one more arm here.
    pub fn world_point_at_screen(&self, x: f32, y: f32) -> Option<(PointNm, SceneSurface)> {
        self.surface_passes.iter().find_map(|pass| {
            let field = inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection = Projection::new(field, &pass.bounds, pass.camera);
            editor_viewport(pass.pane_id, pass.surface, &projection)
                .screen_to_world(datum_gui_protocol::ScreenPointPx { x, y })
                .map(|point| (point, pass.surface))
        })
    }

    pub fn surface_passes(&self) -> &[PreparedSurfacePass] {
        &self.surface_passes
    }

    pub fn set_surface_camera(&mut self, pane_id: datum_gui_protocol::PaneId, camera: CameraState) {
        if let Some(pass) = self
            .surface_passes
            .iter_mut()
            .find(|pass| pass.pane_id == pane_id)
        {
            if pass.camera != camera {
                self.composition_revision.invalidate();
            }
            pass.camera = camera;
        }
    }

    pub fn set_surface_grid_lod(
        &mut self,
        pane_id: datum_gui_protocol::PaneId,
        previous: datum_gui_viewport::GridLodState,
        resolved: datum_gui_viewport::GridLodState,
    ) {
        if let Some(pass) = self
            .surface_passes
            .iter_mut()
            .find(|pass| pass.pane_id == pane_id)
        {
            if pass.grid_lod_resolved != resolved {
                self.composition_revision.invalidate();
            }
            pass.grid_lod_previous = previous;
            pass.grid_lod_resolved = resolved;
        }
    }
}

fn editor_viewport(
    pane_id: datum_gui_protocol::PaneId,
    surface: SceneSurface,
    projection: &Projection,
) -> datum_gui_viewport::EditorViewport {
    datum_gui_viewport::EditorViewport {
        pane_id,
        surface: match surface {
            SceneSurface::Board => datum_gui_protocol::PaneContent::Board,
            SceneSurface::Schematic => datum_gui_protocol::PaneContent::Schematic,
        },
        screen: datum_gui_viewport::ScreenRectPx {
            x: projection.viewport.x,
            y: projection.viewport.y,
            width: projection.viewport.width,
            height: projection.viewport.height,
        },
        world: datum_gui_protocol::RectNm {
            min_x: projection.bounds.min_x,
            min_y: projection.bounds.min_y,
            max_x: projection.bounds.max_x,
            max_y: projection.bounds.max_y,
        },
        scale_px_per_nm: projection.scale,
        offset_x_px: projection.offset_x,
        offset_y_px: projection.offset_y,
    }
}

impl RetainedScene {
    /// Board world hit-test — gated by the board visibility filters (the authored
    /// toggle and per-layer visibility). Unchanged from the pre-S3 board path.
    /// Board hit-testing is scoped by scene-rect containment upstream
    /// (`world_point_at_screen` reports the board surface only inside the board
    /// leaf's rect), so a click in the Schematic pane never reaches board geometry
    /// — and a click in the board pane DOES hit it even while another pane is
    /// focused (view/inspect the board while working elsewhere).
    pub fn hit_test_authored_world(
        &self,
        point: PointNm,
        state: &ReviewWorkspaceState,
    ) -> Option<&HitTarget> {
        if !authored_visible(state) {
            return None;
        }
        self.hit_test_world_with(point, |region| {
            region
                .layer_id
                .as_deref()
                .is_none_or(|layer_id| layer_visible(state, layer_id))
        })
    }

    /// Schematic (non-board) world hit-test — no board filters. The schematic's
    /// layers are always visible (mirrors `all_world_ranges`, which renders every
    /// batch), so every emitted hit region is live. Same scan core as the board
    /// path; the S3 schematic pane hit-tests through here.
    pub fn hit_test_world(&self, point: PointNm) -> Option<&HitTarget> {
        self.hit_test_world_with(point, |_| true)
    }

    /// Shared topmost-first world-region scan; the surface-specific layer gate is
    /// the only difference between the board and schematic hit-tests.
    fn hit_test_world_with(
        &self,
        point: PointNm,
        layer_pass: impl Fn(&WorldHitRegion) -> bool,
    ) -> Option<&HitTarget> {
        self.world_hit_index.hit_test(point, layer_pass).target
    }
}

/// Emit retained shapes from explicit schematic interaction metadata. Selection
/// eligibility remains a tool concern; hit construction covers ordinary symbols,
/// pins, wires, buses, labels, junctions, and no-connect markers.
pub(crate) fn schematic_hit_regions(
    scene: &BoardReviewSceneV1,
    admit: impl FnOnce(usize) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<WorldHitRegion>> {
    use super::hit_construction::{Region, Shape};
    hit_construction::build(
        |emit| {
            for graphic in &scene.board_graphics {
                let Some(kind) = graphic.schematic_hit_kind() else {
                    continue;
                };
                if graphic.path.is_empty() {
                    continue;
                }
                let width = graphic.width_nm.unwrap_or(100_000) as f32;
                let shape = match kind {
                    datum_gui_protocol::SchematicHitKind::Symbol
                    | datum_gui_protocol::SchematicHitKind::Label => {
                        Shape::Rect(bounding_rect_nm(&graphic.path).expect("non-empty path"))
                    }
                    datum_gui_protocol::SchematicHitKind::Junction if graphic.path.len() >= 3 => {
                        Shape::Polygon(&graphic.path)
                    }
                    _ => Shape::Polyline {
                        path: &graphic.path,
                        half_width_nm: (width * 0.5).max(150_000.0),
                    },
                };
                emit(Region {
                    target: &graphic.object_id,
                    layer_id: None,
                    shape,
                })?;
            }
            Ok(())
        },
        admit,
    )
}

/// The axis-aligned world bounding box of a point path, or `None` when empty.
fn bounding_rect_nm(path: &[PointNm]) -> Option<datum_gui_protocol::RectNm> {
    let first = path.first()?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in &path[1..] {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Some(datum_gui_protocol::RectNm {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

/// Resolve hover for the pointer-containing pane at screen point `(x, y)`, in that
/// pane's OWN camera/space (UVT-004): screen→world picks the surface, then the
/// matching per-pane world hit-test names the object. This is the per-surface hover
/// S4 unblocks — a schematic-pane cursor over a symbol now resolves that symbol's
/// identity (impossible pre-S3). Board hover is unchanged: same board hit-test,
/// same layer filtering via `state`. Factored as a free function (no `Runtime`) so
/// the per-pane resolution is unit-tested against the real fixture scenes.
pub fn resolve_pane_hover(
    prepared: &PreparedScene,
    board_retained: &RetainedScene,
    schematic_retained: Option<&RetainedScene>,
    state: &ReviewWorkspaceState,
    x: f32,
    y: f32,
) -> datum_gui_protocol::ViewportInteraction {
    let hit_id = |target: Option<&HitTarget>| match target {
        Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => Some(id.clone()),
        _ => None,
    };
    match prepared.world_point_at_screen(x, y) {
        Some((world_point, SceneSurface::Board)) => {
            let object_id = hit_id(board_retained.hit_test_authored_world(world_point, state));
            datum_gui_viewport::InteractionEngine::resolve(
                datum_gui_protocol::PaneContent::Board,
                datum_gui_protocol::ScreenPointPx { x, y },
                object_id,
                datum_gui_viewport::HoverConfig::default(),
                datum_gui_viewport::CursorConfig::default(),
            )
        }
        Some((world_point, SceneSurface::Schematic)) => {
            let object_id = hit_id(
                schematic_retained.and_then(|retained| retained.hit_test_world(world_point)),
            );
            datum_gui_viewport::InteractionEngine::resolve(
                datum_gui_protocol::PaneContent::Schematic,
                datum_gui_protocol::ScreenPointPx { x, y },
                object_id,
                datum_gui_viewport::HoverConfig::default(),
                datum_gui_viewport::CursorConfig::default(),
            )
        }
        None => datum_gui_viewport::InteractionEngine::clear(),
    }
}

#[cfg(test)]
#[path = "coordinate_hit_duplicate_tests.rs"]
mod coordinate_hit_duplicate_tests;

#[cfg(test)]
#[path = "coordinate_hit_tests.rs"]
mod coordinate_hit_tests;
