//! Author saved Symbol literal text through the ordinary shared form and placement flow.
use super::*;
use eda_engine::substrate::{DesignModel, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;
const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const TARGET: &str = "f22cf09d-67d1-4b71-abae-1129a8e8af8e";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn canonical(root: &Path) -> BTreeMap<String, Vec<u8>> {
    source_bytes(root)
        .into_iter()
        .filter(|(path, _)| {
            !path.starts_with(".datum/")
                || [
                    ".datum/component_instances/",
                    ".datum/electrical_identities/",
                    ".datum/proposals/",
                ]
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .collect()
}
fn authored(before: &Value, phase: usize) -> Value {
    let mut value = before.clone();
    if phase > 0 {
        let rows = value["drawings"].as_array_mut().unwrap();
        rows.push(json!({"Text":{"text":if phase==1 {"DRAFT"} else {"READY"},"position":{"x":if phase==1 {0} else {-500000},"y":if phase==1 {-1000000} else {-2000000}},"rotation":if phase==1 {90} else {0}}}));
        if phase == 1 {
            rows.push(
                json!({"Text":{"text":"REMOVE","position":{"x":500000,"y":2000000},"rotation":0}}),
            );
        }
    }
    value
}
fn assert_text(scene: &datum_gui_protocol::BoardReviewSceneV1, id: &str) {
    let text = scene
        .board_texts
        .iter()
        .find(|t| t.source_object_uuid.as_deref() == Some(id) && t.text == "READY")
        .expect("new placement contains actual revised literal");
    assert_eq!(
        text.position,
        datum_gui_protocol::PointNm {
            x: 3500000,
            y: -4000000
        }
    );
    assert_eq!(text.rotation_degrees, 0);
    assert!(
        scene
            .board_text_geometries
            .iter()
            .any(|g| g.text_uuid == text.text_uuid
                && (!g.glyphs.is_empty() || !g.strokes.is_empty() || !g.fills.is_empty())),
        "revised literal has actual glyph geometry"
    );
}
fn instance_ink(scene: &Value, id: &str, placed: &Value) -> Value {
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
    fn text_graphics_intent(&self) -> Value {
        read(&self.output.join("symbol-text-intent.json"))
    }
    fn text_graphics_frozen(&self, model: &DesignModel) {
        for (id, raw) in self.text_graphics_intent()["old_definitions"]
            .as_object()
            .unwrap()
        {
            assert_eq!(
                model
                    .materialized_placed_symbol_definition(
                        id.parse().unwrap(),
                        SYMBOL.parse().unwrap()
                    )
                    .unwrap(),
                Some(raw.clone())
            );
        }
    }
    fn text_graphics_core(&self, phase: usize) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        let data = self.text_graphics_intent();
        let old = self.original_footprint.as_ref().unwrap();
        assert_eq!(
            read(&self.root.join(format!("pool/symbols/{SYMBOL}.json"))),
            authored(&data["before"], phase)
        );
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        let mut now = canonical(&self.root);
        let path = format!("pool/symbols/{SYMBOL}.json");
        before.remove(&path);
        now.remove(&path);
        assert_eq!(
            now, before,
            "all other authored sources, Unit pins, CI, maps and root/child circuits remain byte-exact"
        );
        assert_eq!(json!(model.component_instances), old["ci"]);
        assert_eq!(json!(model.electrical_identities), old["ei"]);
        for (id, revision) in old["counters"].as_object().unwrap() {
            assert_eq!(
                model.objects[&id.parse().unwrap()].object_revision.0,
                revision.as_u64().unwrap()
            );
        }
        self.text_graphics_frozen(model);
    }
    fn text_graphics_choice(&self) -> usize {
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
    fn text_graphics_old_scene(&self) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let scene =
            datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap();
        assert_eq!(
            scene
                .schematic_document
                .as_ref()
                .unwrap()
                .source_sheet
                .to_string(),
            ROOT
        );
        let before = read(&self.output.join("entity-edit-scene-5.json"));
        let now = json!({"board_graphics":scene.board_graphics,"board_texts":scene.board_texts,"text_geometry":scene.board_text_geometries});
        for (id, placed) in self.original_footprint.as_ref().unwrap()["sheets"][ROOT]["symbols"]
            .as_object()
            .unwrap()
        {
            let expected = instance_ink(&before, id, placed);
            assert!(!expected["graphics"].as_array().unwrap().is_empty());
            assert_eq!(
                instance_ink(&now, id, placed),
                expected,
                "old instance {id} retains exact body, terminals, texts and glyph geometry in actual warm Root preview"
            );
        }
    }
    fn text_graphics_future(&mut self, record: bool) {
        self.text_graphics_old_scene();
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        let old = self.original_footprint.as_ref().unwrap();
        let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
        let source = snap.schematic_source().unwrap();
        let root = &source.sheets[&ROOT.parse().unwrap()];
        let added: Vec<_> = root
            .symbols
            .iter()
            .filter(|(id, _)| old["sheets"][ROOT]["symbols"].get(id.to_string()).is_none())
            .collect();
        assert_eq!(added.len(), 1);
        let (id, placed) = added[0];
        assert_eq!(placed.lib_id.as_deref(), Some(SYMBOL));
        assert_eq!(
            placed.position,
            eda_engine::ir::geometry::Point::new(4000000, -2000000)
        );
        assert_eq!(placed.pins.len(), 1);
        let pin = &placed.pins[0];
        assert_eq!(pin.library_pin, Some(TARGET.parse().unwrap()));
        assert_eq!(pin.name, "Auxiliary supply");
        assert_eq!(
            pin.electrical_type,
            eda_engine::schematic::PinElectricalType::PowerIn
        );
        let data = self.text_graphics_intent();
        let expected = authored(&data["before"], 2);
        assert_eq!(
            pin.position.x - placed.position.x,
            expected["pin_anchors"][0]["position"]["x"]
                .as_i64()
                .unwrap()
        );
        assert_eq!(
            pin.position.y - placed.position.y,
            expected["pin_anchors"][0]["position"]["y"]
                .as_i64()
                .unwrap()
        );
        assert_eq!(
            model
                .materialized_placed_symbol_definition(*id, SYMBOL.parse().unwrap())
                .unwrap(),
            Some(expected.clone())
        );
        let mut sheets = json!(source.sheets);
        sheets[ROOT]["symbols"]
            .as_object_mut()
            .unwrap()
            .remove(&id.to_string());
        assert_eq!(
            sheets, old["sheets"],
            "every old symbol, pin and sheet field stays unchanged"
        );
        for (id, raw) in old["ci"].as_object().unwrap() {
            assert_eq!(json!(model.component_instances[&id.parse().unwrap()]), *raw);
        }
        let ci = model
            .component_instances
            .values()
            .find(|ci| ci.placed_symbol_refs.contains(id));
        assert_eq!(
            model.component_instances.len(),
            old["ci"].as_object().unwrap().len() + usize::from(ci.is_some())
        );
        let mut allowed_ei_files = BTreeMap::new();
        for (id, raw) in old["ei"].as_object().unwrap() {
            let eid: Uuid = id.parse().unwrap();
            let actual = json!(model.electrical_identities[&eid]);
            let mut expected = raw.clone();
            if actual != *raw
                && raw["identity"]["kind"] == "net"
                && raw["identity"]["anchor_reason"] == "unclaimed_final_group"
            {
                expected["identity"]["anchor_reason"] = json!("surviving_anchor");
                expected["identity"]["predecessors"] = json!([eid]);
                expected["object_revision"] = json!(raw["object_revision"].as_u64().unwrap() + 1);
                assert!(model.journal.iter().flat_map(|tx|&tx.operations).any(|op|matches!(op,eda_engine::substrate::Operation::SetElectricalIdentity{previous,record}if json!(previous)==*raw&&json!(record)==expected)));
                allowed_ei_files.insert(
                    format!(".datum/electrical_identities/{eid}.json"),
                    expected.clone(),
                );
            }
            assert_eq!(
                actual, expected,
                "only an actual initial Net lineage transition may change old EI"
            );
        }
        let files: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        for (path, bytes) in files {
            if path == format!("pool/symbols/{SYMBOL}.json") {
                continue;
            }
            let current = std::fs::read(self.root.join(&path)).unwrap();
            if path == format!("schematic/sheets/{ROOT}.json") {
                let mut value: Value = serde_json::from_slice(&current).unwrap();
                assert_eq!(
                    value["symbols"]
                        .as_object_mut()
                        .unwrap()
                        .remove(&id.to_string()),
                    Some(json!(placed))
                );
                assert_eq!(value, serde_json::from_slice::<Value>(&bytes).unwrap());
            } else if let Some(record) = allowed_ei_files.get(&path) {
                let mut expected: Value = serde_json::from_slice(&bytes).unwrap();
                expected["record"] = record.clone();
                assert_eq!(serde_json::from_slice::<Value>(&current).unwrap(), expected);
            } else {
                assert_eq!(current, bytes, "old source {path}");
            }
        }
        self.text_graphics_frozen(model);
        assert!(ci.is_none() && placed.part.is_none() && placed.gate.is_none());
        let scene = datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
            .expect("new placement is displayed in the focused Root pane");
        assert_eq!(
            scene.schematic_document.as_ref(),
            state
                .schematic_documents
                .rows
                .iter()
                .find(|row| row.parent.is_none())
                .map(|row| &row.key)
        );
        assert_text(&scene, &id.to_string());
        let persisted = json!({"placed":placed,"ci":ci,"definition":expected});
        if record {
            self.edited_footprint = Some(persisted.clone());
            std::fs::write(
                self.output.join("symbol-text-consumer.json"),
                serde_json::to_vec_pretty(&persisted).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(self.edited_footprint.as_ref(), Some(&persisted));
        }
    }
    pub(super) fn symbol_text(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            0 => {
                let model = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .basis
                    .as_ref()
                    .unwrap();
                let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
                let data = self.text_graphics_intent();
                let unit = data["before"]["unit"].as_str().unwrap();
                let logical = read(&self.root.join(format!("pool/units/{unit}.json")));
                let counters: BTreeMap<_, _> = std::iter::once(unit)
                    .chain(
                        logical["pins"]
                            .as_object()
                            .unwrap()
                            .keys()
                            .map(String::as_str),
                    )
                    .map(|id| (id, model.objects[&id.parse().unwrap()].object_revision.0))
                    .collect();
                self.original_footprint = Some(
                    json!({"files":canonical(&self.root),"ci":model.component_instances,"ei":model.electrical_identities,"sheets":snap.schematic_source().unwrap().sheets,"counters":counters}),
                );
                self.text_graphics_core(0);
            }
            1 | 2 | 3 | 20 | 21 => {
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
                match stage {
                    1 => {
                        assert_eq!(d.electrical.graphics.selected, Some(2));
                        assert!(d.fields.iter().any(|f| f.key == "text_literal"));
                    }
                    2 | 3 => {
                        assert_eq!(
                            d.preview,
                            Some(authored(&self.text_graphics_intent()["before"], stage - 1))
                        );
                    }
                    20 => {
                        assert_eq!(d.electrical.graphics.selected, Some(2));
                        for (key, text) in [
                            ("text_literal", "READY"),
                            ("text_x", "-500000"),
                            ("text_y", "-2000000"),
                            ("text_rotation", "0"),
                        ] {
                            assert_eq!(d.value(key), text);
                        }
                        assert!(d.request().unwrap().fields.is_empty());
                    }
                    21 => {
                        assert_eq!(
                            d.electrical
                                .graphics
                                .rows
                                .keys()
                                .copied()
                                .collect::<Vec<_>>(),
                            [0, 1, 2]
                        );
                        assert_eq!(d.electrical.graphics.selected, None);
                        assert!(d.fields.iter().all(|f| !f.key.starts_with("text_")));
                        let f = d.request().unwrap().fields;
                        assert_eq!(f["text_removals"][0]["index"], 3);
                        assert_eq!(f["text_edits"][0]["index"], 2);
                    }
                    _ => unreachable!(),
                }
            }
            10..=12 => self.text_graphics_core(stage - 10),
            30 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            31 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            32 => {
                let index = self.text_graphics_choice();
                let offset = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps.insert(self.stage + 1, Step::SymbolText(32));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            33 => {
                let index = self.text_graphics_choice();
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            34 => self.control(events, HitTarget::SchematicPlace),
            35 | 36 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: 4000000,
                    y: -2000000,
                },
                stage == 36,
            ),
            37 => self.text_graphics_future(true),
            38 => self.text_graphics_future(false),
            39 => {
                let l = &self.app.runtime.as_ref().unwrap().workspace().ui.layout;
                if l.zoomed != Some(l.focused) {
                    assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                        "View",
                        "Maximize Pane",
                        EntrySurface::MenuPointer
                    ));
                }
            }
            42 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "View",
                "Zoom Out",
                EntrySurface::MenuPointer
            )),
            40 => self.text_graphics_old_scene(),
            41 => {
                self.text_graphics_old_scene();
                let state = self.app.runtime.as_ref().unwrap().workspace();
                let scene =
                    datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
                        .unwrap();
                assert_text(&scene, &Uuid::nil().to_string());
            }
            _ => panic!("unknown Symbol Text stage"),
        }
    }
}
#[test]
#[ignore = "requires native root/child captures and owned isolated Wayland"]
fn ordinary_saved_symbol_text_add_edit_remove_cancel_history_cold_and_new_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = read(&root.join(format!("pool/symbols/{SYMBOL}.json")));
    assert_eq!(before["drawings"].as_array().unwrap().len(), 2);
    assert!(before["drawings"][0].get("Line").is_some());
    assert!(before["drawings"][1].get("Rect").is_some());
    assert_eq!(before["pin_anchors"][0]["pin"], TARGET);
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let frozen: BTreeMap<_, _> = snap
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .flat_map(|s| s.symbols.values())
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
        output.join("symbol-text-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"old_definitions":frozen,"setup_transactions":0}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::SymbolText(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(1),
        Step::Search("Shared body".into()),
        Step::Select("Shared body".into()),
    ];
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::Control(C::AddText),
            Step::SymbolText(1),
            Step::Field("text_literal", "DRAFT".into()),
            Step::Field("text_x", "0".into()),
            Step::Field("text_y", "-1mm".into()),
            Step::Field("text_rotation", "90".into()),
            Step::Control(C::AddText),
            Step::Field("text_literal", "REMOVE".into()),
            Step::Field("text_x", "0.5mm".into()),
            Step::Field("text_y", "2mm".into()),
            Step::Control(C::Preview),
            Step::SymbolText(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-text-added-preview"),
                Step::Control(C::Commit),
                Step::SymbolText(11),
                Step::Capture("symbol-text-added"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::SymbolText(10),
                Step::Invariant,
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolText(10),
        Step::History(true),
        Step::SymbolText(11),
        Step::Snapshot,
    ]);
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::Control(C::Text(2)),
            Step::Field("text_literal", "READY".into()),
            Step::Field("text_x", "-0.5mm".into()),
            Step::Field("text_y", "-2mm".into()),
            Step::Field("text_rotation", "0".into()),
            Step::Control(C::Text(3)),
            Step::Field("text_rotation", "invalid".into()),
            Step::Control(C::RemoveText),
            Step::SymbolText(21),
            Step::Control(C::Text(2)),
            Step::Control(C::Preview),
            Step::SymbolText(3),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-text-corrected-preview"),
                Step::Control(C::Commit),
                Step::SymbolText(12),
                Step::Capture("symbol-text-corrected"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::SymbolText(11),
                Step::Invariant,
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolText(11),
        Step::History(true),
        Step::SymbolText(12),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolText(12),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::Capture("symbol-text-cold-root"),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Open,
        Step::Kind(1),
        Step::Search("Shared body".into()),
        Step::Select("Shared body".into()),
        Step::Control(C::Edit),
        Step::Control(C::Text(2)),
        Step::SymbolText(20),
        Step::Capture("symbol-text-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SymbolText(40),
        Step::SymbolText(30),
        Step::SymbolText(31),
        Step::SymbolText(32),
        Step::SymbolText(33),
        Step::SymbolText(34),
        Step::SymbolText(39),
        Step::PaneContext(Fit),
        Step::SymbolText(35),
        Step::PaneContext(Fit),
        Step::SymbolText(42),
        Step::SymbolText(41),
        Step::Invariant,
        Step::Capture("symbol-text-new-ghost"),
        Step::SymbolText(36),
        Step::SymbolText(37),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::SymbolText(39),
        Step::PaneContext(Fit),
        Step::SymbolText(38),
        Step::Capture("symbol-text-new-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
