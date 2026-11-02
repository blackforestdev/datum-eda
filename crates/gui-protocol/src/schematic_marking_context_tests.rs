//! Real native label context: prior selection is not Rename authority.
use super::*;
use crate::selection_resolution::NativeSelectionResolution;
use crate::selection_scene::SceneSelectionBindings;
use crate::{MarkingMenuState, PaneId, SelectionSubject, SelectionTarget};
use eda_engine::api::native_write::{
    commit_prepared, context::WriteProvenance, schematic_connectivity::build_create_schematic_label,
};
use eda_engine::schematic::{LabelKind, NetLabel};
use eda_engine::substrate::CommitSource;

#[test]
fn typed_schematic_context_uses_clicked_label_and_refuses_stale_or_ambiguous_sources() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    let labels: Vec<_> = (0..2)
        .map(|index| NetLabel {
            uuid: Uuid::new_v4(),
            name: format!("SIGNAL_{index}"),
            kind: LabelKind::Local,
            position: eda_engine::ir::geometry::Point::new(
                40_000_000 + index * 10_000_000,
                40_000_000,
            ),
        })
        .collect();
    for label in &labels {
        let write = build_create_schematic_label(
            &model,
            WriteProvenance::new("test", CommitSource::Test, "typed context"),
            sheet,
            label,
        )
        .unwrap();
        commit_prepared(&mut model, root, write).unwrap();
    }
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
    state.schematic_scene = state.schematic_documents.scene_for_pane(PaneId(1)).cloned();
    state.ui.layout.focused = PaneId(1);
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
        native_write_verbs: ["datum.schematic.rename_label".into()]
            .into_iter()
            .collect(),
    });
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let scene = state.schematic_scene.as_ref().unwrap();
    let bindings = SceneSelectionBindings::capture(scene, &authority);
    let (key, binding) = bindings
        .iter()
        .find(|(_, b)| b.acquired.id == labels[0].uuid)
        .unwrap();
    let clicked = binding.acquired.clone();
    let mut selection = crate::selection_menu::SelectionMenu::capture(
        &authority,
        vec![(clicked.clone(), binding.projected.clone())],
        crate::PointNm {
            x: labels[0].position.x,
            y: labels[0].position.y,
        },
        state.ui.filters.clone(),
    );
    selection.schematic = true;
    selection.source_scene = scene.scene_id.clone();
    selection.bind_presentation(scene, &bindings, &authority);
    let mut marking = MarkingMenuState {
        menu_key: "schematic.selection".into(),
        target_object_id: Some(key.into()),
        anchor_x_px: 0,
        anchor_y_px: 0,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: Some(selection),
        submenu_path: vec![],
    };
    marking.menu_key = marking.schematic_context_key().into();
    assert_eq!(marking.menu_key, "schematic.label");
    let model_menu = crate::gui_menu_model::default_gui_menu_model().unwrap();
    let rename = &model_menu.marking_menus["schematic.label"].cardinal["N"];
    assert!(
        marking
            .schematic_action_unavailable_reason(rename, &state)
            .is_none(),
        "unselected label is actionable"
    );
    let other = bindings
        .iter()
        .find(|(_, b)| b.acquired.id == labels[1].uuid)
        .unwrap()
        .1
        .acquired
        .clone();
    state.selection = SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            &model,
            &authority,
            SelectionSubject::Object(other.clone()),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap(),
    ));
    assert!(
        marking
            .schematic_action_unavailable_reason(rename, &state)
            .is_none()
    );
    assert_eq!(marking.schematic_context_target(), Some(&clicked));
    assert_ne!(marking.schematic_context_target(), Some(&other));
    state.ui.marking_menu = Some(marking.clone());
    state.ui.focus = crate::ApplicationFocus::Overlay;
    let nodes = crate::gui_menu_model::accessibility::menu_accessibility_nodes(model_menu, &state);
    assert!(nodes.iter().any(|n| n.name == "Rename" && n.available));
    assert!(
        nodes
            .iter()
            .any(|n| n.name == "Highlight Net" && !n.available)
    );
    for (class, expected) in [
        (crate::AuthoredSelectionClass::Symbol, "schematic.symbol"),
        (crate::AuthoredSelectionClass::Wire, "schematic.wire"),
        (crate::AuthoredSelectionClass::Label, "schematic.label"),
    ] {
        let mut typed = marking.clone();
        let menu = typed.selection.as_mut().unwrap();
        let mut id = clicked.clone();
        id.class = class;
        menu.items = vec![crate::selection_menu::Item {
            label: "source".into(),
            command: crate::selection_menu::Command::Object(id.clone()),
            unavailable: None,
        }];
        menu.preview_keys = [(
            id,
            [typed.target_object_id.clone().unwrap()]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();
        assert_eq!(typed.schematic_context_key(), expected);
    }
    let unsupported = &model_menu.marking_menus["schematic.label"].cardinal["W"];
    assert!(
        marking
            .schematic_action_unavailable_reason(unsupported, &state)
            .is_some()
    );
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    assert!(
        marking
            .schematic_action_unavailable_reason(rename, &state)
            .is_some()
    );
    state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.schematic.rename_label".into());
    for field in ["revision", "scene", "key"] {
        let mut stale = marking.clone();
        match field {
            "revision" => stale.selection.as_mut().unwrap().revision.push_str("stale"),
            "scene" => stale
                .selection
                .as_mut()
                .unwrap()
                .source_scene
                .push_str("foreign"),
            _ => stale.target_object_id = Some("unrelated".into()),
        }
        assert!(
            stale
                .schematic_action_unavailable_reason(rename, &state)
                .is_some(),
            "{field}"
        );
    }
    let mut overlap = marking.clone();
    overlap
        .selection
        .as_mut()
        .unwrap()
        .items
        .push(crate::selection_menu::Item {
            label: "other".into(),
            command: crate::selection_menu::Command::Object(other),
            unavailable: None,
        });
    assert_eq!(overlap.schematic_context_key(), "schematic.selection");
    assert!(
        overlap
            .schematic_action_unavailable_reason(rename, &state)
            .is_some()
    );
    let mut blank = marking.clone();
    blank.target_object_id = None;
    blank.selection.as_mut().unwrap().items.clear();
    assert_eq!(blank.schematic_context_key(), "schematic.empty");
    state.ui.layout.focused = PaneId(0);
    assert!(
        marking
            .schematic_action_unavailable_reason(rename, &state)
            .is_some()
    );
}
