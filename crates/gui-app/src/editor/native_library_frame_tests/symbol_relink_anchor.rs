//! Correct a saved logical-pin association through ordinary shared controls.
use super::*;
use eda_engine::substrate::{DesignModel, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;
const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const OLD: &str = "35c36e99-78c5-4bb5-a50e-19a2bf0d47ed";
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
fn corrected(before: &Value) -> Value {
    let mut value = before.clone();
    let row = value["pin_anchors"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| {
            row["pin"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(OLD.parse().unwrap())
        })
        .unwrap();
    row["pin"] = json!(TARGET);
    value
}
impl Fixture {
    fn correction_intent(&self) -> Value {
        read(&self.output.join("anchor-correction-intent.json"))
    }
    fn correction_frozen(&self, model: &DesignModel) {
        for (id, raw) in self.correction_intent()["old_definitions"]
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
    fn correction_core(&self, changed: bool) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        let data = self.correction_intent();
        let old = self.original_footprint.as_ref().unwrap();
        assert_eq!(
            read(&self.root.join(format!("pool/symbols/{SYMBOL}.json"))),
            if changed {
                corrected(&data["before"])
            } else {
                data["before"].clone()
            }
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
        self.correction_frozen(model);
    }
    fn correction_choice(&self) -> usize {
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
    fn correction_future(&mut self, record: bool) {
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
            eda_engine::ir::geometry::Point::new(0, 2000000)
        );
        assert_eq!(placed.pins.len(), 1);
        let pin = &placed.pins[0];
        assert_eq!(pin.library_pin, Some(TARGET.parse().unwrap()));
        assert_eq!(pin.name, "Auxiliary supply");
        assert_eq!(
            pin.electrical_type,
            eda_engine::schematic::PinElectricalType::PowerIn
        );
        let data = self.correction_intent();
        let expected = corrected(&data["before"]);
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
        self.correction_frozen(model);
        let persisted = json!({"placed":placed,"ci":ci,"definition":expected});
        if record {
            self.edited_footprint = Some(persisted.clone());
            std::fs::write(
                self.output.join("anchor-correction-consumer.json"),
                serde_json::to_vec_pretty(&persisted).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(self.edited_footprint.as_ref(), Some(&persisted));
        }
    }
    pub(super) fn symbol_relink_anchor(&mut self, events: &ActiveEventLoop, stage: usize) {
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
                let data = self.correction_intent();
                let counters: BTreeMap<_, _> =
                    [data["before"]["unit"].as_str().unwrap(), OLD, TARGET]
                        .into_iter()
                        .map(|id| (id, model.objects[&id.parse().unwrap()].object_revision.0))
                        .collect();
                self.original_footprint = Some(
                    json!({"files":canonical(&self.root),"ci":model.component_instances,"ei":model.electrical_identities,"sheets":snap.schematic_source().unwrap().sheets,"counters":counters}),
                );
                self.correction_core(false);
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
                assert!(!d.create && !d.electrical.new_anchor);
                assert_eq!(d.electrical.anchor_id, Some(OLD.parse().unwrap()));
                assert_eq!(d.value("anchor_pin"), OLD);
                assert!(d.fields.iter().any(|f| f.key == "anchor_pin"));
                assert!(d.fields.iter().all(|f| f.key != "unit"));
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
                assert_eq!(d.electrical.anchor_id, Some(OLD.parse().unwrap()));
                assert!(!d.electrical.new_anchor);
                assert_eq!(d.value("anchor_pin"), TARGET);
                assert_eq!(
                    json!(d.request().unwrap().fields),
                    json!({"anchor_relinks":{OLD:TARGET}})
                );
                assert!(d.electrical.anchors.contains_key(OLD));
                assert!(!d.electrical.anchors.contains_key(TARGET));
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
                assert!(
                    d.refusal
                        .as_ref()
                        .is_some_and(|s| s.contains("before removing the same row")),
                    "{:?}",
                    d.refusal
                );
                assert_eq!(d.value("anchor_pin"), TARGET);
                assert_eq!(d.electrical.anchor_id, Some(OLD.parse().unwrap()));
                self.correction_core(false);
            }
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
                assert_eq!(
                    d.preview,
                    Some(corrected(&self.correction_intent()["before"]))
                );
            }
            5 | 6 => self.correction_core(stage == 5),
            7 => {
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
                assert_eq!(d.electrical.anchor_id, Some(TARGET.parse().unwrap()));
                assert_eq!(d.value("anchor_pin"), TARGET);
                assert!(!d.electrical.new_anchor);
                assert!(d.request().unwrap().fields.is_empty());
            }
            8 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            16 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            9 => {
                let index = self.correction_choice();
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let offset = s.schematic_authoring.choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps
                        .insert(self.stage + 1, Step::SymbolRelinkAnchor(9));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            10 => {
                let index = self.correction_choice();
                let choice = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .choices[index];
                assert!(choice.refusal.is_none());
                assert_eq!(choice.pins.len(), 1);
                assert_eq!(choice.pins[0].library_pin, Some(TARGET.parse().unwrap()));
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            11 => self.control(events, HitTarget::SchematicPlace),
            12 | 13 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm { x: 0, y: 2000000 },
                stage == 13,
            ),
            14 => self.correction_future(true),
            15 => self.correction_future(false),
            _ => panic!("unknown anchor correction stage"),
        }
    }
}
#[test]
#[ignore = "requires native root/child captures and owned isolated Wayland"]
fn ordinary_saved_symbol_correct_anchor_pin_cancel_history_cold_and_new_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = read(&root.join(format!("pool/symbols/{SYMBOL}.json")));
    assert_eq!(before["pin_anchors"].as_array().unwrap().len(), 1);
    assert_eq!(before["pin_anchors"][0]["pin"], json!(OLD));
    let unit = read(&root.join(format!(
        "pool/units/{}.json",
        before["unit"].as_str().unwrap()
    )));
    assert_eq!(unit["pins"][TARGET]["name"], "Auxiliary supply");
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let frozen: BTreeMap<_, _> = snap
        .schematic_source()
        .unwrap()
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
        output.join("anchor-correction-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"old_definitions":frozen,"setup_transactions":0}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::SymbolRelinkAnchor(0),
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
            Step::Control(C::Anchor(0)),
            Step::SymbolRelinkAnchor(1),
        ]);
        if !commit {
            steps.push(Step::Capture("anchor-correction-before"));
        }
        steps.extend([
            Step::Choice("anchor_pin", "Auxiliary supply".into()),
            Step::Control(C::Anchor(0)),
            Step::SymbolRelinkAnchor(2),
        ]);
        if !commit {
            steps.extend([
                Step::Control(C::RemoveAnchor),
                Step::SymbolRelinkAnchor(3),
                Step::Invariant,
                Step::Capture("anchor-correction-conflict"),
            ]);
        }
        steps.extend([
            Step::Control(C::Preview),
            Step::SymbolRelinkAnchor(4),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("anchor-correction-preview"),
                Step::Control(C::Commit),
                Step::SymbolRelinkAnchor(5),
                Step::Capture("anchor-correction-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::SymbolRelinkAnchor(6),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolRelinkAnchor(6),
        Step::History(true),
        Step::SymbolRelinkAnchor(5),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRelinkAnchor(5),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::Capture("anchor-correction-cold-root"),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Open,
        Step::Kind(1),
        Step::Search("Shared body".into()),
        Step::Select("Shared body".into()),
        Step::Control(C::Edit),
        Step::Control(C::Anchor(0)),
        Step::SymbolRelinkAnchor(7),
        Step::Capture("anchor-correction-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::SymbolRelinkAnchor(8),
        Step::SymbolRelinkAnchor(16),
        Step::SymbolRelinkAnchor(9),
        Step::SymbolRelinkAnchor(10),
        Step::SymbolRelinkAnchor(11),
        Step::SymbolRelinkAnchor(12),
        Step::Invariant,
        Step::Capture("anchor-correction-new-ghost"),
        Step::SymbolRelinkAnchor(13),
        Step::SymbolRelinkAnchor(14),
        Step::Capture("anchor-correction-new-committed"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRelinkAnchor(15),
        Step::Capture("anchor-correction-new-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
