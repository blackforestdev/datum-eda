//! Canonical source/revision/non-mutation and current-fill CAM witness oracles.
use super::*;
use crate::export::{
    ArcOutputDisposition, NativeArcOutputError, OutputMarginResult, native_arc_output_report,
    render_rs274x_copper_layer,
};

#[test]
fn native_m1_witness_binds_history_source_and_reopen_without_query_mutation() {
    let (root, mut model, net) = fixture("native_m1_history");
    let arc = Track {
        midpoint: Some(Point::new(0, 10_000)),
        ..line(net, (-10_000, 0), (10_000, 0))
    };
    tracks(&mut model, &root, std::slice::from_ref(&arc));
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let report = native_arc_output_report(&model, arc.uuid, 2, 0).unwrap();
    assert_eq!(report.model_revision, model.model_revision);
    assert_eq!(report.project_id, model.project.project_id);
    assert_eq!(report.witness.source(), &arc);
    assert_eq!(report.witness.disposition(), ArcOutputDisposition::Exact);
    assert!(!report.source_basis.is_empty());
    assert!(
        !report
            .constraints
            .requires_model_bound_fill_and_complete_inventory
    );
    assert_eq!(
        report.constraints.minimum_width,
        OutputMarginResult::Preserved
    );
    assert!(
        render_rs274x_copper_layer(8, &[], std::slice::from_ref(&arc), &[], &[])
            .unwrap()
            .contains("G02*")
    );
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let rational = Track {
        midpoint: Some(Point::new(1_000_000, 3_000_000)),
        from: Point::new(0, 0),
        to: Point::new(3_000_000, 0),
        ..arc.clone()
    };
    let write = build_set_board_track(&model, provenance(), &rational).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        native_arc_output_report(&model, arc.uuid, 2, 0)
            .unwrap()
            .witness
            .disposition(),
        ArcOutputDisposition::AllowanceRequired
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        native_arc_output_report(&model, arc.uuid, 2, 0)
            .unwrap()
            .witness,
        report.witness
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        native_arc_output_report(&model, arc.uuid, 2, 0).unwrap(),
        native_arc_output_report(&reopened, arc.uuid, 2, 0).unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_m1_fill_requires_current_certified_copper_not_outline_or_raw_empty() {
    let (root, mut model, net) = fixture("native_m1_fill");
    let arc = Track {
        midpoint: Some(Point::new(15, 10)),
        ..line(net, (5, 0), (25, 0))
    };
    tracks(&mut model, &root, std::slice::from_ref(&arc));
    let zone = zone(&mut model, &root, net);
    assert_eq!(
        native_arc_output_report(&model, arc.uuid, 2, 0),
        Err(NativeArcOutputError::UnverifiedFill(zone.uuid))
    );
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(10, 10)],
    );
    let r = native_arc_output_report(&model, arc.uuid, 2, 0).unwrap();
    assert_eq!(r.constraints.pairs.len(), 1);
    assert_eq!(r.constraints.pairs[0].other_source_id, zone.uuid);
    assert!(r.constraints.pairs[0].nominal_contact);
    fill(&mut model, &root, &zone, ZoneFillState::Filled, vec![]);
    assert!(
        !native_arc_output_report(&model, arc.uuid, 2, 0)
            .unwrap()
            .constraints
            .pairs[0]
            .nominal_contact
    );
    for state in [
        ZoneFillState::Stale,
        ZoneFillState::Unfilled,
        ZoneFillState::Unsupported,
    ] {
        fill(&mut model, &root, &zone, state, vec![]);
        assert_eq!(
            native_arc_output_report(&model, arc.uuid, 2, 0),
            Err(NativeArcOutputError::UnverifiedFill(zone.uuid))
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
