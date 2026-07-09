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
