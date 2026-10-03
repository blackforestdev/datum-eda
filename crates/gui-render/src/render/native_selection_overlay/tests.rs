use super::*;
use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
use datum_gui_protocol::selection_view::build_selection_view;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject,
};
use eda_engine::ir::geometry::Point;
pub(crate) struct Fixture {
    root: std::path::PathBuf,
    model: eda_engine::substrate::DesignModel,
    board: uuid::Uuid,
}
impl Fixture {
    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }
    pub(crate) fn source_bytes(&self) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        let mut files: Vec<_> = self
            .model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let journal = eda_engine::substrate::transaction_journal_path(&self.root);
        files.push((journal.clone(), std::fs::read(journal).unwrap()));
        files
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
pub(crate) fn selected_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance, genesis},
        board::{Net, Track},
        substrate::{CommitSource, ProjectResolver},
    };
    let root = std::env::temp_dir().join(format!("s5a-renderer-{}", uuid::Uuid::new_v4()));
    let report = genesis::bootstrap_native_project(
        &root,
        genesis::GenesisSpec {
            project_name: "selection-renderer".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut f = Fixture {
        model: ProjectResolver::new(&root).resolve().unwrap(),
        root,
        board: report.board_uuid,
    };
    let provenance = || {
        WriteProvenance::new(
            "selection-renderer-fixture",
            CommitSource::Test,
            "source-bound immediate proof",
        )
    };
    let net = Net::new(uuid::Uuid::new_v4(), "N", uuid::Uuid::nil());
    let write =
        native_write::board_routing::build_place_board_net(&f.model, provenance(), &net).unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let track = Track::straight(
        uuid::Uuid::new_v4(),
        net.uuid,
        Point::new(0, 0),
        Point::new(10_000_000, 0),
        200_000,
        1,
    );
    let write =
        native_write::board_routing::build_place_board_track(&f.model, provenance(), &track)
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.project_uuid = f.model.project.project_id.to_string();
    state.scene.board_uuid = f.board.to_string();
    state.scene.components.clear();
    state.scene.pads.clear();
    state.scene.vias.clear();
    state.scene.zones.clear();
    state.scene.board_texts.clear();
    state.scene.board_text_geometries.clear();
    state.scene.board_graphics.clear();
    state.scene.outline.clear();
    state.scene.component_graphics.clear();
    state.scene.component_texts.clear();
    state.scene.unrouted_primitives.clear();
    let mut primitive = state.scene.tracks[0].clone();
    primitive.object_id = "stable-track-display".into();
    primitive.source_object_uuid = track.uuid.to_string();
    primitive.track_uuid = track.uuid.to_string();
    primitive.path = vec![
        PointNm { x: 0, y: 0 },
        PointNm {
            x: 10_000_000,
            y: 0,
        },
    ];
    state.scene.tracks = vec![primitive];
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Object(Id {
        class: Class::Track,
        id: track.uuid,
        instance_path: vec![],
    });
    let view =
        build_selection_view(&f.model, &authority, subject, &state.scene, None, |_| true).unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    (f, state)
}
#[test]
fn native_selection_is_immediate_and_retained_authored_bytes_and_resize_eligibility_are_stable() {
    let (f, mut state) = selected_state();
    let before = f.source_bytes();
    let selected = RetainedScene::from_workspace(&state, 1600, 1000);
    state.selection = SelectionTarget::None;
    let baseline = RetainedScene::from_workspace(&state, 1600, 1000);
    assert_eq!(
        bytemuck::cast_slice::<Vertex, u8>(selected.world_vertices()),
        bytemuck::cast_slice::<Vertex, u8>(baseline.world_vertices())
    );
    assert_eq!(
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(selected.world_strokes().as_ref()),
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(baseline.world_strokes().as_ref())
    );
    assert_eq!(
        selected.can_reuse_for_surface_resize(),
        baseline.can_reuse_for_surface_resize()
    );
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn native_cues_are_clipped_to_each_pane_and_reproject_after_warm_camera_assignment() {
    let (_f, state) = selected_state();
    let viewport = RectPx {
        x: 100.0,
        y: 200.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut first = Vec::new();
    let mut text = Vec::new();
    push_native(
        &mut first,
        &mut text,
        &state.scene,
        &state,
        viewport,
        camera,
    );
    assert!(!first.is_empty());
    assert!(
        first
            .iter()
            .flat_map(|q| q.points)
            .all(|(x, y)| x >= viewport.x
                && x <= viewport.x + viewport.width
                && y >= viewport.y
                && y <= viewport.y + viewport.height)
    );
    let mut second = Vec::new();
    let mut warm = camera;
    warm.center_x_nm += 5_000_000.0;
    push_native(&mut second, &mut text, &state.scene, &state, viewport, warm);
    assert_ne!(format!("{first:?}"), format!("{second:?}"));
    let hover = datum_gui_protocol::HoverTarget {
        surface: datum_gui_protocol::PaneContent::Board,
        object_id: "stable-track-display".into(),
    };
    assert!(selection_beats_hover(&state, &hover));
}

#[test]
fn region_evaluation_feedback_is_pane_clipped_and_preserves_authored_sources() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let pane = state.ui.layout.focused;
    state.ui.selection_region = Some(datum_gui_protocol::SelectionRegionFeedback {
        pane,
        path: vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 10, y: 0 },
            PointNm { x: 10, y: 10 },
        ],
        subtractive: false,
        phase: 0,
        evaluating: Some(4096),
    });
    let rect = RectPx {
        x: 20.0,
        y: 30.0,
        width: 200.0,
        height: 100.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut quads = Vec::new();
    let mut text = Vec::new();
    push_region(
        &mut quads,
        &mut text,
        &state,
        pane,
        &state.scene,
        rect,
        camera,
    );
    assert!(
        text.iter()
            .any(|run| run.text.contains("Evaluating selection")
                && run.text.contains("Escape cancels")
                && run.clip_bounds == Some(rect))
    );
    let mut other_text = Vec::new();
    push_region(
        &mut Vec::new(),
        &mut other_text,
        &state,
        datum_gui_protocol::PaneId(pane.0 + 1),
        &state.scene,
        rect,
        camera,
    );
    assert!(other_text.is_empty());
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
