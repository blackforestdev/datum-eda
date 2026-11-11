//! Saved Circle authoring with authenticated old users and a separate fresh placement.
use super::*;
use datum_gui_protocol::{BoardReviewSceneV1, PointNm};
use eda_engine::substrate::{ElectricalOccurrence, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;
const SYMBOL: &str = "4164b3da-29cc-4bfc-aaa1-7c862ba9b00b";
const PART: &str = "7f7d6e07-8a7c-484c-beaf-e98f962d699d";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
pub(super) fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
pub(super) fn write(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
pub(super) fn canonical(root: &Path) -> BTreeMap<String, Vec<u8>> {
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
                .any(|prefix| p.starts_with(prefix))
        })
        .collect()
}
fn circle_body(scene: &BoardReviewSceneV1, owner: &str) {
    let graphic = scene
        .board_graphics
        .iter()
        .find(|g| g.object_id == format!("schematic-symbol:{owner}:library:0"))
        .unwrap();
    assert_eq!(graphic.primitive_kind, "polyline");
    assert!(graphic.path.len() > 4);
    assert_eq!(graphic.path.first(), graphic.path.last());
    assert_eq!(graphic.path.iter().map(|p| p.x).min(), Some(-2000000));
    assert_eq!(graphic.path.iter().map(|p| p.x).max(), Some(2000000));
    assert_eq!(graphic.path.iter().map(|p| p.y).min(), Some(-7000000));
    assert_eq!(graphic.path.iter().map(|p| p.y).max(), Some(-3000000));
    assert_eq!(graphic.width_nm, Some(100000));
}
pub(super) fn instance_ink(scene: &Value, id: &str, placed: &Value) -> Value {
    let ids: std::collections::BTreeSet<String> = std::iter::once(id.to_owned())
        .chain(
            placed["pins"]
                .as_array()
                .unwrap()
                .iter()
                .map(|pin| pin["uuid"].as_str().unwrap().to_owned()),
        )
        .collect();
    let matches = |row: &Value| {
        row["source_object_uuid"]
            .as_str()
            .is_some_and(|id| ids.contains(id))
            || row["object_id"]
                .as_str()
                .is_some_and(|key| key.contains(id))
    };
    let graphics: Vec<_> = scene["board_graphics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| matches(row))
        .cloned()
        .collect();
    let texts: Vec<_> = scene["board_texts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| matches(row))
        .cloned()
        .collect();
    let text_ids: std::collections::BTreeSet<_> = texts
        .iter()
        .filter_map(|row| row["text_uuid"].as_str())
        .collect();
    let geometry: Vec<_> = scene["text_geometry"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["text_uuid"]
                .as_str()
                .is_some_and(|id| text_ids.contains(id))
        })
        .cloned()
        .collect();
    let mut value = json!({"graphics":graphics,"texts":texts,"geometry":geometry});
    for values in value.as_object_mut().unwrap().values_mut() {
        values
            .as_array_mut()
            .unwrap()
            .sort_by_cached_key(Value::to_string);
    }
    value
}
impl Fixture {
    fn circle_expected(&self, radius: Option<i64>) -> Value {
        let mut raw = read(&self.output.join("circle-intent.json"))["original"].clone();
        raw["drawings"] = match radius {
            Some(radius) => {
                json!([{"Circle":{"center":{"x":0,"y":0},"radius":radius,"width":100000}}])
            }
            None => json!([]),
        };
        raw
    }
    fn circle_old_scene(&self, which: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let scene =
            datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap();
        let document = scene.schematic_document.as_ref().unwrap();
        assert_eq!(document.source_sheet.to_string() == ROOT, which == 0);
        let file = self.output.join(format!("circle-old-scene-{which}.json"));
        let old = if file.exists() {
            read(&file)
        } else {
            let sheet = read(
                &self
                    .root
                    .join(format!("schematic/sheets/{}.json", document.source_sheet)),
            );
            json!({"sheet":sheet,"document":document})
        };
        let now = json!({"board_graphics":scene.board_graphics,"board_texts":scene.board_texts,"text_geometry":scene.board_text_geometries});
        let mut ink = serde_json::Map::new();
        for (id, placed) in old["sheet"]["symbols"].as_object().unwrap() {
            let value = instance_ink(&now, id, placed);
            assert!(!value["graphics"].as_array().unwrap().is_empty());
            ink.insert(id.clone(), value);
        }
        if old.get("ink").is_some() {
            assert_eq!(
                json!(ink),
                old["ink"],
                "all old body/pin/text/glyph ink stays frozen on sheet{which}"
            );
        } else {
            let mut record = old;
            record["ink"] = json!(ink);
            write(&file, &record);
        }
    }
    fn circle_frozen(&self) {
        let intent = read(&self.output.join("circle-intent.json"));
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
        for user in intent["users"].as_array().unwrap() {
            let id = user["id"].as_str().unwrap().parse().unwrap();
            let actual = model
                .materialized_placed_symbol_definition(id, SYMBOL.parse().unwrap())
                .unwrap()
                .unwrap();
            assert_eq!(
                actual, user["definition"],
                "authenticated old Symbol definitions must not consume the new Circle"
            );
            for pin in user["pins"].as_array().unwrap() {
                let origin: ElectricalOccurrence =
                    serde_json::from_value(pin["origin"].clone()).unwrap();
                let run = snap.schematic_run(snap.revision(), &origin).unwrap();
                assert_eq!(json!(run.subject_id), pin["net"]);
                assert_eq!(json!(run.owned), pin["members"]);
            }
        }
        let future = self.output.join("circle-new-placement.json");
        let new = if future.exists() {
            Some(read(&future))
        } else {
            None
        };
        let before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(intent["sources"].clone()).unwrap();
        for (path, bytes) in before {
            if path == format!("pool/symbols/{SYMBOL}.json")
                || path.starts_with(".datum/electrical_identities/")
            {
                continue;
            }
            let current = std::fs::read(self.root.join(&path)).unwrap();
            if path == format!("schematic/sheets/{ROOT}.json")
                && let Some(new) = &new
            {
                let mut sheet: Value = serde_json::from_slice(&current).unwrap();
                assert_eq!(
                    sheet["symbols"]
                        .as_object_mut()
                        .unwrap()
                        .remove(new["placed"]["uuid"].as_str().unwrap()),
                    Some(new["placed"].clone())
                );
                assert_eq!(sheet, serde_json::from_slice::<Value>(&bytes).unwrap());
            } else {
                assert_eq!(current, bytes, "unrelated canonical source {path}");
            }
        }
    }
    fn circle_saved(&self, radius: Option<i64>) {
        assert_eq!(
            read(&self.root.join(format!("pool/symbols/{SYMBOL}.json"))),
            self.circle_expected(radius)
        );
        self.circle_frozen();
    }
    fn circle_future(&self) {
        let sheet = read(&self.root.join(format!("schematic/sheets/{ROOT}.json")));
        let placed = sheet["symbols"]
            .as_object()
            .unwrap()
            .values()
            .find(|p| p["reference"] == "TC3")
            .unwrap();
        assert_eq!(placed["position"], json!({"x":0,"y":-5000000}));
        assert_eq!(placed["part"], PART);
        assert_eq!(placed["lib_id"], SYMBOL);
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let id: Uuid = placed["uuid"].as_str().unwrap().parse().unwrap();
        let definition = model
            .materialized_placed_symbol_definition(id, SYMBOL.parse().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(definition, self.circle_expected(Some(2000000)));
        for pin in placed["pins"].as_array().unwrap() {
            let anchor = definition["pin_anchors"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["pin"] == pin["library_pin"])
                .unwrap();
            assert_eq!(pin["position"]["x"], anchor["position"]["x"]);
            assert_eq!(
                pin["position"]["y"].as_i64().unwrap(),
                anchor["position"]["y"].as_i64().unwrap() - 5000000
            );
        }
        assert_eq!(placed["pins"].as_array().unwrap().len(), 2);
        let ci = model
            .component_instances
            .values()
            .find(|ci| ci.placed_symbol_refs.contains(&id))
            .unwrap();
        assert_eq!(ci.part_ref, Some(PART.parse().unwrap()));
        let record = json!({"placed":placed,"ci":ci,"definition":definition});
        let path = self.output.join("circle-new-placement.json");
        if path.exists() {
            assert_eq!(
                read(&path),
                record,
                "cold new placement retains Circle and identities"
            );
        } else {
            write(&path, &record);
        }
        self.circle_frozen();
        self.circle_old_scene(0);
        let state = self.app.runtime.as_ref().unwrap().workspace();
        circle_body(
            &datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap(),
            placed["uuid"].as_str().unwrap(),
        );
    }
    fn circle_choice(&self) -> usize {
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .position(|c| {
                c.explicit_part
                    .as_ref()
                    .is_some_and(|p| p.part.to_string() == PART)
            })
            .unwrap()
    }
    pub(super) fn symbol_circle(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            0 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
                let mut users = Vec::new();
                for which in 0..2 {
                    let old = read(&self.output.join(format!("circle-old-scene-{which}.json")));
                    let path: Vec<Uuid> =
                        serde_json::from_value(old["document"]["instance_path"].clone()).unwrap();
                    for placed in old["sheet"]["symbols"]
                        .as_object()
                        .unwrap()
                        .values()
                        .filter(|p| p["lib_id"] == SYMBOL)
                    {
                        let id: Uuid = placed["uuid"].as_str().unwrap().parse().unwrap();
                        let definition = model
                            .materialized_placed_symbol_definition(id, SYMBOL.parse().unwrap())
                            .unwrap()
                            .unwrap();
                        let pins: Vec<_> = placed["pins"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|pin| {
                                let origin = ElectricalOccurrence {
                                    class: "pins".into(),
                                    source_id: pin["uuid"].as_str().unwrap().parse().unwrap(),
                                    instance_path: path.clone(),
                                };
                                let run = snap.schematic_run(snap.revision(), &origin).unwrap();
                                json!({"origin":origin,"net":run.subject_id,"members":run.owned})
                            })
                            .collect();
                        users.push(json!({"id":id,"definition":definition,"pins":pins}));
                    }
                }
                assert_eq!(users.len(), 2);
                write(
                    &self.output.join("circle-intent.json"),
                    &json!({"original":read(&self.root.join(format!("pool/symbols/{SYMBOL}.json"))),"sources":canonical(&self.root),"users":users}),
                );
            }
            1 => self.circle_saved(Some(1500000)),
            2 => self.circle_saved(Some(2000000)),
            3 => self.circle_saved(None),
            4 => {
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
                assert!(
                    d.refusal
                        .as_ref()
                        .is_some_and(|r| r.contains("positive radius"))
                );
                assert!(d.preview.is_none());
                self.circle_saved(Some(2000000));
            }
            5 => {
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
                for (k, v) in [
                    ("circle_x", "0"),
                    ("circle_y", "0"),
                    ("circle_radius", "2000000"),
                    ("circle_width", "100000"),
                ] {
                    assert_eq!(d.value(k), v);
                }
            }
            10 | 11 => self.circle_old_scene(stage - 10),
            30 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            31 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            32 => {
                let index = self.circle_choice();
                let offset = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps.insert(self.stage + 1, Step::SymbolCircle(32));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            33 => self.control(
                events,
                HitTarget::SchematicLibraryChoice(self.circle_choice()),
            ),
            34 => self.control(events, HitTarget::SchematicPlace),
            35 => self.placement_mouse(events, PointNm { x: 0, y: -5000000 }, false),
            36 => {
                let state = self.app.runtime.as_ref().unwrap().workspace();
                circle_body(
                    &datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
                        .unwrap(),
                    &Uuid::nil().to_string(),
                );
                self.circle_frozen();
                self.circle_old_scene(0);
            }
            37 => self.placement_mouse(events, PointNm { x: 0, y: -5000000 }, true),
            38 => self.circle_future(),
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires saved TERU circuit and owned isolated display"]
fn ordinary_saved_circle_add_edit_remove_history_cold_and_new_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let c = Step::KeyControl;
    let s = Step::SymbolCircle;
    let f = |k, v: &str| Step::Field(k, v.into());
    let mut steps = vec![
        Step::Snapshot,
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(false)),
        s(10),
        Step::PaneContext(Navigate(true)),
        s(11),
        Step::Open,
        Step::Kind(1),
        Step::Search("Teru component".into()),
        Step::Select("Teru component".into()),
        s(0),
        c(C::Edit),
        c(C::RemoveRect),
        c(C::AddCircle),
        f("circle_radius", "1.5mm"),
        c(C::Preview),
        Step::Capture("circle-add-preview"),
        c(C::Cancel),
        Step::Invariant,
        c(C::Edit),
        c(C::RemoveRect),
        c(C::AddCircle),
        f("circle_radius", "1.5mm"),
        c(C::Preview),
        c(C::Commit),
        s(1),
        c(C::Edit),
        c(C::Circle(0)),
        f("circle_radius", "2mm"),
        c(C::Preview),
        Step::Capture("circle-edit-preview"),
        c(C::Commit),
        s(2),
        Step::Snapshot,
        c(C::Edit),
        c(C::Circle(0)),
        f("circle_radius", "0"),
        c(C::Preview),
        s(4),
        c(C::RemoveCircle),
        c(C::Preview),
        Step::Capture("circle-remove-preview"),
        c(C::Cancel),
        Step::Invariant,
        c(C::Edit),
        c(C::Circle(0)),
        c(C::RemoveCircle),
        c(C::Preview),
        c(C::Commit),
        s(3),
        Step::History(false),
        s(2),
        Step::History(true),
        s(3),
        Step::History(false),
        s(2),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(false)),
        s(10),
        Step::PaneContext(Navigate(true)),
        s(11),
        Step::Save,
        Step::Reopen,
        s(2),
        Step::Open,
        Step::Kind(1),
        Step::Search("Teru component".into()),
        Step::Select("Teru component".into()),
        c(C::Edit),
        c(C::Circle(0)),
        s(5),
        Step::Capture("circle-cold-form"),
        c(C::Cancel),
    ];
    for child in [false, true] {
        steps.extend([
            Step::PaneContext(Expand),
            Step::PaneContext(Navigate(child)),
            Step::PinLabelClearance(2),
            Step::PaneContext(Fit),
            s(if child { 11 } else { 10 }),
            Step::Capture(if child {
                "circle-old-child"
            } else {
                "circle-old-root"
            }),
        ]);
    }
    steps.extend([
        Step::PaneContext(Navigate(false)),
        Step::PinLabelClearance(2),
        Step::PaneContext(Fit),
        s(30),
        s(31),
        s(32),
        s(33),
        s(34),
        s(35),
        s(36),
        Step::Capture("circle-new-ghost"),
        s(37),
        s(38),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::PinLabelClearance(2),
        Step::PaneContext(Fit),
        s(38),
        Step::Capture("circle-new-cold"),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::PinLabelClearance(2),
        Step::PaneContext(Fit),
        s(11),
        Step::Capture("circle-old-child-final"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
