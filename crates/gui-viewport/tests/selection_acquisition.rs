use datum_gui_protocol::selection_resolution::{SelectionModelRevision, SelectionResolution};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    SelectionProjectId, SelectionSubject as Subject,
};
use datum_gui_viewport::selection_acquisition::{
    AcquisitionEntry as Entry, AcquisitionMode as Mode, acquire_authored,
};
use std::collections::{BTreeMap, BTreeSet};
fn uuid(n: u64) -> SelectionProjectId {
    format!("00000000-0000-4000-8000-{n:012}").parse().unwrap()
}
fn id(n: u64) -> Id {
    Id {
        class: Class::Wire,
        id: uuid(n),
        instance_path: vec![],
    }
}
fn authority() -> SelectionResolution {
    let full: BTreeSet<_> = (1..=257).map(id).collect();
    SelectionResolution {
        project: uuid(900),
        revision: SelectionModelRevision("r1".into()),
        authored: (1..=258).map(id).collect(),
        runs: Some(BTreeMap::from([(id(1), full.clone())])),
        nets: Some(BTreeMap::from([(uuid(800), full.clone())])),
        buses: Some(BTreeMap::from([(uuid(700), full)])),
        proposals: BTreeSet::new(),
        reviews: BTreeSet::new(),
        diagnostics: BTreeSet::new(),
    }
}
#[test]
fn derived_click_and_mixed_region_refuse_wholly_with_complete_uncapped_membership() {
    let a = authority();
    for subject in [
        Subject::Run(id(1).into()),
        Subject::GlobalNet(uuid(800)),
        Subject::Bus(uuid(700)),
    ] {
        for entry in [Entry::Individual, Entry::Region] {
            for (mode, candidates) in [
                (Mode::Add, BTreeSet::from([id(257), id(258)])),
                (Mode::Remove, BTreeSet::from([id(257), id(258)])),
            ] {
                let r = acquire_authored(&subject, candidates, mode, entry, &a).unwrap();
                assert_eq!(r.subject, subject);
                assert_eq!(r.members, (1..=257).map(id).collect());
                assert!(!r.changed);
                let refusal = r.refusal.unwrap();
                assert_eq!(refusal.mode, mode);
                assert_eq!(refusal.subject_kind, subject.kind());
                assert!(
                    refusal
                        .explanation()
                        .contains("Membership follows connectivity")
                );
            }
        }
    }
}
#[test]
fn derived_noops_preserve_subject_without_refusal_or_focus_conversion() {
    let a = authority();
    for subject in [
        Subject::Run(id(1).into()),
        Subject::GlobalNet(uuid(800)),
        Subject::Bus(uuid(700)),
    ] {
        for entry in [Entry::Individual, Entry::Region] {
            for (mode, candidates) in [
                (Mode::Add, BTreeSet::from([id(257)])),
                (Mode::Remove, BTreeSet::from([id(258)])),
                (Mode::Add, BTreeSet::new()),
                (Mode::Remove, BTreeSet::new()),
            ] {
                let r = acquire_authored(&subject, candidates, mode, entry, &a).unwrap();
                assert_eq!(r.subject, subject);
                assert!(!r.changed);
                assert!(r.refusal.is_none());
                assert_eq!(r.members.len(), 257);
            }
        }
    }
}
#[test]
fn explicit_acquisition_focus_and_global_region_grammar_have_no_arbitrary_promotion() {
    let a = authority();
    let first = acquire_authored(
        &Subject::None,
        BTreeSet::from([id(1)]),
        Mode::Replace,
        Entry::Individual,
        &a,
    )
    .unwrap();
    assert_eq!(first.subject, Subject::Object(id(1)));
    let added = acquire_authored(
        &first.subject,
        BTreeSet::from([id(2)]),
        Mode::Add,
        Entry::Individual,
        &a,
    )
    .unwrap();
    assert_eq!(
        added.subject,
        Subject::Compound(CompoundSelection::new([id(1), id(2)], Some(id(2))).unwrap())
    );
    let removed = acquire_authored(
        &added.subject,
        BTreeSet::from([id(2)]),
        Mode::Remove,
        Entry::Individual,
        &a,
    )
    .unwrap();
    assert_eq!(
        removed.subject,
        Subject::Compound(CompoundSelection::new([id(1)], None).unwrap())
    );
    let region = acquire_authored(
        &first.subject,
        BTreeSet::from([id(2)]),
        Mode::Add,
        Entry::Region,
        &a,
    )
    .unwrap();
    assert_eq!(
        region.subject,
        Subject::Compound(CompoundSelection::new([id(1), id(2)], Some(id(1))).unwrap())
    );
    let global = acquire_authored(
        &first.subject,
        (1..=258).map(id).collect(),
        Mode::Replace,
        Entry::Global,
        &a,
    )
    .unwrap();
    assert_eq!(
        global.subject,
        Subject::Compound(CompoundSelection::new((1..=258).map(id), None).unwrap())
    );
}
#[test]
fn no_op_and_refusal_use_final_revision_authority_and_do_not_write_or_freeze_members() {
    let mut a = authority();
    let subject = Subject::GlobalNet(uuid(800));
    a.revision = SelectionModelRevision("r2".into());
    a.nets
        .as_mut()
        .unwrap()
        .get_mut(&uuid(800))
        .unwrap()
        .insert(id(258));
    let before = a.clone();
    let r = acquire_authored(
        &subject,
        BTreeSet::from([id(258)]),
        Mode::Add,
        Entry::Region,
        &a,
    )
    .unwrap();
    assert!(r.refusal.is_none());
    assert_eq!(r.members.len(), 258);
    assert_eq!(a, before);
    a.nets = None;
    assert!(
        acquire_authored(
            &subject,
            BTreeSet::from([id(258)]),
            Mode::Add,
            Entry::Region,
            &a
        )
        .is_err()
    );
    assert_eq!(subject, Subject::GlobalNet(uuid(800)));
}
#[test]
fn unresolved_candidates_refuse_without_publishing_a_partial_compound() {
    let a = authority();
    let prior = Subject::Object(id(1));
    assert!(
        acquire_authored(
            &prior,
            BTreeSet::from([id(2), id(999)]),
            Mode::Add,
            Entry::Region,
            &a
        )
        .is_err()
    );
    assert_eq!(prior, Subject::Object(id(1)));
}

#[test]
fn replacement_can_leave_an_unavailable_derived_subject_without_weakening_modifiers() {
    let mut a = authority();
    a.nets = None;
    let prior = Subject::GlobalNet(uuid(800));
    let replacement = acquire_authored(
        &prior,
        BTreeSet::from([id(258)]),
        Mode::Replace,
        Entry::Individual,
        &a,
    )
    .unwrap();
    assert_eq!(replacement.subject, Subject::Object(id(258)));
    assert!(
        acquire_authored(
            &prior,
            BTreeSet::from([id(258)]),
            Mode::Add,
            Entry::Individual,
            &a
        )
        .is_err()
    );
}
