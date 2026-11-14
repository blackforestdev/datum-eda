//! Actual board-field Proposal paint and hit safety under constrained views.
use super::tests::review_fixture;
use super::*;
use datum_gui_protocol::board_transfer::BoardFieldRevisionEvidence;
use datum_gui_protocol::schematic_authoring::field_table::FieldTableControl as C;
use eda_engine::api::native_write::{
    schematic_board_transfer::field_revision::{BoardFieldChange, BoardFieldRevision},
    schematic_delete_proposal::NativeProposalAction,
};

#[test]
fn board_field_payload_and_controls_remain_complete_or_truthfully_unavailable() {
    let (base, scene, _, _) = review_fixture();
    let id = uuid::Uuid::new_v4();
    let old = "original value Ω 漢字🙂\t1234567890 ".repeat(6);
    let new = "replacement value μF 漢字🙂\n9876543210 ".repeat(6);
    let evidence = BoardFieldRevisionEvidence {
        summary: BoardFieldRevision {
            project_id: id,
            sheet: id,
            board: id,
            model_revision: eda_engine::substrate::ModelRevision(scene.source_revision.clone()),
            changes: vec![BoardFieldChange {
                component_instance: id,
                symbol: id,
                package: id,
                previous_reference: "R1".into(),
                reference: "AMP1".into(),
                previous_value: old.clone(),
                value: new.clone(),
            }],
            source_revisions: Default::default(),
        },
        contexts: vec!["Part: Demo Resistor · Footprint: R-0603 — read-only".into()],
    };
    let full =
        crate::Renderer::board_field_review_layout(&evidence, None, 800., 6000., 0.).unwrap();
    let (apply_width, dismiss_width) = full.control_widths.unwrap();
    let boundary = (apply_width + 8. + dismiss_width + 48.).ceil();
    let mut cases = vec![
        (800., 900., 0., false, false, true),
        (800., 900., 212., false, false, true),
        (340., 1200., 36., false, false, true),
        (200., 400., 36., false, false, false),
        (800., 250., 36., false, false, false),
        (800., 180., 180., false, false, false),
        (800., 900., 36., true, false, true),
        (800., 900., 36., false, true, true),
    ];
    cases.extend([
        (boundary + 1., 6000., 0., false, false, true),
        (boundary - 1., 6000., 0., false, false, false),
    ]);
    for (width, height, clearance, pending, stale, complete) in cases {
        let mut state = base.clone();
        let review = state.schematic_authoring.delete_review.as_mut().unwrap();
        review.targets.clear();
        review.board_field_revision = Some(evidence.clone());
        if pending {
            review.pending = Some(NativeProposalAction::Preview {
                proposal_id: review.proposal_id,
            });
        }
        if stale {
            review.revision = "stale".into();
        }
        let viewport = RectPx {
            x: 0.,
            y: 0.,
            width,
            height,
        };
        let camera = CameraState::fit_to_bounds(&scene.bounds);
        let projection = Projection::new(viewport, &scene.bounds, camera);
        let mut plan =
            native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        let foreground = render(
            &mut quads,
            &mut text,
            &scene,
            viewport,
            &projection,
            &state,
            clearance,
            &mut hits,
            &mut plan,
        );
        if let Some(foreground) = foreground {
            quads.extend(foreground.quads);
            text.extend(foreground.text);
        }
        let visible = text.iter().map(|run| run.text.as_str()).collect::<String>();
        if complete {
            assert!(visible.contains(&old));
            assert!(visible.contains(&new));
            assert!(visible.contains("Reference: old R1 → new AMP1"));
            assert!(visible.contains("Footprint: R-0603"));
        } else {
            assert!(!visible.contains(&old));
            assert!(!visible.contains(&new));
            assert!(visible.contains("Enlarge the pane"));
        }
        let apply = hits
            .iter()
            .find(|hit| hit.target == HitTarget::SchematicFieldTable(C::Accept));
        assert_eq!(apply.is_some(), complete && !pending && !stale);
        if let Some(hit) = apply {
            let label = text
                .iter()
                .find(|run| run.text == "Apply changes [Tab]")
                .unwrap();
            let width =
                crate::measured_text_run_width_px(&label.text, label.size, label.face).unwrap();
            assert!(label.x >= hit.rect.x + 3.);
            assert!(label.x + width <= hit.rect.x + hit.rect.width - 3.);
        }
        let dismiss = hits
            .iter()
            .find(|hit| hit.target == HitTarget::SchematicFieldTable(C::Cancel))
            .unwrap();
        assert!(dismiss.rect.y >= viewport.y);
        assert!(
            dismiss.rect.y + dismiss.rect.height
                <= if complete { height - clearance } else { height }
        );
        assert!(visible.contains("Dismiss"));
    }
    // Exercise actual PreparedScene console/foreground ownership, not just paint.
    let mut state = base;
    let pane = state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    state.ui.layout.zoomed = Some(pane);
    state.schematic_documents.project = None;
    state.schematic_scene = Some(scene.clone());
    let review = state.schematic_authoring.delete_review.as_mut().unwrap();
    review.basis.pane = pane;
    review.targets.clear();
    review.board_field_revision = Some(evidence);
    state.ui.console.set_history_expanded(true);
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    for menu in [None, Some("Edit".to_string())] {
        state.ui.active_menu = menu.clone();
        let retained = RetainedScene::from_workspace(&state, 1280, 480);
        let mut frame =
            PreparedScene::from_workspace(&state, 1280, 480, camera, &retained).unwrap();
        frame.set_surface_camera(pane, camera);
        frame.append_native_selection(&state).unwrap();
        assert!(state.ui.console.history_expanded());
        assert!(
            frame
                .console_overlay_layout
                .unwrap()
                .history_panel
                .is_some()
        );
        let refusal = frame
            .menu_overlay_text_runs
            .iter()
            .any(|run| run.text == "PCB update");
        let dismiss = frame
            .hit_regions
            .iter()
            .any(|hit| hit.target == HitTarget::SchematicFieldTable(C::Cancel));
        assert_eq!(refusal, menu.is_none());
        assert_eq!(dismiss, menu.is_none());
        assert!(
            !frame
                .hit_regions
                .iter()
                .any(|hit| hit.target == HitTarget::SchematicFieldTable(C::Accept))
        );
    }
}
