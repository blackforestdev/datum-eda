//! The GUI consumes a complete engine plan without inventing annotation policy.
use super::*;
use crate::schematic_authoring::annotation::AnnotationEvidence;
use eda_engine::api::native_write::schematic_delete_proposal::{
    AnnotationChange, AnnotationSummary,
};

fn plan(review: &DeleteReview) -> AnnotationSummary {
    let mut symbols = review.source.symbols.values().collect::<Vec<_>>();
    symbols.sort_by_key(|s| s.uuid);
    AnnotationSummary {
        sheet: review.basis.source,
        instance_path: review
            .basis
            .document
            .as_ref()
            .unwrap()
            .instance_path
            .clone(),
        symbol_count: symbols.len(),
        changes: symbols
            .iter()
            .enumerate()
            .map(|(i, s)| AnnotationChange {
                symbol: s.uuid,
                previous_reference: s.reference.clone(),
                reference: format!("U{}", i + 8),
            })
            .collect(),
    }
}
#[test]
fn sheet_annotation_review_projects_every_old_new_reference_and_preserves_source() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    let source = review.source.clone();
    let scene = review.scene.clone();
    let summary = plan(&review);
    // These fixtures have physical bindings: engine eligibility is tested by its
    // canonical fixture. Here a typed complete plan tests GUI projection only.
    let evidence =
        AnnotationEvidence::from_summary(&state, &review, summary.clone(), &fixture.0).unwrap();
    assert_eq!(review.source, source);
    assert_eq!(review.scene, scene);
    assert_eq!(
        state.schematic_authoring.source.as_ref().unwrap().sheets[&summary.sheet],
        source
    );
    assert_eq!(
        evidence.preview.schematic_document,
        scene.schematic_document
    );
    assert_eq!(evidence.preview.bounds, scene.bounds);
    for change in &summary.changes {
        let label = format!("{} -> {}", change.previous_reference, change.reference);
        let old = scene
            .board_texts
            .iter()
            .find(|t| {
                t.source_object_uuid.as_deref() == Some(&change.symbol.to_string())
                    && t.text == change.previous_reference
            })
            .unwrap();
        let shown = evidence
            .preview
            .board_texts
            .iter()
            .find(|t| t.object_id == old.object_id)
            .unwrap();
        assert_eq!(shown.text, label);
        assert_eq!(shown.position, old.position);
        assert_eq!(shown.source_object_uuid, old.source_object_uuid);
        assert!(
            evidence
                .preview
                .board_text_geometries
                .iter()
                .any(|g| g.object_id == shown.object_id && !g.glyphs.is_empty())
        );
    }
    for original in &scene.board_texts {
        if !summary.changes.iter().any(|c| {
            original.source_object_uuid.as_deref() == Some(&c.symbol.to_string())
                && original.text == c.previous_reference
        }) {
            assert!(
                evidence.preview.board_texts.contains(original),
                "unrelated value/pin/field text stays unchanged"
            );
        }
    }
    review.targets.clear();
    review.pending = None;
    review.annotation = Some(evidence);
    let mut receipt = response(&review);
    receipt.current_model_revision = review.revision.clone();
    receipt.transaction_tip = review.tip;
    receipt.annotation = Some(summary.clone());
    review.observe(receipt.clone()).unwrap();
    assert!(review.ready());
    assert!(review.summary().contains("references change"));
    for change in &summary.changes {
        assert!(review.names().contains(&format!(
            "{} -> {}",
            change.previous_reference, change.reference
        )));
    }
    assert_eq!(
        review.preview_scene(),
        review.annotation.as_ref().unwrap().preview
    );
    let frozen = review.clone();
    receipt.annotation.as_mut().unwrap().changes[0]
        .reference
        .push('9');
    assert!(review.observe(receipt.clone()).is_err());
    assert_eq!(
        review, frozen,
        "mismatching response never replaces accepted preview evidence"
    );
    receipt.annotation = None;
    for status in [
        ProposalStatus::Draft,
        ProposalStatus::Accepted,
        ProposalStatus::Applied,
    ] {
        receipt.status = status;
        assert!(review.observe(receipt.clone()).is_err());
    }
    receipt.status = ProposalStatus::Rejected;
    review.observe(receipt).unwrap();
    assert!(review.finished());
    assert_eq!(
        review.annotation, frozen.annotation,
        "terminal cancellation retains immutable review evidence"
    );
    assert_eq!(review.source, source);
}

#[test]
fn sheet_annotation_review_refuses_incomplete_or_foreign_plan_and_preserves_noop() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let state = workspace(&model, &fixture.0);
    let review = DeleteReview::capture(&state).unwrap();
    let summary = plan(&review);
    for foreign_project in [true, false] {
        let mut stale = review.clone();
        if foreign_project {
            stale.basis.project = Uuid::new_v4();
        } else {
            stale.basis.revision.push_str("-stale");
        }
        assert_eq!(
            AnnotationEvidence::capture(&state, &stale).unwrap_err(),
            "Annotation source model no longer matches the frozen review basis",
            "same source geometry never substitutes for project/revision identity"
        );
    }
    for mode in 0..5 {
        let mut wrong = summary.clone();
        match mode {
            0 => wrong.symbol_count += 1,
            1 => wrong.instance_path.push(Uuid::new_v4()),
            2 => wrong.changes[0].symbol = Uuid::new_v4(),
            3 => wrong.changes[0].previous_reference.push('9'),
            _ => wrong.changes.push(wrong.changes[0].clone()),
        }
        assert!(AnnotationEvidence::from_summary(&state, &review, wrong, &fixture.0).is_err());
    }
    let mut empty = summary;
    empty.changes.clear();
    let noop = AnnotationEvidence::from_summary(&state, &review, empty, &fixture.0).unwrap();
    assert!(noop.summary.changes.is_empty());
    assert_eq!(noop.preview, review.scene);
    let item = crate::gui_menu_model::default_gui_menu_model()
        .unwrap()
        .marking_menus["schematic.empty"]
        .overflow
        .iter()
        .find(|i| i.label == "Annotate This Sheet…")
        .unwrap()
        .clone();
    assert_eq!(
        item.gui_local.as_deref(),
        Some("schematic.annotation.review")
    );
    assert!(item.is_phase_one_enabled());
    assert!(
        item.unavailable_reason(&state).is_some(),
        "physical owners cannot enable whole-sheet annotation"
    );
    let mut stale = state.clone();
    stale.scene.source_revision.push_str("-stale");
    assert!(DeleteReview::capture_annotation(&stale).is_err());
    assert_eq!(
        model.journal.len(),
        eda_engine::substrate::ProjectResolver::new(&fixture.0)
            .resolve()
            .unwrap()
            .journal
            .len()
    );
}
