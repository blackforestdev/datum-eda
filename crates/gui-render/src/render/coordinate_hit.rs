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
//   * `push_schematic_hit_regions` — typed retained regions for the schematic
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
) -> (datum_gui_protocol::PaneId, Option<datum_gui_protocol::PaneId>) {
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
pub(crate) fn push_schematic_hit_regions(
    out: &mut Vec<WorldHitRegion>,
    scene: &BoardReviewSceneV1,
) {
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
                WorldHitShape::Rect(bounding_rect_nm(&graphic.path).expect("non-empty path"))
            }
            datum_gui_protocol::SchematicHitKind::Junction if graphic.path.len() >= 3 => {
                WorldHitShape::Polygon(graphic.path.clone())
            }
            _ => WorldHitShape::Polyline {
                path: graphic.path.clone(),
                half_width_nm: (width * 0.5).max(150_000.0),
            },
        };
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(graphic.object_id.clone()),
            layer_id: None,
            shape,
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
            !retained.world_hit_index.regions().is_empty(),
            "projected schematic symbols must emit world hit regions (was always empty pre-S3)"
        );
        let targets: std::collections::BTreeSet<_> = retained
            .world_hit_index
            .regions()
            .iter()
            .filter_map(|region| match &region.target {
                HitTarget::AuthoredObject(id) => Some(id.as_str()),
                _ => None,
            })
            .collect();
        let schematic = state
            .schematic_scene
            .as_ref()
            .expect("fixture must retain its projected schematic scene");
        let eligible: Vec<_> = schematic
            .board_graphics
            .iter()
            .filter(|graphic| graphic.schematic_hit_kind().is_some())
            .collect();
        assert!(!eligible.is_empty(), "typed hit metadata must not be vacuous");
        for graphic in eligible {
            assert!(
                targets.contains(graphic.object_id.as_str()),
                "typed schematic primitive {} must have a hit region",
                graphic.object_id
            );
        }
        for expected in [
            datum_gui_protocol::SchematicHitKind::Symbol,
            datum_gui_protocol::SchematicHitKind::Pin,
            datum_gui_protocol::SchematicHitKind::Wire,
        ] {
            assert!(
                schematic
                    .board_graphics
                    .iter()
                    .any(|graphic| graphic.schematic_hit_kind() == Some(expected)),
                "simple schematic must exercise {expected:?} metadata"
            );
        }
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
        let schematic =
            RetainedScene::from_workspace_schematic_for_surface(&state, 1600, 1000, 1.0);
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
            hover.hover.as_ref().map(|target| target.object_id.as_str()),
            Some(symbol_id.as_str()),
            "a schematic-pane cursor over a symbol resolves that symbol's identity"
        );
        assert_eq!(
            hover.hover.map(|target| target.surface),
            Some(datum_gui_protocol::PaneContent::Schematic)
        );
    }

    /// S4 (b, schematic): hovering a schematic symbol emits the class-A hover
    /// pre-highlight ring into the SCHEMATIC pane's underlay buffer (not the board).
    #[test]
    fn schematic_hover_ring_lands_in_the_schematic_underlay() {
        let mut state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);

        let baseline = prepared_for(&state, &board);
        let base_rings = count_color(baseline.schematic_overlay_vertices(), HOVER_HIGHLIGHT);

        let symbol_id = {
            let scene = state.schematic_scene.as_ref().unwrap();
            first_symbol(scene).0.to_string()
        };
        state.ui.hovered_object = Some(datum_gui_protocol::HoverTarget {
            object_id: symbol_id,
            surface: datum_gui_protocol::PaneContent::Schematic,
        });
        let hovered = prepared_for(&state, &board);
        assert!(
            count_color(hovered.schematic_overlay_vertices(), HOVER_HIGHLIGHT) > base_rings,
            "a hovered schematic symbol adds a hover ring to the schematic overlay"
        );
    }

    /// S4 (b, board): hovering a board object emits the hover ring into the BOARD
    /// overlay buffer, proving per-pane routing of the same overlay.
    #[test]
    fn board_hover_ring_lands_in_the_board_overlay() {
        let mut state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
        let board_id = board
            .world_hit_index
            .regions()
            .iter()
            .find_map(|region| match &region.target {
                HitTarget::AuthoredObject(id) | HitTarget::ReviewAction(id) => Some(id.clone()),
                _ => None,
            })
            .expect("the board fixture must expose at least one hoverable object");

        let base_rings = count_color(
            prepared_for(&state, &board).board_interaction_vertices(),
            HOVER_HIGHLIGHT,
        );
        state.ui.hovered_object = Some(datum_gui_protocol::HoverTarget {
            object_id: board_id,
            surface: datum_gui_protocol::PaneContent::Board,
        });
        let hovered = prepared_for(&state, &board);
        assert!(
            count_color(hovered.board_interaction_vertices(), HOVER_HIGHLIGHT) > base_rings,
            "a hovered board object adds a hover ring to the board overlay"
        );
    }

    /// Cursor motion is a high-frequency overlay update. It must neither resolve
    /// authored geometry again nor disturb static prepared buffers.
    #[test]
    fn interaction_refresh_preserves_retained_and_static_scene_work() {
        let mut state = schematic_workspace_state();
        let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
        let mut prepared = prepared_for(&state, &board);
        let resolves_before = retained_scene_resolve_count();
        let world_ranges_before = prepared.visible_world_ranges.clone();
        let grid_before = prepared.schematic_underlay_vertices.clone();

        state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx { x: 320.0, y: 240.0 });
        prepared.refresh_interaction(&state, &board);

        assert_eq!(
            retained_scene_resolve_count(),
            resolves_before,
            "pointer chrome must not resolve retained world geometry"
        );
        assert_eq!(prepared.visible_world_ranges, world_ranges_before);
        assert_eq!(prepared.schematic_underlay_vertices, grid_before);
    }
}
