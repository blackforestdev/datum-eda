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

/// The hover resolved for the pane a screen point lands in (S4 HoverEngine): the
/// hovered object's identity, the surface it lives on, and the live cursor while it
/// is over any scene surface. All fields are `None` off every scene pane (over
/// chrome), so the caller clears hover + crosshair state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaneHover {
    pub object_id: Option<String>,
    pub surface: Option<datum_gui_protocol::PaneContent>,
    pub cursor: Option<(f32, f32)>,
}

/// Resolve hover for the FOCUSED/containing pane at screen point `(x, y)`, in that
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
) -> PaneHover {
    let hit_id = |target: Option<&HitTarget>| match target {
        Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => Some(id.clone()),
        _ => None,
    };
    match prepared.world_point_at_screen(x, y) {
        Some((world_point, SceneSurface::Board)) => {
            let object_id = hit_id(board_retained.hit_test_authored_world(world_point, state));
            PaneHover {
                surface: object_id
                    .is_some()
                    .then_some(datum_gui_protocol::PaneContent::Board),
                object_id,
                cursor: Some((x, y)),
            }
        }
        Some((world_point, SceneSurface::Schematic)) => {
            let object_id =
                hit_id(schematic_retained.and_then(|retained| retained.hit_test_world(world_point)));
            PaneHover {
                surface: object_id
                    .is_some()
                    .then_some(datum_gui_protocol::PaneContent::Schematic),
                object_id,
                cursor: Some((x, y)),
            }
        }
        None => PaneHover::default(),
    }
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

    fn prepared_for(state: &ReviewWorkspaceState, board: &RetainedScene) -> PreparedScene {
        PreparedScene::from_workspace_for_surface(
            state,
            1600,
            1000,
            1.0,
            CameraState::fit_to_bounds(&state.scene.bounds),
            board,
        )
    }

    fn count_color(vertices: &[Vertex], color: [f32; 3]) -> usize {
        vertices.iter().filter(|v| v.color == color).count()
    }

    /// S4 (a): a SCHEMATIC-pane cursor over a symbol now resolves that symbol's
    /// identity and the Schematic surface — impossible pre-S3, when hover was a
    /// single board-only global. This is the per-surface hover the slice unblocks.
    #[test]
    fn schematic_cursor_over_symbol_resolves_symbol_identity() {
        let state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
        let schematic = RetainedScene::from_workspace_schematic_for_surface(&state, 1600, 1000, 1.0);
        let prepared = prepared_for(&state, &board);

        let (symbol_id, symbol_center) = {
            let scene = state.schematic_scene.as_ref().unwrap();
            let (id, center) = first_symbol(scene);
            (id.to_string(), center)
        };
        // Project the symbol's world centre to a schematic-pane SCREEN point through
        // the same fit projection `PreparedScene` seeds, then resolve hover there.
        let schematic_viewport = prepared.schematic_scene_viewport.unwrap();
        let field = inset_rect(schematic_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(
            field,
            &state.schematic_scene.as_ref().unwrap().bounds,
            CameraState::fit_to_bounds(&state.schematic_scene.as_ref().unwrap().bounds),
        );
        let (sx, sy) = projection.project_point(symbol_center);

        let hover = resolve_pane_hover(&prepared, &board, schematic.as_ref(), &state, sx, sy);
        assert_eq!(
            hover.object_id.as_deref(),
            Some(symbol_id.as_str()),
            "a schematic-pane cursor over a symbol resolves that symbol's identity"
        );
        assert_eq!(hover.surface, Some(datum_gui_protocol::PaneContent::Schematic));
    }

    /// S4 (b, schematic): hovering a schematic symbol emits the class-A hover
    /// pre-highlight ring into the SCHEMATIC pane's underlay buffer (not the board).
    #[test]
    fn schematic_hover_ring_lands_in_the_schematic_underlay() {
        let mut state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);

        let baseline = prepared_for(&state, &board);
        let base_rings = count_color(baseline.schematic_underlay_vertices(), HOVER_HIGHLIGHT);

        let symbol_id = {
            let scene = state.schematic_scene.as_ref().unwrap();
            first_symbol(scene).0.to_string()
        };
        state.ui.hovered_object_id = Some(symbol_id);
        let hovered = prepared_for(&state, &board);
        assert!(
            count_color(hovered.schematic_underlay_vertices(), HOVER_HIGHLIGHT) > base_rings,
            "a hovered schematic symbol adds a hover ring to the schematic underlay"
        );
    }

    /// S4 (b, board): hovering a board object emits the hover ring into the BOARD
    /// overlay buffer, proving per-pane routing of the same overlay.
    #[test]
    fn board_hover_ring_lands_in_the_board_overlay() {
        let mut state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
        let board_id = board
            .world_hit_regions
            .iter()
            .find_map(|region| match &region.target {
                HitTarget::AuthoredObject(id) | HitTarget::ReviewAction(id) => Some(id.clone()),
                _ => None,
            })
            .expect("the board fixture must expose at least one hoverable object");

        let base_rings = count_color(prepared_for(&state, &board).viewport_overlay_vertices(), HOVER_HIGHLIGHT);
        state.ui.hovered_object_id = Some(board_id);
        let hovered = prepared_for(&state, &board);
        assert!(
            count_color(hovered.viewport_overlay_vertices(), HOVER_HIGHLIGHT) > base_rings,
            "a hovered board object adds a hover ring to the board overlay"
        );
    }
}
