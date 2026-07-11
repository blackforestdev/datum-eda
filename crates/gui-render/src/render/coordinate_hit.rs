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
//   * `push_schematic_symbol_hit_regions` — the schematic pane's first-ever hit
//     regions, one per placed symbol, tagged with the symbol's stable projected
//     identity (mirroring how the board tags authored objects).
//
// It is a real `#[path] mod` child of the crate root (declared in `scene.rs`), so
// as a DESCENDANT of the module that defines them it can still reach the private
// `PreparedScene`/`RetainedScene` fields and the private `WorldHitShape`/
// `WorldHitRegion` types exactly as the code did when it lived in `scene.rs`. Split
// out to keep `scene.rs` under its ceiling with real (non-`include!`) extraction.

use super::*;

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
        if self.scene_viewport.contains(x, y) {
            let board_field = inset_rect(self.scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection = Projection::new(board_field, &self.scene_bounds, self.camera);
            return Some((projection.screen_to_world(x, y), SceneSurface::Board));
        }
        if let Some(schematic_viewport) = self.schematic_scene_viewport
            && schematic_viewport.contains(x, y)
        {
            let field = inset_rect(schematic_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection = Projection::new(field, &self.schematic_bounds, self.schematic_camera);
            return Some((projection.screen_to_world(x, y), SceneSurface::Schematic));
        }
        None
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
        self.world_hit_regions
            .iter()
            .rev()
            .find(|region| layer_pass(region) && region.shape.contains(point))
            .map(|region| &region.target)
    }
}

impl WorldHitShape {
    fn contains(&self, point: PointNm) -> bool {
        match self {
            Self::Rect(rect) => point_in_rect(point, *rect),
            Self::Polyline {
                path,
                half_width_nm,
            } => polyline_contains_world_point(path, point, *half_width_nm),
            Self::Polygon(path) => point_in_polygon_world(path, point),
            Self::Circle { center, radius_nm } => {
                let dx = point.x as f32 - center.x as f32;
                let dy = point.y as f32 - center.y as f32;
                dx * dx + dy * dy <= radius_nm * radius_nm
            }
        }
    }
}

/// Emit one selectable world hit region per placed schematic SYMBOL from the
/// projected schematic scene (S3 / UVT-004 — the schematic pane's first hit
/// regions). Symbol bodies project as `board_graphics` whose `object_id` is
/// `schematic-symbol:<uuid>` — the stable projected identity, the schematic
/// mirror of a board authored object's `object_id`. The pin lines/terminals carry
/// the distinct `schematic-symbol-pin*` ids and are deliberately NOT hit targets
/// this slice (symbols are the required S5/S7 target; wires/labels may follow).
/// The body outline is an axis-aligned rectangle, so its path bounding box is an
/// exact hit rect — the same `WorldHitShape::Rect` the board uses for a component
/// body, so no parallel hit model is introduced.
pub(crate) fn push_schematic_symbol_hit_regions(
    out: &mut Vec<WorldHitRegion>,
    scene: &BoardReviewSceneV1,
) {
    for graphic in &scene.board_graphics {
        if !graphic.object_id.starts_with("schematic-symbol:") {
            continue;
        }
        let Some(bounds) = bounding_rect_nm(&graphic.path) else {
            continue;
        };
        out.push(WorldHitRegion {
            // Tag with the symbol's projected identity, exactly as the board tags
            // its authored objects — so S5 selection routes the same way.
            target: HitTarget::AuthoredObject(graphic.object_id.clone()),
            // Schematic layers are always visible; `hit_test_world` does not filter,
            // and `None` keeps the region live under any (board) layer predicate too.
            layer_id: None,
            shape: WorldHitShape::Rect(bounds),
        });
    }
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

#[cfg(test)]
mod coordinate_hit_tests {
    // A descendant of the crate root (the `include!` module), so it reaches the
    // private `world_hit_regions` field / `WorldHitRegion.target` exactly like the
    // sibling board hit-test tests do — no public accessor is invented for a test.
    use super::*;

    /// A Board|Schematic workspace with the real simple-demo schematic projected
    /// into `schematic_scene`. `load_fixture_workspace_state` already defaults to
    /// the Board|Schematic split (Board focused), so the schematic pane exists.
    fn schematic_workspace_state() -> ReviewWorkspaceState {
        let schematic = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let projected = datum_gui_protocol::load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic fixture should load");
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.schematic_scene = Some(projected.scene);
        state
    }

    /// The world bbox center of the first projected symbol body — a point that
    /// must fall inside that symbol's hit region.
    fn first_symbol(scene: &BoardReviewSceneV1) -> (&str, PointNm) {
        let symbol = scene
            .board_graphics
            .iter()
            .find(|graphic| graphic.object_id.starts_with("schematic-symbol:"))
            .expect("fixture should project at least one symbol body");
        let bounds = bounding_rect_nm(&symbol.path).expect("symbol body has geometry");
        (
            symbol.object_id.as_str(),
            PointNm {
                x: (bounds.min_x + bounds.max_x) / 2,
                y: (bounds.min_y + bounds.max_y) / 2,
            },
        )
    }

    /// (a) The schematic pane emits hit regions for the first time — one per placed
    /// symbol, each tagged with the symbol's stable projected identity.
    #[test]
    fn schematic_scene_emits_symbol_hit_regions() {
        let state = schematic_workspace_state();
        let retained = RetainedScene::from_workspace_schematic_for_surface(&state, 1600, 1000, 1.0)
            .expect("a Schematic pane + projected scene must yield a retained scene");
        assert!(
            !retained.world_hit_regions.is_empty(),
            "projected schematic symbols must emit world hit regions (was always empty pre-S3)"
        );
        assert!(
            retained.world_hit_regions.iter().all(|region| matches!(
                &region.target,
                HitTarget::AuthoredObject(id) if id.starts_with("schematic-symbol:")
            )),
            "every schematic hit region targets a symbol identity"
        );
    }

    /// (b) A screen point inside the SCHEMATIC pane resolves to a world point via
    /// the schematic camera and reports the Schematic surface; a point in the board
    /// pane still reports Board (the board resolve is unchanged).
    #[test]
    fn screen_point_resolves_to_the_containing_panes_surface() {
        let state = schematic_workspace_state();
        let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
        let prepared = PreparedScene::from_workspace_for_surface(
            &state,
            1600,
            1000,
            1.0,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        let schematic_viewport = prepared
            .schematic_scene_viewport
            .expect("Board|Schematic layout must expose a schematic scene viewport");
        let sx = schematic_viewport.x + schematic_viewport.width * 0.5;
        let sy = schematic_viewport.y + schematic_viewport.height * 0.5;
        let (_, schematic_surface) = prepared
            .world_point_at_screen(sx, sy)
            .expect("a point inside the schematic pane must resolve to a world point");
        assert_eq!(
            schematic_surface,
            SceneSurface::Schematic,
            "a schematic-pane point must resolve on the Schematic surface"
        );

        let board_viewport = prepared.scene_viewport;
        let bx = board_viewport.x + board_viewport.width * 0.5;
        let by = board_viewport.y + board_viewport.height * 0.5;
        let (_, board_surface) = prepared
            .world_point_at_screen(bx, by)
            .expect("a point inside the board pane must still resolve");
        assert_eq!(
            board_surface,
            SceneSurface::Board,
            "a board-pane point must still resolve on the Board surface (unchanged)"
        );
    }

    /// (c) Hit-testing a schematic symbol's own world location returns that
    /// symbol's identity — the selection target S5 will act on.
    #[test]
    fn hit_test_at_symbol_location_returns_its_identity() {
        let state = schematic_workspace_state();
        let (symbol_id, symbol_center) = {
            let scene = state.schematic_scene.as_ref().unwrap();
            let (id, center) = first_symbol(scene);
            (id.to_string(), center)
        };
        let retained = RetainedScene::from_workspace_schematic_for_surface(&state, 1600, 1000, 1.0)
            .expect("schematic retained scene");
        let hit = retained
            .hit_test_world(symbol_center)
            .expect("the symbol's world center must land inside its hit region");
        assert_eq!(hit, &HitTarget::AuthoredObject(symbol_id));
    }
}
