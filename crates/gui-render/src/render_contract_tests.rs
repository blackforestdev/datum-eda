//! Render-contract regression tests for the M7 semantic lanes:
//! declared render-stack ordering (M7-REN-006), material-first copper
//! appearance, dim-unrelated legibility (M7-REN-004), and
//! proposed-overlay emphasis discipline (M7-REN-003).

use super::*;
use datum_gui_protocol::PointNm;

#[test]
fn render_stack_policy_follows_declared_contract() {
    // M7-REN-006: the declared render-stack rule is layer type group
    // first, then back-to-front side. These are the memo's named
    // relations plus the full declared stage ladder.
    let priority = |name: &str| scene_layer_stack_priority(name, &[]);

    let declared_ladder = [
        "B.Cu",
        "In1.Cu",
        "F.Cu",
        "B.Mask",
        "F.Mask",
        "B.Paste",
        "F.Paste",
        "B.SilkS",
        "F.SilkS",
        "F.CrtYd",
        "Edge.Cuts",
    ];
    for pair in declared_ladder.windows(2) {
        assert!(
            priority(pair[0]) < priority(pair[1]),
            "{} must render below {}",
            pair[0],
            pair[1]
        );
    }

    // Memo-named relations, asserted independently of the ladder above.
    assert!(priority("F.Paste") > priority("B.Paste"));
    assert!(priority("F.Mask") > priority("B.Mask"));
    assert!(priority("F.Paste") > priority("F.Mask"));
    assert!(priority("F.SilkS") > priority("F.Paste"));
}

#[test]
fn render_stage_declaration_order_is_the_only_priority_encoding() {
    // M7-REN-006: the enum declaration order, derived Ord, and
    // render_stage_priority must agree — one encoding, not three.
    let declared = [
        RenderStage::BottomCopper,
        RenderStage::InnerCopper,
        RenderStage::TopCopper,
        RenderStage::BottomMask,
        RenderStage::TopMask,
        RenderStage::BottomPaste,
        RenderStage::TopPaste,
        RenderStage::BottomSilk,
        RenderStage::TopSilk,
        RenderStage::Mechanical,
        RenderStage::Edge,
        RenderStage::Other,
    ];
    for (index, stage) in declared.iter().enumerate() {
        assert_eq!(
            render_stage_priority(*stage),
            index as u32,
            "{stage:?} priority must equal its declaration position"
        );
    }
    let mut sorted = declared;
    sorted.sort();
    assert_eq!(sorted, declared, "derived Ord must match declaration order");

    let mut walk_priorities: Vec<u32> = POST_COPPER_STAGES
        .iter()
        .map(|stage| render_stage_priority(*stage))
        .collect();
    let unsorted = walk_priorities.clone();
    walk_priorities.sort();
    assert_eq!(
        walk_priorities, unsorted,
        "POST_COPPER_STAGES must iterate in declared draw order"
    );
}

// (Removed project_panel_renders_source_shard_health — the source-shard health
// line + attention rows were pulled from the Project panel as debug-HUD clutter;
// shard status is not part of the designed panel.)

#[test]
fn dimmed_copper_stays_legible_against_board_field() {
    // M7-REN-004: dim-unrelated must keep authored context readable.
    // Dimmed copper on every known family must stay clearly separated
    // from the board field it sits on.
    for layer in ["F.Cu", "In1.Cu", "B.Cu"] {
        let base = resolve_layer_appearance(Some(layer)).authored_track;
        let dimmed = dim_authored_color(base, true);
        let distance: f32 = dimmed
            .iter()
            .zip(BOARD_INNER_FIELD.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            distance > 0.35,
            "{layer}: dimmed copper {dimmed:?} too close to board field"
        );
    }
}

#[test]
fn copper_layer_appearance_is_material_first() {
    // M7-REN-006: authored copper primitive families inherit one base
    // material color per known copper layer family.
    for layer in ["F.Cu", "In1.Cu", "B.Cu"] {
        let appearance = resolve_layer_appearance(Some(layer));
        assert_eq!(
            appearance.authored_track, appearance.pad_copper,
            "{layer}: tracks and pads must share the layer material"
        );
        // Zone fill is a DERIVED shade of the same material (M7-REN-004
        // boundary readability), never an independent color system.
        assert_eq!(
            appearance.zone_fill,
            mix_color(
                appearance.authored_track,
                BOARD_INNER_FIELD,
                ZONE_FILL_FIELD_MIX
            ),
            "{layer}: zone fill must be the declared derived shade of the layer material"
        );
        assert_eq!(
            appearance.authored_track, appearance.zone_outline,
            "{layer}: zone outline must share the layer material"
        );
    }
}

#[test]
fn conformance_region_token_bindings_follow_design_book() {
    assert_eq!(APP_BG, design_tokens::chrome::BG_BASE);
    assert_eq!(PANEL_BG, design_tokens::chrome::SURFACE_01);
    // Passive panel bodies are the flat SURFACE_01 material (flush stacked
    // panels); SURFACE_02 is reserved for interactive fields/hover/tool-buttons.
    assert_eq!(PANEL_CARD_BG, design_tokens::chrome::SURFACE_01);
    assert_eq!(PANEL_CARD_BORDER, design_tokens::chrome::BORDER_SUBTLE);
    assert_eq!(TEXT_ACCENT, design_tokens::chrome::ACCENT);
    assert_eq!(REVIEW_ROW_ACTIVE_BG, design_tokens::chrome::ACCENT_TINT);
    assert_eq!(
        design_tokens::content::DRC_ERROR,
        design_tokens::chrome::STATUS_ERROR
    );
    assert_eq!(
        design_tokens::content::DRC_WARN,
        design_tokens::chrome::STATUS_WARN
    );

    let layers = [
        datum_gui_protocol::SceneLayer {
            layer_id: "f".to_string(),
            name: "F.Cu".to_string(),
            kind: "copper".to_string(),
            render_order: 0,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "b".to_string(),
            name: "B.Cu".to_string(),
            kind: "copper".to_string(),
            render_order: 1,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "silk".to_string(),
            name: "F.SilkS".to_string(),
            kind: "silk".to_string(),
            render_order: 2,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "edge".to_string(),
            name: "Edge.Cuts".to_string(),
            kind: "edge".to_string(),
            render_order: 3,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "rat".to_string(),
            name: "Ratsnest".to_string(),
            kind: "ratsnest".to_string(),
            render_order: 4,
            visible_by_default: false,
        },
    ];
    assert_eq!(
        layer_swatch_color_with_scene(Some("f"), &layers),
        design_tokens::content::COPPER_FRONT
    );
    assert_eq!(
        layer_swatch_color_with_scene(Some("b"), &layers),
        design_tokens::content::COPPER_BACK
    );
    assert_eq!(
        layer_swatch_color_with_scene(Some("silk"), &layers),
        design_tokens::content::SILK_TOP
    );
    assert_eq!(
        layer_swatch_color_with_scene(Some("edge"), &layers),
        design_tokens::content::EDGE
    );
    assert_eq!(
        layer_swatch_color_with_scene(Some("rat"), &layers),
        design_tokens::content::RATSNEST
    );

    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let has_panel_vertex = |rect: RectPx, color: [f32; 3]| {
        prepared
            .panel_vertices()
            .iter()
            .any(|vertex| vertex.color == color && vertex.pos == [rect.x, rect.y])
    };
    assert!(has_panel_vertex(prepared.layout.top_menu_bar, APP_BG));
    assert!(has_panel_vertex(prepared.layout.bottom_strip, APP_BG));
    assert!(has_panel_vertex(prepared.layout.status_bar, PANEL_BG));
}

#[test]
fn conformance_pane_header_tools_and_binding_chips_render() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let labels = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();

    // The default tree tiles a Board leaf and a Schematic leaf; each leaf emits
    // its content-derived header title, driven from the tree (not fixed literals).
    let panes = prepared.layout.viewport_panes(&state.ui.layout);
    for leaf in &panes.panes {
        let title = match leaf.content {
            datum_gui_protocol::PaneContent::Board => "Board \u{00B7} Layout",
            datum_gui_protocol::PaneContent::Schematic => "Schematic \u{00B7} Sheet 1",
        };
        assert!(
            labels.contains(&title),
            "missing header title {title} for leaf {:?}",
            leaf.id
        );
    }
    for tool in ["S", "M", "R", "V", "Z"] {
        assert!(labels.contains(&tool), "missing board-pane tool {tool}");
    }
    // A Schematic leaf is a real pane with its own (unfocused) chrome and a
    // labeled "Schematic (coming)" placeholder caption — not schematic world
    // geometry / authoring this commit.
    assert!(
        panes
            .panes
            .iter()
            .any(|l| l.content == datum_gui_protocol::PaneContent::Schematic),
        "default tree should carry a schematic leaf"
    );
    assert!(
        labels.contains(&"Schematic (coming)"),
        "missing schematic-leaf placeholder caption"
    );
}

/// A non-focused Board pane cannot render live under single-live-scene, so it
/// carries the "Inactive - click to focus" placeholder caption instead of a blank
/// canvas. Focusing the Schematic leaf leaves the Board leaf non-focused.
#[test]
fn non_scene_board_pane_shows_inactive_caption() {
    // The inactive placeholder is for a Board leaf that is NOT the live scene leaf
    // — e.g. a SECOND board pane. (The scene-leaf board renders the PCB regardless
    // of focus; that is locked by board_scene_stays_in_board_pane_regardless_of_focus.)
    // Build a Board|Board layout by retargeting the schematic leaf to Board, and
    // focus one of them: the focused board is the scene leaf, the other is inactive.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.layout.focus_next(); // focus the Schematic leaf
    state
        .ui
        .layout
        .set_focused_content(datum_gui_protocol::PaneContent::Board); // now Board|Board, 2nd focused
    let board_leaves = state
        .ui
        .layout
        .leaves()
        .into_iter()
        .filter(|id| {
            let mut probe = state.ui.layout.clone();
            probe.focused = *id;
            probe.focused_content() == datum_gui_protocol::PaneContent::Board
        })
        .count();
    assert_eq!(board_leaves, 2, "test precondition: two board leaves");

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let labels = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();
    assert!(
        labels.contains(&"Inactive \u{00b7} click to focus"),
        "a non-scene Board pane should show the inactive placeholder caption"
    );
}

/// The tiling differentiates focus: the focused leaf (whichever it is) draws the
/// accent focus dot + inset ACCENT pane frame; non-focused leaves draw neither.
/// Each lives inside its own pane rect. This locks the focus differentiation that
/// makes context-follows-focus legible (docs/gui/DATUM_GUI_DESIGN_SPEC.md).
#[test]
fn focus_frame_belongs_to_the_focused_leaf_only() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let panes = prepared.layout.viewport_panes(&state.ui.layout);
    // The accent pane frame is emitted as panel vertices inset 1px inside the
    // focused leaf; an accent quad must fall inside its frame and inside no
    // other (non-focused) leaf's interior.
    let has_accent_vertex_in = |rect: RectPx| {
        prepared.panel_vertices().iter().any(|v| {
            let [r, g, b] = TEXT_ACCENT;
            (v.color[0] - r).abs() < 0.01
                && (v.color[1] - g).abs() < 0.01
                && (v.color[2] - b).abs() < 0.01
                && rect.contains(v.pos[0], v.pos[1])
        })
    };
    for leaf in &panes.panes {
        if leaf.id == panes.focused {
            assert!(
                has_accent_vertex_in(leaf.rect.frame),
                "focused leaf {:?} must carry accent focus chrome",
                leaf.id
            );
        } else {
            // A non-focused leaf's interior (inside its frame but excluding the
            // shared divider edge) carries no accent focus frame.
            let interior = inset_rect(leaf.rect.frame, 2.0, 2.0, 2.0, 2.0);
            assert!(
                !has_accent_vertex_in(interior),
                "non-focused leaf {:?} must not carry an accent focus frame",
                leaf.id
            );
        }
    }
}

#[test]
fn diagnostic_evidence_marks_endpoints_only_over_proposed_copper() {
    // M7-REN-003: diagnostic emphasis over a proposed route may mark the
    // evidence span's endpoints, but per-vertex dots read as generic
    // path-editing handles and are forbidden.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let primitive = state
        .scene
        .review_primitives
        .first_mut()
        .expect("fixture should carry one review evidence primitive");
    let first = primitive.path[0];
    let last = *primitive.path.last().unwrap();
    primitive.path = vec![
        first,
        PointNm {
            x: (first.x + last.x) / 2,
            y: first.y,
        },
        PointNm {
            x: (first.x + last.x) / 2,
            y: (first.y + last.y) / 2,
        },
        last,
    ];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let focus_markers = prepared
        .viewport_overlay_vertices()
        .chunks_exact(6)
        .filter(|quad| {
            if quad[0].color != DIAGNOSTIC_FOCUS {
                return false;
            }
            let xs: Vec<f32> = quad.iter().map(|v| v.pos[0]).collect();
            let ys: Vec<f32> = quad.iter().map(|v| v.pos[1]).collect();
            let w = xs.iter().cloned().fold(f32::MIN, f32::max)
                - xs.iter().cloned().fold(f32::MAX, f32::min);
            let h = ys.iter().cloned().fold(f32::MIN, f32::max)
                - ys.iter().cloned().fold(f32::MAX, f32::min);
            (w - 4.0).abs() < 0.5 && (h - 4.0).abs() < 0.5
        })
        .count();
    assert_eq!(
        focus_markers, 2,
        "active evidence span must mark exactly its two endpoints, \
             not every path vertex"
    );
}

// Terminal-lane contract coverage lives in terminal_dock_contract_tests.rs
// (T0-C02 moved the activity-span and output-lane assertions there beside the
// rest of the terminal dock contract).

#[test]
fn scene_underlay_has_no_decorative_gold_edge_frame() {
    // Bug A: the spurious gold rounded-rect frame around the board (a fixed 10px
    // viewport-inset stroke in the InnerField-mixed EDGE color) is removed. The
    // ONLY board outline is the REAL projected Edge.Cuts in the retained world
    // pass. Assert the underlay still fills the inner field but no longer emits
    // any stroke in the retired decorative-edge color, while the real board
    // outline batches still exist in the retained scene.
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let inner_field = board_surface_color(BoardSurfaceRole::InnerField);
    let decorative_edge = mix_color(design_tokens::content::EDGE, inner_field, 0.18);

    let underlay = prepared.viewport_underlay_vertices();
    assert!(
        underlay.iter().any(|v| v.color == inner_field),
        "underlay must still fill the inner board field"
    );
    assert!(
        !underlay.iter().any(|v| v.color == decorative_edge),
        "underlay must not emit the retired decorative gold edge frame color"
    );
    // The real Edge.Cuts outline source is still present on the scene (projected
    // separately in the retained world pass, not synthesized in the underlay).
    assert!(
        !state.scene.outline.is_empty(),
        "real board outline (scene.outline / Edge.Cuts) must still exist"
    );
    let _ = &retained;
}

#[test]
fn board_scene_stays_in_board_pane_regardless_of_focus() {
    // The board scene is bound to the BOARD leaf, not the focused leaf. Focusing a
    // Schematic pane must NOT move the PCB into it (the earlier bug) NOR make the
    // PCB vanish from the board pane (the second bug) — the board must stay put and
    // visible so a user can view it while working another pane. Locks: (1) the
    // scene viewport is the same board-pane rect whether the board or the schematic
    // is focused, and (2) the board substrate still renders under both.
    let inner_field = board_surface_color(BoardSurfaceRole::InnerField);

    // Board focused (fixture default).
    let board_state = datum_gui_protocol::load_fixture_workspace_state();
    assert_eq!(
        board_state.ui.layout.focused_content(),
        datum_gui_protocol::PaneContent::Board,
        "fixture default should focus the Board leaf"
    );
    let retained_b = RetainedScene::from_workspace(&board_state, 1280, 800);
    let prepared_b = PreparedScene::from_workspace(
        &board_state,
        1280,
        800,
        CameraState::fit_to_bounds(&board_state.scene.bounds),
        &retained_b,
    );
    assert!(
        prepared_b
            .viewport_underlay_vertices()
            .iter()
            .any(|v| v.color == inner_field),
        "board-focused underlay must carry the board substrate field"
    );

    // Same layout, but focus the Schematic leaf.
    let mut schem_state = datum_gui_protocol::load_fixture_workspace_state();
    let schem_id = schem_state
        .ui
        .layout
        .leaves()
        .into_iter()
        .find(|id| {
            let mut probe = schem_state.ui.layout.clone();
            probe.focused = *id;
            probe.focused_content() == datum_gui_protocol::PaneContent::Schematic
        })
        .expect("default two-pane layout has a Schematic leaf");
    schem_state.ui.layout.focused = schem_id;
    assert_eq!(
        schem_state.ui.layout.focused_content(),
        datum_gui_protocol::PaneContent::Schematic
    );

    let retained_s = RetainedScene::from_workspace(&schem_state, 1280, 800);
    let prepared_s = PreparedScene::from_workspace(
        &schem_state,
        1280,
        800,
        CameraState::fit_to_bounds(&schem_state.scene.bounds),
        &retained_s,
    );

    // (1) The scene viewport does not move when focus shifts to the Schematic pane:
    // it stays bound to the board pane's rect (so the PCB neither migrates into the
    // schematic pane nor disappears).
    assert_eq!(
        prepared_s.scene_viewport, prepared_b.scene_viewport,
        "board scene viewport must stay on the board pane regardless of focus"
    );
    // (1b) The scene LEAF id is the same board leaf under both focus states. The
    // app binds the active board camera to this id and only swaps the camera when
    // it changes — so a board<->schematic focus change never resets/refits the
    // board's zoom (the "PCB zooms to fit on focus" bug).
    let board_scene_leaf = prepared_b
        .layout
        .viewport_panes(&board_state.ui.layout)
        .scene_leaf_id();
    let schem_scene_leaf = prepared_s
        .layout
        .viewport_panes(&schem_state.ui.layout)
        .scene_leaf_id();
    assert!(board_scene_leaf.is_some(), "board layout has a scene leaf");
    assert_eq!(
        board_scene_leaf, schem_scene_leaf,
        "scene leaf (the active camera's owner) must not change on a board<->schematic focus change"
    );
    // (2) The board substrate still renders (the board pane keeps showing the PCB
    // field) even though the Schematic pane is focused.
    assert!(
        prepared_s
            .viewport_underlay_vertices()
            .iter()
            .any(|v| v.color == inner_field),
        "board substrate must persist in the board pane when the Schematic is focused"
    );
}

#[test]
fn narrow_pane_header_does_not_bleed_into_neighbor() {
    // Divider-drag can shrink a pane hard (down to the 0.1 ratio clamp). Its header
    // (title + tool cluster) must clip/cull to the pane instead of spilling into
    // the adjacent enlarged pane — the panel/text passes are not scissored
    // per-pane, so a bleeding tool letter would float over the neighbor's header.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    // Shrink the (left, focused) Board pane hard; the Schematic pane enlarges.
    state.ui.layout.set_ratio_at_path(&[], 0.12);
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let panes = prepared.layout.viewport_panes(&state.ui.layout);
    let board = panes.panes[0].rect;
    let schem = panes.panes[1].rect;
    assert!(
        board.frame.width < schem.frame.width,
        "the board pane must be the shrunk one"
    );

    // Every header-band text run that STARTS inside the board pane must not extend
    // (after its clip bounds) into the schematic pane.
    for run in &prepared.text_runs {
        let in_header_band = run.y >= board.header.y && run.y <= board.header.y + board.header.height;
        let starts_in_board = run.x >= board.frame.x && run.x < schem.frame.x;
        if !in_header_band || !starts_in_board {
            continue;
        }
        let natural_right = run.x + measured_text_run_width_px(&run.text, run.size, run.face);
        let effective_right = match run.clip_bounds {
            Some(cb) => natural_right.min(cb.x + cb.width),
            None => natural_right,
        };
        assert!(
            effective_right <= schem.frame.x + 1.0,
            "board-pane header run '{}' (right {effective_right:.1}) bleeds into the schematic pane (x {:.1})",
            run.text,
            schem.frame.x
        );
    }
}
