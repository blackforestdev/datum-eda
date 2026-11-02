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
        visibility: None,
        placing: false,
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
        CustomFieldControl::Show,
        CustomFieldControl::Hide,
        CustomFieldControl::Place,
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

#[test]
fn custom_field_presentation_requires_explicit_position_and_preserves_hidden_moves() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let original = draft(&mut state);
    let source = state.source.clone();
    let mut hidden = original.clone();
    hidden
        .navigate(CustomFieldControl::Next, "Reference", "imported metadata")
        .unwrap();
    hidden
        .navigate(CustomFieldControl::Show, "Reference", "separate metadata")
        .unwrap();
    assert!(hidden.placing);
    assert!(hidden.request("Reference", "separate metadata").is_err());
    let at = PointNm {
        x: 31_000_000,
        y: 29_000_000,
    };
    let (_, params) = hidden.point_request(at).unwrap().verb_params();
    assert_eq!(
        params,
        serde_json::json!({"symbol":original.original.uuid,"instance_path":[],
        "field":"custom","field_id":original.original.fields[1].uuid,"visible":true,
        "position":{"x":at.x,"y":at.y}})
    );
    state.picker = false;
    let mut canceled_placement = hidden.clone();
    assert!(canceled_placement.return_to_form());
    assert!(!canceled_placement.placing);
    assert_eq!(canceled_placement.visibility, None);
    assert_eq!(canceled_placement.fields(), hidden.fields());
    state.editing = Some(FieldsEditDraft::Custom(hidden));
    assert!(state.active());
    state.cancel();
    assert_eq!(state.source, source);
    assert!(!state.active());

    let mut hiding = original.clone();
    hiding
        .navigate(CustomFieldControl::Hide, "Reference", "imported metadata")
        .unwrap();
    let (_, params) = hiding
        .request("Reference", "imported metadata")
        .unwrap()
        .unwrap()
        .verb_params();
    assert_eq!(params["visible"], false);
    for key in ["position", "key", "value"] {
        assert!(params.get(key).is_none());
    }
    assert_eq!(hiding.original, original.original);
    assert!(
        !hiding.return_to_form(),
        "ordinary Apply refusal retains staged Hide"
    );
    assert_eq!(hiding.visibility, Some(false));

    let mut moving = original.clone();
    moving
        .navigate(CustomFieldControl::Next, "Reference", "imported metadata")
        .unwrap();
    moving
        .navigate(CustomFieldControl::Place, "Reference", "separate metadata")
        .unwrap();
    let (_, params) = moving.point_request(at).unwrap().verb_params();
    assert!(
        params.get("visible").is_none(),
        "moving hidden text must not show it"
    );
    let mut preview_sheet = state.source.as_ref().unwrap().sheets[&state.sheet.unwrap()].clone();
    moving.preview_at(at, &mut preview_sheet).unwrap();
    let preview = &preview_sheet.symbols[&original.original.uuid].fields[1];
    assert!(
        preview.visible,
        "hidden field is legible during manipulation only"
    );
    assert_eq!(preview.position, Some(Point::new(at.x, at.y)));
    assert_eq!(state.source, source);
}

#[test]
fn custom_field_presentation_projects_explicit_owned_text_in_library_and_fallback_scenes() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = SchematicAuthoring::from_model(&model).unwrap();
    let original = draft(&mut state);
    let sheet = state.sheet.unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, &fixture.0)
        .unwrap()
        .unwrap();
    let owner = original.original.uuid;
    let mut source = state.source.clone().unwrap();
    let fields = &mut source
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .symbols
        .get_mut(&owner)
        .unwrap()
        .fields;
    let mut missing = fields[0].clone();
    missing.uuid = Uuid::new_v4();
    missing.position = None;
    let mut empty = fields[0].clone();
    empty.uuid = Uuid::new_v4();
    empty.value.clear();
    fields.extend([missing, empty]);
    let mut stable_key = None;
    for lib_id in [None, Some(state.choices[0].id.to_string())] {
        source
            .sheets
            .get_mut(&sheet)
            .unwrap()
            .symbols
            .get_mut(&owner)
            .unwrap()
            .lib_id = lib_id;
        let scene = state
            .project_source(&source, sheet, &base, &fixture.0)
            .unwrap();
        let texts: Vec<_> = scene
            .board_texts
            .iter()
            .filter(|t| t.text == "imported metadata")
            .collect();
        assert_eq!(
            texts.len(),
            1,
            "only the visible positioned field has glyphs"
        );
        let text = texts[0];
        assert_eq!(text.source_object_uuid, Some(owner.to_string()));
        assert_eq!(text.height_nm, 1_270_000);
        assert_eq!(text.position, PointNm { x: 17, y: -23 });
        assert!(
            !scene
                .board_texts
                .iter()
                .any(|t| t.text == "separate metadata")
        );
        if let Some(key) = &stable_key {
            assert_eq!(&text.object_id, key);
        }
        stable_key = Some(text.object_id.clone());
        let tethers: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.object_kind == "schematic_field_tether")
            .collect();
        assert_eq!(tethers.len(), 1);
        assert_eq!(
            tethers[0].object_id,
            format!("{}:field-tether", text.object_id)
        );
        assert!(tethers[0].schematic_hit_kind().is_none());
        assert_eq!(tethers[0].source_object_uuid, owner.to_string());
    }
}
