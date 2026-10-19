use super::*;
use crate::terminal_accessibility_platform::selection_fixture as fixture;
use datum_gui_protocol::{AuthoredSelectionClass, CompoundSelection, SelectionSubject};
use datum_gui_viewport::selection_lifetime::reconcile_native_selection;
use eda_engine::{ir::geometry::Point, substrate::Operation};

#[test]
fn singleton_inspector_preserves_explicit_scope_through_refresh_and_revision_shrink() {
    let mut f = fixture::Fixture::new("singleton-scope-lifetime");
    let a = f.line(fixture::id(1), Point::new(0, 0), Point::new(1000, 0));
    f.line(fixture::id(2), Point::new(2000, 0), Point::new(3000, 0));
    let previous = Arc::new(NativeSelectionResolution::capture(&f.model).unwrap());
    let members: Vec<_> = previous
        .authored_identities()
        .iter()
        .filter(|id| id.class == AuthoredSelectionClass::Track)
        .cloned()
        .collect();
    let subject = SelectionSubject::Compound(CompoundSelection::new(members, None).unwrap());
    let workspace = |model: &eda_engine::substrate::DesignModel| {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.project_uuid = model.project.project_id.to_string();
        state.scene.source_revision = model.model_revision.0.clone();
        state.supervision.journal.accepted_transaction_tip =
            eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
        state
    };
    let state = workspace(&f.model);
    let initial = build_workspace_selection_view(&f.model, &previous, subject, &state).unwrap();
    let mut priors = Vec::new();
    let before = f.source_bytes();
    for scope in [None, Some(AuthoredSelectionClass::Track)] {
        let mut prior = initial.clone();
        prior.scope = scope;
        let mut next = state.clone();
        let filters = next.ui.filters.clone();
        let prepared = prepare_from_model(
            &mut next,
            Inputs {
                selected: SelectionTarget::Native(Box::new(prior.clone())),
                filters,
                prior: Some(previous.clone()),
            },
            Arc::new(f.model.clone()),
            Default::default(),
        )
        .unwrap();
        let SelectionTarget::Native(refreshed) = prepared.selected else {
            panic!("native refresh")
        };
        assert_eq!(refreshed.scope, scope);
        assert_eq!(refreshed.subject, prior.subject);
        assert_eq!(refreshed.members, prior.members);
        assert_eq!(refreshed.focus(), None);
        priors.push(prior);
    }
    fixture::Fixture::assert_bytes(&before);
    f.commit([Operation::DeleteBoardTrack {
        track_id: a.uuid,
        track: serde_json::to_value(a).unwrap(),
    }]);
    let before = f.source_bytes();
    for prior in priors {
        let mut next = workspace(&f.model);
        let filters = next.ui.filters.clone();
        let prepared = prepare_from_model(
            &mut next,
            Inputs {
                selected: SelectionTarget::Native(Box::new(prior.clone())),
                filters,
                prior: Some(previous.clone()),
            },
            Arc::new(f.model.clone()),
            Default::default(),
        )
        .unwrap();
        let SelectionTarget::Native(view) = prepared.selected else {
            panic!("native survivor")
        };
        assert_eq!(view.scope, prior.scope);
        assert!(matches!(view.subject, SelectionSubject::Compound(_)));
        assert_eq!(view.members.len(), 1);
        assert_eq!(view.focus(), None);
        assert_eq!(
            view.fields().contains("copper_width_nm"),
            prior.scope.is_some()
        );
        assert_eq!(
            view.inspector_title(),
            view.member_display_label(view.members.first().unwrap(), 1)
        );
    }
    fixture::Fixture::assert_bytes(&before);
}

fn report(
    f: &fixture::Fixture,
    previous: &NativeSelectionResolution,
    subject: &SelectionSubject,
) -> SelectionTarget {
    let current = NativeSelectionResolution::capture(&f.model).unwrap();
    let reconciled =
        reconcile_native_selection(subject, f.model.project.project_id, previous, &current)
            .unwrap();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    let view = datum_gui_protocol::selection_view::build_selection_view(
        &f.model,
        &current,
        reconciled.subject,
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    target_after_reconciliation(view, reconciled.dropped, reconciled.state.explanation())
}

#[test]
fn last_identity_drop_report_is_empty_and_undo_never_resurrects_selection() {
    let mut f = fixture::Fixture::new("empty-lifetime-report");
    let track = f.line(fixture::id(1), Point::new(0, 0), Point::new(1000, 0));
    let previous = NativeSelectionResolution::capture(&f.model).unwrap();
    let id = previous
        .authored_identities()
        .iter()
        .find(|id| id.class == AuthoredSelectionClass::Track)
        .unwrap()
        .clone();
    let subject = SelectionSubject::Object(id.clone());
    f.commit([Operation::DeleteBoardTrack {
        track_id: track.uuid,
        track: serde_json::to_value(&track).unwrap(),
    }]);
    let source = f.source_bytes();
    let SelectionTarget::Native(view) = report(&f, &previous, &subject) else {
        panic!("actual dropped report retained")
    };
    assert_eq!(view.subject, SelectionSubject::None);
    assert!(
        view.members.is_empty() && view.board_keys.is_empty() && view.schematic_keys.is_empty()
    );
    assert!(view.focus().is_none());
    assert!(
        view.related.is_empty()
            && view.related_board_keys.is_empty()
            && view.related_schematic_keys.is_empty()
    );
    assert!(view.values.is_empty() && view.authored_anchors.is_empty());
    assert!(view.board_presentations.is_empty() && view.schematic_presentations.is_empty());
    assert!(view.hidden.is_empty() && view.locked.is_empty() && view.unprojected.is_empty());
    assert_eq!(view.dropped, [id].into_iter().collect());
    assert!(
        view.notice
            .as_ref()
            .unwrap()
            .contains("1 deleted or stale identities dropped")
    );
    fixture::Fixture::assert_bytes(&source);
    let deleted_authority = NativeSelectionResolution::capture(&f.model).unwrap();
    f.undo();
    let source = f.source_bytes();
    assert_eq!(
        report(&f, &deleted_authority, &view.subject),
        SelectionTarget::None,
        "restored source is never selected from report"
    );
    fixture::Fixture::assert_bytes(&source);
    let current = NativeSelectionResolution::capture(&f.model).unwrap();
    let mut empty = datum_gui_protocol::selection_view::build_selection_view(
        &f.model,
        &current,
        SelectionSubject::None,
        &datum_gui_protocol::load_fixture_workspace_state().scene,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(
        target_after_reconciliation(empty.clone(), Default::default(), None),
        SelectionTarget::None
    );
    empty.notice = Some("discarded old message".into());
    let SelectionTarget::Native(explanation) = target_after_reconciliation(
        empty,
        Default::default(),
        Some("Current lifetime explanation".into()),
    ) else {
        panic!("explanation retained")
    };
    assert_eq!(
        explanation.notice.as_deref(),
        Some("Current lifetime explanation")
    );
    assert!(explanation.members.is_empty() && explanation.focus().is_none());
}

#[test]
fn focused_compound_drop_preserves_survivor_without_promoting_or_resurrecting() {
    let mut f = fixture::Fixture::new("surviving-lifetime-report");
    let a = f.line(fixture::id(1), Point::new(0, 0), Point::new(1000, 0));
    f.line(fixture::id(2), Point::new(2000, 0), Point::new(3000, 0));
    let previous = NativeSelectionResolution::capture(&f.model).unwrap();
    let members: std::collections::BTreeSet<_> = previous
        .authored_identities()
        .iter()
        .filter(|id| id.class == AuthoredSelectionClass::Track)
        .cloned()
        .collect();
    let focus = members.iter().find(|id| id.id == a.uuid).unwrap().clone();
    let survivor = members.iter().find(|id| id.id != a.uuid).unwrap().clone();
    let subject = SelectionSubject::Compound(CompoundSelection::new(members, Some(focus)).unwrap());
    f.commit([Operation::DeleteBoardTrack {
        track_id: a.uuid,
        track: serde_json::to_value(a).unwrap(),
    }]);
    let source = f.source_bytes();
    let SelectionTarget::Native(view) = report(&f, &previous, &subject) else {
        panic!("surviving report")
    };
    assert_eq!(view.members, [survivor].into_iter().collect());
    assert_eq!(view.dropped.len(), 1);
    assert!(view.focus().is_none(), "survivor must not be promoted");
    fixture::Fixture::assert_bytes(&source);
    let deleted = NativeSelectionResolution::capture(&f.model).unwrap();
    f.undo();
    let SelectionTarget::Native(restored) = report(&f, &deleted, &view.subject) else {
        panic!("survivor remains")
    };
    assert_eq!(restored.members, view.members);
    assert!(restored.focus().is_none());
}
