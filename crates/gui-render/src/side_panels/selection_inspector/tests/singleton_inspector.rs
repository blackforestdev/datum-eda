use super::*;
use datum_gui_protocol::{CompoundSelection, SelectionSubject};

#[test]
fn unfocused_group_renders_identity_and_existing_class_fields() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let SelectionTarget::Native(previous) = &state.selection else {
        panic!("native fixture")
    };
    let id = previous.members.first().unwrap().clone();
    fixture.select(&mut state, SelectionSubject::Object(id.clone()));
    let SelectionTarget::Native(object) = state.selection.clone() else {
        unreachable!()
    };
    fixture.select(
        &mut state,
        SelectionSubject::Compound(CompoundSelection::new([id], None).unwrap()),
    );
    let SelectionTarget::Native(group) = state.selection.clone() else {
        unreachable!()
    };
    let rect = RectPx {
        x: 100.0,
        y: 0.0,
        width: 900.0,
        height: 2400.0,
    };
    state.selection = SelectionTarget::Native(object.clone());
    let (_, object_text, _) = frame(&state, rect);
    state.selection = SelectionTarget::Native(group.clone());
    let (_, group_text, _) = frame(&state, rect);
    let object_text: Vec<_> = object_text.iter().map(|t| t.text.as_str()).collect();
    let group_text: Vec<_> = group_text.iter().map(|t| t.text.as_str()).collect();
    assert!(group_text.contains(&group.inspector_title().as_str()));
    for field in group.fields() {
        let label = datum_gui_protocol::selection_view::field_label(field);
        assert!(object_text.iter().any(|t| t.contains(&label)), "{label}");
        assert!(group_text.iter().any(|t| t.contains(&label)), "{label}");
    }
    assert_eq!(group.fields(), object.fields());
    assert!(group.focus().is_none());
    assert_eq!(state.selection, SelectionTarget::Native(group));
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
