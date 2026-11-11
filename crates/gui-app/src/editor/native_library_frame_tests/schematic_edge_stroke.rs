//! Existing prepared/retained admission observation for the canonical edge line.
use super::*;
use datum_gui_render::SceneSurface;
use serde_json::json;
impl Fixture {
    pub(super) fn schematic_edge_stroke(&mut self, events: &ActiveEventLoop, stage: usize) {
        self.schematic_bounds(events, stage);
        let runtime = self.app.runtime.as_ref().unwrap();
        let state = runtime.workspace();
        let pane = state.ui.layout.focused;
        let scene = datum_gui_protocol::camera_scene_for_pane(state, pane).unwrap();
        let graphic = scene
            .board_graphics
            .iter()
            .find(|g| g.object_kind == "schematic_drawing")
            .unwrap();
        let session = runtime.renderer.render_session();
        let prepared = session.prepared().unwrap();
        let pass = prepared
            .surface_passes()
            .iter()
            .find(|p| p.pane_id == pane)
            .unwrap();
        assert_eq!(pass.surface, SceneSurface::Schematic);
        assert_eq!(pass.bounds, scene.bounds);
        let retained = session.schematic_for_pane(pane).unwrap();
        let board = session.board().unwrap();
        let admission = prepared
            .geometry_admission(board, session.schematic())
            .find(|a| a.pane_id == pane)
            .unwrap();
        let counts = admission.counts().unwrap();
        // The fixture has exactly one source segment. This is production draw
        // admission, not a raster-pixel or arbitrary geometry-count verdict.
        assert_eq!(counts.retained_stroke_instances, 1);
        assert_eq!(counts.prepared_stroke_instances, 1);
        let rect = pass.scene_viewport;
        let camera = pass.camera;
        let record = json!({"focused_pane":format!("{pane:?}"),"scene_id":scene.scene_id,"graphic":graphic,"bounds":scene.bounds,"viewport":{"x":rect.x,"y":rect.y,"width":rect.width,"height":rect.height},"camera":{"center_x_nm":camera.center_x_nm,"center_y_nm":camera.center_y_nm,"zoom":camera.zoom},"show_authored":state.ui.filters.show_authored,"layer_visibility":state.ui.filters.layer_visibility,"scene_layers":scene.layers,"admission":{"retained_strokes":counts.retained_stroke_instances,"prepared_strokes":counts.prepared_stroke_instances,"prepared_commands":counts.prepared_commands,"ranges":format!("{:?}",admission.ranges().collect::<Vec<_>>())},"prepared_pass":format!("{pass:?}")});
        super::symbol_circle::write(
            &self.output.join(format!("edge-prepared-{stage}.json")),
            &record,
        );
        std::fs::write(
            self.output.join(format!("edge-retained-{stage}.txt")),
            format!("{retained:#?}"),
        )
        .unwrap();
    }
}
#[test]
#[ignore = "requires canonical edge drawing and owned isolated display"]
fn ordinary_edge_stroke_prepared_admission_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let steps = vec![
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SchematicEdgeStroke(0),
        Step::Capture("edge-stroke-fit"),
        Step::Invariant,
        Step::Reopen,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SchematicEdgeStroke(1),
        Step::Capture("edge-stroke-cold"),
        Step::Invariant,
    ];
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
