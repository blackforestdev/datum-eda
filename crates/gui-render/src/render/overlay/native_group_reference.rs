//! Shared transient command reference over the original qualified schematic pane.
use super::*;
/// Command-owned pickup reference; never retained, selectable, or electrical.
pub(super) fn render(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    let gesture = &state.authoring.gesture;
    if gesture.tool != datum_gui_protocol::WorkspaceTool::Move
        || !gesture
            .target_object_id
            .as_deref()
            .is_some_and(|id| id.starts_with("schematic-symbols:"))
        || !gesture.view_basis.as_ref().is_some_and(|basis| {
            basis.content == datum_gui_protocol::PaneContent::Schematic
                && basis
                    .document
                    .as_ref()
                    .is_some_and(|document| Some(document) == scene.schematic_document.as_ref())
                && basis.project.to_string() == scene.project_uuid
                && basis.revision == scene.source_revision
        })
    {
        return;
    }
    let Some(at) = gesture.preview.or(gesture.anchor) else {
        return;
    };
    let point = project_point(at, projection);
    if projection.viewport.contains(point.0, point.1) {
        interaction_overlay::push_local_crosshair(out, point, projection.scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_symbol_reference_tracks_pose_and_refuses_other_document_stale_and_cancelled_state() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let project = uuid::Uuid::new_v4();
        let sheet = uuid::Uuid::new_v4();
        let document = datum_gui_protocol::schematic_documents::SchematicDocumentKey {
            project,
            schematic: uuid::Uuid::new_v4(),
            source_sheet: sheet,
            instance_path: vec![],
        };
        let mut scene = state.scene.clone();
        scene.project_uuid = project.to_string();
        scene.schematic_document = Some(document.clone());
        scene.bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 10_000_000,
            max_y: 10_000_000,
        };
        let projection = Projection::new(
            RectPx {
                x: 0.,
                y: 0.,
                width: 600.,
                height: 600.,
            },
            &scene.bounds,
            CameraState::fit_to_bounds(&scene.bounds),
        );
        state.authoring.gesture = datum_gui_protocol::AuthoringGestureState {
            tool: datum_gui_protocol::WorkspaceTool::Move,
            anchor: Some(PointNm {
                x: 2_000_000,
                y: 3_000_000,
            }),
            preview: Some(PointNm {
                x: 6_000_000,
                y: 7_000_000,
            }),
            target_object_id: Some("schematic-symbols:2".into()),
            view_basis: Some(datum_gui_protocol::AuthoringViewBasis {
                project,
                revision: scene.source_revision.clone(),
                pane: datum_gui_protocol::PaneId(1),
                content: datum_gui_protocol::PaneContent::Schematic,
                source: sheet,
                document: Some(document),
            }),
        };
        let mut quads = vec![];
        render(&mut quads, &scene, &projection, &state);
        assert_eq!(quads.len(), 2);
        let center = project_point(state.authoring.gesture.preview.unwrap(), &projection);
        for quad in &quads {
            let x = quad.points.iter().map(|p| p.0).sum::<f32>() / 4.;
            let y = quad.points.iter().map(|p| p.1).sum::<f32>() / 4.;
            assert_eq!(
                (x, y),
                center,
                "reference moves with common destination, not a member"
            );
        }
        let pane = state.ui.layout.open_beside_root(
            datum_gui_protocol::PaneContent::Schematic,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            true,
        );
        state.ui.layout.zoomed = Some(pane);
        state.schematic_documents.project = None;
        state.schematic_scene = Some(scene.clone());
        state.authoring.gesture.view_basis.as_mut().unwrap().pane = pane;
        let camera = CameraState::fit_to_bounds(&scene.bounds);
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let mut frame =
            PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        frame.set_surface_camera(pane, camera);
        frame.append_native_selection(&state).unwrap();
        assert!(
            frame
                .viewport_overlay_vertices
                .iter()
                .any(|v| v.color == CURSOR_CROSSHAIR)
        );
        let original = scene.clone();
        scene.schematic_document.as_mut().unwrap().source_sheet = uuid::Uuid::new_v4();
        quads.clear();
        render(&mut quads, &scene, &projection, &state);
        assert!(quads.is_empty());
        scene = original.clone();
        scene.source_revision.push_str("-stale");
        render(&mut quads, &scene, &projection, &state);
        assert!(quads.is_empty());
        state.authoring.gesture = datum_gui_protocol::AuthoringGestureState::idle(
            datum_gui_protocol::WorkspaceTool::Move,
        );
        render(&mut quads, &original, &projection, &state);
        assert!(quads.is_empty());
    }
}
