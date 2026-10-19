//! Singleton presentation does not change selection identity or navigation.
use super::*;

#[test]
fn authored_group_uses_object_inspection_without_promoting_focus() {
    let (f, scene, ids) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let object = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Object(ids[0].clone()),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    let subject =
        SelectionSubject::Compound(CompoundSelection::new([ids[0].clone()], None).unwrap());
    let mut group =
        build_selection_view(&f.model, &authority, subject.clone(), &scene, None, |_| {
            true
        })
        .unwrap();
    assert_eq!(group.authored_singleton(), Some(&ids[0]));
    assert_eq!(group.inspector_title(), object.inspector_title());
    assert_eq!(group.scope, Some(Class::Track));
    assert_eq!(group.fields(), object.fields());
    for field in object.fields() {
        assert_eq!(group.field(field), object.field(field), "{field}");
        assert_eq!(
            group.field_quantity_presentation(field),
            object.field_quantity_presentation(field),
            "{field}"
        );
        assert_eq!(
            group.field_display_preview(field, usize::MAX),
            object.field_display_preview(field, usize::MAX),
            "{field}"
        );
    }
    assert_eq!(
        group.field("copper_width_nm"),
        InspectionValue::Common(serde_json::json!(5_080_000))
    );
    assert_eq!(group.subject, subject);
    assert_eq!(group.members, object.members);
    assert_eq!(group.focus(), None);
    assert_eq!(object.focus(), Some(&ids[0]));
    let context =
        DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(group.clone())));
    assert_eq!(context.kind, "compound");
    let details = context.details.unwrap();
    assert_eq!(details.scope, Some(Class::Track));
    assert_eq!(details.focus, None);
    assert_eq!(details.members.unwrap(), vec![ids[0].clone()]);
    let title = group.inspector_title();
    group.scope = None;
    assert!(!group.fields().contains("copper_width_nm"));
    assert_eq!(group.inspector_title(), title);
    assert_eq!(group.subject, subject);
    group.scope = Some(Class::Track);
    assert_eq!(group.fields(), object.fields());
    assert_eq!(group.focus(), None);
    Fixture::assert_bytes(&before);
}

#[test]
fn semantic_and_artifact_presentations_never_become_authored_singletons() {
    let (f, scene, ids) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let multi = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap()),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(multi.authored_singleton(), None);
    assert_eq!(multi.scope, None);
    assert_eq!(multi.inspector_title(), "Compound Selection — 2 objects");
    let singleton = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Object(ids[0].clone()),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    // Deliberately provide one member to each presentation consumer: cardinality
    // cannot erase a semantic/artifact subject. This is not resolution evidence.
    for subject in [
        SelectionSubject::Run(ids[0].clone().into()),
        SelectionSubject::GlobalNet(f.net),
        SelectionSubject::Bus(id(70)),
        SelectionSubject::Proposal("proposal".into()),
        SelectionSubject::Review("review".into()),
        SelectionSubject::Diagnostic("diagnostic".into()),
    ] {
        let mut view = singleton.clone();
        view.subject = subject.clone();
        assert_eq!(view.authored_singleton(), None, "{subject:?}");
        assert_eq!(
            view.inspector_title(),
            format!("{} — 1 objects", subject.display_name())
        );
        assert_eq!(view.subject, subject);
    }
    // Actual resolution control is Run-only. The board-local fixture Net is
    // not a canonical GlobalNet; the six controls above test presentation only.
    let subject = authority.acquire_run(ids[0].clone(), None).unwrap();
    let view = build_selection_view(&f.model, &authority, subject.clone(), &scene, None, |_| {
        true
    })
    .unwrap();
    assert_eq!(view.scope, None);
    assert_eq!(view.subject, subject);
    assert!(view.authored_singleton().is_none());
    Fixture::assert_bytes(&before);
}
