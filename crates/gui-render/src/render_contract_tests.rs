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
    assert_eq!(PANEL_CARD_BG, design_tokens::chrome::SURFACE_02);
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

    assert!(labels.contains(&"Board / Layout"));
    for tool in ["S", "M", "R", "V", "Z"] {
        assert!(labels.contains(&tool), "missing board-pane tool {tool}");
    }
    // (The "FOLLOWS PANE A" diagnostic dump was removed from the chrome; the
    // pane header's title + tools are the conformance surface here.)
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

#[test]
fn terminal_dock_surfaces_recent_activity_spans() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.terminal.activity_summary =
        vec!["#3 command datum.artifact.generate in:7B out:12B".to_string()];
    state.ui.dock_height_px = 260;

    for tab in [datum_gui_protocol::DockTab::Terminal] {
        state.ui.active_dock_tab = Some(tab);
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text.contains("ACTIVITY")),
            "{tab:?} dock should label the terminal activity block"
        );
        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text.contains("datum.artifact.generate")),
            "{tab:?} dock should render the terminal activity summary"
        );
        let activity_region = prepared.hit_regions.iter().find(|region| {
            matches!(
                &region.target,
                HitTarget::TerminalActivitySummary(summary)
                    if summary.contains("datum.artifact.generate")
            )
        });
        assert!(
            activity_region.is_some(),
            "{tab:?} dock should expose a clickable activity summary hit region"
        );
        let rect = activity_region.unwrap().rect;
        assert!(matches!(
            prepared.hit_test(rect.x + 1.0, rect.y + 1.0),
            Some(HitTarget::TerminalActivitySummary(summary))
                if summary.contains("datum.artifact.generate")
        ));
    }
}

#[test]
fn terminal_dock_does_not_render_output_lane_findings() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 300;
    state.checks = datum_gui_protocol::check_run_review_state_from_json(
        r#"{
          "contract": "check_run_v1",
          "check_run_id": "00000000-0000-0000-0000-00000000chk2",
          "profile_id": "standards",
          "status": "error",
          "finding_count": 1,
          "findings": [{
            "finding_id": "00000000-0000-0000-0000-00000000f002",
            "source": "drc",
            "code": "pad_mask_expansion_missing",
            "severity": "error",
            "fingerprint": "sha256:process-aperture",
            "domain": "drc",
            "rule_id": "process_aperture_policy",
            "status": "active",
            "evidence": [{
              "evidence_kind": "standards_basis",
              "basis_id": "datum.process_aperture_and_geometry.current"
            }]
          }]
        }"#,
    )
    .expect("check-run fixture should decode");

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        !prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("BASIS DATUM.PROCESS_APERTURE")),
        "Phase 1 terminal dock must not render the retired Output lane"
    );
}
