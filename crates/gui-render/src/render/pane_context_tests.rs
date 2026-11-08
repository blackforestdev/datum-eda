//! Pane-local identity, geometry and unavailable-source regression.
use crate::*;
use datum_gui_protocol::{PaneContent, PaneId, SplitOrientation};

#[test]
fn qualified_child_context_survives_narrow_tools_focus_and_missing_scene() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library = datum_gui_protocol::library_browser::LibraryBrowser::capture(
        std::sync::Arc::new(model.clone()),
    );
    let snapshot = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    state.schematic_documents =
        datum_gui_protocol::schematic_documents::SchematicDocuments::from_source(
            model.project.project_id,
            datum_gui_protocol::selection_resolution::SelectionModelRevision(
                model.model_revision.0.clone(),
            ),
            "Pane regression".into(),
            snapshot.schematic_source().unwrap().clone(),
        )
        .unwrap();
    state
        .schematic_documents
        .bind_initial_panes(&state.ui.layout);
    let root_key = state.schematic_documents.panes[&PaneId(1)].clone();
    state
        .schematic_documents
        .prepare_scene(&root_key, &root, &model)
        .unwrap();
    state.schematic_scene = state.schematic_documents.scene_for_pane(PaneId(1)).cloned();
    let child = state
        .schematic_documents
        .rows
        .iter()
        .find(|row| !row.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    // Short view aliases isolate reservation from long-name clipping. These
    // admitted fixture instances are root occurrences, not parent/child proof;
    // the ordinary App case covers actual nested breadcrumbs. Source keys and
    // prepared scenes remain those of the serialized fixture.
    for row in &mut state.schematic_documents.rows {
        if row.key == child {
            row.name = "Child".into();
            row.source_name = "Child".into();
            row.breadcrumb = vec!["Child".into()];
        } else if row.parent.is_none() {
            row.name = "Root".into();
            row.source_name = "Root".into();
            row.breadcrumb = vec!["Root".into()];
        }
    }
    state.ui.layout.focused = PaneId(0);
    state
        .schematic_documents
        .prepare_scene(&child, &root, &model)
        .unwrap();
    let child_pane = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &child)
        .unwrap();
    let retained = RetainedScene::from_workspace(&state, 1344, 840);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    for focus in [child_pane, PaneId(0), PaneId(1)] {
        state.ui.layout.focused = focus;
        let prepared = PreparedScene::from_workspace(&state, 1344, 840, camera, &retained).unwrap();
        let panes = prepared.layout.viewport_panes(&state.ui.layout);
        assert_eq!(panes.panes.len(), 3);
        for leaf in &panes.panes {
            let pass = prepared
                .surface_passes()
                .iter()
                .find(|pass| pass.pane_id == leaf.id)
                .unwrap();
            assert_eq!(pass.scene_viewport, leaf.rect.scene);
            let title =
                datum_gui_protocol::inspector_context::pane_header(&state, leaf.id, leaf.content)
                    .unwrap();
            if leaf.content == PaneContent::Board {
                assert!(title.0.starts_with("Board:"));
            } else {
                assert_eq!(
                    pass.schematic_document.as_ref(),
                    state.schematic_documents.panes.get(&leaf.id)
                );
            }
        }
        let leaf = panes
            .panes
            .iter()
            .find(|leaf| leaf.id == child_pane)
            .unwrap();
        let reserve = crate::menu_chrome::pane_tools::reserve_for(
            &state,
            child_pane,
            leaf.rect.header,
            focus == child_pane,
        );
        assert_eq!(
            reserve, 0.,
            "the actual narrow child cannot fit context and inline tools"
        );
        let clip = crate::menu_chrome::pane_tools::title_bounds(
            leaf.rect.header,
            reserve,
            focus == child_pane,
        );
        let child_text = prepared
            .text_runs
            .iter()
            .find(|run| {
                run.text == "Child"
                    && run.y >= leaf.rect.header.y
                    && run.y < leaf.rect.header.y + leaf.rect.header.height
                    && run.x >= clip.x
                    && run.x < clip.x + clip.width
            })
            .expect("current occurrence must actually be emitted");
        assert!(
            child_text.x
                + crate::control_text::measure_text_width(
                    "Child",
                    child_text.size,
                    child_text.face
                )
                .unwrap()
                <= clip.x + clip.width,
            "emitted child x={} y={} size={} face={:?}, clip={:?}, leaf={:?}, reserved={}",
            child_text.x,
            child_text.y,
            child_text.size,
            child_text.face,
            clip,
            leaf.rect.header,
            reserve
        );
        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text.starts_with("Source: Child")
                    && run.y >= leaf.rect.header.y
                    && run.y < leaf.rect.header.y + leaf.rect.header.height
                    && run.x >= clip.x
                    && run.x < clip.x + clip.width)
        );
        assert!(
            !prepared.hit_regions.iter().any(|hit| matches!(&hit.target,
            HitTarget::SchematicBreadcrumb { pane, .. } if *pane == child_pane)),
            "the admitted fixture occurrence is a root, so it has no invented ancestor link"
        );
        assert!(
            !prepared.hit_regions.iter().any(|hit| leaf
                .rect
                .header
                .contains(hit.rect.x, hit.rect.y)
                && matches!(hit.target, HitTarget::SetWorkspaceTool(_)))
        );
    }
    state.ui.layout.focused = child_pane;
    state.ui.layout.zoomed = Some(child_pane);
    let wide = PreparedScene::from_workspace(&state, 1344, 840, camera, &retained).unwrap();
    let wide_panes = wide.layout.viewport_panes(&state.ui.layout);
    assert_eq!(wide_panes.panes.len(), 1);
    assert!(
        crate::menu_chrome::pane_tools::reserve_for(
            &state,
            child_pane,
            wide_panes.panes[0].rect.header,
            true
        ) > 0.
    );
    let header = wide_panes.panes[0].rect.header;
    assert_eq!(
        wide.hit_regions
            .iter()
            .filter(|hit| header.contains(hit.rect.x, hit.rect.y)
                && match &hit.target {
                    HitTarget::SetWorkspaceTool(_) => true,
                    HitTarget::MenuItem { menu, .. } => menu == "Place",
                    _ => false,
                })
            .count(),
        5
    );
    state.ui.layout.zoomed = None;
    // A row without its matching scene never borrows the other pane's source.
    let missing_pane = state.ui.layout.split_focused_with(
        PaneContent::Schematic,
        SplitOrientation::Horizontal,
        true,
    );
    let mut missing = child.clone();
    missing.instance_path.push(uuid::Uuid::new_v4());
    let mut row = state.schematic_documents.row(&child).unwrap().clone();
    row.key = missing.clone();
    row.name = "Unavailable child".into();
    state.schematic_documents.rows.push(row);
    state
        .schematic_documents
        .panes
        .insert(missing_pane, missing);
    assert!(datum_gui_protocol::camera_scene_for_pane(&state, missing_pane).is_none());
    assert!(
        datum_gui_protocol::inspector_context::pane_header(
            &state,
            missing_pane,
            PaneContent::Schematic
        )
        .unwrap()
        .0
        .starts_with("Source: unavailable")
    );
    let prepared = PreparedScene::from_workspace(&state, 1344, 840, camera, &retained).unwrap();
    assert!(
        !prepared
            .surface_passes()
            .iter()
            .any(|pass| pass.pane_id == missing_pane)
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text == "Source unavailable")
    );
}
