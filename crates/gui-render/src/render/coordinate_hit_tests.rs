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
fn schematic_hit_storage_is_admitted_before_cloning_and_matches_allocator() {
    let state = schematic_workspace_state();
    let scene = state.schematic_scene.as_ref().unwrap();
    let scope = crate::cpu_alloc::Scope::new("schematic-hit-construction-proof");
    let mut required = 0;
    let regions = scope
        .with(|| {
            schematic_hit_regions(scene, |bytes| {
                required = bytes;
                assert_eq!(scope.usage().allocations, 0, "admission precedes clones");
                Ok(())
            })
        })
        .unwrap();
    assert!(!regions.is_empty());
    assert_eq!(regions.capacity(), regions.len());
    let usage = scope.usage();
    assert_eq!(required as u64, usage.payload_bytes + usage.tracking_bytes);
    let eligible = scene
        .board_graphics
        .iter()
        .filter(|graphic| graphic.schematic_hit_kind().is_some() && !graphic.path.is_empty());
    for (region, graphic) in regions.iter().zip(eligible) {
        assert_eq!(
            region.target,
            HitTarget::AuthoredObject(graphic.object_id.clone())
        );
        match &region.shape {
            WorldHitShape::Polyline { path, .. } | WorldHitShape::Polygon(path) => {
                assert_eq!(path, &graphic.path);
            }
            WorldHitShape::Rect(rect) => {
                assert_eq!(Some(*rect), bounding_rect_nm(&graphic.path))
            }
            _ => panic!("unexpected schematic hit shape"),
        }
    }
    drop(regions);
    assert_eq!(scope.usage().allocations, 0);
    let refusal = schematic_hit_regions(scene, |bytes| {
        assert_eq!(bytes, required);
        assert_eq!(scope.usage().allocations, 0);
        anyhow::bail!("test admission refusal")
    });
    assert!(refusal.is_err());
    assert_eq!(scope.usage().allocations, 0);
}

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
    assert!(
        !eligible.is_empty(),
        "typed hit metadata must not be vacuous"
    );
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
    )
    .unwrap();

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
    .unwrap()
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
    let draw_commands_before = prepared.visible_draw_commands.clone();
    let grid_before = prepared.schematic_underlay_vertices.clone();

    state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx { x: 320.0, y: 240.0 });
    prepared.refresh_interaction(&state, &board);

    assert_eq!(
        retained_scene_resolve_count(),
        resolves_before,
        "pointer chrome must not resolve retained world geometry"
    );
    assert_eq!(prepared.visible_draw_commands, draw_commands_before);
    assert_eq!(prepared.schematic_underlay_vertices, grid_before);
}
#[test]
fn interaction_damage_revision_tracks_static_camera_changes_but_not_pointer_refresh() {
    let mut state = schematic_workspace_state();
    let board = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
    let mut prepared = prepared_for(&state, &board);
    let original = prepared.composition_revision.identity();
    let clone = prepared.clone();
    assert_eq!(clone.composition_revision.identity(), original);
    state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx { x: 320.0, y: 240.0 });
    prepared.refresh_interaction(&state, &board);
    assert_eq!(prepared.composition_revision.identity(), original);

    let pass = prepared
        .surface_passes()
        .iter()
        .find(|pass| pass.surface == SceneSurface::Schematic)
        .unwrap()
        .clone();
    let scalar_camera = prepared.schematic_camera;
    let mut moved = pass.camera;
    moved.center_x_nm += 500_000.0;
    assert_ne!(moved, pass.camera);
    prepared.set_surface_camera(pass.pane_id, moved);
    let moved_revision = prepared.composition_revision.identity();
    assert_ne!(moved_revision, original);
    prepared.set_surface_camera(pass.pane_id, moved);
    assert_eq!(prepared.composition_revision.identity(), moved_revision);

    // The scalar camera never changed: restoring it still changes the pass.
    assert_eq!(prepared.schematic_camera, scalar_camera);
    prepared.set_schematic_camera(scalar_camera);
    assert_ne!(prepared.composition_revision.identity(), moved_revision);
    assert_eq!(
        prepared
            .surface_passes()
            .iter()
            .find(|candidate| candidate.pane_id == pass.pane_id)
            .unwrap()
            .camera,
        scalar_camera
    );
    assert_eq!(clone.composition_revision.identity(), original);
    assert_ne!(
        prepared_for(&state, &board).composition_revision.identity(),
        original
    );
}
