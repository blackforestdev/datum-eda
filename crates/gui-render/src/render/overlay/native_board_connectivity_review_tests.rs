//! Display fixtures exercise actual font measurement, without claiming native authority.
use super::tests::review_fixture;
use super::*;
use datum_gui_protocol::{
    board_connectivity_revision::BoardConnectivityRevisionEvidence,
    schematic_authoring::field_table::FieldTableControl as C,
};
use eda_engine::{
    api::native_write::{
        schematic_board_transfer::connectivity_revision::{
            BoardConnectivityRevision, PadNetChange,
        },
        schematic_delete_proposal::NativeProposalAction,
    },
    substrate::{ElectricalOccurrence, ModelRevision},
};

#[test]
fn connectivity_review_preserves_ids_unknowns_and_complete_action_visibility() {
    let (base, scene, _, _) = review_fixture();
    let [component, terminal, pad, old, new, track, via, zone] =
        std::array::from_fn(|_| uuid::Uuid::new_v4());
    let rows = vec![
        format!("R1 10Ω · component {component} · read-only physical choice"),
        "Part: Resistor · Footprint: 0603 (read-only)".into(),
        format!("Terminal {terminal} → pad {pad} (retained)"),
        format!("Logical net: old {old} → new {new}"),
        format!("Board net: old {old} → new {new}"),
        format!("Track {track} · old net {old} · affected net"),
        format!("Via {via} · old net {new} · conservative candidate"),
        format!("Zone {zone} · old net {old} · fill already stale"),
        format!("Previous and proposed contacts unavailable: Zone {zone}; board nets {old}, {new}"),
        "Apply changes only pad/net correspondence. Copper retains geometry and old net assignments. Repair shorts and unrouted connections, explicitly refill and run DRC. Complete review does not certify physical connectivity or a complete Global Net.".into(),
    ];
    let evidence = BoardConnectivityRevisionEvidence {
        summary: BoardConnectivityRevision {
            schema_version: 1,
            project_id: component,
            sheet: component,
            board: component,
            model_revision: ModelRevision(scene.source_revision.clone()),
            changes: vec![PadNetChange {
                component_instance: component,
                terminal: ElectricalOccurrence {
                    class: "pin".into(),
                    source_id: terminal,
                    instance_path: vec![],
                },
                pad,
                previous_logical_net: old,
                logical_net: new,
                previous_board_net: old,
                board_net: new,
            }],
            relationships: vec![],
            retained_copper: vec![],
            contact_basis: vec![],
            zone_fills: vec![],
            source_revisions: Default::default(),
        },
        rows: rows.clone(),
    };
    let measured =
        crate::Renderer::board_connectivity_review_layout(&evidence, None, 800., 6000., 0.)
            .unwrap();
    let (apply, dismiss) = measured.control_widths.unwrap();
    let boundary = (apply + 8. + dismiss + 48.).ceil();
    for (width, height, clearance, pending, stale, complete) in [
        (800., 1500., 0., false, false, true),
        (340., 3000., 100., false, false, true),
        (800., 200., 0., false, false, false),
        (200., 6000., 0., false, false, false),
        (800., 1500., 212., true, false, true),
        (800., 1500., 212., false, true, true),
        (boundary + 1., 6000., 0., false, false, true),
        (boundary - 1., 6000., 0., false, false, false),
    ] {
        let mut state = base.clone();
        let review = state.schematic_authoring.delete_review.as_mut().unwrap();
        review.targets.clear();
        review.board_connectivity_revision = Some(evidence.clone());
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
        if let Some(foreground) = render(
            &mut quads,
            &mut text,
            &scene,
            viewport,
            &projection,
            &state,
            clearance,
            &mut hits,
            &mut plan,
        ) {
            quads.extend(foreground.quads);
            text.extend(foreground.text);
        }
        let visible = text.iter().map(|run| run.text.as_str()).collect::<String>();
        if complete {
            for row in &rows {
                assert!(visible.contains(row), "missing row: {row}");
            }
        } else {
            assert!(visible.contains("Enlarge the pane"));
            assert!(!visible.contains(&terminal.to_string()));
            assert!(!visible.contains("Tab accept deletion"));
        }
        let apply_hit = hits
            .iter()
            .find(|hit| hit.target == HitTarget::SchematicFieldTable(C::Accept));
        assert_eq!(apply_hit.is_some(), complete && !pending && !stale);
        if let Some(hit) = apply_hit {
            let label = text
                .iter()
                .find(|run| run.text == "Apply changes [Tab]")
                .unwrap();
            let measured =
                crate::measured_text_run_width_px(&label.text, label.size, label.face).unwrap();
            assert!(label.x + measured <= hit.rect.x + hit.rect.width - 3.);
        }
        let dismiss_hit = hits
            .iter()
            .find(|hit| hit.target == HitTarget::SchematicFieldTable(C::Cancel))
            .unwrap();
        assert!(dismiss_hit.rect.y >= viewport.y);
        assert!(
            dismiss_hit.rect.y + dismiss_hit.rect.height
                <= if complete { height - clearance } else { height }
        );
    }
}
