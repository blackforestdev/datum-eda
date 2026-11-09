//! Saved Unit pin addition and later named-pin Symbol authoring in ordinary App.
use super::*;
use eda_engine::substrate::ProjectResolver;
use serde_json::{Value, json};
use uuid::Uuid;
const UNIT: &str = "f69c04fd-a070-4985-b5ed-c42426f4f259";
const PIN_NAME: &str = "Auxiliary supply";
const SYMBOL_NAME: &str = "New supply body";
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn save(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn canonical(root: &Path) -> BTreeMap<String, Vec<u8>> {
    source_bytes(root)
        .into_iter()
        .filter(|(p, _)| {
            !p.starts_with(".datum/")
                || [
                    ".datum/component_instances/",
                    ".datum/electrical_identities/",
                    ".datum/proposals/",
                ]
                .iter()
                .any(|s| p.starts_with(s))
        })
        .collect()
}
impl Fixture {
    fn unit_pin_intent(&self) -> Value {
        read(&self.output.join("unit-pin-intent.json"))
    }
    fn unit_pin_expected(&self, added: bool) -> Value {
        let i = self.unit_pin_intent();
        let mut value = i["before"].clone();
        if added {
            let id = i["pin"].as_str().unwrap();
            value["pins"][id] = json!({"uuid":id,"name":PIN_NAME,"electrical_type":"PowerIn","direction":"PowerIn","swap_group":0,"alternates":[]});
        }
        value
    }
    fn assert_unit_pin_sources(&self, added: bool, symbol: bool) {
        let intent = self.unit_pin_intent();
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        assert_eq!(
            read(&self.root.join(format!("pool/units/{UNIT}.json"))),
            self.unit_pin_expected(added)
        );
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
        let mut now = canonical(&self.root);
        let path = format!("pool/units/{UNIT}.json");
        before.remove(&path);
        now.remove(&path);
        if symbol {
            assert!(
                now.remove(&format!(
                    "pool/symbols/{}.json",
                    intent["symbol"].as_str().unwrap()
                ))
                .is_some()
            );
        }
        assert_eq!(
            now, before,
            "every other authored source, CI, old pin, reference, Map and Net stays byte-exact"
        );
        if let Some(pin) = intent["pin"].as_str() {
            assert_eq!(model.objects.contains_key(&pin.parse().unwrap()), added);
        }
    }
    pub(super) fn unit_add_pin(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            0 => {
                self.original_footprint = Some(json!(canonical(&self.root)));
            }
            1 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert!(!d.create);
                assert!(d.request().unwrap().fields.is_empty());
            }
            2 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert_eq!(d.value("pin_name"), PIN_NAME);
                assert!(d.value("pin_type").is_empty());
                assert!(d.request().is_err());
                assert!(d.preview.is_none());
            }
            3 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                let pin = d.electrical.pin_id.unwrap();
                assert!(d.electrical.selected_new_pin());
                assert_eq!(
                    json!(d.request().unwrap().fields),
                    json!({"pin_additions":{pin.to_string():{"name":PIN_NAME,"electrical_type":"PowerIn"}}})
                );
                let mut intent = self.unit_pin_intent();
                intent["pin"] = json!(pin);
                save(&self.output.join("unit-pin-intent.json"), &intent);
            }
            4 | 5 => {
                let intent = self.unit_pin_intent();
                let key = if stage == 4 {
                    intent["before"]["pins"]
                        .as_object()
                        .unwrap()
                        .keys()
                        .next()
                        .unwrap()
                        .as_str()
                } else {
                    intent["pin"].as_str().unwrap()
                };
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                let index = d.electrical.pins.keys().position(|k| k == key).unwrap();
                self.control(events, HitTarget::Library(C::Pin(index)));
                if stage == 5 {
                    let d = self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .as_ref()
                        .unwrap();
                    assert_eq!(d.value("pin_name"), PIN_NAME);
                    assert_eq!(d.value("pin_type"), "PowerIn");
                }
            }
            6 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert_eq!(d.preview, Some(self.unit_pin_expected(true)));
            }
            7 | 8 => self.assert_unit_pin_sources(stage == 7, false),
            9 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert!(!d.electrical.selected_new_pin());
                assert!(d.request().unwrap().fields.is_empty());
                assert_eq!(
                    d.electrical.pins.len(),
                    self.unit_pin_intent()["before"]["pins"]
                        .as_object()
                        .unwrap()
                        .len()
                        + 1
                );
                assert!(
                    !self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .presented_hits
                        .regions()
                        .iter()
                        .any(|h| matches!(h.target, HitTarget::Library(C::RemovePin)))
                );
            }
            10 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert!(d.create);
                let pin = self.unit_pin_intent()["pin"].as_str().unwrap().to_string();
                assert!(
                    d.fields
                        .iter()
                        .find(|f| f.key == "anchor_pin")
                        .unwrap()
                        .choices
                        .contains(&(pin, PIN_NAME.into()))
                );
            }
            11 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                let raw = d.preview.as_ref().unwrap();
                assert_eq!(raw["unit"], json!(UNIT));
                assert_eq!(raw["pin_anchors"].as_array().unwrap().len(), 1);
                assert_eq!(raw["pin_anchors"][0]["pin"], self.unit_pin_intent()["pin"]);
            }
            12 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let mut intent = self.unit_pin_intent();
                if intent["symbol"].is_null() {
                    let browser = datum_gui_protocol::library_browser::LibraryBrowser::capture(
                        std::sync::Arc::new(model),
                    );
                    let e = browser
                        .entries
                        .iter()
                        .find(|e| e.kind == "symbols" && e.name == SYMBOL_NAME)
                        .unwrap();
                    intent["symbol"] = json!(e.id);
                    save(&self.output.join("unit-pin-intent.json"), &intent);
                }
                let raw = read(&self.root.join(format!(
                    "pool/symbols/{}.json",
                    intent["symbol"].as_str().unwrap()
                )));
                assert_eq!(raw["unit"], json!(UNIT));
                assert_eq!(raw["pin_anchors"][0]["pin"], intent["pin"]);
                assert_eq!(raw["pin_anchors"].as_array().unwrap().len(), 1);
                self.assert_unit_pin_sources(true, true);
            }
            13 => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert!(d.electrical.pin_id.is_none());
                assert!(d.preview.is_none());
                assert!(d.request().unwrap().fields.is_empty());
                assert_eq!(
                    json!(d.electrical.pins),
                    self.unit_pin_intent()["before"]["pins"]
                );
            }
            _ => panic!("unknown Unit Add pin stage"),
        }
    }
}
#[test]
#[ignore = "requires real native root/child circuit and owned isolated Wayland"]
fn ordinary_saved_unit_add_pin_cancel_history_cold_and_later_symbol() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    save(
        &output.join("unit-pin-intent.json"),
        &json!({"before":read(&root.join(format!("pool/units/{UNIT}.json")))}),
    );
    let mut steps = vec![
        Step::UnitAddPin(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(0),
        Step::Search("Logic".into()),
        Step::Select("Logic".into()),
        Step::Control(C::Edit),
        Step::UnitAddPin(1),
        Step::Capture("unit-pin-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Control(C::AddPin),
            Step::Field("pin_name", PIN_NAME.into()),
            Step::Control(C::Preview),
            Step::UnitAddPin(2),
            Step::Invariant,
        ]);
        if commit {
            steps.push(Step::Capture("unit-pin-invalid"));
        }
        steps.extend([
            Step::Choice("pin_type", "PowerIn".into()),
            Step::UnitAddPin(3),
            Step::UnitAddPin(4),
            Step::UnitAddPin(5),
            Step::Control(C::Preview),
            Step::UnitAddPin(6),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("unit-pin-preview"),
                Step::Control(C::Commit),
                Step::UnitAddPin(7),
                Step::Capture("unit-pin-committed"),
            ]);
        } else {
            steps.extend([
                Step::Control(C::RemovePin),
                Step::UnitAddPin(13),
                Step::Invariant,
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::UnitAddPin(8),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::UnitAddPin(8),
        Step::History(true),
        Step::UnitAddPin(7),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::UnitAddPin(7),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("unit-pin-cold-child"),
        Step::Open,
        Step::Kind(0),
        Step::Search("Logic".into()),
        Step::Select("Logic".into()),
        Step::Control(C::Edit),
        Step::UnitAddPin(5),
        Step::UnitAddPin(9),
        Step::Capture("unit-pin-cold-form"),
        Step::Control(C::Cancel),
        Step::Kind(1),
        Step::Control(C::New),
        Step::Field("name", SYMBOL_NAME.into()),
        Step::Choice("unit", "Logic".into()),
        Step::UnitAddPin(10),
    ]);
    for (key, value) in [
        ("rect_min_x", "-1000000"),
        ("rect_min_y", "-1000000"),
        ("rect_max_x", "1000000"),
        ("rect_max_y", "1000000"),
        ("rect_width", "100000"),
    ] {
        steps.push(Step::Field(key, value.into()));
    }
    steps.extend([
        Step::Choice("anchor_pin", PIN_NAME.into()),
        Step::Field("anchor_x", "-2000000".into()),
        Step::Field("anchor_y", "0".into()),
        Step::Choice("anchor_orientation", "Right".into()),
        Step::Field("anchor_length", "1000000".into()),
        Step::Control(C::Preview),
        Step::UnitAddPin(11),
        Step::Capture("unit-pin-symbol-preview"),
        Step::Control(C::Commit),
        Step::UnitAddPin(12),
        Step::Capture("unit-pin-symbol-created"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::UnitAddPin(12),
        Step::Open,
        Step::Kind(1),
        Step::Search(SYMBOL_NAME.into()),
        Step::Select(SYMBOL_NAME.into()),
        Step::Capture("unit-pin-symbol-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
