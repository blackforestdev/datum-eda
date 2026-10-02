//! Native engine-to-shared-adapter identity, projection and refusal proofs.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::selection_resolution::{
    NativeSelectionResolution as Native, SelectionAuthority, SelectionResolutionError,
};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity,
    SelectionSubject as Subject,
};
use eda_engine::{
    api::native_write::{self, electrical, schematic_sheets},
    ir::geometry::Point,
    schematic::{Bus, BusEntry, SheetDefinition, SheetInstance},
    substrate::*,
};
use fixture::*;
use std::collections::BTreeSet;
fn identity(class: Class, n: u128, path: &[Id]) -> Identity {
    Identity {
        class,
        id: id(n),
        instance_path: path.into(),
    }
}
fn declaration(n: u128, reps: BTreeSet<ElectricalOccurrence>) -> ElectricalIdentityRecord {
    ElectricalIdentityRecord {
        id: id(n),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "same".into(),
            scalar_nets: BTreeSet::new(),
            representations: reps,
            retired: false,
        },
    }
}
fn occurrence(class: &str, n: u128, path: &[Id]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id(n),
        instance_path: path.into(),
    }
}
fn create(f: &mut Fixture, record: ElectricalIdentityRecord) {
    let write =
        electrical::build_create_electrical_identity(&f.model, provenance(), record).unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
}

#[test]
fn native_board_run_keeps_complete_owned_projection_and_separate_faults() {
    let mut f = Fixture::new("native-board-adapter");
    f.line(id(1), Point::new(0, 0), Point::new(10, 0));
    f.line(id(2), Point::new(10, 0), Point::new(20, 0));
    f.line(id(3), Point::new(100, 0), Point::new(110, 0));
    let foreign = eda_engine::board::Net::new(id(906), "foreign", Id::nil());
    let write =
        native_write::board_routing::build_place_board_net(&f.model, provenance(), &foreign)
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let cross = eda_engine::board::Track::straight(
        id(4),
        foreign.uuid,
        Point::new(5, -10),
        Point::new(5, 10),
        2,
        1,
    );
    f.commit([Operation::CreateBoardTrack {
        track_id: cross.uuid,
        track: serde_json::to_value(cross).unwrap(),
    }]);
    let files = f.source_bytes();
    let resolver = Native::capture(&f.model).unwrap();
    let subject = resolver
        .acquire_run(identity(Class::Track, 1, &[]), None)
        .unwrap();
    let projection = resolver.resolve(&subject).unwrap();
    assert_eq!(projection.project, f.model.project.project_id);
    assert_eq!(projection.revision, f.model.model_revision);
    assert_eq!(
        projection.members,
        BTreeSet::from([
            identity(Class::Track, 1, &[]),
            identity(Class::Track, 2, &[])
        ])
    );
    let run = projection.board_run.unwrap();
    assert_eq!(run.board_net, f.net);
    assert_eq!(run.contact_evidence.unwrap().len(), 1);
    assert!(!projection.source_basis.is_empty());
    Fixture::assert_bytes(&files);
    assert!(matches!(
        resolver.acquire_run(identity(Class::Pad, 1, &[]), None),
        Err(SelectionResolutionError::InvalidSubject(_))
    ));
    assert!(matches!(
        resolver.acquire_run(identity(Class::Track, 1, &[id(999)]), None),
        Err(SelectionResolutionError::Engine(
            ElectricalQueryFailure::InvalidOccurrence { .. }
        ))
    ));
    let reopened = Native::capture(&f.reopen()).unwrap();
    assert_eq!(reopened.members(&subject).unwrap(), projection.members);
}

#[test]
fn native_bus_owned_entries_are_projection_not_extra_subject_members_or_scalar_wires() {
    let mut f = Fixture::new("native-bus-adapter");
    let buses = [(1, 0, 10), (2, 10, 20), (3, 100, 110), (4, 20, 30)];
    let mut operations = vec![];
    for (n, a, b) in buses {
        let spine = Bus {
            uuid: id(n),
            name: "same".into(),
            members: vec![],
            segments: vec![Point::new(a, 0), Point::new(b, 0)],
        };
        operations.push(Operation::CreateSchematicBus {
            sheet_id: f.sheet,
            bus_id: id(n),
            bus: serde_json::to_value(spine).unwrap(),
        });
    }
    let entry = BusEntry {
        uuid: id(5),
        bus: id(2),
        wire: None,
        position: Point::new(15, 0),
        size: Point::new(5, 5),
    };
    operations.push(Operation::CreateSchematicBusEntry {
        sheet_id: f.sheet,
        bus_entry_id: entry.uuid,
        bus_entry: serde_json::to_value(entry).unwrap(),
    });
    f.commit(operations);
    create(
        &mut f,
        declaration(800, [1, 2, 3].map(|n| occurrence("buses", n, &[])).into()),
    );
    create(
        &mut f,
        declaration(801, BTreeSet::from([occurrence("buses", 4, &[])])),
    );
    let files = f.source_bytes();
    let native = Native::capture(&f.model).unwrap();
    let subject = native
        .acquire_run(identity(Class::BusSection, 1, &[]), None)
        .unwrap();
    let projection = native.resolve(&subject).unwrap();
    assert_eq!(
        projection.members,
        BTreeSet::from([
            identity(Class::BusSection, 1, &[]),
            identity(Class::BusSection, 2, &[])
        ])
    );
    let run = projection.bus_run.unwrap();
    assert_eq!(run.contacts.len(), 1);
    assert_eq!(run.contacts.first().unwrap().foreign_bus, id(801));
    assert!(
        run.membership
            .owned
            .contains(&occurrence("bus_entries", 5, &[]))
    );
    assert_eq!(projection.electrical.unwrap().owned, run.membership.owned);
    assert_eq!(native.members(&Subject::Bus(id(800))).unwrap().len(), 3);
    Fixture::assert_bytes(&files);
}

#[test]
fn native_shared_identity_preserves_repeated_sheet_paths_and_source_domains() {
    let mut f = Fixture::new("native-source-occurrences");
    let text = eda_engine::schematic::SchematicText {
        uuid: id(1),
        text: "same".into(),
        position: Point::zero(),
        rotation: 0,
    };
    let board_text:eda_engine::board::BoardText=serde_json::from_value(serde_json::json!({"uuid":id(2),"text":"same","position":{"x":0,"y":0},"rotation":0,"layer":3})).unwrap();
    f.commit([
        Operation::CreateSchematicText {
            sheet_id: f.sheet,
            text_id: text.uuid,
            text: serde_json::to_value(text).unwrap(),
        },
        Operation::CreateBoardText {
            text_id: board_text.uuid,
            text: serde_json::to_value(board_text).unwrap(),
        },
    ]);
    let definition = SheetDefinition {
        uuid: id(700),
        root_sheet: f.sheet,
        name: "same".into(),
    };
    let write = schematic_sheets::build_create_schematic_definition(
        &f.model,
        provenance(),
        f.schematic,
        definition.uuid,
        "definitions/source.json",
        serde_json::to_value(definition).unwrap(),
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    for n in [701, 702] {
        let instance = SheetInstance {
            uuid: id(n),
            definition: id(700),
            parent_sheet: None,
            name: "same".into(),
            position: Point::zero(),
            ports: vec![],
        };
        let write = schematic_sheets::build_create_schematic_sheet_instance(
            &f.model,
            provenance(),
            f.schematic,
            instance.uuid,
            serde_json::to_value(instance).unwrap(),
        )
        .unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let native = Native::capture(&f.model).unwrap();
    let a = identity(Class::SchematicText, 1, &[id(701)]);
    let b = identity(Class::SchematicText, 1, &[id(702)]);
    assert_ne!(a, b);
    assert!(native.authored_contains(&a));
    assert!(native.authored_contains(&b));
    assert!(!native.authored_contains(&identity(Class::SchematicText, 1, &[])));
    assert!(native.authored_contains(&identity(Class::BoardText, 2, &[])));
    assert!(!native.authored_contains(&identity(Class::SchematicText, 2, &[])));
    let subject = Subject::Compound(
        datum_gui_protocol::CompoundSelection::new([a.clone(), b.clone()], Some(a.clone()))
            .unwrap(),
    );
    assert_eq!(native.members(&subject).unwrap(), BTreeSet::from([a, b]));
    let outline = Identity {
        class: Class::BoardOutline,
        id: f.board,
        instance_path: vec![],
    };
    assert!(native.authored_contains(&outline));
    assert_eq!(
        serde_json::from_slice::<Subject>(&serde_json::to_vec(&subject).unwrap()).unwrap(),
        subject
    );
}

#[test]
fn native_binding_failure_is_not_empty_membership_or_deletion() {
    let mut f = Fixture::new("native-binding-refusal");
    let track = f.line(id(1), Point::new(0, 0), Point::new(10, 0));
    let logical = ElectricalIdentityRecord {
        id: id(800),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Net {
            anchor: occurrence("tracks", 1, &[]),
            anchor_reason: NetAnchorReason::Authored,
            retired: false,
            predecessors: BTreeSet::new(),
        },
    };
    create(&mut f, logical);
    let relation = ElectricalIdentityRecord {
        id: id(801),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: id(800),
            board_net: Some(RevisionedRef {
                object_id: f.net,
                object_revision: f.model.objects[&f.net].object_revision,
            }),
            intent: NetRelationshipIntent::BoardOnly,
            evidence: vec![],
        },
    };
    create(&mut f, relation);
    let good = Native::capture(&f.model).unwrap();
    let subject = Subject::GlobalNet(id(800));
    assert_eq!(
        good.members(&subject).unwrap(),
        BTreeSet::from([identity(Class::Track, 1, &[])])
    );
    let mut relation = f.model.electrical_identities[&id(801)].clone();
    if let ElectricalIdentity::NetRelationship { intent, .. } = &mut relation.identity {
        *intent = NetRelationshipIntent::Pending;
    }
    let write =
        electrical::build_set_electrical_identity(&f.model, provenance(), relation).unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let pending = Native::capture(&f.model).unwrap();
    assert!(pending.derived_exists(&subject).unwrap());
    assert!(matches!(
        pending.members(&subject),
        Err(SelectionResolutionError::Engine(
            ElectricalQueryFailure::UnavailableBinding {
                status: NetCorrespondenceStatus::Pending,
                ..
            }
        ))
    ));
    assert_eq!(
        good.members(&subject).unwrap(),
        BTreeSet::from([identity(Class::Track, 1, &[])])
    );
    assert_eq!(track.uuid, id(1));
    f.undo();
    assert_eq!(
        Native::capture(&f.reopen())
            .unwrap()
            .members(&subject)
            .unwrap(),
        good.members(&subject).unwrap()
    );
}
