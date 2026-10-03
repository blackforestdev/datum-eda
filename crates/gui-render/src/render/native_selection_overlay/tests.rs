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
    schematic: uuid::Uuid,
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
    selected_track_state(10_000_000)
}
pub(crate) fn selected_track_state(length_nm: i64) -> (Fixture, ReviewWorkspaceState) {
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
        schematic: report.schematic_uuid,
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
        Point::new(length_nm, 0),
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
    primitive.width_nm = track.width;
    primitive.path = vec![PointNm { x: 0, y: 0 }, PointNm { x: length_nm, y: 0 }];
    state.scene.tracks = vec![primitive];
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    state.scene.source_revision = f.model.model_revision.0.clone();
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
    let mut first_strokes = Vec::new();
    let mut text = Vec::new();
    push_native(
        &mut first_strokes,
        &mut first,
        &mut text,
        &state.scene,
        &state,
        viewport,
        camera,
        datum_gui_protocol::PaneContent::Board,
    );
    assert!(!first_strokes.is_empty());
    assert!(
        first_strokes
            .iter()
            .all(|s| s.clip == [100.0, 200.0, 600.0, 600.0])
    );
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
    let mut second_strokes = Vec::new();
    let mut warm = camera;
    warm.center_x_nm += 5_000_000.0;
    push_native(
        &mut second_strokes,
        &mut second,
        &mut text,
        &state.scene,
        &state,
        viewport,
        warm,
        datum_gui_protocol::PaneContent::Board,
    );
    assert_ne!(first_strokes, second_strokes);
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

#[test]
fn native_projection_uses_typed_surface_and_refuses_foreign_or_stale_scene_basis() {
    use datum_gui_protocol::PaneContent;
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    // Names carry no surface authority: this is still the configured board pane.
    state.scene.scene_id = "schematic-misleading-display-name".into();
    let render = |state: &ReviewWorkspaceState, surface| {
        let mut quads = Vec::new();
        let mut strokes = Vec::new();
        push_native(
            &mut strokes,
            &mut quads,
            &mut Vec::new(),
            &state.scene,
            state,
            viewport,
            camera,
            surface,
        );
        strokes
    };
    assert!(!render(&state, PaneContent::Board).is_empty());
    assert!(render(&state, PaneContent::Schematic).is_empty());
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.schematic_keys.insert("stable-track-display".into());
        view.board_keys.clear();
    }
    state.scene.scene_id = "board-misleading-display-name".into();
    assert!(render(&state, PaneContent::Board).is_empty());
    assert!(!render(&state, PaneContent::Schematic).is_empty());
    let revision = state.scene.source_revision.clone();
    state.scene.source_revision = "older-scene-revision".into();
    assert!(render(&state, PaneContent::Schematic).is_empty());
    state.scene.source_revision = revision;
    state.scene.project_uuid = uuid::Uuid::new_v4().to_string();
    assert!(render(&state, PaneContent::Schematic).is_empty());
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

pub(crate) fn selected_points_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, BatchComposer, WriteProvenance},
        substrate::{CommitSource, Operation},
    };
    let (mut fixture, mut state) = selected_state();
    let sheet = uuid::Uuid::new_v4();
    let junction = uuid::Uuid::new_v4();
    let second_junction = uuid::Uuid::new_v4();
    let provenance = || {
        WriteProvenance::new(
            "selection-point-fixture",
            CommitSource::Test,
            "canonical low-zoom marker source",
        )
    };
    let mut source = fixture
        .model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::SchematicRoot)
        .unwrap();
    source.as_object_mut().unwrap().clear();
    let empty = source.clone();
    source["schema_version"] = 1.into();
    source["uuid"] = sheet.to_string().into();
    source["name"] = "Main".into();
    for field in [
        "symbols",
        "wires",
        "labels",
        "ports",
        "junctions",
        "buses",
        "bus_entries",
        "noconnects",
        "texts",
        "drawings",
    ] {
        source[field] = empty.clone();
    }
    let mut junction_source = empty.clone();
    junction_source["uuid"] = junction.to_string().into();
    junction_source["position"] = empty.clone();
    junction_source["position"]["x"] = 0.into();
    junction_source["position"]["y"] = 0.into();
    let mut second_source = junction_source.clone();
    second_source["uuid"] = second_junction.to_string().into();
    second_source["position"]["x"] = 1_000_000.into();
    let write = native_write::schematic_sheets::build_create_schematic_sheet(
        &fixture.model,
        provenance(),
        fixture.schematic,
        sheet,
        "sheets/main.json",
        source,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let write = BatchComposer::compose(&fixture.model, provenance())
        .push_ops([
            Operation::CreateSchematicJunction {
                sheet_id: sheet,
                junction_id: junction,
                junction: junction_source,
            },
            Operation::CreateSchematicJunction {
                sheet_id: sheet,
                junction_id: second_junction,
                junction: second_source,
            },
        ])
        .finish()
        .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let schematic =
        datum_gui_protocol::project_native_schematic_scene(&fixture.model, &fixture.root)
            .unwrap()
            .unwrap();
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(
            [
                Id {
                    class: Class::Junction,
                    id: junction,
                    instance_path: vec![],
                },
                Id {
                    class: Class::Junction,
                    id: second_junction,
                    instance_path: vec![],
                },
            ],
            None,
        )
        .unwrap(),
    );
    let view = build_selection_view(
        &fixture.model,
        &authority,
        subject,
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    state.schematic_scene = Some(schematic);
    state.selection = SelectionTarget::Native(Box::new(view));
    (fixture, state)
}

pub(crate) fn selected_wire_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance},
        substrate::CommitSource,
    };
    let (mut f, mut state) = selected_points_state();
    let before = NativeSelectionResolution::capture(&f.model).unwrap();
    let sheets = &before.snapshot().schematic_source().unwrap().sheets;
    assert_eq!(sheets.len(), 1);
    let sheet = *sheets.keys().next().unwrap();
    let wire = eda_engine::schematic::SchematicWire {
        uuid: uuid::Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(10_000_000, 0),
    };
    let write = native_write::schematic_connectivity::build_create_schematic_wire(
        &f.model,
        WriteProvenance::new(
            "selection-wire-fixture",
            CommitSource::Test,
            "canonical selection stroke source",
        ),
        sheet,
        &wire,
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let schematic = datum_gui_protocol::project_native_schematic_scene(&f.model, &f.root)
        .unwrap()
        .unwrap();
    state.scene.source_revision = f.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Object(Id {
        class: Class::Wire,
        id: wire.uuid,
        instance_path: vec![],
    });
    let view = build_selection_view(
        &f.model,
        &authority,
        subject,
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    state.schematic_scene = Some(schematic);
    state.selection = SelectionTarget::Native(Box::new(view));
    (f, state)
}
