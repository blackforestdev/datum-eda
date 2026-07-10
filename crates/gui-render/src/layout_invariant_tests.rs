//! Enforces the UI layout invariants from PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM
//! and docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md over the real Taffy-solved
//! rects across the supported scale matrix. Previously ungated (only scattered
//! single-scale unit tests + #[ignore] goldens); wired into
//! scripts/run_migration_proof_gates.sh as PG-UI-LAYOUT-INVARIANTS.
//!
//! Invariants asserted (the contract's "at minimum" set):
//! - shell rectangles are non-degenerate and stay inside the window;
//! - the left/viewport/right columns do not overlap;
//! - every interactive hit region stays inside the window and inside a shell
//!   panel (no control/card overflow);
//! - right-panel inspector/review cards stay inside the sidebar and do not
//!   overlap.

use super::*;

const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];
const EPS: f32 = 0.5;

fn within(inner: RectPx, outer: RectPx) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.x + inner.width <= outer.x + outer.width + EPS
        && inner.y + inner.height <= outer.y + outer.height + EPS
}

fn overlaps(a: RectPx, b: RectPx) -> bool {
    a.x + a.width > b.x + EPS
        && b.x + b.width > a.x + EPS
        && a.y + a.height > b.y + EPS
        && b.y + b.height > a.y + EPS
}

#[test]
fn shell_and_hit_regions_hold_layout_invariants_across_scale_matrix() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let logical_w = 1280u32;
    let logical_h = 800u32;

    for scale in SCALES {
        let pw = ((logical_w as f32) * scale).round() as u32;
        let ph = ((logical_h as f32) * scale).round() as u32;
        let window = RectPx {
            x: 0.0,
            y: 0.0,
            width: pw as f32,
            height: ph as f32,
        };
        let dock = dock_height_for_state(&state);
        let layout = ShellLayout::for_surface(pw, ph, scale, dock);

        for (name, rect) in [
            ("top_menu_bar", layout.top_menu_bar),
            ("left_sidebar", layout.left_sidebar),
            ("right_sidebar", layout.right_sidebar),
            ("viewport", layout.viewport),
            ("bottom_strip", layout.bottom_strip),
            ("status_bar", layout.status_bar),
        ] {
            assert!(
                rect.width > 0.0 && rect.height > 0.0,
                "shell rect {name} is degenerate at scale {scale}"
            );
            assert!(
                within(rect, window),
                "shell rect {name} escapes the window at scale {scale}"
            );
        }

        assert!(
            !overlaps(layout.left_sidebar, layout.viewport),
            "left sidebar overlaps viewport at scale {scale}"
        );
        assert!(
            !overlaps(layout.right_sidebar, layout.viewport),
            "right sidebar overlaps viewport at scale {scale}"
        );
        assert!(
            !overlaps(layout.left_sidebar, layout.right_sidebar),
            "left sidebar overlaps right sidebar at scale {scale}"
        );

        let retained = RetainedScene::from_workspace(&state, pw, ph);
        let prepared = PreparedScene::from_workspace_for_surface(
            &state,
            pw,
            ph,
            scale,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let panels = [
            prepared.layout.top_menu_bar,
            prepared.layout.left_sidebar,
            prepared.layout.right_sidebar,
            prepared.layout.viewport,
            prepared.layout.bottom_strip,
            prepared.layout.status_bar,
        ];
        for (i, region) in prepared.hit_regions.iter().enumerate() {
            assert!(
                within(region.rect, window),
                "hit region #{i} escapes the window at scale {scale}"
            );
            assert!(
                panels.iter().any(|panel| within(region.rect, *panel)),
                "hit region #{i} is not contained in any shell panel at scale {scale}"
            );
        }
        for run in prepared
            .text_runs
            .iter()
            .filter(|run| run.text.contains("REVIEW NAV"))
        {
            let bounds = run.clip_bounds.expect("filtered clipped text run");
            assert!(
                within(bounds, prepared.layout.viewport),
                "viewport overlay hint clip escapes viewport at scale {scale}"
            );
            assert!(
                run.x + bounds.width
                    <= prepared.layout.viewport.x + prepared.layout.viewport.width + EPS,
                "viewport overlay hint escapes viewport at scale {scale}"
            );
        }

        if let Some(right) =
            side_panels::solve_right_panel_layout_with_taffy(&state, prepared.layout.right_sidebar)
        {
            assert!(
                within(right.inspector_rect, prepared.layout.right_sidebar),
                "inspector card escapes the right sidebar at scale {scale}"
            );
            assert!(
                right.inspector_rect.height
                    >= prepared.layout.right_sidebar.height - 2.0 * UI_CARD_MARGIN - EPS,
                "inspector card does not occupy the Phase 1 right sidebar at scale {scale}"
            );
        }
    }
}

/// Workspace pane-tiling invariants (decision 021): the central viewport is
/// tiled per the `WorkspaceLayout` tree. Asserted across the scale matrix AND a
/// set of tree shapes (single Board leaf; the default vertical V-split; an
/// H-split; a nested Split; and a zoomed default): every leaf rect is
/// non-degenerate and inside `viewport`; leaf rects don't overlap; dividers lie
/// strictly between siblings; `scene_viewport()` equals the focused leaf's scene;
/// and a zoomed leaf fills the whole viewport with no dividers.
#[test]
fn viewport_tiling_holds_pane_tree_invariants_across_scale_matrix() {
    use datum_gui_protocol::{PaneContent, PaneId, SplitOrientation, WorkspaceLayout};

    let logical_w = 1280u32;
    let logical_h = 800u32;

    // A few representative tree shapes to exercise the generalized tile walk.
    let shapes: Vec<(&str, WorkspaceLayout)> = vec![
        ("single_board", WorkspaceLayout::single()),
        ("default_v_split", WorkspaceLayout::default()),
        ("h_split", {
            let mut l = WorkspaceLayout::single();
            l.split_focused(SplitOrientation::Horizontal);
            l
        }),
        ("nested_split", {
            // Vertical outer split; then split the second (right) leaf
            // horizontally, producing Board | [Board / Board] with three leaves
            // and two dividers of differing orientation.
            let mut l = WorkspaceLayout::single();
            l.split_focused(SplitOrientation::Vertical); // ids 0 | 1, focus 0
            l.focus_next(); // focus the right leaf (id 1)
            l.split_focused(SplitOrientation::Horizontal); // right becomes [1 / 2]
            l
        }),
        ("zoomed_default", {
            let mut l = WorkspaceLayout::default();
            l.toggle_zoom(); // maximize the focused (Board) leaf
            l
        }),
    ];

    for scale in SCALES {
        let pw = ((logical_w as f32) * scale).round() as u32;
        let ph = ((logical_h as f32) * scale).round() as u32;
        let layout = ShellLayout::for_surface(pw, ph, scale, None);

        for (shape_name, workspace) in &shapes {
            let panes = layout.viewport_panes(workspace);
            assert!(
                !panes.panes.is_empty(),
                "{shape_name}: tile walk produced no leaves at scale {scale}"
            );

            // Every leaf rect: non-degenerate, inside the viewport, with its
            // header and scene inside its own frame.
            for leaf in &panes.panes {
                let pane = leaf.rect;
                assert!(
                    pane.frame.width > 0.0 && pane.frame.height > 0.0,
                    "{shape_name}: leaf {:?} frame is degenerate at scale {scale}",
                    leaf.id
                );
                assert!(
                    within(pane.frame, layout.viewport),
                    "{shape_name}: leaf {:?} frame escapes the viewport at scale {scale}",
                    leaf.id
                );
                assert!(
                    within(pane.header, pane.frame),
                    "{shape_name}: leaf {:?} header escapes its own pane at scale {scale}",
                    leaf.id
                );
                assert!(
                    within(pane.scene, pane.frame),
                    "{shape_name}: leaf {:?} scene escapes its own pane at scale {scale}",
                    leaf.id
                );
            }

            // Leaf frames do not overlap one another.
            for (i, a) in panes.panes.iter().enumerate() {
                for b in panes.panes.iter().skip(i + 1) {
                    assert!(
                        !overlaps(a.rect.frame, b.rect.frame),
                        "{shape_name}: leaf {:?} overlaps leaf {:?} at scale {scale}",
                        a.id,
                        b.id
                    );
                }
            }

            // Every divider lies strictly between two leaf frames (no leaf frame
            // overlaps a divider gutter) and stays inside the viewport.
            for divider in &panes.dividers {
                assert!(
                    within(divider.rect, layout.viewport),
                    "{shape_name}: divider escapes the viewport at scale {scale}"
                );
                for leaf in &panes.panes {
                    assert!(
                        !overlaps(leaf.rect.frame, divider.rect),
                        "{shape_name}: leaf {:?} overlaps a divider at scale {scale}",
                        leaf.id
                    );
                }
            }

            // scene_viewport (world board canvas + gpu scissor) equals the BOARD
            // SCENE leaf's scene — bound to the board pane so the PCB stays put
            // regardless of focus (falling back to the focused leaf only when no
            // board leaf exists). The single-live-scene invariant, board-bound.
            let expected_scene = panes
                .scene_leaf()
                .map(|leaf| leaf.rect.scene)
                .unwrap_or_else(|| panes.focused_pane().rect.scene);
            assert_eq!(
                layout.scene_viewport(workspace),
                expected_scene,
                "{shape_name}: scene_viewport must follow the board scene leaf at scale {scale}"
            );
            assert_eq!(
                panes.focused_document(),
                workspace.focused_content(),
                "{shape_name}: focused_document must match the tree's focused leaf at scale {scale}"
            );
        }

        // Zoom: the maximized leaf fills the WHOLE viewport and no dividers are
        // emitted; scene_viewport follows that single leaf.
        {
            let mut zoomed = WorkspaceLayout::default();
            zoomed.toggle_zoom();
            let panes = layout.viewport_panes(&zoomed);
            assert_eq!(
                panes.panes.len(),
                1,
                "zoom must collapse to a single visible leaf at scale {scale}"
            );
            assert!(
                panes.dividers.is_empty(),
                "zoom must emit no dividers at scale {scale}"
            );
            assert_eq!(
                panes.panes[0].rect.frame, layout.viewport,
                "zoomed leaf must fill the whole viewport at scale {scale}"
            );
        }

        // Guard the intended default: the default tree reproduces the former
        // fixed two-pane split — two leaves (Board focused | Schematic), one
        // vertical divider, and the focused Board leaf's scene as scene_viewport.
        {
            let default = WorkspaceLayout::default();
            let panes = layout.viewport_panes(&default);
            assert_eq!(panes.panes.len(), 2, "default must be two leaves at scale {scale}");
            assert_eq!(panes.dividers.len(), 1, "default must have one divider at scale {scale}");
            assert_eq!(panes.panes[0].content, PaneContent::Board);
            assert_eq!(panes.panes[1].content, PaneContent::Schematic);
            assert_eq!(panes.focused, PaneId(0));
            // Divider strictly between the two panes.
            let a = panes.panes[0].rect.frame;
            let b = panes.panes[1].rect.frame;
            let d = panes.dividers[0].rect;
            assert!(
                d.x + EPS >= a.x + a.width && d.x + d.width <= b.x + EPS,
                "default divider is not between the panes at scale {scale}"
            );
        }
    }
}

/// Click-to-focus hit map (decision 021): a point inside a NON-focused pane maps
/// to that pane's leaf id (the seam `Runtime::pane_at_screen` calls), and a point
/// outside every pane (the left sidebar) maps to nothing so the click falls
/// through to normal behavior. Verifies the exact `ViewportPanes::leaf_at` logic
/// click-to-focus relies on, across the scale matrix.
#[test]
fn leaf_at_maps_points_for_click_to_focus() {
    use datum_gui_protocol::{PaneContent, WorkspaceLayout};

    let logical_w = 1280u32;
    let logical_h = 800u32;
    for scale in SCALES {
        let pw = ((logical_w as f32) * scale).round() as u32;
        let ph = ((logical_h as f32) * scale).round() as u32;
        let layout = ShellLayout::for_surface(pw, ph, scale, None);

        // Default: vertical Board | Schematic split, Board (PaneId 0) focused.
        let workspace = WorkspaceLayout::default();
        let panes = layout.viewport_panes(&workspace);
        assert_eq!(panes.panes.len(), 2, "default is two panes at scale {scale}");

        // A point at the center of each leaf frame maps back to that leaf.
        for leaf in &panes.panes {
            let f = leaf.rect.frame;
            let cx = f.x + f.width * 0.5;
            let cy = f.y + f.height * 0.5;
            assert_eq!(
                panes.leaf_at(cx, cy),
                Some(leaf.id),
                "center of leaf {:?} must hit it at scale {scale}",
                leaf.id
            );
        }

        // Concretely: a click in the RIGHT (Schematic) pane is NOT the focused
        // (Board, id 0) pane -> click-to-focus would swap focus.
        let schematic = panes
            .panes
            .iter()
            .find(|p| p.content == PaneContent::Schematic)
            .expect("default has a schematic pane");
        let f = schematic.rect.frame;
        let hit = panes
            .leaf_at(f.x + f.width * 0.5, f.y + f.height * 0.5)
            .expect("schematic-pane point must hit a leaf");
        assert_eq!(hit, schematic.id);
        assert_ne!(hit, panes.focused, "schematic pane differs from focused");

        // A point in the left sidebar (outside every pane frame) maps to nothing.
        let side = layout.left_sidebar;
        assert_eq!(
            panes.leaf_at(side.x + side.width * 0.5, side.y + side.height * 0.5),
            None,
            "left-sidebar point must not hit any pane at scale {scale}"
        );
    }
}

/// Divider-drag resize (decision 021): each divider carries the path of the Split
/// it controls, its orientation, and the full split frame, and `divider_at` hits
/// the (grab-widened) gutter. This is the render-side contract the runtime relies
/// on to map a grabbed gutter to `WorkspaceLayout::set_ratio_at_path`.
#[test]
fn dividers_carry_split_paths_and_are_grabbable() {
    use datum_gui_protocol::{SplitChild, SplitOrientation, WorkspaceLayout};
    let layout = ShellLayout::for_surface(1280, 800, 1.0, None);

    // Board|Schematic: exactly one root vertical divider, path [].
    let ws = WorkspaceLayout::board_schematic();
    let panes = layout.viewport_panes(&ws);
    assert_eq!(panes.dividers.len(), 1);
    let root_div = &panes.dividers[0];
    assert!(root_div.path.is_empty(), "root split divider has the empty path");
    assert_eq!(root_div.orientation, SplitOrientation::Vertical);
    // The split frame spans the whole viewport for the root split.
    assert_eq!(root_div.split_frame, layout.viewport);
    // divider_at hits the gutter center and misses a point far from it.
    let cx = root_div.rect.x + root_div.rect.width * 0.5;
    let cy = root_div.rect.y + root_div.rect.height * 0.5;
    assert!(panes.divider_at(cx, cy).is_some(), "gutter center is grabbable");
    assert!(
        panes.divider_at(root_div.rect.x - 100.0, cy).is_none(),
        "a point far from every gutter is not a divider grab"
    );

    // Nested: focus the schematic leaf and split it horizontally, producing
    // root = Split[ Board , Split[ Schematic / Board ] ]. Two dividers: the root
    // vertical (path []) and the inner horizontal (path [Second]).
    let mut nested = WorkspaceLayout::board_schematic();
    nested.focus_next();
    nested.split_focused(SplitOrientation::Horizontal);
    let np = layout.viewport_panes(&nested);
    assert_eq!(np.dividers.len(), 2);
    assert!(
        np.dividers
            .iter()
            .any(|d| d.path.is_empty() && d.orientation == SplitOrientation::Vertical),
        "root vertical divider present at path []"
    );
    assert!(
        np.dividers
            .iter()
            .any(|d| d.path == vec![SplitChild::Second]
                && d.orientation == SplitOrientation::Horizontal),
        "inner horizontal divider present at path [Second]"
    );
}
