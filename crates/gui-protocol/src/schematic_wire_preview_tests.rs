//! Preview/action/commit equivalence for shared wire handles.
use super::*;
use eda_engine::api::native_write::{
    schematic_wire_edit::{WireEditBasis, WireHandle, WireRouting},
    schematic_wire_edit_adapter::WireEditParams,
};
use eda_engine::ir::geometry::Point;
use std::collections::BTreeSet;

#[test]
fn contact_span_and_vertex_previews_match_canonical_replacements_and_locks() {
    for (vertex, child_sheet) in [(false, false), (true, false), (false, true), (true, true)] {
        let fixture = isolated_fixture();
        let root = &fixture.0;
        let mut model = eda_engine::substrate::ProjectResolver::new(root)
            .resolve()
            .unwrap();
        let mut sheet_id = SchematicAuthoring::from_model(&model)
            .unwrap()
            .sheet
            .unwrap();
        let mut instance_path = vec![];
        if child_sheet {
            use eda_engine::api::native_write::{
                schematic_child_sheet::ChildSheetRequest,
                schematic_delete_proposal::{
                    self as proposal, NativeProposalAction as Action, NativeProposalRequest,
                },
            };
            let child = ChildSheetRequest {
                proposal_id: uuid::Uuid::new_v4(),
                parent: sheet_id,
                instance_path: vec![],
                sheet: uuid::Uuid::new_v4(),
                definition: uuid::Uuid::new_v4(),
                instance: uuid::Uuid::new_v4(),
                name: "Wire editing child".into(),
                position: Point::new(0, 0),
            };
            for action in [
                Action::CreateChildSheet(child.clone()),
                Action::AcceptApply {
                    proposal_id: child.proposal_id,
                },
            ] {
                let request = NativeProposalRequest {
                    project_root: root.clone(),
                    expected_model_revision: model.model_revision.0.clone(),
                    expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
                    action,
                };
                proposal::execute(&mut model, &request).unwrap();
            }
            sheet_id = child.sheet;
            instance_path = vec![child.instance];
        }
        let prepared = (find_native_write_verb("datum.schematic.draw_wire").unwrap().build)(
            &NativeWriteContext { model: &model, project_root: root },
            WriteProvenance::new("test", eda_engine::substrate::CommitSource::Test, "preview source"),
            json!({"sheet":sheet_id,"instance_path":instance_path,"points":[{"x":70_000_000,"y":80_000_000},{"x":80_000_000,"y":80_000_000},{"x":80_000_000,"y":90_000_000}]}),
        ).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        let authoring = SchematicAuthoring::from_model(&model).unwrap();
        let sheet = &authoring.source.as_ref().unwrap().sheets[&sheet_id];
        let wire = sheet
            .wires
            .values()
            .find(|w| w.from == Point::new(70_000_000, 80_000_000))
            .unwrap();
        let handle = if vertex {
            WireHandle::Vertex { point: wire.to }
        } else {
            WireHandle::Segment {
                from: wire.from,
                to: wire.to,
            }
        };
        let at = PointNm {
            x: 82_000_000,
            y: 82_000_000,
        };
        let edit = WireEditParams {
            action: eda_engine::api::native_write::schematic_wire_edit::WireEditAction::Move,
            handle,
            routing: WireRouting::Free,
            edit_seed: uuid::Uuid::new_v4(),
        };
        let plan = WireEditBasis::capture(sheet, wire.uuid, edit.edit_seed)
            .unwrap()
            .plan_move(&handle, Point::new(at.x, at.y), edit.routing)
            .unwrap();
        assert!(!plan.wires.is_empty());
        let mut documents = crate::native_schematic_scene::project_native_schematic_documents(
            &model,
            root,
            &crate::WorkspaceLayout::board_schematic(),
        )
        .unwrap()
        .unwrap();
        let key = documents
            .rows
            .iter()
            .find(|r| r.key.source_sheet == sheet_id && r.key.instance_path == instance_path)
            .unwrap()
            .key
            .clone();
        documents.prepare_scene(&key, root, &model).unwrap();
        documents.panes.insert(crate::PaneId(1), key);
        let base = documents.scene_for_pane(crate::PaneId(1)).unwrap().clone();
        let identity = AuthoredSelectionIdentity {
            class: AuthoredSelectionClass::Wire,
            id: wire.uuid,
            instance_path: base
                .schematic_document
                .as_ref()
                .unwrap()
                .instance_path
                .clone(),
        };
        let snap = crate::AuthoringSnapState {
            enabled: false,
            grid_nm: 100_000,
        };
        let mut preview = NativeMovePreview::connectivity_with_wire_edit(
            &base,
            &authoring,
            &identity,
            PointNm {
                x: wire.from.x,
                y: wire.from.y,
            },
            snap,
            root.clone(),
            Some(edit),
        )
        .unwrap();
        let before: Vec<_> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let scene = preview.scene_at(at).unwrap();
        assert_eq!(scene.schematic_document, base.schematic_document);
        for (id, wire) in &plan.wires {
            let graphic = scene
                .board_graphics
                .iter()
                .find(|g| {
                    g.object_kind == "schematic_wire" && g.source_object_uuid == id.to_string()
                })
                .unwrap();
            assert_eq!(
                graphic.path,
                vec![
                    PointNm {
                        x: wire.from.x,
                        y: wire.from.y
                    },
                    PointNm {
                        x: wire.to.x,
                        y: wire.to.y
                    }
                ]
            );
        }
        preview.set_wire_locked_members(BTreeSet::from([wire.uuid]));
        assert!(preview.scene_at(at).is_err());
        assert!(preview.action(at).is_err());
        preview.set_wire_locked_members(BTreeSet::new());
        let NativeEditAction::MoveConnectivity {
            object,
            x_nm,
            y_nm,
            wire_edit,
        } = preview.action(at).unwrap()
        else {
            panic!("wire action")
        };
        assert_eq!(object, wire.uuid);
        assert_eq!(wire_edit, Some(edit));
        for (path, bytes) in before {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
        if child_sheet {
            assert!((find_native_write_verb("datum.schematic.move").unwrap().build)(
                &NativeWriteContext{model:&model,project_root:root},
                WriteProvenance::new("test",eda_engine::substrate::CommitSource::Test,"wrong root occurrence"),
                json!({"object":object,"instance_path":[],"x_nm":x_nm,"y_nm":y_nm,"wire_edit":wire_edit}),
            ).is_err());
        }
        let prepared = (find_native_write_verb("datum.schematic.move").unwrap().build)(
            &NativeWriteContext{model:&model,project_root:root},
            WriteProvenance::new("test",eda_engine::substrate::CommitSource::Test,"preview commit equivalence"),
            json!({"object":object,"instance_path":identity.instance_path,"x_nm":x_nm,"y_nm":y_nm,"wire_edit":wire_edit}),
        ).unwrap();
        let count = model.journal.len();
        commit_prepared(&mut model, root, prepared).unwrap();
        assert_eq!(model.journal.len(), count + 1);
        let cold = eda_engine::substrate::ProjectResolver::new(root)
            .resolve()
            .unwrap();
        let committed = SchematicAuthoring::from_model(&cold).unwrap();
        let mut expected = authoring.source.as_ref().unwrap().clone();
        expected.sheets.insert(sheet_id, plan.apply_to_sheet(sheet));
        assert_eq!(committed.source.as_ref().unwrap(), &expected);
    }
}

#[test]
fn selected_wire_context_retains_exact_source_at_an_overlapping_bend() {
    use crate::{
        AuthoredSelectionClass as Class, MarkingMenuState, SelectionSubject,
        selection_resolution::NativeSelectionResolution, selection_scene::SceneSelectionBindings,
    };
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    let write=(find_native_write_verb("datum.schematic.draw_wire").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:root},
        WriteProvenance::new("test",eda_engine::substrate::CommitSource::Test,"wire context source"),
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":70_000_000,"y":80_000_000},{"x":80_000_000,"y":80_000_000},{"x":80_000_000,"y":90_000_000}]}),
    ).unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    let source = SchematicAuthoring::from_model(&model).unwrap();
    let wires: Vec<_> = source.source.as_ref().unwrap().sheets[&sheet]
        .wires
        .values()
        .filter(|w| w.from.x == 70_000_000 || w.from.x == 80_000_000)
        .map(|w| w.uuid)
        .collect();
    assert_eq!(wires.len(), 2);
    let scene = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let bindings = SceneSelectionBindings::capture(&scene, &authority);
    let entries: Vec<_> = bindings
        .iter()
        .filter(|(_, b)| b.acquired.class == Class::Wire && wires.contains(&b.acquired.id))
        .collect();
    assert_eq!(entries.len(), 2);
    let mut menu = crate::selection_menu::SelectionMenu::capture(
        &authority,
        entries
            .iter()
            .map(|(_, b)| (b.acquired.clone(), b.projected.clone()))
            .collect(),
        PointNm {
            x: 80_000_000,
            y: 80_000_000,
        },
        crate::load_fixture_workspace_state().ui.filters,
    );
    menu.schematic = true;
    menu.source_scene = scene.scene_id.clone();
    menu.bind_presentation(&scene, &bindings, &authority);
    let mut marking = MarkingMenuState {
        menu_key: "schematic.selection".into(),
        target_object_id: Some(entries[0].0.to_owned()),
        anchor_x_px: 0,
        anchor_y_px: 0,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: Some(menu),
        submenu_path: vec![],
    };
    assert_eq!(marking.schematic_context_key(), "schematic.selection");
    marking.selection.as_mut().unwrap().inspection_subject =
        Some(SelectionSubject::Object(entries[0].1.acquired.clone()));
    assert_eq!(marking.schematic_context_key(), "schematic.wire");
    assert_eq!(
        marking.schematic_context_target(),
        Some(&entries[0].1.acquired)
    );
    // The real menu must admit the shared Delete consumer using this captured
    // wire's source and the engine's existing deletion capability.
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = source;
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
        native_write_verbs: ["datum.schematic.delete_wire".into()].into_iter().collect(),
    });
    marking.menu_key = "schematic.wire".into();
    let inventory = crate::gui_menu_model::default_gui_menu_model().unwrap();
    let delete = inventory.marking_menus["schematic.wire"]
        .overflow
        .iter()
        .find(|item| item.label == "Delete")
        .unwrap();
    assert_eq!(
        marking.schematic_action_unavailable_reason(delete, &state),
        None
    );
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    assert!(
        marking
            .schematic_action_unavailable_reason(delete, &state)
            .is_some()
    );
    state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.schematic.delete_wire".into());
    let mut stale = state.clone();
    stale.scene.source_revision = "stale".into();
    assert!(
        marking
            .schematic_action_unavailable_reason(delete, &stale)
            .is_some()
    );
    state.ui.filters.show_authored = !state.ui.filters.show_authored;
    assert!(
        marking
            .schematic_action_unavailable_reason(delete, &state)
            .is_some()
    );
    marking.target_object_id = Some(entries[1].0.to_owned());
    assert_eq!(
        marking.schematic_context_key(),
        "schematic.selection",
        "a selected ID cannot substitute a different captured presentation key"
    );
}
