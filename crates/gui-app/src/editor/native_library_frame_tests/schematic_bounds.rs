//! Read-only open/Fit/cold on a small canonically authored boundary drawing.
use super::*;
use datum_gui_protocol::PointNm;
use serde_json::json;
impl Fixture {
    pub(super) fn schematic_bounds(&mut self, _events: &ActiveEventLoop, stage: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let scene =
            datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap();
        let document = scene.schematic_document.as_ref().unwrap();
        let path = self
            .root
            .join(format!("schematic/sheets/{}.json", document.source_sheet));
        let sheet = super::symbol_circle::read(&path);
        let drawings = sheet["drawings"].as_object().unwrap();
        assert_eq!(drawings.len(), 1);
        let line = &drawings.values().next().unwrap()["Line"];
        let max = i64::MAX;
        assert_eq!(line["from"], json!({"x":max-1000000,"y":max}));
        assert_eq!(line["to"], json!({"x":max,"y":max}));
        let graphic = scene
            .board_graphics
            .iter()
            .find(|g| g.object_kind == "schematic_drawing")
            .unwrap();
        assert_eq!(
            graphic.path,
            vec![
                PointNm {
                    x: max - 1000000,
                    y: max
                },
                PointNm { x: max, y: max }
            ]
        );
        let bounds = &scene.bounds;
        assert!(bounds.min_x < bounds.max_x && bounds.min_y < bounds.max_y);
        assert_eq!(bounds.max_x, max);
        assert_eq!(bounds.max_y, max);
        for p in &graphic.path {
            assert!((bounds.min_x..=bounds.max_x).contains(&p.x));
            assert!((bounds.min_y..=bounds.max_y).contains(&p.y));
        }
        let record = json!({"source_sheet":document.source_sheet,"source_line":line,"projected_path":graphic.path,"bounds":bounds});
        let baseline = self.output.join("bounds-before-cold.json");
        if stage == 0 {
            super::symbol_circle::write(&baseline, &record);
        } else {
            assert_eq!(
                super::symbol_circle::read(&baseline),
                record,
                "cold projection retains source endpoints and clipped padding"
            );
        }
    }
}
#[test]
#[ignore = "requires canonically authored boundary drawing and owned isolated display"]
fn ordinary_boundary_schematic_open_fit_and_cold_preserve_sources() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let steps = vec![
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SchematicBounds(0),
        Step::Capture("bounds-positive-fit"),
        Step::Invariant,
        Step::Reopen,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SchematicBounds(1),
        Step::Capture("bounds-positive-cold"),
        Step::Invariant,
    ];
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
