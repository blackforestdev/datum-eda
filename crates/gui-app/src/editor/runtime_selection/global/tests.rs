use super::*;
use crate::terminal_accessibility_platform::selection_fixture::{self as fixture, Fixture};
use datum_gui_protocol::{
    CompoundSelection, SelectionSubject, selection_resolution::SelectionAuthority,
};
use eda_engine::{board::PlacedPad, ir::geometry::Point, substrate::Operation};

#[test]
fn global_authored_selection_retains_standalone_pad_complete_projection_and_reopen() {
    let mut f = Fixture::new("global-standalone-pad");
    let track = f.line(fixture::id(1), Point::new(0, 0), Point::new(10000, 0));
    let pad: PlacedPad = serde_json::from_value(serde_json::json!({
        "uuid":fixture::id(2),"package":fixture::Id::nil(),"name":"P","net":f.net,
        "position":{"x":20000,"y":0},"layer":1
    }))
    .unwrap();
    f.commit([Operation::CreateBoardPad {
        pad_id: pad.uuid,
        pad: serde_json::to_value(&pad).unwrap(),
    }]);
    let files = f.source_bytes();
    let expected = BTreeSet::from([
        AuthoredSelectionIdentity {
            class: C::Pad,
            id: pad.uuid,
            instance_path: vec![],
        },
        AuthoredSelectionIdentity {
            class: C::Track,
            id: track.uuid,
            instance_path: vec![],
        },
        AuthoredSelectionIdentity {
            class: C::BoardOutline,
            id: f.board,
            instance_path: vec![],
        },
    ]);
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let actual = candidates(&authority, false, Some(f.sheet)).unwrap();
    assert_eq!(actual, expected);
    assert!(
        candidates(&authority, true, Some(f.sheet))
            .unwrap()
            .is_empty()
    );
    let subject = SelectionSubject::Compound(CompoundSelection::new(actual, None).unwrap());
    assert_eq!(authority.members(&subject).unwrap(), expected);
    let reopened = NativeSelectionResolution::capture(&f.reopen()).unwrap();
    assert_eq!(
        candidates(&reopened, false, Some(f.sheet)).unwrap(),
        expected
    );
    assert_eq!(reopened.members(&subject).unwrap(), expected);
    Fixture::assert_bytes(&files);
}
