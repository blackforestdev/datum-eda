use super::*;
use crate::native_selection_overlay::tests::selected_state;

impl PanePlan {
    fn overflow(&self, emitted_glyphs: usize) -> bool {
        self.detail_primitives.saturating_add(emitted_glyphs) > DETAILED_PRIMITIVES
    }
}

fn viewport() -> RectPx {
    RectPx {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 200.0,
    }
}
fn visible() -> Quad {
    Quad::from_rect(
        RectPx {
            x: 20.0,
            y: 20.0,
            width: 8.0,
            height: 8.0,
        },
        TEXT_ACCENT,
    )
}

#[test]
fn emitted_clipped_detail_and_glyphs_share_exact_boundary_excluding_feedback() {
    let (_fixture, state) = selected_state();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut plan = PanePlan::new(&state, viewport(), camera).unwrap();
    let mut output = vec![visible(); 65_535];
    output.push(Quad::from_rect(
        RectPx {
            x: 400.0,
            y: 20.0,
            width: 8.0,
            height: 8.0,
        },
        TEXT_ACCENT,
    ));
    output.push(Quad::from_rect(
        RectPx {
            x: -20.0,
            y: 20.0,
            width: 8.0,
            height: 8.0,
        },
        TEXT_ACCENT,
    ));
    plan.finish_detail(&mut output, 0);
    assert_eq!(plan.detail_primitives, 65_535);
    assert_eq!(output.len(), 65_535);
    assert!(
        !plan.overflow(1),
        "65,536 actual visible primitives remain detailed"
    );
    assert!(
        plan.overflow(2),
        "one more clipped glyph triggers the whole-pane decision"
    );
    // Later region feedback remains painted but is not selected-detail admission.
    output.push(visible());
    assert_eq!(plan.detail[0].end, 65_535 * 6);
    assert_eq!(plan.detail_primitives, 65_535);
    assert!(
        plan.subject.is_none(),
        "authored membership is not duplicated as artifact metadata"
    );
    plan.freeze().unwrap();
}

#[test]
fn separate_producers_join_one_pane_limit_and_voids_remain_ordered() {
    let (_fixture, state) = selected_state();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut first = PanePlan::new(&state, viewport(), camera).unwrap();
    first.finish_detail(&mut vec![visible(); 30_000], 0);
    let mut second = PanePlan::new(&state, viewport(), camera).unwrap();
    second.finish_detail(&mut vec![visible(); 35_536], 0);
    let mut base = Vec::new();
    {
        let mut output = second.base_output(&mut base, true);
        output.push(visible());
        output.role(crate::presentation_ranges::Role::Void);
        output.push(visible());
        output.role(crate::presentation_ranges::Role::Material);
        output.push(visible());
    }
    assert_eq!(
        second
            .screen
            .iter()
            .map(|s| (s.selected, s.void))
            .collect::<Vec<_>>(),
        vec![(true, false), (true, true), (true, false)]
    );
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("fixture selection")
    };
    let mut request =
        mask::Request::capture(view, &state.scene, datum_gui_protocol::PaneContent::Board)
            .unwrap()
            .unwrap();
    let plans = [
        std::sync::Arc::new(first.freeze().unwrap()),
        std::sync::Arc::new(second.freeze().unwrap()),
    ];
    request.set_screen(plans.iter().cloned()).unwrap();
    assert!(!request.detailed_overflow(0));
    assert!(
        request.detailed_overflow(1),
        "both producers share the pane's existing limit"
    );
    assert_eq!(request.screen_count(), 3);
    assert!(request.screen().nth(1).unwrap().void);
}

#[test]
fn original_selected_hits_and_geometry_remain_distinct_from_baseline_and_typed_lifetime() {
    let (_fixture, mut state) = selected_state();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let projection = Projection::new(viewport(), &state.scene.bounds, camera);
    let overlay = ProposalOverlayPrimitive {
        overlay_id: "source-overlay".into(),
        primitive_kind: "route".into(),
        proposal_action_id: "same-id".into(),
        layer_id: None,
        render_role: "proposed_overlay".into(),
        width_nm: Some(500_000),
        drill_nm: None,
        diameter_nm: None,
        path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 5_000_000, y: 0 }],
    };
    let mut selected = Vec::new();
    let expected_hits = crate::overlay_channels::push_overlay(
        &mut selected,
        &overlay,
        &projection,
        PROPOSAL_BASE,
        true,
        false,
    );
    crate::hit_clipping::clip_content(
        &mut selected,
        &mut Vec::new(),
        &mut Vec::new(),
        0,
        0,
        0,
        viewport(),
    );
    let mut plan = PanePlan::new(&state, viewport(), camera).unwrap();
    let mut combined = Vec::new();
    let actual_hits = crate::overlay_channels::push_overlay_channels(
        &mut combined,
        &mut plan,
        &overlay,
        &projection,
        PROPOSAL_BASE,
        true,
    );
    assert_eq!(actual_hits, expected_hits);
    assert!(!selected.is_empty());
    let detail = &plan.detail[0];
    assert_eq!(
        &combined[detail.start as usize / 6..detail.end as usize / 6],
        selected.as_slice()
    );
    let mut baseline = Vec::new();
    crate::overlay_channels::push_overlay(
        &mut baseline,
        &overlay,
        &projection,
        PROPOSAL_BASE,
        false,
        false,
    );
    crate::hit_clipping::clip_content(
        &mut baseline,
        &mut Vec::new(),
        &mut Vec::new(),
        0,
        0,
        0,
        viewport(),
    );
    assert_eq!(&combined[..detail.start as usize / 6], baseline.as_slice());
    let SelectionTarget::Native(view) = &mut state.selection else {
        panic!("fixture selection")
    };
    view.subject = datum_gui_protocol::SelectionSubject::Proposal("same-id".into());
    let proposal = PanePlan::new(&state, viewport(), camera).unwrap();
    let SelectionTarget::Native(view) = &mut state.selection else {
        unreachable!()
    };
    view.subject = datum_gui_protocol::SelectionSubject::Review("same-id".into());
    let review = PanePlan::new(&state, viewport(), camera).unwrap();
    assert_ne!(
        proposal, review,
        "typed subjects cannot reuse same-ID cue intent"
    );
    let mut moved = camera;
    moved.center_x_nm += 32.0;
    assert_ne!(review, PanePlan::new(&state, viewport(), moved).unwrap());
    let SelectionTarget::Native(view) = &mut state.selection else {
        unreachable!()
    };
    view.stale = true;
    assert!(
        PanePlan::new(&state, viewport(), camera)
            .unwrap()
            .subject
            .is_none()
    );
}

#[test]
fn overflow_compaction_preserves_other_panes_base_voids_and_feedback_order() {
    let (_fixture, state) = selected_state();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let retained = crate::RetainedScene::from_workspace(&state, 300, 200);
    let mut prepared = PreparedScene::from_workspace(&state, 300, 200, camera, &retained).unwrap();
    prepared.pane_cues.clear();
    prepared.selection_masks.clear();
    let mut first = PanePlan::new(&state, viewport(), camera).unwrap();
    let mut quads = Vec::new();
    {
        let mut base = first.base_output(&mut quads, true);
        base.push(visible());
        base.role(crate::presentation_ranges::Role::Void);
        base.push(visible());
    }
    let before = quads.len();
    quads.extend(std::iter::repeat_n(visible(), DETAILED_PRIMITIVES + 1));
    first.finish_detail(&mut quads, before);
    let feedback = Quad::from_rect(
        RectPx {
            x: 50.0,
            y: 50.0,
            width: 10.0,
            height: 10.0,
        },
        TEXT_PRIMARY,
    );
    quads.push(feedback);
    let mut second = PanePlan::new(&state, viewport(), camera).unwrap();
    let offset = quads.len() as u32 * 6;
    let mut other = vec![visible()];
    second.finish_detail(&mut other, 0);
    for range in second.detail.iter_mut() {
        range.start += offset;
        range.end += offset;
    }
    quads.extend(other);
    prepared.viewport_overlay_vertices = quads_to_vertices(&quads);
    prepared.selection_overlay_split = offset as usize;
    prepared.pane_cues = vec![
        (
            datum_gui_protocol::PaneId(1),
            std::sync::Arc::new(first.freeze().unwrap()),
        ),
        (
            datum_gui_protocol::PaneId(2),
            std::sync::Arc::new(second.freeze().unwrap()),
        ),
    ];
    let expected = quads_to_vertices(&[visible(), visible(), feedback, visible()]);
    prepared.compact_overflow_detail().unwrap();
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&prepared.viewport_overlay_vertices),
        bytemuck::cast_slice::<_, u8>(&expected)
    );
    assert_eq!(prepared.selection_overlay_split, 18);
    let first = &prepared.pane_cues[0].1;
    assert_eq!(first.detail_primitives, DETAILED_PRIMITIVES + 1);
    assert!(first.detail.is_empty());
    assert_eq!(first.screen[0].range, 0..6);
    assert_eq!(first.screen[1].range, 6..12);
    assert!(first.screen[1].void);
    assert_eq!(prepared.pane_cues[1].1.detail[0], 18..24);
}
