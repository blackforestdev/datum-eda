use datum_gui_protocol::selection_resolution::{
    SelectionModelRevision, SelectionResolution, SelectionResolutionError,
};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity, CompoundSelection,
    SelectionSubject as Subject, SelectionSubjectError,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

fn id(n: u128) -> Identity {
    Identity {
        class: Class::Wire,
        id: Uuid::from_u128(n),
    }
}

fn resolution() -> SelectionResolution {
    SelectionResolution {
        project: Uuid::from_u128(900),
        revision: SelectionModelRevision("r1".into()),
        authored: (1..=257).map(id).collect(),
        runs: Some(BTreeMap::from([(id(1), (1..=257).map(id).collect())])),
        nets: Some(BTreeMap::from([(
            Uuid::from_u128(800),
            (1..=257).map(id).collect(),
        )])),
        buses: None,
        proposals: BTreeSet::new(),
        reviews: BTreeSet::new(),
        diagnostics: BTreeSet::new(),
    }
}

#[test]
fn nine_kinds_round_trip_without_collapsing_run_bus_or_review() {
    let bus_origin = Identity {
        class: Class::BusSection,
        ..id(1)
    };
    let values = [
        Subject::None,
        Subject::Object(bus_origin),
        Subject::Compound(CompoundSelection::new([id(1), id(2)], Some(id(2))).unwrap()),
        Subject::Run(bus_origin),
        Subject::GlobalNet(Uuid::from_u128(800)),
        Subject::Bus(Uuid::from_u128(700)),
        Subject::Proposal("same-action".into()),
        Subject::Review("same-action".into()),
        Subject::Diagnostic("finding".into()),
    ];
    let kinds: BTreeSet<_> = values.iter().map(Subject::kind).collect();
    assert_eq!(kinds.len(), 9);
    for value in values {
        let encoded = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<Subject>(&encoded).unwrap(), value);
        value.validate().unwrap();
    }
}

#[test]
fn compound_is_canonical_and_rejects_focus_outside_membership() {
    let a = CompoundSelection::new([id(2), id(1), id(2)], Some(id(1))).unwrap();
    let b = CompoundSelection::new([id(1), id(2)], Some(id(1))).unwrap();
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
    assert_eq!(
        CompoundSelection::new([], None),
        Err(SelectionSubjectError::EmptyCompound)
    );
    assert_eq!(
        CompoundSelection::new([id(1)], Some(id(2))),
        Err(SelectionSubjectError::FocusOutsideMembership)
    );
    // Payload deserialization never grants authority: resolution validates too.
    let invalid = Subject::Compound(CompoundSelection {
        members: BTreeSet::from([id(1)]),
        focus: Some(id(2)),
    });
    assert_eq!(
        resolution().members(&invalid),
        Err(SelectionResolutionError::InvalidSubject(
            SelectionSubjectError::FocusOutsideMembership
        ))
    );
}

#[test]
fn full_derived_resolution_is_independent_of_transport_or_view_subset() {
    let model = resolution();
    for subject in [
        Subject::Run(id(1)),
        Subject::GlobalNet(Uuid::from_u128(800)),
    ] {
        let full = model.members(&subject).unwrap();
        let visible_subset = BTreeSet::from([id(1), id(257)]);
        assert_eq!(full.len(), 257);
        assert!(visible_subset.is_subset(&full));
        assert!(full.contains(&id(128)));
        assert_eq!(model.members(&subject).unwrap(), full);
        assert_eq!(
            serde_json::from_slice::<Subject>(&serde_json::to_vec(&subject).unwrap()).unwrap(),
            subject
        );
    }
}

#[test]
fn missing_or_inconsistent_derivation_never_silently_truncates() {
    let mut model = resolution();
    assert_eq!(
        model.members(&Subject::Bus(Uuid::from_u128(700))),
        Err(SelectionResolutionError::MissingDerivedAuthority)
    );
    model.authored.remove(&id(257));
    assert_eq!(
        model.members(&Subject::Run(id(1))),
        Err(SelectionResolutionError::UnresolvedDerivedMember(id(257)))
    );
    let pad = Identity {
        class: Class::Pad,
        ..id(1)
    };
    assert_eq!(
        model.members(&Subject::Run(pad)),
        Err(SelectionResolutionError::InvalidSubject(
            SelectionSubjectError::InvalidRunOrigin
        ))
    );
}
