//! Remove saved anchors through presented controls while old placed users stay frozen.
use super::*;
use eda_engine::substrate::{DesignModel, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;
const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
const PIN: &str = "f22cf09d-67d1-4b71-abae-1129a8e8af8e";
const RETAINED: &str = "35c36e99-78c5-4bb5-a50e-19a2bf0d47ed";
const PIN_NAME: &str = "Auxiliary supply";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
const PART: &str = "88175c8c-8479-4291-8ea2-3ccac699cc60";
const GATE: &str = "df2605d7-853b-40b0-ba7e-4afff49a249e";
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

fn intent(output: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(output.join("symbol-remove-anchor-intent.json")).unwrap())
        .unwrap()
}
fn reduced(before: &Value) -> Value {
    let mut v = before.clone();
    v["pin_anchors"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["pin"] != json!(PIN));
    v
}
fn members(model: &DesignModel) -> Value {
    let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
    let values: BTreeMap<_, _> = snap
        .net_ids()
        .into_iter()
        .map(|id| {
            let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                panic!("Net")
            };
            (
                id,
                snap.schematic_run(snap.revision(), anchor).unwrap().owned,
            )
        })
        .collect();
    json!(values)
}

impl Fixture {
    fn removed_anchor_frozen(&self, model: &DesignModel) {
        for (id, raw) in intent(&self.output)["old_definitions"].as_object().unwrap() {
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
    fn removed_anchor_core(&self, removed: bool) {
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
        let data = intent(&self.output);
        let old = self.original_footprint.as_ref().unwrap();
        let now: Value = serde_json::from_slice(
            &std::fs::read(self.root.join(format!("pool/symbols/{SYMBOL}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            now,
            if removed {
                reduced(&data["before"])
            } else {
                data["before"].clone()
            }
        );
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        let mut actual = canonical(&self.root);
        let path = format!("pool/symbols/{SYMBOL}.json");
        before.remove(&path);
        actual.remove(&path);
        assert_eq!(
            actual, before,
            "all other25 canonical sources, Unit pin and independent Supply Symbol stay byte-exact"
        );
        assert!(model.objects.contains_key(&PIN.parse().unwrap()));
        assert_eq!(json!(model.component_instances), old["ci"]);
        assert_eq!(json!(model.electrical_identities), old["ei"]);
        assert_eq!(members(model), old["members"]);
        self.removed_anchor_frozen(model);
    }
    fn removed_anchor_choice(&self) -> usize {
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
                    .is_some_and(|p| p.part.to_string() == PART && p.gate.to_string() == GATE)
            })
            .unwrap()
    }
    fn removed_anchor_future(&mut self, record: bool) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        let old = self.original_footprint.as_ref().unwrap();
        let data = intent(&self.output);
        let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
        let source = snap.schematic_source().unwrap();
        let mut sheets = serde_json::to_value(&source.sheets).unwrap();
        let root = &source.sheets[&ROOT.parse().unwrap()];
        let added: Vec<_> = root
            .symbols
            .iter()
            .filter(|(id, _)| old["sheets"][ROOT]["symbols"].get(id.to_string()).is_none())
            .collect();
        assert_eq!(added.len(), 1);
        let (id, placed) = added[0];
        assert_eq!(placed.part, Some(PART.parse().unwrap()));
        assert_eq!(placed.gate, Some(GATE.parse().unwrap()));
        assert_eq!(
            placed.position,
            eda_engine::ir::geometry::Point {
                x: 2000000,
                y: 4000000
            }
        );
        assert_eq!(placed.pins.len(), 1);
        assert_eq!(placed.pins[0].library_pin, Some(RETAINED.parse().unwrap()));
        assert_eq!(placed.pins[0].name, "Input");
        assert_eq!(
            placed.pins[0].electrical_type,
            eda_engine::schematic::PinElectricalType::Input
        );
        assert_eq!(
            placed.pins[0].position,
            eda_engine::ir::geometry::Point::new(0, 4000000)
        );
        assert_eq!(placed.rotation, 0);
        assert!(!placed.mirrored);
        sheets[ROOT]["symbols"]
            .as_object_mut()
            .unwrap()
            .remove(&id.to_string());
        assert_eq!(
            sheets, old["sheets"],
            "every old symbol/pin/pose/sheet field is unchanged"
        );
        let expected = reduced(&data["before"]);
        assert_eq!(
            model
                .materialized_placed_symbol_definition(*id, SYMBOL.parse().unwrap())
                .unwrap(),
            Some(expected.clone())
        );
        let ci = model
            .component_instances
            .values()
            .find(|ci| ci.placed_symbol_refs.contains(id))
            .unwrap();
        assert_eq!(
            ci.library_bindings.len(),
            1,
            "ordinary initial Part placement stays Part-only"
        );
        let old_cis: BTreeMap<Uuid, Value> = serde_json::from_value(old["ci"].clone()).unwrap();
        assert_eq!(model.component_instances.len(), old_cis.len() + 1);
        for (id, raw) in old_cis {
            assert_eq!(
                serde_json::to_value(&model.component_instances[&id]).unwrap(),
                raw
            );
        }
        for (id, raw) in old["ei"].as_object().unwrap() {
            let id: Uuid = id.parse().unwrap();
            let actual = serde_json::to_value(&model.electrical_identities[&id]).unwrap();
            let mut expected = raw.clone();
            if actual != *raw
                && raw["identity"]["kind"] == "net"
                && raw["identity"]["anchor_reason"] == "unclaimed_final_group"
            {
                expected["identity"]["anchor_reason"] = json!("surviving_anchor");
                expected["identity"]["predecessors"] = json!([id]);
                expected["object_revision"] = json!(raw["object_revision"].as_u64().unwrap() + 1);
                assert!(model.journal.iter().flat_map(|t|&t.operations).any(|op|matches!(op,eda_engine::substrate::Operation::SetElectricalIdentity{previous,record} if serde_json::to_value(previous).unwrap()==*raw && serde_json::to_value(record).unwrap()==expected)),"only the existing canonically recorded initial lineage transition is qualified, after new placement");
            }
            assert_eq!(actual, expected);
        }
        let current = members(model);
        for (id, expected) in old["members"].as_object().unwrap() {
            assert_eq!(
                &current[id], expected,
                "old actual Net membership stays exact"
            );
        }
        let files: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        for (path, bytes) in files {
            if path == format!("pool/symbols/{SYMBOL}.json")
                || path.starts_with(".datum/electrical_identities/")
            {
                continue;
            }
            let actual = std::fs::read(self.root.join(&path)).unwrap();
            if path == format!("schematic/sheets/{ROOT}.json") {
                let mut current: Value = serde_json::from_slice(&actual).unwrap();
                current["symbols"]
                    .as_object_mut()
                    .unwrap()
                    .remove(&id.to_string());
                assert_eq!(current, serde_json::from_slice::<Value>(&bytes).unwrap());
            } else {
                assert_eq!(actual, bytes, "old source {path}");
            }
        }
        self.removed_anchor_frozen(model);
        let persisted = json!({"symbol":placed,"ci":ci,"definition":expected});
        if record {
            self.edited_footprint = Some(persisted);
        } else {
            assert_eq!(self.edited_footprint.as_ref(), Some(&persisted));
        }
    }

    pub(super) fn symbol_remove_anchor(&mut self, events: &ActiveEventLoop, stage: usize) {
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
                self.original_footprint = Some(
                    json!({"files":canonical(&self.root),"ci":model.component_instances,"ei":model.electrical_identities,"members":members(model),"sheets":snap.schematic_source().unwrap().sheets}),
                );
                self.removed_anchor_core(false);
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
                assert_eq!(d.electrical.anchor_id, Some(PIN.parse().unwrap()));
                assert!(!d.electrical.new_anchor);
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
                assert!(d.request().is_err());
                assert_eq!(d.value("anchor_x"), "invalid selected pose");
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
                assert_eq!(
                    json!(d.request().unwrap().fields),
                    json!({"anchor_removals":[PIN]})
                );
                assert_eq!(d.electrical.anchor_id, None);
                assert!(!d.electrical.new_anchor);
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
                assert!(d.request().unwrap_err().contains("removed in this draft"));
                assert_eq!(d.value("anchor_pin"), PIN);
                assert!(d.preview.is_none());
                assert_eq!(d.electrical.anchors.len(), 1);
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
                assert_eq!(d.preview, Some(reduced(&intent(&self.output)["before"])));
                assert_eq!(d.request().unwrap().fields.len(), 1);
            }
            6 => self.removed_anchor_core(true),
            7 => self.removed_anchor_core(false),
            8 => {
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
                assert!(d.request().unwrap().fields.is_empty());
                assert_eq!(d.electrical.anchors.len(), 1);
                assert!(!d.electrical.new_anchor);
                assert!(
                    !self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .presented_hits
                        .regions()
                        .iter()
                        .any(|h| matches!(h.target, HitTarget::Library(C::RemoveAnchor)))
                );
            }
            10 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            11 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            12 => {
                if self.removed_anchor_choice() >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            13 => {
                let index = self.removed_anchor_choice();
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let selected = s.schematic_authoring.choices[index]
                    .explicit_part
                    .as_ref()
                    .unwrap();
                assert_eq!(selected.symbol.to_string(), SYMBOL);
                assert_eq!(
                    selected.symbol_revision,
                    s.library.basis.as_ref().unwrap().objects[&SYMBOL.parse().unwrap()]
                        .object_revision
                        .0
                );
                std::fs::write(
                    self.output.join("symbol-remove-anchor-choice.json"),
                    serde_json::to_vec_pretty(selected).unwrap(),
                )
                .unwrap();
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            14 => self.control(events, HitTarget::SchematicPlace),
            15 | 16 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: 2000000,
                    y: 4000000,
                },
                stage == 16,
            ),
            17 => self.removed_anchor_future(true),
            18 => self.removed_anchor_future(false),
            _ => panic!("unknown saved anchor removal stage"),
        }
    }
}
#[test]
#[ignore = "requires native root/child and frozen two-pin placement in owned isolated Wayland"]
fn ordinary_saved_symbol_remove_anchor_cancel_history_cold_and_new_retained_pin_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let symbol = SYMBOL.parse().unwrap();
    let before = eda_engine::api::native_write::library::pool_library_object_payload(
        &model,
        &format!("pool/symbols/{SYMBOL}.json"),
        symbol,
    )
    .unwrap();
    assert_eq!(before["pin_anchors"].as_array().unwrap().len(), 2);
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
                    .materialized_placed_symbol_definition(p.uuid, symbol)
                    .unwrap()
                    .unwrap(),
            )
        })
        .collect();
    assert_eq!(frozen.len(), 4);
    assert_eq!(
        frozen
            .values()
            .filter(|v| v["pin_anchors"].as_array().unwrap().len() == 2)
            .count(),
        1
    );
    std::fs::write(
        output.join("symbol-remove-anchor-intent.json"),
        serde_json::to_vec_pretty(&json!({"before":before,"old_definitions":frozen})).unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::SymbolRemoveAnchor(0),
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
        Step::Control(C::Edit),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([Step::Control(C::Anchor(1)), Step::SymbolRemoveAnchor(1)]);
        if !commit {
            steps.extend([
                Step::Focus(C::RemoveAnchor),
                Step::Capture("symbol-remove-before"),
            ]);
        }
        steps.extend([
            Step::Field("anchor_x", "invalid selected pose".into()),
            Step::SymbolRemoveAnchor(2),
            Step::Control(C::RemoveAnchor),
            Step::SymbolRemoveAnchor(3),
            Step::Invariant,
            Step::Control(C::AddAnchor),
            Step::Choice("anchor_pin", PIN_NAME.into()),
            Step::Control(C::Preview),
            Step::SymbolRemoveAnchor(4),
            Step::Invariant,
        ]);
        if commit {
            steps.push(Step::Capture("symbol-remove-conflict"));
        }
        steps.extend([
            Step::Control(C::RemoveAnchor),
            Step::SymbolRemoveAnchor(3),
            Step::Control(C::Preview),
            Step::SymbolRemoveAnchor(5),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-remove-preview"),
                Step::Control(C::Commit),
                Step::SymbolRemoveAnchor(6),
                Step::Capture("symbol-remove-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::SymbolRemoveAnchor(7),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolRemoveAnchor(7),
        Step::History(true),
        Step::SymbolRemoveAnchor(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRemoveAnchor(6),
        Step::Capture("symbol-remove-cold-root"),
        Step::Open,
        Step::Kind(1),
        Step::Search("Shared body".into()),
        Step::Select("Shared body".into()),
        Step::Control(C::Edit),
        Step::SymbolRemoveAnchor(8),
        Step::Capture("symbol-remove-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::SymbolRemoveAnchor(10),
        Step::SymbolRemoveAnchor(11),
        Step::SymbolRemoveAnchor(12),
        Step::SymbolRemoveAnchor(13),
        Step::SymbolRemoveAnchor(14),
        Step::SymbolRemoveAnchor(15),
        Step::Invariant,
        Step::Capture("symbol-remove-new-ghost"),
        Step::SymbolRemoveAnchor(16),
        Step::SymbolRemoveAnchor(17),
        Step::Capture("symbol-remove-new-committed"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRemoveAnchor(18),
        Step::Capture("symbol-remove-new-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
