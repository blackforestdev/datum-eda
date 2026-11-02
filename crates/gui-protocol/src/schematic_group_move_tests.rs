//! Real shared preview: complete source set, explicit pickup, immutable cancel, exact payload.
use super::*;
use crate::footprint_move_preview::native::{
    NativeMovePreview,
    group::{SymbolGroupPreview, subject_contains, symbol_members},
};
use crate::{
    AuthoredSelectionClass as C, AuthoredSelectionIdentity as Id, CompoundSelection,
    SelectionSubject,
};
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_symbol_move::SymbolMoveSetBasis,
};
use serde_json::json;

#[test]
fn group_symbol_preview_preserves_full_set_fields_wire_pickup_and_cancel_source() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let sheet = authoring.sheet.unwrap();
    let library = authoring
        .choices
        .iter()
        .find(|c| c.name == "Datum single op amp symbol")
        .unwrap()
        .id;
    for (verb, params) in [
        (
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":library,"reference":"U3","value":"group","x_nm":40_000_000,"y_nm":40_000_000}),
        ),
        (
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":library,"reference":"U4","value":"group","x_nm":60_000_000,"y_nm":40_500_000}),
        ),
        (
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"points":[{"x":41_500_000,"y":40_000_000},{"x":58_500_000,"y":40_000_000}]}),
        ),
    ] {
        let edit = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: root,
            },
            WriteProvenance::new(
                "test",
                eda_engine::substrate::CommitSource::Test,
                "group preview",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, edit).unwrap();
    }
    let original = SchematicAuthoring::from_model(&model).unwrap();
    let owner = original.source.as_ref().unwrap().sheets[&sheet]
        .symbols
        .values()
        .find(|s| s.reference == "U3")
        .unwrap()
        .uuid;
    for params in [
        json!({"symbol":owner,"instance_path":[],"field":"custom","key":"Note","value":"group field"}),
    ] {
        let edit = (find_native_write_verb("datum.schematic.set_symbol_field")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &model,
                project_root: root,
            },
            WriteProvenance::new(
                "test",
                eda_engine::substrate::CommitSource::Test,
                "group field",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, edit).unwrap();
    }
    let field = SchematicAuthoring::from_model(&model)
        .unwrap()
        .source
        .unwrap()
        .sheets[&sheet]
        .symbols[&owner]
        .fields[0]
        .uuid;
    let edit=(find_native_write_verb("datum.schematic.set_symbol_field").unwrap().build)(&NativeWriteContext{model:&model,project_root:root},WriteProvenance::new("test",eda_engine::substrate::CommitSource::Test,"group field position"),json!({"symbol":owner,"instance_path":[],"field":"custom","field_id":field,"visible":true,"position":{"x":42_000_000,"y":44_000_000}})).unwrap();
    commit_prepared(&mut model, root, edit).unwrap();
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let ids: Vec<_> = source.sheets[&sheet]
        .symbols
        .values()
        .filter(|s| ["U3", "U4"].contains(&s.reference.as_str()))
        .map(|s| Id {
            class: C::Symbol,
            id: s.uuid,
            instance_path: vec![],
        })
        .collect();
    assert_eq!(ids.len(), 2);
    let subject = SelectionSubject::Compound(
        CompoundSelection::new(ids.clone(), Some(ids[1].clone())).unwrap(),
    );
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let basis = SymbolMoveSetBasis::capture(
        authority.snapshot(),
        &ids.iter().map(|id| id.id).collect::<Vec<_>>(),
        &[],
    )
    .unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    assert_group_menu(&model, root, &subject, &ids[0], reference_for_menu());
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let snap = crate::AuthoringSnapState {
        enabled: true,
        grid_nm: 1_000_000,
    };
    let reference = PointNm {
        x: 45_000_000,
        y: 50_000_000,
    };
    let mut preview = NativeMovePreview::Symbols(
        SymbolGroupPreview::begin(
            &base,
            &authoring,
            &subject,
            basis,
            reference,
            snap,
            root.clone(),
        )
        .unwrap(),
    );
    let at = preview
        .destination(
            PointNm {
                x: 49_200_000,
                y: 54_100_000,
            },
            snap,
        )
        .unwrap();
    assert_eq!(
        at,
        PointNm {
            x: 49_000_000,
            y: 54_000_000
        }
    );
    assert!(preview.transform_symbol(Rotate, at).unwrap());
    let scene = preview.scene_at(at).unwrap();
    let wire = source.sheets[&sheet]
        .wires
        .values()
        .find(|w| w.from.x == 41_500_000 && w.to.x == 58_500_000)
        .unwrap()
        .uuid;
    let owned: std::collections::BTreeSet<_> = ids
        .iter()
        .map(|id| id.id.to_string())
        .chain([wire.to_string()])
        .collect();
    let mut covered = std::collections::BTreeSet::new();
    for before in base
        .board_graphics
        .iter()
        .filter(|g| owned.contains(&g.source_object_uuid))
    {
        let after = scene
            .board_graphics
            .iter()
            .find(|g| g.object_id == before.object_id)
            .unwrap();
        let expected: Vec<_> = before
            .path
            .iter()
            .map(|p| PointNm {
                x: at.x - (p.y - reference.y),
                y: at.y + (p.x - reference.x),
            })
            .collect();
        assert_eq!(after.path, expected, "{}", before.object_id);
        covered.insert(before.source_object_uuid.clone());
    }
    assert_eq!(
        covered, owned,
        "both native symbol bodies and connected wire are projected"
    );
    let field_before = base
        .board_texts
        .iter()
        .find(|t| {
            t.text == "group field"
                && t.source_object_uuid.as_deref() == Some(owner.to_string().as_str())
        })
        .unwrap();
    let field_after = scene
        .board_texts
        .iter()
        .find(|t| t.object_id == field_before.object_id)
        .unwrap();
    assert_eq!(
        field_after.position,
        PointNm {
            x: 55_000_000,
            y: 51_000_000
        }
    );
    let request = preview.group_request(at).unwrap();
    let params = serde_json::to_value(&request).unwrap();
    assert_eq!(params["reference"], json!({"x":45_000_000,"y":50_000_000}));
    assert_eq!(params["rotation_deg"], 90);
    assert_eq!(params["symbols"].as_array().unwrap().len(), 2);
    assert_eq!(params["x_nm"], at.x);
    let refocused = SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap());
    assert!(
        preview.matches_subject(&refocused),
        "focus is never transform authority"
    );
    assert!(!preview.matches_subject(&SelectionSubject::Object(ids[0].clone())));
    assert!(subject_contains(&subject, &ids[1]));
    drop(preview);
    assert_eq!(authoring.source.as_ref().unwrap(), source);
    for (path, bytes) in bytes {
        assert_eq!(
            std::fs::read(path).unwrap(),
            bytes,
            "preview/cancel never writes"
        );
    }
}

#[test]
fn group_symbol_admission_refuses_mixed_or_cross_occurrence_without_subset() {
    let first = Id {
        class: C::Symbol,
        id: Uuid::new_v4(),
        instance_path: vec![],
    };
    for second in [
        Id {
            class: C::Wire,
            id: Uuid::new_v4(),
            instance_path: vec![],
        },
        Id {
            class: C::Symbol,
            id: Uuid::new_v4(),
            instance_path: vec![Uuid::new_v4()],
        },
    ] {
        let subject = SelectionSubject::Compound(
            CompoundSelection::new([first.clone(), second], None).unwrap(),
        );
        assert!(symbol_members(&subject).is_err());
        assert!(
            subject_contains(&subject, &first),
            "pickup preserves the subject so admission refuses the whole operation"
        );
    }
}

fn reference_for_menu() -> PointNm {
    PointNm {
        x: 45_125_000,
        y: 50_250_000,
    }
}
fn assert_group_menu(
    model: &eda_engine::substrate::DesignModel,
    root: &std::path::Path,
    subject: &SelectionSubject,
    clicked: &Id,
    pickup: PointNm,
) {
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = SchematicAuthoring::from_model(model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        model,
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
            project_root: root.into(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: root.join("board/board.json"),
        native_write_verbs: ["datum.schematic.move_symbol".into()].into_iter().collect(),
    });
    let authority = crate::selection_resolution::NativeSelectionResolution::capture(model).unwrap();
    let scene = state.schematic_scene.as_ref().unwrap();
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, &authority);
    let (key, binding) = bindings
        .iter()
        .find(|(_, b)| &b.acquired == clicked)
        .unwrap();
    let mut menu = crate::selection_menu::SelectionMenu::capture(
        &authority,
        vec![(clicked.clone(), binding.projected.clone())],
        pickup,
        state.ui.filters.clone(),
    );
    menu.schematic = true;
    menu.source_scene = scene.scene_id.clone();
    menu.bind_presentation(scene, &bindings, &authority);
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            model,
            &authority,
            subject.clone(),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap(),
    ));
    let marking = crate::MarkingMenuState {
        menu_key: "schematic.symbol".into(),
        target_object_id: Some(key.into()),
        anchor_x_px: 999,
        anchor_y_px: 888,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: Some(menu),
        submenu_path: vec![],
    };
    let menus = crate::gui_menu_model::default_gui_menu_model().unwrap();
    let item = &menus.marking_menus["schematic.symbol"].cardinal["S"];
    assert!(
        marking
            .schematic_action_unavailable_reason(item, &state)
            .is_none(),
        "existing Move slot must be actionable for a selected member"
    );
    assert_eq!(
        marking.selection.as_ref().unwrap().pickup_world,
        pickup,
        "capture preserves exact world pickup independently of menu pixel anchor"
    );
    if let crate::SelectionTarget::Native(view) = &mut state.selection {
        let other = view
            .members
            .iter()
            .find(|id| *id != clicked)
            .unwrap()
            .clone();
        view.locked.insert(other);
    }
    assert!(
        marking
            .schematic_action_unavailable_reason(item, &state)
            .unwrap()
            .contains("locked"),
        "locked nonclicked member disables the whole Move action"
    );
    if let crate::SelectionTarget::Native(view) = &mut state.selection {
        view.locked.clear();
    }
    let original_subject = subject.clone();
    if let crate::SelectionTarget::Native(view) = &mut state.selection {
        if let SelectionSubject::Compound(compound) = &mut view.subject {
            compound.members.insert(Id {
                class: C::Wire,
                id: Uuid::new_v4(),
                instance_path: vec![],
            });
        }
    }
    assert!(
        marking
            .schematic_action_unavailable_reason(item, &state)
            .unwrap()
            .contains("only Symbols"),
        "no silent type-filtered subset"
    );
    if let crate::SelectionTarget::Native(view) = &mut state.selection {
        view.subject = original_subject;
    }
    state.schematic_authoring.delete_review =
        Some(crate::schematic_authoring::delete_review::DeleteReview::capture(&state).unwrap());
    assert_eq!(
        marking
            .schematic_action_unavailable_reason(item, &state)
            .as_deref(),
        Some("Accept or dismiss the deletion proposal first")
    );
    state.schematic_authoring.delete_review = None;
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    assert!(
        marking
            .schematic_action_unavailable_reason(item, &state)
            .is_some(),
        "missing canonical operation remains visibly unavailable"
    );
}
