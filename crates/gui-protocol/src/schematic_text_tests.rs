//! Real source-backed standalone annotation, shared form/Move, and nonmutation proof.
use super::*;
use crate::footprint_move_preview::native::NativeMovePreview;
use crate::schematic_authoring::selected::{NamedPointEditDraft, TextEditDraft};
use crate::{AuthoredSelectionClass as C, AuthoredSelectionIdentity as Id, SelectionSubject};
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
};

#[test]
fn standalone_text_literal_form_preview_move_and_source_guards_share_one_owner() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    let literal = "  Check supply R/X/Y  ";
    let request = SchematicEditRequest::Text {
        sheet,
        text: None,
        instance_path: vec![],
        value: Some(literal.into()),
        x_nm: Some(40_000_000),
        y_nm: Some(42_000_000),
        rotation_deg: Some(0),
    };
    let (verb, params) = request.verb_params();
    assert_eq!(verb, "datum.schematic.place_text");
    assert_eq!(params["value"], literal);
    let prepared = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model: &model,
            project_root: root,
        },
        WriteProvenance::new(
            "test",
            eda_engine::substrate::CommitSource::Test,
            "standalone text GUI",
        ),
        params,
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = SchematicAuthoring::from_model(&model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        &model,
        root,
        &state.ui.layout,
    )
    .unwrap()
    .unwrap();
    state.schematic_scene = state
        .schematic_documents
        .scene_for_pane(crate::PaneId(1))
        .cloned();
    state.ui.layout.focused = crate::PaneId(1);
    state.backing = Some(crate::WorkspaceBacking {
        request: crate::LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: root.join("board/board.json"),
        native_write_verbs: [
            "datum.schematic.place_text".into(),
            "datum.schematic.delete_text".into(),
        ]
        .into_iter()
        .collect(),
    });
    let original = state.schematic_authoring.source.as_ref().unwrap().sheets[&sheet]
        .texts
        .values()
        .find(|t| t.text == literal)
        .unwrap()
        .clone();
    let id = Id {
        class: C::SchematicText,
        id: original.uuid,
        instance_path: vec![],
    };
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let base = state.schematic_scene.as_ref().unwrap().clone();
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            SelectionSubject::Object(id.clone()),
            &state.scene,
            Some(&base),
            |_| true,
        )
        .unwrap(),
    ));
    assert!(selected::text_admission(&state).is_ok());
    assert!(selected::admission(&state, false).is_ok());
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(&base, &authority);
    assert!(
        bindings.iter().any(|(_, binding)| binding.acquired == id),
        "real text glyph owns standalone subject"
    );
    let basis = crate::AuthoringViewBasis::capture(&state, crate::PaneId(1)).unwrap();
    let edit = TextEditDraft {
        target: id.clone(),
        basis,
        tip: None,
        original: original.clone(),
    };
    let named = NamedPointEditDraft::Text(edit.clone());
    let mut form = named.form();
    assert_eq!(form.name, literal);
    assert!(named.request(&form).unwrap().is_none());
    form.name = "  Revised note  ".into();
    let params = named.request(&form).unwrap().unwrap().verb_params().1;
    assert_eq!(params["value"], "  Revised note  ");
    assert!(params.get("x_nm").is_none());
    assert!(params.get("rotation_deg").is_none());
    let source = state.schematic_authoring.source.as_ref().unwrap().clone();
    let mut sheet_copy = source.sheets[&sheet].clone();
    form.name.clear();
    assert!(named.request(&form).is_err());
    named.preview(&form, &mut sheet_copy);
    assert_eq!(
        sheet_copy, source.sheets[&sheet],
        "invalid intermediate content retains original ink"
    );
    sheet_copy.texts.get_mut(&id.id).unwrap().position.x += 1;
    assert!(!named.source_matches(&sheet_copy));
    let pin = source.sheets[&sheet]
        .symbols
        .values()
        .flat_map(|symbol| symbol.pins.iter())
        .next()
        .unwrap()
        .uuid;
    let mut annotation = state.schematic_authoring.clone();
    assert!(annotation.pin_point(&pin.to_string()).is_some());
    annotation.label = Some(LabelDraft::text(0));
    assert!(
        annotation.pin_point(&pin.to_string()).is_none(),
        "a note uses shared grid snap, not electrical pin attraction"
    );
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let snap = crate::AuthoringSnapState {
        enabled: true,
        grid_nm: 1_000_000,
    };
    let mut preview = NativeMovePreview::text(
        &base,
        &state.schematic_authoring,
        &id,
        PointNm {
            x: 41_000_000,
            y: 43_000_000,
        },
        snap,
        root.clone(),
    )
    .unwrap();
    let at = preview
        .destination(
            PointNm {
                x: 44_100_000,
                y: 47_200_000,
            },
            snap,
        )
        .unwrap();
    assert_eq!(
        at,
        PointNm {
            x: 43_000_000,
            y: 46_000_000
        },
        "pickup offset retained"
    );
    assert!(preview.transform_symbol(Rotate, at).unwrap());
    assert!(
        preview.transform_symbol(FlipX, at).is_err(),
        "unsupported mirror never mutates pose"
    );
    let scene = preview.scene_at(at).unwrap();
    let text = scene
        .board_texts
        .iter()
        .find(|t| t.source_object_uuid.as_deref() == Some(&id.id.to_string()))
        .unwrap();
    assert_eq!(text.text, literal);
    assert_eq!(text.position, at);
    assert_eq!(text.rotation_degrees, 90);
    assert_eq!(text.height_nm, 1_270_000);
    assert!(
        !scene
            .board_graphics
            .iter()
            .any(|g| g.object_kind == "schematic_field_tether"
                && g.source_object_uuid == id.id.to_string())
    );
    assert!(
        scene
            .board_text_geometries
            .iter()
            .any(|g| g.object_id == text.object_id)
    );
    assert!(preview.matches_subject(&SelectionSubject::Object(id.clone())));
    assert!(!preview.matches_subject(&SelectionSubject::Object(Id {
        id: Uuid::new_v4(),
        ..id.clone()
    })));
    let crate::native_edit_client::NativeEditAction::MoveText {
        text: target,
        x_nm,
        y_nm,
        rotation_deg,
        ..
    } = preview.action(at).unwrap()
    else {
        panic!("wrong Move action")
    };
    assert_eq!(
        (target, x_nm, y_nm, rotation_deg),
        (id.id, at.x, at.y, Some(90))
    );
    drop(preview);
    assert_eq!(state.schematic_authoring.source, Some(source));
    for (path, before) in bytes {
        assert_eq!(
            std::fs::read(path).unwrap(),
            before,
            "preview/cancel writes no source"
        );
    }
    assert_eq!(
        selected::DeleteKind::from_class(C::SchematicText)
            .unwrap()
            .verb_and_key(),
        ("datum.schematic.delete_text", "text")
    );
    let symbol = Id {
        class: C::Symbol,
        id: state.schematic_authoring.source.as_ref().unwrap().sheets[&sheet]
            .symbols
            .keys()
            .next()
            .copied()
            .unwrap(),
        instance_path: vec![],
    };
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            SelectionSubject::Compound(crate::CompoundSelection::new([id, symbol], None).unwrap()),
            &state.scene,
            Some(&base),
            |_| true,
        )
        .unwrap(),
    ));
    let review = delete_review::DeleteReview::capture(&state).unwrap();
    assert!(review.targets.iter().any(|target|target.class==eda_engine::api::native_write::schematic_delete_proposal::DeleteClass::SchematicText),"ordinary batch Delete includes Text instead of silently dropping it");
    state.scene.source_revision.push_str("-stale");
    assert!(selected::text_admission(&state).is_err());
}

#[test]
fn standalone_text_configuration_keeps_form_input_and_electrical_snap_separate() {
    let mut authoring = SchematicAuthoring::default();
    authoring.label = Some(LabelDraft::text(0));
    authoring.picker = true;
    authoring.label.as_mut().unwrap().name = "  R X Y  ".into();
    assert!(
        !authoring.transform_placement(Rotate),
        "R in the shared field remains text input"
    );
    let request = authoring
        .label
        .as_ref()
        .unwrap()
        .placement_request(Uuid::nil(), PointNm { x: 2, y: 3 })
        .unwrap();
    assert_eq!(request.verb_params().1["value"], "  R X Y  ");
    authoring.picker = false;
    assert!(authoring.transform_placement(Rotate));
    assert_eq!(authoring.label.as_ref().unwrap().text_rotation, Some(90));
    assert!(!authoring.transform_placement(FlipY));

    for invalid in ["", "  ", "a\nb", "a\tb"] {
        authoring.label.as_mut().unwrap().name = invalid.into();
        assert!(
            authoring
                .label
                .as_ref()
                .unwrap()
                .placement_request(Uuid::nil(), PointNm { x: 0, y: 0 })
                .is_err()
        );
    }
    let target = Id {
        class: C::SchematicText,
        id: Uuid::new_v4(),
        instance_path: vec![],
    };
    let edit = TextEditDraft {
        target: target.clone(),
        basis: crate::AuthoringViewBasis {
            project: Uuid::nil(),
            revision: "r".into(),
            pane: crate::PaneId(1),
            content: crate::PaneContent::Schematic,
            source: Uuid::new_v4(),
            document: None,
        },
        tip: None,
        original: eda_engine::schematic::SchematicText {
            uuid: target.id,
            text: "Imported".into(),
            position: eda_engine::ir::geometry::Point::new(1, 2),
            rotation: 37,
        },
    };
    let params = edit
        .request("Imported revised", 37)
        .unwrap()
        .unwrap()
        .verb_params()
        .1;
    assert!(
        params.get("rotation_deg").is_none(),
        "literal editing preserves an imported angle without rewriting it"
    );
    let mut imported = NamedPointEditDraft::Text(edit).form();
    imported.cycle_kind();
    assert_eq!(imported.text_rotation, Some(90));
    imported.text_rotation = Some(i32::MAX);
    imported.cycle_kind();
    assert!(
        imported.text_rotation.unwrap().rem_euclid(90) == 0,
        "choice arithmetic is bounded before addition"
    );
    authoring.cancel();
    assert!(authoring.label.is_none() && !authoring.active());
}
