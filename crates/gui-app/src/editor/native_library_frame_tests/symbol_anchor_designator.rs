//! Saved schematic pin designator through the ordinary Library and placement flows.
use super::*;
use eda_engine::substrate::{ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;

const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
const PIN: &str = "f22cf09d-67d1-4b71-abae-1129a8e8af8e";
const SOURCE: &str = "pool/symbols/01974935-759f-4259-84d1-0c7d53be8bbf.json";
const DESIGNATOR: &str = "S17";
const PLACEMENT: datum_gui_protocol::PointNm = datum_gui_protocol::PointNm {
    x: 9000000,
    y: 4000000,
};

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn expected(mut raw: Value, set: bool) -> Value {
    if set {
        raw["pin_anchors"][0]["number"] = json!(DESIGNATOR);
    }
    raw
}

impl Fixture {
    fn designator_choice(&self) -> usize {
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .position(|choice| choice.id.to_string() == SYMBOL && choice.explicit_part.is_none())
            .unwrap()
    }

    pub(super) fn symbol_anchor_designator(&mut self, events: &ActiveEventLoop, stage: usize) {
        let record = read(&self.output.join("symbol-anchor-designator-intent.json"));
        let state = self.app.runtime.as_ref().unwrap().workspace();
        match stage {
            0 | 1 | 30 => {
                let candidate = expected(record["before"].clone(), stage != 0);
                assert_eq!(read(&self.root.join(SOURCE)), candidate);
                let model = state.library.basis.as_ref().unwrap();
                for (id, raw) in record["frozen"].as_object().unwrap() {
                    assert_eq!(
                        model
                            .materialized_placed_symbol_definition(
                                id.parse().unwrap(),
                                SYMBOL.parse().unwrap()
                            )
                            .unwrap()
                            .as_ref(),
                        Some(raw)
                    );
                }
                let capture = ElectricalSelectionSnapshot::capture(model).unwrap();
                let source = capture.schematic_source().unwrap();
                let mut sheets = json!(source.sheets);
                if stage == 30 {
                    let root = &source.sheets[&ROOT.parse().unwrap()];
                    let (id, placed) = root
                        .symbols
                        .iter()
                        .find(|(id, _)| {
                            record["sheets"][ROOT]["symbols"]
                                .get(id.to_string())
                                .is_none()
                        })
                        .unwrap();
                    assert_eq!(placed.lib_id.as_deref(), Some(SYMBOL));
                    assert_eq!(
                        placed.position,
                        eda_engine::ir::geometry::Point::new(PLACEMENT.x, PLACEMENT.y)
                    );
                    assert_eq!(placed.pins.len(), 1);
                    let pin = &placed.pins[0];
                    assert_eq!(pin.library_pin, Some(PIN.parse().unwrap()));
                    assert_eq!(pin.number, DESIGNATOR);
                    assert_eq!(pin.name, "Auxiliary supply");
                    assert_eq!(
                        pin.position,
                        eda_engine::ir::geometry::Point::new(PLACEMENT.x - 2000000, PLACEMENT.y)
                    );
                    assert_eq!(
                        model
                            .materialized_placed_symbol_definition(*id, SYMBOL.parse().unwrap())
                            .unwrap(),
                        Some(candidate)
                    );
                    let scene =
                        datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
                            .unwrap();
                    let pin_id = pin.uuid.to_string();
                    let number = scene
                        .board_texts
                        .iter()
                        .find(|text| {
                            text.text == DESIGNATOR
                                && text.source_object_uuid.as_deref() == Some(&pin_id)
                        })
                        .expect("native Root shows the authored pin designator");
                    assert!(scene.board_text_geometries.iter().any(|geometry| {
                        geometry.text_uuid == number.text_uuid
                            && (!geometry.glyphs.is_empty()
                                || !geometry.strokes.is_empty()
                                || !geometry.fills.is_empty())
                    }));
                    sheets[ROOT]["symbols"]
                        .as_object_mut()
                        .unwrap()
                        .remove(&id.to_string());
                    std::fs::write(
                        self.output
                            .join("symbol-anchor-designator-new-placement.json"),
                        serde_json::to_vec_pretty(&json!(placed)).unwrap(),
                    )
                    .unwrap();
                }
                assert_eq!(sheets, record["sheets"]);
            }
            10 | 11 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert!(draft.refusal.is_none());
                assert_eq!(
                    draft.preview,
                    Some(expected(record["before"].clone(), stage == 11))
                );
            }
            12 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert_eq!(draft.value("anchor_designator"), DESIGNATOR);
                assert!(draft.request().unwrap().fields.is_empty());
                self.focus_field("anchor_designator");
            }
            20 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            21 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            22 => {
                let index = self.designator_choice();
                let offset = state.schematic_authoring.choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps
                        .insert(self.stage + 1, Step::SymbolAnchorDesignator(22));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            23 => self.control(
                events,
                HitTarget::SchematicLibraryChoice(self.designator_choice()),
            ),
            24 => self.control(events, HitTarget::SchematicPlace),
            25 => {
                if state.ui.layout.zoomed != Some(state.ui.layout.focused) {
                    assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                        "View",
                        "Maximize Pane",
                        EntrySurface::MenuPointer
                    ));
                }
            }
            26 | 27 => self.placement_mouse(events, PLACEMENT, stage == 27),
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires native Root project and owned isolated Wayland"]
fn ordinary_saved_anchor_designator_preview_cancel_history_cold_and_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = read(&root.join(SOURCE));
    assert_eq!(before["pin_anchors"][0]["pin"], PIN);
    assert!(before["pin_anchors"][0].get("number").is_none());
    let capture = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = capture.schematic_source().unwrap();
    let frozen: BTreeMap<_, _> = source
        .sheets
        .values()
        .flat_map(|sheet| sheet.symbols.values())
        .filter(|p| p.lib_id.as_deref() == Some(SYMBOL))
        .map(|p| {
            (
                p.uuid,
                model
                    .materialized_placed_symbol_definition(p.uuid, SYMBOL.parse().unwrap())
                    .unwrap()
                    .unwrap(),
            )
        })
        .collect();
    assert!(!frozen.is_empty());
    std::fs::write(
        output.join("symbol-anchor-designator-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"frozen":frozen,"sheets":source.sheets,"setup_transactions":0}),
        )
        .unwrap(),
    )
    .unwrap();

    let open = || {
        [
            Step::Open,
            Step::Kind(1),
            Step::Search("Shared body".into()),
            Step::Select("Shared body".into()),
        ]
    };
    let s = Step::SymbolAnchorDesignator;
    let mut steps = vec![
        s(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
    ];
    steps.extend(open());
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::KeyControl(C::Anchor(0)),
            Step::Field("anchor_designator", DESIGNATOR.into()),
            Step::KeyControl(C::Preview),
            s(11),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-designator-set-preview"),
                Step::KeyControl(C::Commit),
                s(1),
            ]);
        } else {
            steps.extend([Step::KeyControl(C::Cancel), Step::Invariant, s(0)]);
        }
    }
    steps.extend([
        Step::History(false),
        s(0),
        Step::History(true),
        s(1),
        Step::Snapshot,
        Step::Control(C::Edit),
        Step::KeyControl(C::Anchor(0)),
        Step::KeyControl(C::ClearAnchorDesignator),
        Step::KeyControl(C::Preview),
        s(10),
        Step::Invariant,
        Step::Capture("symbol-designator-clear-preview"),
        Step::KeyControl(C::Commit),
        s(0),
        Step::History(false),
        s(1),
        Step::History(true),
        s(0),
        Step::History(false),
        s(1),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        s(1),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
    ]);
    steps.extend(open());
    steps.extend([
        Step::Control(C::Edit),
        Step::KeyControl(C::Anchor(0)),
        s(12),
        Step::Capture("symbol-designator-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        s(25),
        s(20),
        s(21),
        s(22),
        s(23),
        s(24),
        Step::PaneContext(Fit),
        s(26),
        Step::Capture("symbol-designator-placement-ghost"),
        s(27),
        s(30),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        s(25),
        Step::PaneContext(Fit),
        s(30),
        Step::Capture("symbol-designator-placement-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
