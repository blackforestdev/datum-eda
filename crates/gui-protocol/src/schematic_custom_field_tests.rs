//! Custom metadata exercises source-preserving drafts and exact native request boundaries.
use super::*;
use crate::{
    AuthoredSelectionClass, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent, PaneId,
};
use eda_engine::{ir::geometry::Point, schematic::SymbolField};

fn draft(state: &mut SchematicAuthoring) -> CustomFieldDraft {
    let sheet = state.sheet.unwrap();
    let symbol = state
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .symbols
        .values_mut()
        .next()
        .unwrap();
    symbol.fields = vec![
        SymbolField {
            uuid: Uuid::new_v4(),
            key: "Reference".into(),
            value: "imported metadata".into(),
            visible: true,
            position: Some(Point::new(17, -23)),
        },
        SymbolField {
            uuid: Uuid::new_v4(),
            key: "Reference".into(),
            value: "separate metadata".into(),
            visible: false,
            position: None,
        },
    ];
    CustomFieldDraft {
        symbol: selected::SymbolEditDraft {
            target: AuthoredSelectionIdentity {
                class: AuthoredSelectionClass::Symbol,
                id: symbol.uuid,
                instance_path: vec![],
            },
            basis: AuthoringViewBasis {
                project: Uuid::new_v4(),
                revision: "custom-fields-test".into(),
                pane: PaneId(1),
                content: PaneContent::Schematic,
                source: sheet,
                document: None,
            },
            tip: None,
            original_reference: symbol.reference.clone(),
            original_value: symbol.value.clone(),
        },
        field_id: Some(symbol.fields[0].uuid),
        original: symbol.clone(),
        remove: false,
    }
}
#[test]
fn custom_metadata_uuid_requests_preserve_duplicate_keys_and_display_metadata() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let mut draft = draft(&mut state);
    let original = draft.original.clone();
    let source = state.source.clone();
    draft.check_source(&state).unwrap();
    assert!(
        draft
            .request("Reference", "imported metadata")
            .unwrap()
            .is_none()
    );
    let (verb, params) = draft
        .request("Reference", "changed metadata")
        .unwrap()
        .unwrap()
        .verb_params();
    assert_eq!(verb, "datum.schematic.set_symbol_field");
    assert_eq!(
        params,
        serde_json::json!({"symbol":original.uuid,"instance_path":[],"field":"custom",
        "field_id":original.fields[0].uuid,"key":"Reference","value":"changed metadata"})
    );
    assert_eq!(draft.original, original);
    draft
        .navigate(CustomFieldControl::Next, "Reference", "imported metadata")
        .unwrap();
    assert_eq!(draft.field_id, Some(original.fields[1].uuid));
    assert_eq!(
        draft.fields(),
        ("Reference".into(), "separate metadata".into())
    );
    draft
        .navigate(CustomFieldControl::Remove, "Reference", "separate metadata")
        .unwrap();
    let (_, params) = draft
        .request("Reference", "separate metadata")
        .unwrap()
        .unwrap()
        .verb_params();
    assert_eq!(
        params,
        serde_json::json!({"symbol":original.uuid,"instance_path":[],"field":"custom",
        "field_id":original.fields[1].uuid,"clear":true})
    );
    state.editing = Some(FieldsEditDraft::Custom(draft));
    assert!(!state.editing.as_ref().unwrap().text_editable());
    state.cancel();
    assert_eq!(state.source, source);
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[test]
fn custom_metadata_navigation_and_source_guards_keep_unsaved_text_local() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let mut draft = draft(&mut state);
    let before = draft.clone();
    for control in [
        CustomFieldControl::Previous,
        CustomFieldControl::Next,
        CustomFieldControl::New,
        CustomFieldControl::Remove,
        CustomFieldControl::Symbol,
    ] {
        assert!(
            draft
                .navigate(control, "unsaved", "imported metadata")
                .is_err()
        );
        assert_eq!(draft, before);
    }
    draft
        .navigate(CustomFieldControl::New, "Reference", "imported metadata")
        .unwrap();
    assert_eq!(draft.fields(), (String::new(), String::new()));
    for key in ["", "  ", "line\nbreak"] {
        assert!(draft.request(key, "value").is_err());
    }
    assert!(draft.request("key", "line\nbreak").is_err());
    let (_, params) = draft.request(" key ", "").unwrap().unwrap().verb_params();
    assert_eq!(params["key"], " key ", "do not normalize authored text");
    for key in ["field_id", "visible", "position", "reference", "clear"] {
        assert!(params.get(key).is_none());
    }
    assert_eq!(params["value"], "");
    state
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&draft.symbol.basis.source)
        .unwrap()
        .symbols
        .get_mut(&draft.symbol.target.id)
        .unwrap()
        .fields[0]
        .visible = false;
    assert!(draft.check_source(&state).is_err());
    state.reference = " key ".into();
    state.value.clear();
    assert!(FieldsEditDraft::Custom(draft).request(&state).is_err());
}

#[test]
fn custom_metadata_ambiguous_uuid_is_refused_instead_of_selecting_first() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let mut draft = draft(&mut state);
    draft.original.fields[1].uuid = draft.original.fields[0].uuid;
    state
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&draft.symbol.basis.source)
        .unwrap()
        .symbols
        .insert(draft.original.uuid, draft.original.clone());
    assert!(
        draft
            .check_source(&state)
            .unwrap_err()
            .contains("ambiguous")
    );
}
