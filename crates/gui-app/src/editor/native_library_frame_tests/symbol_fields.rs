//! Edit saved definition fields and place a bare Symbol through the ordinary App.
use super::*;
use eda_engine::substrate::{ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;

const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
const SOURCE: &str = "pool/symbols/01974935-759f-4259-84d1-0c7d53be8bbf.json";
const PREFIX: &str = "XQ";

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn expected(mut raw: Value, phase: usize) -> Value {
    match phase {
        0 => (),
        1 => {
            raw["fields"] = json!([
                {"key":"MPN","value":"Draft","position":{"x":500000,"y":-1000000},"visible":true},
                {"key":"Note","value":"Remove me"}
            ]);
            raw["default_refdes_prefix"] = json!("Q");
        }
        2 | 3 => {
            raw["fields"] = json!([
                {"key":"MPN","value":"  Ready Ω  ","position":{"x":1000000,"y":-1000000},"visible":true}
            ]);
            if phase == 2 {
                raw["default_refdes_prefix"] = json!(PREFIX);
            }
        }
        _ => unreachable!(),
    }
    raw
}

impl Fixture {
    fn symbol_field_choice(&self) -> usize {
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

    pub(super) fn symbol_fields(&mut self, events: &ActiveEventLoop, stage: usize) {
        let record = read(&self.output.join("symbol-fields-intent.json"));
        let state = self.app.runtime.as_ref().unwrap().workspace();
        match stage {
            0..=3 | 30 => {
                let phase = if stage == 30 { 2 } else { stage };
                let expected = expected(record["before"].clone(), phase);
                assert_eq!(read(&self.root.join(SOURCE)), expected);
                let model = state.library.basis.as_ref().unwrap();
                for (id, raw) in record["frozen"].as_object().unwrap() {
                    assert_eq!(
                        model
                            .materialized_placed_symbol_definition(
                                id.parse().unwrap(),
                                SYMBOL.parse().unwrap(),
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
                    let sheet = &source.sheets[&ROOT.parse().unwrap()];
                    let (id, placed) = sheet
                        .symbols
                        .iter()
                        .find(|(id, _)| {
                            record["sheets"][ROOT]["symbols"]
                                .get(id.to_string())
                                .is_none()
                        })
                        .unwrap();
                    assert_eq!(placed.lib_id.as_deref(), Some(SYMBOL));
                    let placed_raw = json!(placed);
                    assert_eq!(placed_raw["reference"], format!("{PREFIX}1"));
                    assert_eq!(placed_raw["value"], "Shared body");
                    assert!(
                        placed_raw
                            .get("fields")
                            .is_none_or(|fields| fields.as_array().is_some_and(Vec::is_empty))
                    );
                    assert_eq!(
                        model
                            .materialized_placed_symbol_definition(*id, SYMBOL.parse().unwrap(),)
                            .unwrap(),
                        Some(expected)
                    );
                    sheets[ROOT]["symbols"]
                        .as_object_mut()
                        .unwrap()
                        .remove(&id.to_string());
                    std::fs::write(
                        self.output.join("symbol-fields-new-placement.json"),
                        serde_json::to_vec_pretty(&placed_raw).unwrap(),
                    )
                    .unwrap();
                }
                assert_eq!(sheets, record["sheets"], "existing placed and sheet data");
            }
            11..=13 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert!(draft.refusal.is_none());
                assert_eq!(
                    draft.preview,
                    Some(expected(record["before"].clone(), stage - 10))
                );
            }
            14 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert_eq!(draft.symbol_prefix(), Some(PREFIX));
                assert_eq!(draft.value("symbol_field_key"), "MPN");
                assert_eq!(draft.value("symbol_field_value"), "  Ready Ω  ");
                assert_eq!(draft.value("symbol_field_x"), "1000000");
                assert_eq!(draft.value("symbol_field_y"), "-1000000");
                assert_eq!(draft.value("symbol_field_visible"), "true");
                assert!(draft.request().unwrap().fields.is_empty());
            }
            20 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            21 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            22 => {
                let index = self.symbol_field_choice();
                let offset = state.schematic_authoring.choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps.insert(self.stage + 1, Step::SymbolFields(22));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            23 => self.control(
                events,
                HitTarget::SchematicLibraryChoice(self.symbol_field_choice()),
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
            26 | 27 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: 7000000,
                    y: -3000000,
                },
                stage == 27,
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires native root/child project and owned isolated Wayland"]
fn ordinary_saved_symbol_fields_prefix_preview_cancel_history_cold_and_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = read(&root.join(SOURCE));
    assert!(before.get("fields").is_none());
    assert!(before.get("default_refdes_prefix").is_none());
    let capture = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = capture.schematic_source().unwrap();
    let frozen: BTreeMap<_, _> = source
        .sheets
        .values()
        .flat_map(|sheet| sheet.symbols.values())
        .filter(|placed| placed.lib_id.as_deref() == Some(SYMBOL))
        .map(|placed| {
            (
                placed.uuid,
                model
                    .materialized_placed_symbol_definition(placed.uuid, SYMBOL.parse().unwrap())
                    .unwrap()
                    .unwrap(),
            )
        })
        .collect();
    assert!(!frozen.is_empty());
    std::fs::write(
        output.join("symbol-fields-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"frozen":frozen,"sheets":source.sheets,"setup_transactions":0}),
        )
        .unwrap(),
    )
    .unwrap();

    let field = |key, text: &str| Step::Field(key, text.into());
    let open = || {
        [
            Step::Open,
            Step::Kind(1),
            Step::Search("Shared body".into()),
            Step::Select("Shared body".into()),
        ]
    };
    let mut steps = vec![Step::SymbolFields(0), Step::Snapshot];
    steps.extend(open());
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::Control(C::AddSymbolField),
            field("symbol_field_key", "MPN"),
            field("symbol_field_value", "Draft"),
            field("symbol_field_x", "0.5mm"),
            field("symbol_field_y", "-1mm"),
            Step::Choice("symbol_field_visible", "Visible".into()),
            Step::KeyControl(C::AddSymbolField),
            field("symbol_field_key", "Note"),
            field("symbol_field_value", "Remove me"),
            field("symbol_prefix", "Q"),
            Step::KeyControl(C::Preview),
            Step::SymbolFields(11),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-fields-add-preview"),
                Step::KeyControl(C::Commit),
                Step::SymbolFields(1),
            ]);
        } else {
            steps.extend([Step::KeyControl(C::Cancel), Step::Invariant]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolFields(0),
        Step::History(true),
        Step::SymbolFields(1),
        Step::Snapshot,
        Step::Control(C::Edit),
        Step::KeyControl(C::SymbolField(0)),
        field("symbol_field_value", "  Ready Ω  "),
        field("symbol_field_x", "1mm"),
        Step::KeyControl(C::SymbolField(1)),
        Step::KeyControl(C::RemoveSymbolField),
        field("symbol_prefix", PREFIX),
        Step::KeyControl(C::Preview),
        Step::SymbolFields(12),
        Step::Invariant,
        Step::Capture("symbol-fields-edit-preview"),
        Step::KeyControl(C::Commit),
        Step::SymbolFields(2),
        Step::History(false),
        Step::SymbolFields(1),
        Step::History(true),
        Step::SymbolFields(2),
        Step::Snapshot,
        Step::Control(C::Edit),
        Step::KeyControl(C::ClearSymbolPrefix),
        Step::KeyControl(C::Preview),
        Step::SymbolFields(13),
        Step::Invariant,
        Step::Capture("symbol-fields-clear-preview"),
        Step::KeyControl(C::Commit),
        Step::SymbolFields(3),
        Step::History(false),
        Step::SymbolFields(2),
        Step::History(true),
        Step::SymbolFields(3),
        Step::History(false),
        Step::SymbolFields(2),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolFields(2),
    ]);
    steps.extend(open());
    steps.extend([
        Step::Control(C::Edit),
        Step::KeyControl(C::SymbolField(0)),
        Step::SymbolFields(14),
        Step::Capture("symbol-fields-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SymbolFields(25),
        Step::SymbolFields(20),
        Step::SymbolFields(21),
        Step::SymbolFields(22),
        Step::SymbolFields(23),
        Step::SymbolFields(24),
        Step::PaneContext(Fit),
        Step::SymbolFields(26),
        Step::Capture("symbol-fields-placement-ghost"),
        Step::SymbolFields(27),
        Step::SymbolFields(30),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolFields(30),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::Capture("symbol-fields-placement-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
