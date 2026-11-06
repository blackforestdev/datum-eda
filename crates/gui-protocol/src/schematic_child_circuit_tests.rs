//! Real saved hierarchy verifies qualified circuit previews and admission isolation.
use super::*;
use crate::{AuthoringViewBasis, PaneContent, PaneId};
use eda_engine::substrate::ProjectResolver;
#[test]
#[ignore = "requires DATUM_CHILD_CIRCUIT_PROJECT saved run85 project"]
fn child_circuit_preview_requests_and_tool_admission() {
    let root = std::path::PathBuf::from(std::env::var_os("DATUM_CHILD_CIRCUIT_PROJECT").unwrap());
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.project_unopened = false;
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = SchematicAuthoring::from_model(&model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        &model,
        &root,
        &state.ui.layout,
    )
    .unwrap()
    .unwrap();
    let primary = state
        .schematic_documents
        .scene_for_pane(PaneId(1))
        .unwrap()
        .clone();
    state.ui.layout.focused = PaneId(0);
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    state
        .schematic_documents
        .panes
        .insert(PaneId(0), primary.schematic_document.clone().unwrap());
    state.ui.layout.focused = PaneId(1);
    let key = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.name == "First child")
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&key, &root, &model)
        .unwrap();
    state
        .schematic_documents
        .panes
        .insert(PaneId(1), key.clone());
    state.schematic_scene = Some(primary.clone());
    state.supervision.journal.accepted_transaction_tip =
        eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
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
        native_write_verbs: eda_engine::api::native_write::registry::native_write_verbs()
            .iter()
            .map(|v| v.id.to_string())
            .collect(),
    });
    let basis = AuthoringViewBasis::capture_qualified_schematic(&state, PaneId(1)).unwrap();
    assert_eq!(basis.document, Some(key.clone()));
    assert!(AuthoringViewBasis::capture(&state, PaneId(1)).is_err());
    assert!(AuthoringViewBasis::capture_unique_root_schematic(&state, PaneId(1)).is_err());
    let child_command: crate::GuiMenuItem = serde_json::from_value(serde_json::json!({
        "label":"Sheet", "gui_local":"schematic.child_sheet.place", "requires_project":true
    }))
    .unwrap();
    assert_eq!(
        child_command.unavailable_reason(&state),
        None,
        "existing unique nested child creation remains available"
    );
    assert_eq!(
        crate::gui_menu_model::native_edit_unavailable_reason(
            &state,
            "datum.schematic.create_sheet"
        ),
        None,
        "new root document is independent of the focused occurrence"
    );
    for verb in [
        "datum.schematic.place_symbol",
        "datum.schematic.place_text",
        "datum.schematic.draw_wire",
        "datum.schematic.set_symbol_reference",
        "datum.schematic.set_symbol_value",
        "datum.schematic.place_label",
        "datum.schematic.set_symbol_field",
        "datum.schematic.place_port",
        "datum.schematic.create_bus",
        "datum.schematic.place_bus_entry",
        "datum.schematic.place_junction",
        "datum.schematic.place_noconnect",
        "datum.schematic.delete_wire",
        "datum.schematic.delete_junction",
        "datum.schematic.delete_noconnect",
        "datum.schematic.move_symbol",
        "datum.schematic.delete_symbol",
    ] {
        assert_eq!(
            crate::gui_menu_model::native_edit_unavailable_reason(&state, verb),
            None
        );
    }
    let before = state.schematic_authoring.source.clone();
    let base = crate::camera_scene_for_pane(&state, PaneId(1))
        .unwrap()
        .clone();
    let symbol = *state.schematic_authoring.source.as_ref().unwrap().sheets[&key.source_sheet]
        .symbols
        .keys()
        .next()
        .unwrap();
    let id = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::Symbol,
        id: symbol,
        instance_path: key.instance_path.clone(),
    };
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            crate::SelectionSubject::Object(id.clone()),
            &state.scene,
            Some(&base),
            |_| true,
        )
        .unwrap(),
    ));
    assert_eq!(selected::delete_target(&state).unwrap().0, basis);
    assert!(selected::admission(&state, false).is_ok());
    let assignment = assignment::AssignmentEvidence::selected(&state).unwrap();
    assert_eq!(assignment.options.sheet, key.source_sheet);
    assert_eq!(assignment.target.instance_path, key.instance_path);
    let annotated = delete_review::DeleteReview::capture_annotation(&state)
        .unwrap()
        .expect("saved child has references requiring annotation");
    assert_eq!(annotated.basis, basis);
    assert_eq!(
        annotated.annotation.as_ref().unwrap().summary.instance_path,
        key.instance_path
    );
    let mut table = delete_review::DeleteReview::capture_field_table(&state).unwrap();
    table.fields.as_mut().unwrap().rows[0]
        .value
        .push_str(" child field review");
    assert!(table.stage_field_table(&state).unwrap());
    match table.pending.as_ref().unwrap() {
        eda_engine::api::native_write::schematic_delete_proposal::NativeProposalAction::CreateFieldBatch {
            sheet, instance_path, ..
        } => {
            assert_eq!(*sheet, key.source_sheet);
            assert_eq!(*instance_path, key.instance_path);
        }
        _ => panic!("field review must carry the complete qualified child intent"),
    }
    assert_eq!(
        state.schematic_authoring.source, before,
        "reviewing child fields writes no source"
    );
    let prior_selection = state.selection.clone();
    let label = state.schematic_authoring.source.as_ref().unwrap().sheets[&key.source_sheet]
        .labels
        .values()
        .next()
        .unwrap()
        .clone();
    let label_id = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::Label,
        id: label.uuid,
        instance_path: key.instance_path.clone(),
    };
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            crate::SelectionSubject::Object(label_id.clone()),
            &state.scene,
            Some(&base),
            |_| true,
        )
        .unwrap(),
    ));
    assert!(selected::label_admission(&state).is_ok());
    let label_draft = selected::LabelEditDraft {
        target: label_id.clone(),
        basis: basis.clone(),
        tip: None,
        original: label.clone(),
    };
    let (_, params) = label_draft
        .request("REVIEWED_CHILD_LABEL", label.kind)
        .unwrap()
        .unwrap()
        .verb_params();
    assert_eq!(
        params["instance_path"],
        serde_json::json!(key.instance_path)
    );
    assert_ne!(state.schematic_authoring.sheet, Some(key.source_sheet));
    let label_at = PointNm {
        x: label.position.x,
        y: label.position.y,
    };
    let label_move = crate::footprint_move_preview::native::NativeMovePreview::label(
        &base,
        &state.schematic_authoring,
        &label_id,
        label_at,
        state.authoring.snap,
        root.clone(),
    )
    .unwrap();
    assert_eq!(
        label_move
            .scene_at(PointNm {
                x: label_at.x + 1_000_000,
                y: label_at.y + 1_000_000
            })
            .unwrap()
            .schematic_document,
        Some(key.clone())
    );
    let text_key = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.name == "Nested child")
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&text_key, &root, &model)
        .unwrap();
    state
        .schematic_documents
        .panes
        .insert(PaneId(1), text_key.clone());
    let text_scene = crate::camera_scene_for_pane(&state, PaneId(1))
        .unwrap()
        .clone();
    let text = state.schematic_authoring.source.as_ref().unwrap().sheets[&text_key.source_sheet]
        .texts
        .values()
        .next()
        .unwrap()
        .clone();
    let text_id = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::SchematicText,
        id: text.uuid,
        instance_path: text_key.instance_path.clone(),
    };
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            crate::SelectionSubject::Object(text_id.clone()),
            &state.scene,
            Some(&text_scene),
            |_| true,
        )
        .unwrap(),
    ));
    assert!(selected::text_admission(&state).is_ok());
    let text_basis = selected::tool_target(&state, crate::AuthoredSelectionClass::SchematicText)
        .unwrap()
        .0;
    let text_draft = selected::TextEditDraft {
        target: text_id.clone(),
        basis: text_basis,
        tip: None,
        original: text.clone(),
    };
    let (_, params) = text_draft
        .request("Literal child Text edit", 90)
        .unwrap()
        .unwrap()
        .verb_params();
    assert_eq!(params["sheet"], serde_json::json!(text_key.source_sheet));
    assert_eq!(
        params["instance_path"],
        serde_json::json!(text_key.instance_path)
    );
    assert_ne!(state.schematic_authoring.sheet, Some(text_key.source_sheet));
    let text_at = PointNm {
        x: text.position.x,
        y: text.position.y,
    };
    let text_move = crate::footprint_move_preview::native::NativeMovePreview::text(
        &text_scene,
        &state.schematic_authoring,
        &text_id,
        text_at,
        state.authoring.snap,
        root.clone(),
    )
    .unwrap();
    assert_eq!(
        text_move
            .scene_at(PointNm {
                x: text_at.x + 1_000_000,
                y: text_at.y + 1_000_000
            })
            .unwrap()
            .schematic_document,
        Some(text_key.clone())
    );
    state
        .schematic_documents
        .panes
        .insert(PaneId(1), key.clone());
    state.selection = prior_selection;
    let delete_command: crate::GuiMenuItem = serde_json::from_value(serde_json::json!({
        "label":"Delete", "gui_local":"schematic.selection.delete", "requires_project":true
    }))
    .unwrap();
    assert_eq!(delete_command.unavailable_reason(&state), None);
    let review = delete_review::DeleteReview::capture(&state).unwrap();
    assert!(review.symbol_delete());
    assert_eq!(review.basis, basis);
    assert_eq!(review.preview_scene().schematic_document, Some(key.clone()));
    use crate::footprint_move_preview::native::NativeMovePreview;
    let at = PointNm {
        x: 12_000_000,
        y: 25_000_000,
    };
    let mut moving = NativeMovePreview::symbol(
        &base,
        &state.schematic_authoring,
        &id,
        at,
        state.authoring.snap,
        root.clone(),
    )
    .unwrap();
    assert!(
        NativeMovePreview::symbol(
            &primary,
            &state.schematic_authoring,
            &id,
            at,
            state.authoring.snap,
            root.clone()
        )
        .is_err()
    );
    moving
        .transform_symbol(PlacementTransform::Rotate, at)
        .unwrap();
    moving
        .transform_symbol(PlacementTransform::FlipX, at)
        .unwrap();
    assert_eq!(moving.instance_path(), key.instance_path);
    let moved = moving.scene_at(at).unwrap();
    assert_eq!(moved.schematic_document, Some(key.clone()));
    state.schematic_authoring.circuit_basis = Some(basis.clone());
    state.schematic_authoring.circuit_preview = Some(moved.clone());
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(1)),
        Some(&moved)
    );
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(0)),
        Some(&primary)
    );
    state.schematic_authoring.cancel();
    assert_eq!(crate::camera_scene_for_pane(&state, PaneId(1)), Some(&base));
    assert_eq!(state.schematic_authoring.source, before);
    let choice = state
        .schematic_authoring
        .choices
        .iter()
        .position(|c| c.refusal.is_none())
        .unwrap();
    let draft = &mut state.schematic_authoring;
    draft.circuit_basis = Some(basis.clone());
    draft.base = Some(base.clone());
    draft.select(choice);
    draft.picker = false;
    draft.position = Some(PointNm {
        x: 12_000_000,
        y: 25_000_000,
    });
    let ghost = draft.preview_scene(&base, &root).unwrap();
    assert_eq!(ghost.schematic_document, Some(key.clone()));
    draft.circuit_preview = Some(ghost.clone());
    assert_eq!(draft.placement_sheet(), Some(key.source_sheet));
    assert_eq!(draft.placement_path(), key.instance_path);
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(1)),
        Some(&ghost)
    );
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(0)),
        Some(&primary)
    );
    assert_eq!(state.schematic_scene.as_ref(), Some(&primary));
    let draft = &mut state.schematic_authoring;
    draft.chosen = None;
    let mut label = LabelDraft::new(LabelKind::Hierarchical);
    label.name = "CHILD_SIGNAL".into();
    draft.label = Some(label);
    let label_ghost = draft.preview_scene(&base, &root).unwrap();
    assert_eq!(label_ghost.schematic_document, Some(key.clone()));
    assert_eq!(draft.source, before);
    let mut text = LabelDraft::text(90);
    text.name = "Literal ${CHILD} note".into();
    draft.label = Some(text);
    let at = draft.position.unwrap();
    let (verb, params) = draft.named_point_request(at).unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.place_text");
    assert_eq!(params["sheet"], serde_json::json!(key.source_sheet));
    assert_eq!(
        params["instance_path"],
        serde_json::json!(key.instance_path)
    );
    assert_eq!(params["value"], "Literal ${CHILD} note");
    assert_eq!(params["rotation_deg"], 90);
    let text_ghost = draft.preview_scene(&base, &root).unwrap();
    assert_eq!(text_ghost.schematic_document, Some(key.clone()));
    assert_eq!(
        draft.source, before,
        "Text preview leaves canonical source intact"
    );
    draft.label = None;
    let draft = &mut state.schematic_authoring;
    draft.wire = true;
    draft.vertices = vec![
        PointNm { x: 0, y: 0 },
        PointNm {
            x: 10_000_000,
            y: 0,
        },
    ];
    let (verb, params) = draft.polyline_request().unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.draw_wire");
    assert_eq!(params["sheet"], key.source_sheet.to_string());
    assert_eq!(
        params["instance_path"],
        serde_json::json!(key.instance_path)
    );
    draft.bus = true;
    draft.reference = "CHILD_BUS".into();
    draft.value = "SIGNAL_A,SIGNAL_B".into();
    let (verb, params) = draft.polyline_request().unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.create_bus");
    assert_eq!(params["sheet"], key.source_sheet.to_string());
    assert_eq!(
        params["instance_path"],
        serde_json::json!(key.instance_path),
        "Bus creation retains the same captured child occurrence as Wire creation"
    );
    draft.cancel();
    assert!(draft.circuit_basis.is_none());
    assert!(draft.circuit_preview.is_none());
    assert_eq!(draft.source, before);
    assert_eq!(crate::camera_scene_for_pane(&state, PaneId(1)), Some(&base));
    state.scene.source_revision.push_str("-stale");
    assert!(AuthoringViewBasis::capture_qualified_schematic(&state, PaneId(1)).is_err());
    state.scene.source_revision = model.model_revision.0.clone();
    let mut repeated = state.schematic_documents.row(&key).unwrap().clone();
    repeated.key.instance_path.push(Uuid::new_v4());
    state.schematic_documents.rows.push(repeated);
    assert!(AuthoringViewBasis::capture_qualified_schematic(&state, PaneId(1)).is_err());
    assert!(
        crate::gui_menu_model::native_edit_unavailable_reason(
            &state,
            "datum.schematic.place_symbol"
        )
        .is_some()
    );
    assert!(
        crate::gui_menu_model::native_edit_unavailable_reason(&state, "datum.schematic.place_text")
            .is_some()
    );
    // Parent Port uses a unique root, even when another root is the primary scene.
    let root_key = state
        .schematic_documents
        .rows
        .iter()
        .find(|row| row.name == "Library access" && row.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&root_key, &root, &model)
        .unwrap();
    state
        .schematic_documents
        .panes
        .insert(PaneId(1), root_key.clone());
    assert_ne!(state.schematic_authoring.sheet, Some(root_key.source_sheet));
    let basis = AuthoringViewBasis::capture_unique_root_schematic(&state, PaneId(1)).unwrap();
    let base = crate::camera_scene_for_pane(&state, PaneId(1))
        .unwrap()
        .clone();
    let draft = &mut state.schematic_authoring;
    draft.circuit_basis = Some(basis.clone());
    draft.chosen = None;
    let mut port = LabelDraft::port(PortDirection::Input);
    port.name = "ROOT_INTERFACE".into();
    let at = PointNm {
        x: 10_000_000,
        y: 10_000_000,
    };
    let (verb, params) = port
        .placement_request(draft.placement_sheet().unwrap(), at)
        .unwrap()
        .verb_params();
    assert_eq!(verb, "datum.schematic.place_port");
    assert_eq!(params["sheet"], serde_json::json!(root_key.source_sheet));
    assert_eq!(params["instance_path"], serde_json::json!([]));
    draft.label = Some(port);
    draft.picker = false;
    draft.position = Some(at);
    assert_eq!(
        draft
            .preview_scene(&base, &root)
            .unwrap()
            .schematic_document,
        Some(root_key.clone())
    );
    draft.cancel();
    // Text and Bus share the same root basis and never replace the primary pane.
    let draft = &mut state.schematic_authoring;
    draft.circuit_basis = Some(basis.clone());
    let mut text = LabelDraft::text(90);
    text.name = "Nonprimary root note".into();
    let (_, params) = text
        .placement_request(draft.placement_sheet().unwrap(), at)
        .unwrap()
        .verb_params();
    assert_eq!(params["sheet"], serde_json::json!(root_key.source_sheet));
    draft.label = Some(text);
    draft.position = Some(at);
    draft.picker = false;
    let ghost = draft.preview_scene(&base, &root).unwrap();
    assert_eq!(ghost.schematic_document, Some(root_key.clone()));
    draft.circuit_preview = Some(ghost.clone());
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(1)),
        Some(&ghost)
    );
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(0)),
        Some(&primary)
    );
    let draft = &mut state.schematic_authoring;
    draft.label = None;
    draft.wire = true;
    draft.bus = true;
    draft.reference = "ROOT_BUS".into();
    draft.value = "SIGNAL_A,SIGNAL_B".into();
    draft.vertices = vec![PointNm { x: 0, y: 0 }, at];
    let (verb, params) = draft.polyline_request().unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.create_bus");
    assert_eq!(params["sheet"], serde_json::json!(root_key.source_sheet));
    assert_eq!(params["instance_path"], serde_json::json!([]));
    assert_eq!(
        draft
            .preview_scene(&base, &root)
            .unwrap()
            .schematic_document,
        Some(root_key.clone())
    );
    draft.cancel();
    assert_eq!(state.schematic_authoring.source, before);
    assert_eq!(crate::camera_scene_for_pane(&state, PaneId(1)), Some(&base));
    // Real existing root objects exercise the selected-edit source/revision guards.
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let sheet = &state.schematic_authoring.source.as_ref().unwrap().sheets[&root_key.source_sheet];
    let targets = [
        (
            crate::AuthoredSelectionClass::SchematicText,
            *sheet.texts.keys().next().unwrap(),
        ),
        (
            crate::AuthoredSelectionClass::Wire,
            *sheet.wires.keys().next().unwrap(),
        ),
    ];
    for (class, id) in targets {
        let id = crate::AuthoredSelectionIdentity {
            class,
            id,
            instance_path: vec![],
        };
        state.selection = crate::SelectionTarget::Native(Box::new(
            crate::selection_view::build_selection_view(
                &model,
                &authority,
                crate::SelectionSubject::Object(id.clone()),
                &state.scene,
                Some(&base),
                |_| true,
            )
            .unwrap(),
        ));
        assert_eq!(selected::tool_target(&state, class).unwrap().1, id);
        assert_eq!(
            selected::target(&state).unwrap().1,
            id,
            "shared acquisition retains the focused qualified source"
        );
        if class == crate::AuthoredSelectionClass::SchematicText {
            assert!(selected::text_admission(&state).is_ok());
        }
        state.scene.source_revision.push_str("-stale");
        assert!(selected::tool_target(&state, class).is_err());
        state.scene.source_revision = model.model_revision.0.clone();
    }
    let mut repeated_root = state.schematic_documents.row(&root_key).unwrap().clone();
    repeated_root.key.instance_path.push(Uuid::new_v4());
    state.schematic_documents.rows.push(repeated_root);
    assert!(AuthoringViewBasis::capture_unique_root_schematic(&state, PaneId(1)).is_err());
}
