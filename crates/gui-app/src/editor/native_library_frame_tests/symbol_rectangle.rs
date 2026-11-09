//! Saved numeric rectangle correction, frozen real circuits and later guarded placement.
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    },
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver},
};
use serde_json::{Value, json};
use uuid::Uuid;
const SYMBOL: &str = "01974935-759f-4259-84d1-0c7d53be8bbf";
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
    serde_json::from_slice(&std::fs::read(output.join("symbol-rectangle-intent.json")).unwrap())
        .unwrap()
}
fn corrected(before: &Value) -> Value {
    let mut v = before.clone();
    let r = &mut v["drawings"][1]["Rect"];
    r["min"]["x"] = json!(-3000000);
    r["min"]["y"] = json!(-2000000);
    r["max"]["x"] = json!(4000000);
    r["max"]["y"] = json!(3000000);
    r["width"] = json!(200000);
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
    fn rectangle_frozen_definitions(&self, model: &DesignModel) {
        let data = intent(&self.output);
        for (id, raw) in data["old_definitions"].as_object().unwrap() {
            assert_eq!(
                model
                    .materialized_placed_symbol_definition(
                        id.parse().unwrap(),
                        SYMBOL.parse().unwrap()
                    )
                    .unwrap(),
                Some(raw.clone()),
                "actual pre-setup body stays frozen"
            );
        }
    }
    fn rectangle_core(&self, changed: bool) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        let data = intent(&self.output);
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "symbols" && e.id.to_string() == SYMBOL)
            .unwrap();
        assert_eq!(
            entry.raw,
            if changed {
                corrected(&data["before"])
            } else {
                data["before"].clone()
            }
        );
        let old = self.original_footprint.as_ref().unwrap();
        let mut actual = canonical(&self.root);
        let mut files: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        let path = format!("pool/symbols/{SYMBOL}.json");
        actual.remove(&path);
        files.remove(&path);
        assert_eq!(
            actual, files,
            "only the five rectangle leaves change; all other sources/CI/Nets/Proposals exact"
        );
        assert_eq!(
            serde_json::to_value(&model.component_instances).unwrap(),
            old["ci"]
        );
        assert_eq!(
            serde_json::to_value(&model.electrical_identities).unwrap(),
            old["ei"]
        );
        assert_eq!(members(model), old["members"]);
        self.rectangle_frozen_definitions(model);
    }
    fn rectangle_choice(&self) -> usize {
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
    fn rectangle_future(&mut self, record: bool) {
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
                x: -2000000,
                y: 3000000
            }
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
        let expected = corrected(&data["before"]);
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
        self.rectangle_frozen_definitions(model);
        let persisted = json!({"symbol":placed,"ci":ci,"definition":expected});
        if record {
            self.edited_footprint = Some(persisted);
        } else {
            assert_eq!(self.edited_footprint.as_ref(), Some(&persisted));
        }
    }
    fn rectangle_rendered_future(&self) {
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
        let old: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("entity-edit-scene-5.json")).unwrap(),
        )
        .unwrap();
        let mut actual = json!({"components":scene.components,"texts":scene.component_texts,"graphics":scene.component_graphics,"board_texts":scene.board_texts,"text_geometry":scene.board_text_geometries,"board_graphics":scene.board_graphics,"tracks":scene.tracks,"nets":scene.net_display});
        for (key, values) in old.as_object().unwrap() {
            for expected in values.as_array().unwrap() {
                let a = actual[key].as_array_mut().unwrap();
                let index = a
                    .iter()
                    .position(|v| v == expected)
                    .expect("every old rendered primitive remains, including duplicates");
                a.remove(index);
            }
        }
        let id = self.edited_footprint.as_ref().unwrap()["symbol"]["uuid"]
            .as_str()
            .unwrap();
        let body = scene
            .board_graphics
            .iter()
            .find(|g| g.source_object_uuid == id && g.width_nm == Some(200000))
            .unwrap();
        assert_eq!(body.path.iter().map(|p| p.x).min(), Some(-5000000));
        assert_eq!(body.path.iter().map(|p| p.x).max(), Some(2000000));
        assert_eq!(body.path.iter().map(|p| p.y).min(), Some(1000000));
        assert_eq!(body.path.iter().map(|p| p.y).max(), Some(6000000));
    }
    pub(super) fn symbol_rectangle(&mut self, events: &ActiveEventLoop, stage: usize) {
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
                self.rectangle_core(false);
            }
            1 => {
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let d = s.library.draft.as_ref().unwrap();
                assert!(!d.create);
                assert_eq!(d.saved_rectangle_index(), Some(1));
                assert_eq!(
                    d.fields
                        .iter()
                        .filter(|f| f.key.starts_with("rect_"))
                        .count(),
                    5
                );
                assert!(!d.fields.iter().any(|f| f.key == "unit"));
                assert_eq!(
                    d.electrical.original.as_ref(),
                    Some(
                        &s.library
                            .entries
                            .iter()
                            .find(|e| e.id.to_string() == SYMBOL)
                            .unwrap()
                            .raw
                    )
                );
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
                let expected = corrected(&intent(&self.output)["before"]);
                assert_eq!(d.preview.as_ref(), Some(&expected));
                let request = d.request().unwrap();
                assert_eq!(request.fields["drawings"], expected["drawings"]);
                assert!(!request.fields.contains_key("pin_anchors"));
                assert!(!request.fields.contains_key("fields"));
            }
            3 => self.rectangle_core(true),
            4 => self.rectangle_core(false),
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
                assert!(d.saved_rectangle_index().is_none());
                assert!(!d.fields.iter().any(|f| f.key.starts_with("rect_")));
                assert!(d.fields.iter().any(|f| f.key == "name"));
                assert!(!d.request().unwrap().fields.contains_key("drawings"));
            }
            6 | 7 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: -2000000,
                    y: 3000000,
                },
                stage == 7,
            ),
            8 => self.rectangle_future(true),
            9 => self.rectangle_future(false),
            10 => self.rectangle_rendered_future(),
            11 => {
                if self.rectangle_choice() >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            12 => {
                let index = self.rectangle_choice();
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
                    self.output.join("symbol-rectangle-choice.json"),
                    serde_json::to_vec_pretty(selected).unwrap(),
                )
                .unwrap();
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            13 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "Place",
                "Symbol",
                EntrySurface::MenuPointer
            )),
            14 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            15 => self.control(events, HitTarget::SchematicPlace),
            16 => self.rectangle_core(true),
            _ => panic!("unknown Symbol rectangle stage"),
        }
    }
}
#[test]
#[ignore = "requires saved native root/child circuit and owned isolated native Wayland"]
fn ordinary_saved_symbol_rectangle_cancel_history_cold_and_new_guarded_body() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let symbol: Uuid = SYMBOL.parse().unwrap();
    let original = library::pool_library_object_payload(
        &model,
        &format!("pool/symbols/{SYMBOL}.json"),
        symbol,
    )
    .unwrap();
    assert_eq!(original["drawings"].as_array().unwrap().len(), 1);
    assert!(original["drawings"][0].get("Rect").is_some());
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let old_definitions: BTreeMap<_, _> = snap
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
    assert!(!old_definitions.is_empty());
    let mut before = original.clone();
    let mut rectangle = before["drawings"][0].clone();
    rectangle["Rect"]["min"]["vendor"] = json!([3, 1, 3]);
    rectangle["Rect"]["max"]["optional"] = Value::Null;
    rectangle["Rect"]["extension"] = json!({"retain":["z","a","z"]});
    before["drawings"] = json!([{"Line":{"from":{"x":-4000000,"y":-1000000},"to":{"x":-4000000,"y":1000000},"width":50000,"unknown":true}},rectangle,{"Circle":{"center":{"x":0,"y":0},"radius":300000,"width":80000}}]);
    before["extension"] = json!({"fixture":"raw mixed body","ordered":[3,1,3]});
    let mut zero = before.clone();
    let zero_id = Uuid::new_v4();
    zero["uuid"] = json!(zero_id);
    zero["name"] = json!("No rectangle body");
    zero["drawings"].as_array_mut().unwrap().remove(1);
    let mut multiple = before.clone();
    let multiple_id = Uuid::new_v4();
    multiple["uuid"] = json!(multiple_id);
    multiple["name"] = json!("Multiple rectangle body");
    let repeated = multiple["drawings"][1].clone();
    multiple["drawings"]
        .as_array_mut()
        .unwrap()
        .insert(2, repeated);
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Canonical mixed body and unavailable-form fixtures",
        ),
        Some("pool"),
        vec![
            Spec::Set {
                target: Target::new("pool", "symbols", symbol),
                object: before.clone(),
            },
            Spec::Create {
                target: Target::new("pool", "symbols", zero_id),
                object: zero,
            },
            Spec::Create {
                target: Target::new("pool", "symbols", multiple_id),
                object: multiple,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for (id, raw) in &old_definitions {
        assert_eq!(
            model
                .materialized_placed_symbol_definition(*id, symbol)
                .unwrap(),
            Some(raw.clone())
        );
    }
    std::fs::write(output.join("symbol-rectangle-intent.json"),serde_json::to_vec_pretty(&json!({"before":before,"pre_setup_original":original,"old_definitions":old_definitions,"zero_id":zero_id,"multiple_id":multiple_id})).unwrap()).unwrap();
    let mut steps = vec![
        Step::SymbolRectangle(0),
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
        Step::SymbolRectangle(1),
        Step::Capture("symbol-rect-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Field("rect_min_x", "-3 mm".into()),
            Step::Field("rect_min_y", "-2 mm".into()),
            Step::Field("rect_max_x", "4 mm".into()),
            Step::Field("rect_max_y", "3 mm".into()),
            Step::Field("rect_width", "0.2 mm".into()),
            Step::Control(C::Preview),
            Step::SymbolRectangle(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("symbol-rect-preview"),
                Step::Control(C::Commit),
                Step::SymbolRectangle(3),
                Step::Capture("symbol-rect-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::SymbolRectangle(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::SymbolRectangle(4),
        Step::History(true),
        Step::SymbolRectangle(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRectangle(3),
    ]);
    for (name, capture) in [
        ("No rectangle body", "symbol-rect-unavailable-zero"),
        (
            "Multiple rectangle body",
            "symbol-rect-unavailable-multiple",
        ),
    ] {
        steps.extend([
            Step::Open,
            Step::Kind(1),
            Step::Search(name.into()),
            Step::Select(name.into()),
            Step::Control(C::Edit),
            Step::SymbolRectangle(5),
            Step::Capture(capture),
            Step::Control(C::Preview),
            Step::SymbolRectangle(5),
            Step::Invariant,
            Step::KeyControl(C::Cancel),
            Step::Invariant,
            Step::SymbolRectangle(3),
        ]);
    }
    steps.extend([
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::SymbolRectangle(13),
        Step::SymbolRectangle(14),
        Step::SymbolRectangle(11),
        Step::SymbolRectangle(12),
        Step::SymbolRectangle(15),
        Step::SymbolRectangle(6),
        Step::Invariant,
        Step::Capture("symbol-rect-future-ghost"),
        Step::SymbolRectangle(7),
        Step::SymbolRectangle(8),
        Step::SymbolRectangle(10),
        Step::Capture("symbol-rect-future-committed"),
        Step::History(false),
        Step::SymbolRectangle(16),
        Step::History(true),
        Step::SymbolRectangle(9),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::SymbolRectangle(9),
        Step::PaneContext(Navigate(false)),
        Step::SymbolRectangle(10),
        Step::Capture("symbol-rect-future-cold"),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
