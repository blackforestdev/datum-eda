use datum_gui_protocol::selection_resolution::{SelectionModelRevision, SelectionResolution};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity, CompoundSelection,
    SelectionProjectId, SelectionSubject as Subject,
};
use datum_gui_viewport::selection_lifetime::reconcile_selection;
use std::collections::{BTreeMap, BTreeSet};

fn uuid(n: u64) -> SelectionProjectId {
    format!("00000000-0000-4000-8000-{n:012}").parse().unwrap()
}

fn id(n: u64) -> Identity {
    Identity {
        class: Class::Wire,
        id: uuid(n),
        instance_path: vec![],
    }
}

fn model() -> SelectionResolution {
    SelectionResolution {
        project: uuid(900),
        revision: SelectionModelRevision("r1".into()),
        authored: (1..=3).map(id).collect(),
        runs: Some(BTreeMap::from([(id(1), (1..=3).map(id).collect())])),
        nets: Some(BTreeMap::from([(uuid(800), (1..=3).map(id).collect())])),
        buses: None,
        proposals: BTreeSet::from(["action".into()]),
        reviews: BTreeSet::from(["action".into()]),
        diagnostics: BTreeSet::from(["finding".into()]),
    }
}

#[test]
fn deleted_focus_drops_without_promotion_substitution_or_undo_resurrection() {
    let mut next = model();
    let subject = Subject::Compound(CompoundSelection::new([id(1), id(2)], Some(id(2))).unwrap());
    next.authored.remove(&id(2));
    next.authored.insert(id(4));
    next.revision = SelectionModelRevision("r2".into());
    let changed = reconcile_selection(&subject, next.project, &next).unwrap();
    assert_eq!(changed.project, next.project);
    assert_eq!(changed.revision, next.revision);
    assert_eq!(changed.dropped, BTreeSet::from([id(2)]));
    assert_eq!(
        changed.subject,
        Subject::Compound(CompoundSelection::new([id(1)], None).unwrap())
    );
    next.authored.insert(id(2));
    next.revision = SelectionModelRevision("undo".into());
    let undo = reconcile_selection(&changed.subject, next.project, &next).unwrap();
    assert_eq!(undo.subject, changed.subject);
    assert_eq!(undo.members, BTreeSet::from([id(1)]));
}

#[test]
fn surviving_derived_identity_rederives_wholly_and_missing_origin_dissolves() {
    let mut next = model();
    next.revision = SelectionModelRevision("r2".into());
    next.authored.insert(id(4));
    let full = BTreeSet::from([id(1), id(3), id(4)]);
    next.runs.as_mut().unwrap().insert(id(1), full.clone());
    next.nets.as_mut().unwrap().insert(uuid(800), full.clone());
    for subject in [Subject::Run(id(1).into()), Subject::GlobalNet(uuid(800))] {
        let changed = reconcile_selection(&subject, next.project, &next).unwrap();
        assert_eq!(changed.subject, subject);
        assert_eq!(changed.members, full);
        assert!(changed.dropped.is_empty());
    }
    next.authored.remove(&id(1));
    assert!(
        reconcile_selection(&Subject::Run(id(1).into()), next.project, &next)
            .unwrap()
            .dissolved
    );
    next.nets.as_mut().unwrap().remove(&uuid(800));
    assert!(
        reconcile_selection(&Subject::GlobalNet(uuid(800)), next.project, &next)
            .unwrap()
            .dissolved
    );
}

#[test]
fn project_replacement_and_artifact_expiration_cannot_leak_selection() {
    let mut next = model();
    assert!(
        reconcile_selection(&Subject::Object(id(1)), uuid(901), &next)
            .unwrap()
            .dissolved
    );
    for subject in [
        Subject::Proposal("action".into()),
        Subject::Review("action".into()),
        Subject::Diagnostic("finding".into()),
    ] {
        assert_eq!(
            reconcile_selection(&subject, next.project, &next)
                .unwrap()
                .subject,
            subject
        );
    }
    next.proposals.clear();
    next.reviews.clear();
    next.diagnostics.clear();
    for subject in [
        Subject::Proposal("action".into()),
        Subject::Review("action".into()),
        Subject::Diagnostic("finding".into()),
    ] {
        assert!(
            reconcile_selection(&subject, next.project, &next)
                .unwrap()
                .dissolved
        );
    }
}

#[test]
fn incomplete_authority_refuses_reconciliation_instead_of_partial_selection() {
    let mut next = model();
    let prior = Subject::Run(id(1).into());
    next.runs.as_mut().unwrap().remove(&id(1));
    assert!(reconcile_selection(&prior, next.project, &next).is_err());
    assert_eq!(prior, Subject::Run(id(1).into()));
}

#[test]
fn unavailable_semantic_authority_does_not_masquerade_as_deleted_identity() {
    let mut next = model();
    let selected = Subject::GlobalNet(uuid(800));
    next.nets = None;
    assert!(reconcile_selection(&selected, next.project, &next).is_err());
    next.nets = Some(BTreeMap::new());
    assert!(
        reconcile_selection(&selected, next.project, &next)
            .unwrap()
            .dissolved
    );
    assert!(reconcile_selection(&Subject::Bus(uuid(700)), next.project, &next).is_err());
}
