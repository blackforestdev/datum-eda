//! Original thick-stroke preview and XQ2 native schematic pin-label reproductions.
use super::*;
use datum_gui_protocol::{BoardReviewSceneV1, PointNm};

const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const PLACED: &str = "5e006b68-85f6-429b-8ba3-df6fafc5169a";
const THICK: &str = "Manual Symbol 9gtd";
const STUB: [PointNm; 2] = [
    PointNm {
        x: 7000000,
        y: 4000000,
    },
    PointNm {
        x: 8000000,
        y: 4000000,
    },
];

fn geometry(scene: &BoardReviewSceneV1, owner: &str, content: &str) -> [[i128; 2]; 2] {
    let text = scene
        .board_texts
        .iter()
        .find(|text| text.source_object_uuid.as_deref() == Some(owner) && text.text == content)
        .expect("actual source-bound native pin label");
    let ink = scene
        .board_text_geometries
        .iter()
        .find(|ink| ink.text_uuid == text.text_uuid)
        .unwrap();
    assert!(!ink.glyphs.is_empty());
    let corners = datum_gui_protocol::selection_geometry_catalog::layout_polygon(ink).unwrap();
    let min = std::array::from_fn(|axis| {
        corners
            .iter()
            .map(|point| {
                let v = point[axis];
                v.numerator.div_euclid(v.denominator)
            })
            .min()
            .unwrap()
    });
    let max = std::array::from_fn(|axis| {
        corners
            .iter()
            .map(|point| {
                let v = point[axis];
                v.numerator.div_euclid(v.denominator)
                    + i128::from(v.numerator.rem_euclid(v.denominator) != 0)
            })
            .max()
            .unwrap()
    });
    [min, max]
}

impl Fixture {
    pub(super) fn pin_label_clearance(&mut self, events: &ActiveEventLoop, stage: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        match stage {
            0 => {
                let draft = state.library.draft.as_ref().unwrap();
                let raw = draft.preview.as_ref().unwrap();
                let unit = state
                    .library
                    .entries
                    .iter()
                    .find(|entry| entry.id.to_string() == raw["unit"].as_str().unwrap())
                    .unwrap();
                let scene = datum_gui_protocol::library_browser::symbol_geometry::preview::project(
                    raw,
                    Some(&unit.raw),
                )
                .unwrap();
                let body = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_kind == "schematic_symbol")
                    .unwrap();
                assert_eq!(body.width_nm, Some(254000));
                assert!(body.path.contains(&PointNm {
                    x: -2540000,
                    y: -2540000
                }));
                let input = raw["pin_anchors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row["position"]["x"] == -5080000)
                    .unwrap();
                let id = input["pin"].as_str().unwrap();
                let stub = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_kind == "schematic_pin" && g.source_object_uuid == id)
                    .unwrap();
                assert_eq!(
                    stub.path,
                    [PointNm { x: -5080000, y: 0 }, PointNm { x: -2540000, y: 0 }]
                );
                geometry(&scene, id, "IN");
            }
            1 | 4 => {
                let pane = state.ui.layout.focused;
                let scene = datum_gui_protocol::camera_scene_for_pane(state, pane).unwrap();
                let owner = if stage == 1 {
                    PLACED.to_owned()
                } else {
                    uuid::Uuid::nil().to_string()
                };
                let stub_id = format!("schematic-symbol-pin:{owner}:0");
                let stub = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_id == stub_id)
                    .unwrap();
                let expected = if stage == 1 {
                    STUB
                } else {
                    [PointNm { x: 7000000, y: 0 }, PointNm { x: 8000000, y: 0 }]
                };
                assert_eq!(stub.path, expected);
                assert_eq!(stub.width_nm, Some(100000));
                let terminal = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_id == format!("schematic-symbol-pin-terminal:{owner}:0"))
                    .unwrap();
                assert_eq!(terminal.source_object_uuid, stub.source_object_uuid);
                let body = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_id == format!("schematic-symbol:{owner}:library:1"))
                    .unwrap();
                assert_eq!(body.width_nm, Some(150000));
                let [min, max] = geometry(&scene, &stub.source_object_uuid, "S17");
                let radius = i128::from(
                    90_000 + datum_gui_protocol::schematic_stroke::TERMINAL_OUTLINE_NM / 2,
                );
                let center = [i128::from(expected[0].x), i128::from(expected[0].y)];
                assert!(
                    (0..2).any(|axis| max[axis] <= center[axis] - radius
                        || min[axis] >= center[axis] + radius)
                );
                geometry(&scene, &stub.source_object_uuid, "Auxiliary supply");
            }
            2 => {
                let layout = &state.ui.layout;
                if layout.zoomed != Some(layout.focused) {
                    assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                        "View",
                        "Maximize Pane",
                        EntrySurface::MenuPointer
                    ));
                }
            }
            3 => {
                let layout = state.ui.layout.clone();
                let runtime = self.app.runtime.as_mut().unwrap();
                let prepared = runtime.prepared_scene().unwrap();
                let viewport = prepared.layout.viewport_panes(&layout);
                let rect = viewport
                    .panes
                    .iter()
                    .find(|pane| pane.id == layout.focused)
                    .unwrap()
                    .rect
                    .scene;
                let id = self.app.window.as_ref().unwrap().id();
                self.app.window_event(
                    events,
                    id,
                    WindowEvent::CursorMoved {
                        device_id: winit::event::DeviceId::dummy(),
                        position: winit::dpi::PhysicalPosition::new(
                            f64::from(rect.x + rect.width / 2.),
                            f64::from(rect.y + rect.height / 2.),
                        ),
                    },
                );
                self.app.window_event(
                    events,
                    id,
                    WindowEvent::MouseWheel {
                        device_id: winit::event::DeviceId::dummy(),
                        delta: winit::event::MouseScrollDelta::LineDelta(0., -100.),
                        phase: winit::event::TouchPhase::Moved,
                    },
                );
            }
            10 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            11 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            12 | 13 => {
                let authoring = &state.schematic_authoring;
                let index = authoring
                    .choices
                    .iter()
                    .position(|choice| {
                        choice.id.to_string() == SYMBOL && choice.explicit_part.is_none()
                    })
                    .unwrap();
                let offset = authoring.choice_offset;
                if stage == 13 {
                    self.control(events, HitTarget::SchematicLibraryChoice(index));
                } else if !(offset..offset + 4).contains(&index) {
                    self.steps
                        .insert(self.stage + 1, Step::PinLabelClearance(12));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            14 => self.control(events, HitTarget::SchematicPlace),
            15 => self.placement_mouse(events, PointNm { x: 9000000, y: 0 }, false),
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires original native cases and owned Wayland"]
fn ordinary_native_pin_label_clearance_preview_and_schematic() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let check = Step::PinLabelClearance;
    let steps = match std::env::var("DATUM_LIBRARY_PIN_LABEL_CASE")
        .unwrap()
        .as_str()
    {
        "preview" => vec![
            Step::Snapshot,
            Step::Open,
            Step::Kind(1),
            Step::Search(THICK.into()),
            Step::Select(THICK.into()),
            check(2),
            Step::KeyControl(C::Edit),
            Step::KeyControl(C::Preview),
            check(0),
            Step::Invariant,
            check(3),
            Step::Capture("thick-pin-label-preview"),
            Step::KeyControl(C::Cancel),
            Step::Invariant,
        ],
        "canvas" => vec![
            Step::Snapshot,
            Step::PaneContext(Navigate(false)),
            check(2),
            Step::PaneContext(Fit),
            check(1),
            Step::Capture("xq2-pin-label-open"),
            Step::Reopen,
            Step::Invariant,
            Step::PaneContext(Navigate(false)),
            check(2),
            Step::PaneContext(Fit),
            check(1),
            Step::Capture("xq2-pin-label-cold"),
            check(10),
            check(11),
            check(12),
            check(13),
            check(14),
            Step::PaneContext(Fit),
            check(15),
            check(4),
            Step::Capture("xq2-pin-label-ghost"),
            Step::Invariant,
        ],
        _ => unreachable!(),
    };
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
