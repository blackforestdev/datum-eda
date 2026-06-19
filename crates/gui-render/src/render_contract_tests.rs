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
