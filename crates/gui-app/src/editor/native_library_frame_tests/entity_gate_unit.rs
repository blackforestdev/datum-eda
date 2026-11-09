//! Explicit Unit and Symbol correction through the shared form and later explicit placement.
use super::*;
use eda_engine::{
    api::native_write::{
        commit_prepared,
        context::WriteProvenance,
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    },
    substrate::{CommitSource, ProjectResolver},
};
use serde_json::{Value, json};
use uuid::Uuid;
const ENTITY: &str = "bdc50817-a127-4984-960e-bc26d5f913b2";
const NAME: &str = "Unit pair circuit";
const CORRECTED: &str = "Revised Unit gate body";

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
                .any(|prefix| p.starts_with(prefix))
        })
        .collect()
}
impl Fixture {
    fn gate_unit_intent(&self) -> Value {
        serde_json::from_slice(&std::fs::read(self.output.join("gate-unit-intent.json")).unwrap())
            .unwrap()
    }
    fn gate_unit_expected(&self, edited: bool) -> Value {
        let intent = self.gate_unit_intent();
        let mut raw = intent["before"].clone();
        if edited {
            raw["gates"][intent["gate"].as_str().unwrap()]["unit"] = intent["unit"].clone();
            raw["gates"][intent["gate"].as_str().unwrap()]["symbol"] =
                intent["replacement"].clone();
        }
        raw
    }
    pub(super) fn entity_gate_unit(&mut self, events: &ActiveEventLoop, stage: usize) {
        let intent = self.gate_unit_intent();
        if stage >= 10 {
            self.unit_gate_place(events, stage, &intent);
            return;
        }
        let s = self.app.runtime.as_ref().unwrap().workspace();
        let entry = s
            .library
            .entries
            .iter()
            .find(|e| e.id.to_string() == ENTITY)
            .unwrap();
        match stage {
            0 => {
                assert_eq!(entry.raw, self.gate_unit_expected(false));
                assert_eq!(entry.raw["gates"].as_object().unwrap().len(), 2);
                self.original_footprint = Some(json!(canonical(&self.root)));
            }
            1 | 2 => {
                let draft = s.library.draft.as_ref().unwrap();
                assert!(!draft.create);
                assert!(draft.fields.iter().any(|f| f.key == "gate_unit"));
                let hits = self.app.runtime.as_ref().unwrap().presented_hits.regions();
                assert_eq!(
                    hits.iter()
                        .filter(|h| matches!(h.target, HitTarget::Library(C::Gate(_))))
                        .count(),
                    2
                );
                assert!(
                    !hits.iter().any(|h| matches!(
                        h.target,
                        HitTarget::Library(C::AddGate | C::RemoveGate)
                    ))
                );
                if stage == 2 {
                    assert_eq!(draft.preview.as_ref(), Some(&self.gate_unit_expected(true)));
                    assert_eq!(
                        draft.request().unwrap().fields,
                        *json!({"gate_units":{intent["gate"].as_str().unwrap():intent["unit"]},"gate_symbols":{
                        intent["gate"].as_str().unwrap():intent["replacement"]}})
                        .as_object()
                        .unwrap()
                    );
                }
            }
            5 => {
                let d = s.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_unit"), intent["unit"].as_str().unwrap());
                assert_eq!(
                    d.value("gate_symbol"),
                    intent["before"]["gates"][intent["gate"].as_str().unwrap()]["symbol"]
                        .as_str()
                        .unwrap()
                );
                assert!(d.request().is_err());
                assert!(d.preview.is_none());
            }
            6 => {
                let d = s.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_unit"), intent["unit"].as_str().unwrap());
                assert_eq!(
                    d.value("gate_symbol"),
                    intent["replacement"].as_str().unwrap()
                );
            }
            3 | 4 => {
                assert_eq!(entry.raw, self.gate_unit_expected(stage == 3));
                let mut actual = canonical(&self.root);
                let mut original: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let path = format!("pool/entities/{ENTITY}.json");
                actual.remove(&path);
                original.remove(&path);
                assert_eq!(
                    actual, original,
                    "only the Entity gate Symbol link changes; all old source, CI, electrical and Proposal bytes remain exact"
                );
            }
            _ => panic!("unknown gate proof stage"),
        }
    }
    fn unit_corrected_choice(&self, intent: &Value) -> usize {
        let s = &self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring;
        s.choices
            .iter()
            .position(|c| {
                c.explicit_part.as_ref().is_some_and(|p| {
                    json!(p.part) == intent["part"]
                        && json!(p.gate) == intent["gate"]
                        && json!(p.symbol) == intent["replacement"]
                })
            })
            .expect("explicit Part/gate chooser exposes the corrected Symbol chain")
    }
    fn unit_gate_place(&mut self, events: &ActiveEventLoop, stage: usize, intent: &Value) {
        match stage {
            10 => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Place",
                    "Symbol",
                    EntrySurface::MenuPointer
                ));
            }
            11 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            12 => {
                let index = self.unit_corrected_choice(intent);
                if index >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            13 => {
                let index = self.unit_corrected_choice(intent);
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            14 => self.control(events, HitTarget::SchematicPlace),
            15 | 16 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: -2_000_000,
                    y: 3_000_000,
                },
                stage == 16,
            ),
            17 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let electrical =
                    eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
                for (id, members) in intent["net_members"].as_object().unwrap() {
                    let id: Uuid = id.parse().unwrap();
                    let anchor = match &model.electrical_identities[&id].identity {
                        eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } => anchor,
                        _ => panic!("existing Net identity remains a Net"),
                    };
                    let actual = electrical
                        .schematic_run(electrical.revision(), anchor)
                        .unwrap();
                    assert_eq!(actual.subject_id, id);
                    assert_eq!(
                        serde_json::to_value(actual.owned).unwrap(),
                        *members,
                        "old Net membership remains exact"
                    );
                }
                let original: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let actual = canonical(&self.root);
                let mut fresh = Vec::new();
                for (path, bytes) in &original {
                    if path.starts_with("schematic/") && path.ends_with(".json") {
                        let old: Value = serde_json::from_slice(bytes).unwrap();
                        let now: Value = serde_json::from_slice(&actual[path]).unwrap();
                        if let Some(symbols) = old.get("symbols").and_then(Value::as_object) {
                            for (id, raw) in symbols {
                                assert_eq!(
                                    &now["symbols"][id], raw,
                                    "old placed sources are retained"
                                );
                            }
                            for (id, raw) in now["symbols"].as_object().unwrap() {
                                if !symbols.contains_key(id) {
                                    fresh.push(raw.clone());
                                }
                            }
                        }
                    } else if path.starts_with(".datum/electrical_identities/") {
                        // Reuse the existing explicit-placement proof's one
                        // canonical initial lineage transition, never normalize
                        // source preservation during the gate correction itself.
                        let old: Value = serde_json::from_slice(bytes).unwrap();
                        let mut expected = old.clone();
                        if old["record"]["identity"]["kind"] == "net"
                            && old["record"]["identity"]["anchor_reason"] == "unclaimed_final_group"
                        {
                            expected["record"]["identity"]["anchor_reason"] =
                                json!("surviving_anchor");
                            expected["record"]["identity"]["predecessors"] =
                                json!([old["record"]["id"]]);
                            expected["record"]["object_revision"] =
                                json!(old["record"]["object_revision"].as_u64().unwrap() + 1);
                            assert!(model.journal.iter().flat_map(|tx|&tx.operations).any(|op| matches!(op,
                                eda_engine::substrate::Operation::SetElectricalIdentity { previous,record }
                                    if serde_json::to_value(previous).unwrap()==old["record"] && serde_json::to_value(record).unwrap()==expected["record"])),
                                "only the exact canonical initial lineage transition is permitted");
                        }
                        assert_eq!(
                            serde_json::from_slice::<Value>(&actual[path]).unwrap(),
                            expected,
                            "old identity/anchor/active state is preserved"
                        );
                    } else if path != &format!("pool/entities/{ENTITY}.json") {
                        assert_eq!(
                            &actual[path], bytes,
                            "existing library, Part/map, CI, electrical and Proposal bytes are retained"
                        );
                    }
                }
                assert_eq!(fresh.len(), 1);
                assert_eq!(fresh[0]["lib_id"], intent["replacement"]);
                let unit: Uuid = serde_json::from_value(intent["unit"].clone()).unwrap();
                let logical = library::pool_library_object_payload(
                    &model,
                    &format!("pool/units/{unit}.json"),
                    unit,
                )
                .unwrap();
                let expected: std::collections::BTreeSet<_> = logical["pins"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect();
                let actual: std::collections::BTreeSet<_> = fresh[0]["pins"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p["library_pin"].as_str().unwrap().to_string())
                    .collect();
                assert_eq!(
                    actual, expected,
                    "new explicit placement uses revised Unit pins"
                );
                assert_eq!(fresh[0]["entity"], json!(ENTITY));
                assert_eq!(fresh[0]["gate"], intent["gate"]);
                assert_eq!(fresh[0]["part"], intent["part"]);
                let state = self.app.runtime.as_ref().unwrap().workspace();
                let scene =
                    datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
                        .unwrap();
                let before_scene: Value = serde_json::from_slice(
                    &std::fs::read(self.output.join("entity-edit-scene-5.json")).unwrap(),
                )
                .unwrap();
                let graphics = serde_json::to_value(&scene.board_graphics).unwrap();
                for old in before_scene["board_graphics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|g| {
                        g["object_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("schematic-symbol:"))
                    })
                {
                    assert!(
                        graphics.as_array().unwrap().contains(old),
                        "every old rendered Symbol primitive remains exact after later placement"
                    );
                }
                let prefix = format!(
                    "schematic-symbol:{}:library:",
                    fresh[0]["uuid"].as_str().unwrap()
                );
                let graphic = scene
                    .board_graphics
                    .iter()
                    .find(|g| g.object_id.starts_with(&prefix))
                    .unwrap();
                assert_eq!(graphic.width_nm, Some(150_000));
                assert_eq!(
                    graphic.path.iter().map(|p| p.x).max().unwrap()
                        - graphic.path.iter().map(|p| p.x).min().unwrap(),
                    5_000_000
                );
                assert_eq!(
                    graphic.path.iter().map(|p| p.y).max().unwrap()
                        - graphic.path.iter().map(|p| p.y).min().unwrap(),
                    3_000_000
                );
                std::fs::write(
                    self.output.join("gate-unit-fresh-placed.json"),
                    serde_json::to_vec_pretty(&fresh[0]).unwrap(),
                )
                .unwrap();
            }
            18 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "View",
                "Fit to Board",
                EntrySurface::MenuPointer
            )),
            _ => panic!("unknown fresh placement stage"),
        }
    }
}

#[test]
#[ignore = "requires saved two-gate native Part circuit and an owned isolated native compositor"]
fn ordinary_saved_gate_unit_pair_cancel_history_cold_and_later_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let entity: Uuid = ENTITY.parse().unwrap();
    let journal = root.join(".datum/journal/transactions.jsonl");
    std::fs::write(
        output.join("gate-unit-before-setup-sources.json"),
        serde_json::to_vec(&canonical(&root)).unwrap(),
    )
    .unwrap();
    std::fs::copy(
        &journal,
        output.join("gate-unit-before-setup-journal.jsonl"),
    )
    .unwrap();
    let mut definition = library::pool_library_object_payload(
        &model,
        "pool/entities/6be4c754-4199-4d0d-b848-25ff4b3931c0.json",
        "6be4c754-4199-4d0d-b848-25ff4b3931c0".parse().unwrap(),
    )
    .unwrap();
    definition["uuid"] = json!(entity);
    definition["name"] = json!(NAME);
    let gates: serde_json::Map<_, _> = definition["gates"]
        .as_object()
        .unwrap()
        .values()
        .map(|g| {
            let id = Uuid::new_v4();
            let mut g = g.clone();
            g["uuid"] = json!(id);
            (id.to_string(), g)
        })
        .collect();
    definition["gates"] = json!(gates);
    let (old_gate, old_row) = gates.iter().next().unwrap();
    let old_gate: Uuid = old_gate.parse().unwrap();
    let old_symbol: Uuid = serde_json::from_value(old_row["symbol"].clone()).unwrap();
    let part = Uuid::new_v4();
    let mut orderable = library::pool_library_object_payload(
        &model,
        "pool/parts/0b84fb96-2578-420b-9a03-e9dc0b4d48b3.json",
        "0b84fb96-2578-420b-9a03-e9dc0b4d48b3".parse().unwrap(),
    )
    .unwrap();
    orderable["uuid"] = json!(part);
    orderable["entity"] = json!(entity);
    orderable["mpn"] = json!("UNITPAIR");
    assert_eq!(orderable["pad_map"], json!({}));
    assert!(orderable["default_pin_pad_map"].is_null());
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Distinct map-free native fixture",
        ),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "entities", entity),
                object: definition,
            },
            Spec::Create {
                target: Target::new("pool", "parts", part),
                object: orderable,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for (sheet, path, reference) in [
        ("77ac3f79-5121-4dfe-82e5-2f390da0c99c", vec![], "UROOT"),
        (
            "6c602479-5cb0-40b1-91d5-22447e9ccc07",
            vec!["8e0a7838-65ad-4c4f-86ff-2c49203abca7"],
            "UCHILD",
        ),
    ] {
        let selected=eda_engine::api::native_write::schematic_library::part_placement::PartGateSelection::capture(&model,part,old_gate,old_symbol).unwrap();
        let p=(eda_engine::api::native_write::registry::find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
            &eda_engine::api::native_write::registry::NativeWriteContext{model:&model,project_root:&root},
            WriteProvenance::new("test",CommitSource::Manual,"Original root/child map-free placement"),
            json!({"sheet":sheet,"instance_path":path,"library_symbol":old_symbol,"explicit_part":selected,"reference":reference,"value":"Original pair","x_nm":-5000000,"y_nm":-3000000})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    let before = library::pool_library_object_payload(
        &model,
        &format!("pool/entities/{ENTITY}.json"),
        entity,
    )
    .unwrap();
    let (gate, gate_raw) = before["gates"].as_object().unwrap().iter().next().unwrap();
    let old: Uuid = serde_json::from_value(gate_raw["symbol"].clone()).unwrap();
    let unit = Uuid::new_v4();
    let old_unit: Uuid = serde_json::from_value(gate_raw["unit"].clone()).unwrap();
    let mut logical = library::pool_library_object_payload(
        &model,
        &format!("pool/units/{old_unit}.json"),
        old_unit,
    )
    .unwrap();
    logical["uuid"] = json!(unit);
    logical["name"] = json!("Revised named Unit");
    let mut pins = serde_json::Map::new();
    let mut pin_ids = BTreeMap::new();
    for (old, raw) in logical["pins"].as_object().unwrap() {
        let id = Uuid::new_v4();
        let mut raw = raw.clone();
        raw["uuid"] = json!(id);
        pin_ids.insert(old.clone(), id);
        pins.insert(id.to_string(), raw);
    }
    logical["pins"] = json!(pins);
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Named compatible Unit fixture",
        ),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "units", unit),
            object: logical,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let replacement = Uuid::new_v4();
    let mut body =
        library::pool_library_object_payload(&model, &format!("pool/symbols/{old}.json"), old)
            .unwrap();
    body["uuid"] = json!(replacement);
    body["name"] = json!(CORRECTED);
    body["unit"] = json!(unit);
    for anchor in body["pin_anchors"].as_array_mut().unwrap() {
        anchor["pin"] = json!(pin_ids[anchor["pin"].as_str().unwrap()]);
    }
    body["drawings"] = json!([{"Rect":{"min":{"x":-2500000,"y":-1500000},"max":{"x":2500000,"y":1500000},"width":150000}}]);
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Named same-Unit alternate fixture",
        ),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "symbols", replacement),
            object: body,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let part = model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "parts")
        .find_map(|o| {
            let raw = library::pool_library_object_payload(
                &model,
                &format!("pool/parts/{}.json", o.object_id),
                o.object_id,
            )
            .ok()?;
            (raw["entity"] == json!(ENTITY)).then_some(o.object_id)
        })
        .unwrap();
    std::fs::write(
        output.join("gate-unit-after-setup-sources.json"),
        serde_json::to_vec(&canonical(&root)).unwrap(),
    )
    .unwrap();
    std::fs::copy(&journal, output.join("gate-unit-after-setup-journal.jsonl")).unwrap();
    let electrical = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let net_members: BTreeMap<_, _> = electrical
        .net_ids()
        .into_iter()
        .map(|id| {
            let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                unreachable!()
            };
            let run = electrical
                .schematic_run(electrical.revision(), anchor)
                .unwrap();
            assert_eq!(run.subject_id, id);
            (id, run.owned)
        })
        .collect();
    std::fs::write(
        output.join("gate-unit-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"gate":gate,"replacement":replacement,"unit":unit,"part":part,"net_members":net_members}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::EntityGateUnit(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(2),
        Step::Search(NAME.into()),
        Step::Select(NAME.into()),
        Step::Control(C::Edit),
        Step::EntityGateUnit(1),
        Step::Capture("gate-unit-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Control(C::Gate(0)),
            Step::Choice("gate_unit", "Revised named Unit".into()),
            Step::Control(C::Preview),
            Step::EntityGateUnit(5),
            Step::Invariant,
        ]);
        if commit {
            steps.push(Step::Capture("gate-unit-invalid"));
        }
        steps.extend([
            Step::Choice("gate_symbol", CORRECTED.into()),
            Step::Control(C::Gate(1)),
            Step::Control(C::Gate(0)),
            Step::EntityGateUnit(6),
            Step::Control(C::Preview),
            Step::EntityGateUnit(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("gate-unit-preview"),
                Step::Control(C::Commit),
                Step::EntityGateUnit(3),
                Step::Capture("gate-unit-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::EntityGateUnit(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::EntityGateUnit(4),
        Step::History(true),
        Step::EntityGateUnit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("gate-unit-placed-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::EntityGateUnit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("gate-unit-cold-child"),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(2),
        Step::Search(NAME.into()),
        Step::Select(NAME.into()),
        Step::Control(C::Edit),
        Step::Control(C::Gate(0)),
        Step::EntityGateUnit(6),
        Step::Capture("gate-unit-cold-form"),
        Step::Control(C::Cancel),
        Step::PaneContext(Navigate(false)),
    ]);
    steps.extend((10..=18).map(Step::EntityGateUnit));
    steps.extend([
        Step::Capture("gate-unit-fresh-placement"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::EntityGateUnit(18),
        Step::EntityGateUnit(17),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
