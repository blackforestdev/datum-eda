//! Ordinary saved Part relink, old component editing and new explicit placement.
use super::*;
use eda_engine::substrate::{
    Operation,
    TransactionKind::{Normal, Redo, Undo},
};
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    },
    substrate::{CommitSource, ElectricalSelectionSnapshot, ProjectResolver},
};
use serde_json::{Value, json};
use uuid::Uuid;
const PART: &str = "0b84fb96-2578-420b-9a03-e9dc0b4d48b3";
const NAME: &str = "Explicit replacement physical Package";
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
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn read_sources(path: &Path) -> BTreeMap<String, Vec<u8>> {
    serde_json::from_value(read(path)).unwrap()
}
fn save(path: &Path, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn source_symbol(model: &eda_engine::substrate::DesignModel, id: Uuid) -> Value {
    let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
    snap.schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|s| s.symbols.get(&id).map(|s| json!(s)))
        .unwrap()
}
fn component_source(model: &eda_engine::substrate::DesignModel, id: Uuid) -> Value {
    let owner = &model.objects[&id];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == owner.source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone()
}
fn retained_sources(
    model: &eda_engine::substrate::DesignModel,
    ci: Uuid,
    old: &BTreeMap<String, Vec<u8>>,
    now: &BTreeMap<String, Vec<u8>>,
) {
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        now.keys().collect::<Vec<_>>()
    );
    let owner = &model.objects[&ci];
    let path = &model
        .source_shards
        .iter()
        .find(|s| s.shard_id == owner.source_shard_id)
        .unwrap()
        .relative_path;
    for (key, bytes) in old {
        if key != path {
            assert_eq!(&now[key], bytes);
            continue;
        }
        let before: Value = serde_json::from_slice(bytes).unwrap();
        let mut expected = before.clone();
        expected["component_instance"]["object_revision"] = json!(owner.object_revision.0);
        for reference in expected["component_instance"]["placed_symbol_refs"]
            .as_array_mut()
            .unwrap()
        {
            let member: Uuid = serde_json::from_value(reference["object_id"].clone()).unwrap();
            reference["object_revision"] = json!(model.objects[&member].object_revision.0);
        }
        if expected == before {
            assert_eq!(&now[key], bytes);
        } else {
            assert_eq!(
                serde_json::from_slice::<Value>(&now[key]).unwrap(),
                expected,
                "canonical Move compensation advances only actual CI/member revision counters"
            );
        }
    }
}
fn net_members(model: &eda_engine::substrate::DesignModel) -> BTreeMap<Uuid, Value> {
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    snap.net_ids()
        .into_iter()
        .map(|id| {
            let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                panic!("Net")
            };
            (
                id,
                json!(snap.schematic_run(snap.revision(), anchor).unwrap().owned),
            )
        })
        .collect()
}
impl Fixture {
    fn package_intent(&self) -> Value {
        read(&self.output.join("part-package-intent.json"))
    }
    fn package_expected(&self, edited: bool) -> Value {
        let intent = self.package_intent();
        let mut raw = intent["before"].clone();
        if edited {
            raw["package"] = intent["replacement"].clone();
            raw.as_object_mut().unwrap().remove("default_footprint");
        }
        raw
    }
    fn package_members(&self) {
        let intent = self.package_intent();
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let current = json!(net_members(&model));
        for (id, rows) in intent["net_members"].as_object().unwrap() {
            assert_eq!(current[id], *rows);
        }
    }
    /// The ordinary public window-event path, with the existing pane projection.
    fn package_world(
        &mut self,
        events: &ActiveEventLoop,
        point: datum_gui_protocol::PointNm,
        click: bool,
    ) {
        let r = self.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let prepared = r.prepared_scene().unwrap();
        let pane = prepared
            .layout
            .viewport_panes(&layout)
            .panes
            .into_iter()
            .find(|p| p.content == PaneContent::Schematic && p.id == layout.focused)
            .unwrap();
        let (x, y) = (pane.rect.scene.x + 20., pane.rect.scene.y + 20.);
        let a = prepared
            .world_point_for_pane_unclipped(pane.id, x, y)
            .unwrap();
        let b = prepared
            .world_point_for_pane_unclipped(pane.id, x + 100., y + 100.)
            .unwrap();
        let (px, py) = (
            x + (point.x - a.x) as f32 * 100. / (b.x - a.x) as f32,
            y + (point.y - a.y) as f32 * 100. / (b.y - a.y) as f32,
        );
        assert!(
            px >= pane.rect.scene.x
                && px < pane.rect.scene.x + pane.rect.scene.width
                && py >= pane.rect.scene.y
                && py < pane.rect.scene.y + pane.rect.scene.height
        );
        let window = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            window,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(px as f64, py as f64),
            },
        );
        if click {
            for state in [ElementState::Pressed, ElementState::Released] {
                self.app.window_event(
                    events,
                    window,
                    WindowEvent::MouseInput {
                        device_id: winit::event::DeviceId::dummy(),
                        state,
                        button: MouseButton::Left,
                    },
                );
            }
        }
    }
    pub(super) fn part_package(&mut self, events: &ActiveEventLoop, stage: usize) {
        if stage >= 20 {
            self.package_future(events, stage);
            return;
        }
        let intent = self.package_intent();
        let id: Uuid = intent["old_symbol"].as_str().unwrap().parse().unwrap();
        if stage >= 10 {
            let pin = &intent["old_symbol_raw"]["pins"][0]["position"];
            let mut point = datum_gui_protocol::PointNm {
                x: pin["x"].as_i64().unwrap(),
                y: pin["y"].as_i64().unwrap(),
            };
            if matches!(stage, 12 | 13) {
                point.x += 1_000_000;
                point.y += 1_000_000;
            }
            match stage {
                10 => {
                    save(
                        &self.output.join("part-package-before-move.json"),
                        &canonical(&self.root),
                    );
                    self.package_world(events, point, true);
                }
                11 => {
                    assert!(
                        matches!(&self.app.runtime.as_ref().unwrap().workspace().selection,
                        datum_gui_protocol::SelectionTarget::Native(v) if matches!(&v.subject,
                            datum_gui_protocol::SelectionSubject::Object(o) if o.id==id))
                    );
                    let action = crate::workspace_keyboard::character_action("m", false).unwrap();
                    assert!(crate::workspace_keyboard::apply(
                        self.app.runtime.as_mut().unwrap(),
                        action
                    ));
                    self.app.request_workspace_redraw();
                    assert!(
                        self.app
                            .runtime
                            .as_ref()
                            .unwrap()
                            .native_edit
                            .preview
                            .as_ref()
                            .unwrap()
                            .is_schematic()
                    );
                }
                12 => self.package_world(events, point, false),
                13 => self.package_world(events, point, true),
                14 | 16 => {
                    let model = ProjectResolver::new(&self.root).resolve().unwrap();
                    let current = source_symbol(&model, id);
                    assert_ne!(current["position"], intent["old_symbol_raw"]["position"]);
                    assert_eq!(current["entity"], intent["before"]["entity"]);
                    assert_eq!(current["part"], json!(PART));
                    let dx = current["position"]["x"].as_i64().unwrap()
                        - intent["old_symbol_raw"]["position"]["x"].as_i64().unwrap();
                    let dy = current["position"]["y"].as_i64().unwrap()
                        - intent["old_symbol_raw"]["position"]["y"].as_i64().unwrap();
                    let mut pins = intent["old_symbol_raw"]["pins"].clone();
                    for pin in pins.as_array_mut().unwrap() {
                        pin["position"]["x"] = json!(pin["position"]["x"].as_i64().unwrap() + dx);
                        pin["position"]["y"] = json!(pin["position"]["y"].as_i64().unwrap() + dy);
                    }
                    assert_eq!(
                        current["pins"], pins,
                        "only the old pin coordinates translate; identity and semantics remain exact"
                    );
                    let old_ci: Uuid = intent["old_ci"].as_str().unwrap().parse().unwrap();
                    assert_eq!(
                        model.component_instances[&old_ci].part_ref,
                        Some(PART.parse().unwrap())
                    );
                    let ci = component_source(&model, old_ci);
                    assert_eq!(ci["part_ref"], intent["old_ci_raw"]["part_ref"]);
                    assert_eq!(
                        ci["library_bindings"],
                        intent["old_ci_raw"]["library_bindings"]
                    );
                    if stage == 14 {
                        save(
                            &self.output.join("part-package-moved-symbol.json"),
                            &current,
                        );
                    } else {
                        assert_eq!(
                            current,
                            read(&self.output.join("part-package-moved-symbol.json"))
                        );
                    }
                    self.package_members();
                }
                15 | 17 => {
                    let model = ProjectResolver::new(&self.root).resolve().unwrap();
                    assert_eq!(source_symbol(&model, id), intent["old_symbol_raw"]);
                    let old = read_sources(&self.output.join("part-package-before-move.json"));
                    let ci = intent["old_ci"].as_str().unwrap().parse().unwrap();
                    retained_sources(&model, ci, &old, &canonical(&self.root));
                    if stage == 17 {
                        let history = &model.journal[model.journal.len() - 4..];
                        assert_eq!(
                            history
                                .iter()
                                .map(|t| t.transaction_kind)
                                .collect::<Vec<_>>(),
                            [Normal, Undo, Redo, Undo]
                        );
                        assert!(history.iter().all(|t|t.operations.len()==2
                            && t.operations.iter().any(|op|matches!(op,Operation::SetComponentInstance{component_instance_id,..} if *component_instance_id==ci))
                            && t.operations.iter().any(|op|matches!(op,Operation::SetSchematicSymbol{symbol_id,..} if *symbol_id==id))));
                        save(
                            &self.output.join("part-package-move-complete.json"),
                            &component_source(&model, ci),
                        );
                    }
                }
                _ => panic!("unknown old edit stage"),
            }
            return;
        }
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "parts" && e.id.to_string() == PART)
            .unwrap();
        match stage {
            0 => {
                assert!(!self.output.join("part-package-move-complete.json").exists());
                assert_eq!(entry.raw, self.package_expected(false));
                self.original_footprint = Some(json!(canonical(&self.root)));
            }
            1 => {
                let d = state.library.draft.as_ref().unwrap();
                assert!(!d.create);
                let field = d.fields.iter().find(|f| f.key == "part_package").unwrap();
                assert!(field.selector);
                assert_eq!(field.text, intent["before"]["package"].as_str().unwrap());
                assert!(
                    field
                        .choices
                        .iter()
                        .any(|(id, label)| id == intent["replacement"].as_str().unwrap()
                            && label.starts_with(NAME))
                );
                assert_eq!(d.fields.len(), 10);
            }
            5 => {
                let d = state.library.draft.as_ref().unwrap();
                assert_eq!(
                    d.value("part_footprint"),
                    intent["before"]["default_footprint"].as_str().unwrap()
                );
                assert!(d.request().is_err());
                assert!(
                    d.fields
                        .iter()
                        .find(|f| f.key == "part_footprint")
                        .unwrap()
                        .choices
                        .iter()
                        .any(|(id, label)| id == d.value("part_footprint")
                            && label.contains("Unavailable"))
                );
            }
            2 => {
                let d = state.library.draft.as_ref().unwrap();
                assert_eq!(d.preview, Some(self.package_expected(true)));
                assert_eq!(
                    d.request().unwrap().fields,
                    *json!({"package":intent["replacement"],"default_footprint":null})
                        .as_object()
                        .unwrap()
                );
            }
            3 | 4 => {
                assert_eq!(entry.raw, self.package_expected(stage == 3));
                let mut now = canonical(&self.root);
                let mut old: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let path = format!("pool/parts/{PART}.json");
                now.remove(&path);
                old.remove(&path);
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let completed = self.output.join("part-package-move-complete.json");
                if completed.exists() {
                    let ci = intent["old_ci"].as_str().unwrap().parse().unwrap();
                    assert_eq!(component_source(&model, ci), read(&completed));
                    retained_sources(&model, ci, &old, &now);
                } else {
                    assert_eq!(
                        now, old,
                        "plain Part relink/cancel/history preserves every old CI/source byte"
                    );
                }
                self.package_members();
            }
            _ => panic!("unknown Part stage"),
        }
    }
    fn package_choice(&self) -> usize {
        let intent = self.package_intent();
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .position(|c| {
                c.explicit_part.as_ref().is_some_and(|p| {
                    json!(p.part) == json!(PART)
                        && json!(p.entity) == intent["before"]["entity"]
                        && json!(p.gate) == intent["new_gate"]
                })
            })
            .expect("current explicit Part/gate Entity choice")
    }
    fn package_future(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            20 => {
                save(
                    &self.output.join("part-package-before-new.json"),
                    &canonical(&self.root),
                );
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Place",
                    "Symbol",
                    EntrySurface::MenuPointer
                ));
            }
            21 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            22 => {
                if self.package_choice() >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            23 => self.control(
                events,
                HitTarget::SchematicLibraryChoice(self.package_choice()),
            ),
            24 => self.control(events, HitTarget::SchematicPlace),
            25 | 27 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: -2_000_000,
                    y: 3_000_000,
                },
                stage == 27,
            ),
            28 | 30 => {
                let intent = self.package_intent();
                let now = canonical(&self.root);
                let old = read_sources(&self.output.join("part-package-before-new.json"));
                let mut added = Vec::new();
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                for (path, bytes) in &old {
                    let (before, after): (Value, Value) = if path.starts_with("schematic/")
                        || path.starts_with(".datum/electrical_identities/")
                    {
                        (
                            serde_json::from_slice(bytes).unwrap(),
                            serde_json::from_slice(&now[path]).unwrap(),
                        )
                    } else {
                        (Value::Null, Value::Null)
                    };
                    if path.starts_with("schematic/") && before.get("symbols").is_some() {
                        let mut expected = after.clone();
                        for (id, symbol) in after["symbols"].as_object().unwrap() {
                            if before["symbols"].get(id).is_none() {
                                added.push(symbol.clone());
                                expected["symbols"].as_object_mut().unwrap().remove(id);
                            }
                        }
                        assert_eq!(expected, before);
                    } else if path.starts_with(".datum/electrical_identities/") {
                        let mut expected = before.clone();
                        if before["record"]["identity"]["kind"] == "net"
                            && before["record"]["identity"]["anchor_reason"]
                                == "unclaimed_final_group"
                        {
                            expected["record"]["identity"]["anchor_reason"] =
                                json!("surviving_anchor");
                            expected["record"]["identity"]["predecessors"] =
                                json!([before["record"]["id"]]);
                            expected["record"]["object_revision"] =
                                json!(before["record"]["object_revision"].as_u64().unwrap() + 1);
                            assert!(model.journal.iter().flat_map(|t|&t.operations).any(|op|matches!(op,
                                eda_engine::substrate::Operation::SetElectricalIdentity{previous,record}
                                    if json!(previous)==before["record"] && json!(record)==expected["record"])));
                        }
                        assert_eq!(after, expected);
                    } else {
                        assert_eq!(&now[path], bytes);
                    }
                }
                assert_eq!(added.len(), 1);
                assert_eq!(added[0]["part"], json!(PART));
                assert_eq!(added[0]["entity"], intent["before"]["entity"]);
                assert_eq!(added[0]["gate"], intent["new_gate"]);
                let symbol: Uuid = serde_json::from_value(added[0]["uuid"].clone()).unwrap();
                let ci = model
                    .component_instances
                    .values()
                    .find(|ci| ci.placed_symbol_refs.contains(&symbol))
                    .unwrap();
                let declared = component_source(&model, ci.id)["part_ref"].clone();
                assert_eq!(
                    declared["object_revision"],
                    json!(model.objects[&PART.parse().unwrap()].object_revision.0)
                );
                assert_eq!(
                    read(&self.root.join(format!("pool/parts/{PART}.json")))["package"],
                    intent["replacement"]
                );
                self.package_members();
                if stage == 28 {
                    save(&self.output.join("part-package-after-new.json"), &now);
                } else {
                    let expected = read_sources(&self.output.join("part-package-after-new.json"));
                    assert_eq!(now, expected);
                }
            }
            29 => {
                let expected = read_sources(&self.output.join("part-package-before-new.json"));
                assert_eq!(canonical(&self.root), expected);
            }
            _ => panic!("unknown future stage"),
        }
    }
}
#[test]
#[ignore = "requires saved root/child circuit and owned isolated native Wayland"]
fn ordinary_saved_part_package_cancel_history_cold_old_edit_and_new_explicit_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let part: Uuid = PART.parse().unwrap();
    let before =
        library::pool_library_object_payload(&model, &format!("pool/parts/{PART}.json"), part)
            .unwrap();
    assert!(
        before["pad_map"].as_object().unwrap().is_empty()
            && before["default_pin_pad_map"].is_null()
    );
    let old_entity: Uuid = serde_json::from_value(before["entity"].clone()).unwrap();
    let logical = library::pool_library_object_payload(
        &model,
        &format!("pool/entities/{old_entity}.json"),
        old_entity,
    )
    .unwrap();
    let gates = logical["gates"].as_object().unwrap();
    let new_gate: Uuid = gates.keys().next().unwrap().parse().unwrap();
    let replacement = Uuid::new_v4();
    let package = library::pool_package_payload(replacement, NAME, json!({}));
    let pre_setup = canonical(&root);
    let write = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Existing distinct named Package fixture",
        ),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "packages", replacement),
            object: package,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let after_setup = canonical(&root);
    for (path, bytes) in &pre_setup {
        assert_eq!(
            after_setup[path], *bytes,
            "fixture does not rewrite any old source"
        );
    }
    let old_ci = model
        .component_instances
        .values()
        .find(|ci| ci.part_ref == Some(part))
        .unwrap();
    let old_symbol = old_ci.placed_symbol_refs[0];
    save(
        &output.join("part-package-intent.json"),
        &json!({"before":before,"replacement":replacement,
            "new_gate":new_gate,"old_ci":old_ci.id,"old_ci_raw":component_source(&model,old_ci.id),"old_symbol":old_symbol,
        "old_symbol_raw":source_symbol(&model,old_symbol),"net_members":net_members(&model)}),
    );
    let mut steps = vec![
        Step::PartPackage(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(3),
        Step::Search("FIRST".into()),
        Step::Select("FIRST".into()),
        Step::Control(C::Edit),
        Step::PartPackage(1),
        Step::Capture("part-package-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Choice("part_package", NAME.into()),
            Step::PartPackage(5),
            Step::Choice("part_footprint", "No default Footprint".into()),
            Step::Control(C::Preview),
            Step::PartPackage(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("part-package-preview"),
                Step::Control(C::Commit),
                Step::PartPackage(3),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::PartPackage(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::PartPackage(4),
        Step::History(true),
        Step::PartPackage(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("part-package-old-child"),
        Step::PaneContext(Fit),
        Step::PartPackage(10),
        Step::PartPackage(11),
        Step::PartPackage(12),
        Step::Capture("part-package-old-move-ghost"),
        Step::PartPackage(13),
        Step::PartPackage(14),
        Step::History(false),
        Step::PartPackage(15),
        Step::History(true),
        Step::PartPackage(16),
        Step::History(false),
        Step::PartPackage(17),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PartPackage(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("part-package-cold-child"),
        Step::PaneContext(Navigate(false)),
    ]);
    steps.extend((20..=25).map(Step::PartPackage));
    steps.extend([
        Step::Capture("part-package-future-ghost"),
        Step::PartPackage(27),
        Step::PartPackage(28),
        Step::Capture("part-package-future-committed"),
        Step::History(false),
        Step::PartPackage(29),
        Step::History(true),
        Step::PartPackage(30),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::PartPackage(30),
        Step::Capture("part-package-future-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
