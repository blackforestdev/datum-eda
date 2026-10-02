//! Exact native origin bindings and the selection/manufacturing boundary.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity,
    SelectionSubject as Subject,
    selection_resolution::{
        NativeSelectionResolution as Native, PhysicalQueryFailure, SelectionAuthority,
        SelectionResolutionError,
    },
};
use eda_engine::{
    api::native_write::{self, electrical},
    board::{PlacedPad, Track},
    export::{ExportError, render_rs274x_copper_layer},
    ir::geometry::Point,
    substrate::*,
};
use fixture::*;
use std::collections::BTreeSet;

#[test]
fn rational_center_selection_widens_complete_net_while_cam_remains_refused() {
    let mut f = Fixture::new("selection-not-cam-acceptance");
    let mut arc = Track::straight(
        id(1),
        f.net,
        Point::new(0, 0),
        Point::new(3_000_000, 0),
        200_001,
        1,
    );
    arc.midpoint = Some(Point::new(1_000_000, 3_000_000));
    f.commit([Operation::CreateBoardTrack {
        track_id: arc.uuid,
        track: serde_json::to_value(&arc).unwrap(),
    }]);
    f.line(
        id(2),
        Point::new(100_000_000, 0),
        Point::new(110_000_000, 0),
    );
    let origin = ElectricalOccurrence {
        class: "tracks".into(),
        source_id: arc.uuid,
        instance_path: vec![],
    };
    let logical = id(800);
    for record in [
        ElectricalIdentityRecord {
            id: logical,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Net {
                anchor: origin,
                anchor_reason: NetAnchorReason::Authored,
                retired: false,
                predecessors: BTreeSet::new(),
            },
        },
        ElectricalIdentityRecord {
            id: id(801),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: logical,
                board_net: Some(RevisionedRef {
                    object_id: f.net,
                    object_revision: f.model.objects[&f.net].object_revision,
                }),
                intent: NetRelationshipIntent::BoardOnly,
                evidence: vec![],
            },
        },
    ] {
        let write =
            electrical::build_create_electrical_identity(&f.model, provenance(), record).unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let files = f.source_bytes();
    let before = f.model.clone();
    let native = Native::capture(&f.model).unwrap();
    let track_identity = |n| Identity {
        class: Class::Track,
        id: id(n),
        instance_path: vec![],
    };
    let subject = native.acquire_run(track_identity(1), None).unwrap();
    assert_eq!(
        native.members(&subject).unwrap(),
        BTreeSet::from([track_identity(1)])
    );
    let Subject::Run(run) = &subject else {
        panic!("expected Run")
    };
    let widened = native.widen_run(run).unwrap();
    assert_eq!(widened, Subject::GlobalNet(logical));
    assert_ne!(logical, f.net);
    let expected = BTreeSet::from([track_identity(1), track_identity(2)]);
    assert_eq!(native.members(&widened).unwrap(), expected);
    assert!(matches!(
        render_rs274x_copper_layer(1, &[], &[arc], &[], &[]),
        Err(ExportError::DeferredArcManufacturing { .. })
    ));
    assert_eq!(f.model, before);
    Fixture::assert_bytes(&files);
    let reopened = Native::capture(&f.reopen()).unwrap();
    assert_eq!(reopened.widen_run(run).unwrap(), widened);
    assert_eq!(reopened.members(&widened).unwrap(), expected);
}

#[test]
fn rotated_square_inner_witness_certifies_complete_run_without_rounding_pose() {
    let mut f = Fixture::new("selection-rotated-pad-gap");
    let mut arc = Track::straight(
        id(1),
        f.net,
        Point::new(-5_000_000, 0),
        Point::new(5_000_000, 0),
        1,
        1,
    );
    arc.midpoint = Some(Point::new(0, 5_000_000));
    let pad: PlacedPad = serde_json::from_value(serde_json::json!({
        "uuid":id(2),"package":Id::nil(),"name":"P","net":f.net,
        "position":{"x":0,"y":6_000_000},"layer":1,
        "shape":"rect","width":2_000_000,"height":2_000_000,"rotation":45,
        "layer_connection":{"kind":"separate"}
    }))
    .unwrap();
    f.commit([
        Operation::CreateBoardTrack {
            track_id: arc.uuid,
            track: serde_json::to_value(&arc).unwrap(),
        },
        Operation::CreateBoardPad {
            pad_id: pad.uuid,
            pad: serde_json::to_value(&pad).unwrap(),
        },
    ]);
    // Independent positive-contact witness: the authored arc midpoint differs
    // from the pad center by (0,-1mm). In the 45-degree square's frame both
    // squared coordinates are 1mm²/2, strictly below half-width²=1mm².
    // That on-arc point is inside the pad, proving actual occupied contact.
    let delta = arc.midpoint.unwrap().y - pad.position.y;
    let twice_half_width_squared = 2 * i128::from(pad.width / 2).pow(2);
    assert!(i128::from(delta).pow(2) < twice_half_width_squared);
    let files = f.source_bytes();
    let before = f.model.clone();
    let native = Native::capture(&f.model).unwrap();
    let subject = native
        .acquire_run(
            Identity {
                class: Class::Track,
                id: arc.uuid,
                instance_path: vec![],
            },
            None,
        )
        .unwrap();
    assert_eq!(
        native.members(&subject).unwrap(),
        BTreeSet::from([
            Identity {
                class: Class::Track,
                id: arc.uuid,
                instance_path: vec![]
            },
            Identity {
                class: Class::Pad,
                id: pad.uuid,
                instance_path: vec![]
            },
        ])
    );
    assert_eq!(f.model, before);
    Fixture::assert_bytes(&files);
}

#[test]
fn rotated_corner_contact_is_still_unresolved_and_cannot_publish_partial_run() {
    let mut f = Fixture::new("selection-rotated-corner-gap");
    f.line(id(1), Point::new(0, 1_400_000), Point::new(0, 1_400_001));
    let pad: PlacedPad = serde_json::from_value(serde_json::json!({
        "uuid":id(2),"package":Id::nil(),"name":"P","net":f.net,
        "position":{"x":0,"y":0},"layer":1,
        "shape":"rect","width":2_000_000,"height":2_000_000,"rotation":45,
        "layer_connection":{"kind":"separate"}
    }))
    .unwrap();
    f.commit([Operation::CreateBoardPad {
        pad_id: pad.uuid,
        pad: serde_json::to_value(&pad).unwrap(),
    }]);
    // Exact independent interior witness: at (0,1.4mm), both rotated local
    // squared coordinates are 0.98mm² < 1mm². The inner 1mm disk cannot settle
    // it, and the enclosing 2mm disk cannot prove separation.
    assert!(1_400_000_i128.pow(2) < 2 * 1_000_000_i128.pow(2));
    let files = f.source_bytes();
    let native = Native::capture(&f.model).unwrap();
    assert!(matches!(
        native.acquire_run(
            Identity {
                class: Class::Track,
                id: id(1),
                instance_path: vec![],
            },
            None
        ),
        Err(SelectionResolutionError::Engine(
            ElectricalQueryFailure::Physical(PhysicalQueryFailure::UnavailableGeometry { .. })
        ))
    ));
    Fixture::assert_bytes(&files);
    // Refusal honesty passes; positive {Track1, Pad2} is still an open oracle.
}
