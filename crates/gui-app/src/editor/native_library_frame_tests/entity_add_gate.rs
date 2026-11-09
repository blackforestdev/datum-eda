//! Saved Add Gate and real remaining-gate placement through existing public input.
use super::*;
use eda_engine::substrate::ProjectResolver;
use serde_json::{Value, json};
use uuid::Uuid;
const ENTITY: &str = "6be4c754-4199-4d0d-b848-25ff4b3931c0";
const NAME: &str = "Auxiliary supply";
const CHILD: &str = "6c602479-5cb0-40b1-91d5-22447e9ccc07";
const MEMBER: &str = "91c68649-df5b-4d86-a93a-d8b4757452ac";
const CI: &str = "41c1143e-2e07-559a-ae1b-ae13f6757b8f";
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
fn read(p: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
}
impl Fixture {
    fn added_intent(&self) -> Value {
        read(&self.output.join("add-gate-intent.json"))
    }
    fn added_expected(&self, changed: bool) -> Value {
        let i = self.added_intent();
        let mut raw = i["before"].clone();
        if changed {
            raw["gates"][i["gate"].as_str().unwrap()] = i["row"].clone();
        }
        raw
    }
    pub(super) fn entity_add_gate(&mut self, events: &ActiveEventLoop, stage: usize) {
        if stage >= 10 {
            self.added_placement(events, stage);
            return;
        }
        let s = self.app.runtime.as_ref().unwrap().workspace();
        match stage {
            0 => {
                self.original_footprint = Some(json!(canonical(&self.root)));
            }
            1 => {
                let d = s.library.draft.as_ref().unwrap();
                assert!(!d.create);
                assert!(d.request().unwrap().fields.is_empty());
            }
            2 => {
                let d = s.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_name"), NAME);
                assert!(d.value("gate_symbol").is_empty());
                assert!(d.request().is_err());
                assert!(d.preview.is_none());
                assert!(!d.value("gate_unit").is_empty());
            }
            3 => {
                let d = s.library.draft.as_ref().unwrap();
                let id = d.entity.gate_id.unwrap();
                let rows = d.entity_gates().unwrap();
                let row = rows[&id.to_string()].clone();
                assert_eq!(row["name"], json!(NAME));
                assert_eq!(d.request().unwrap().fields.len(), 1);
                assert_eq!(
                    d.request().unwrap().fields["gate_additions"],
                    json!({id.to_string():row})
                );
                let mut intent = self.added_intent();
                intent["gate"] = json!(id);
                intent["row"] = row;
                std::fs::write(
                    self.output.join("add-gate-intent.json"),
                    serde_json::to_vec_pretty(&intent).unwrap(),
                )
                .unwrap();
            }
            4 => {
                let id = self.added_intent()["gate"].as_str().unwrap().to_string();
                let d = s.library.draft.as_ref().unwrap();
                let index = d.entity.gates.keys().position(|key| key == &id).unwrap();
                self.control(events, HitTarget::Library(C::Gate(index)));
            }
            5 => {
                let d = s.library.draft.as_ref().unwrap();
                assert_eq!(d.preview.as_ref(), Some(&self.added_expected(true)));
            }
            6 | 7 => {
                let changed = stage == 6;
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let raw = eda_engine::api::native_write::library::pool_library_object_payload(
                    &model,
                    &format!("pool/entities/{ENTITY}.json"),
                    ENTITY.parse().unwrap(),
                )
                .unwrap();
                assert_eq!(raw, self.added_expected(changed));
                let original: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let current = canonical(&self.root);
                assert_eq!(current.len(), original.len());
                for (p, bytes) in original {
                    if p != format!("pool/entities/{ENTITY}.json") {
                        assert_eq!(
                            current[&p], bytes,
                            "every other source/CI/Map/Net stays byte-exact"
                        );
                    }
                }
            }
            8 => {
                let d = s.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_name"), NAME);
                assert!(d.request().unwrap().fields.is_empty());
                assert_eq!(d.entity.gates.len(), 3);
            }
            _ => panic!("unknown AddGate stage"),
        }
    }
    fn added_key(&mut self, events: &ActiveEventLoop, key: Key) {
        let mut event = self.key_template.as_ref().unwrap().clone();
        event.logical_key = key.clone();
        event.physical_key = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F24);
        event.text = match key {
            Key::Character(s) => Some(s),
            _ => None,
        };
        event.state = ElementState::Pressed;
        event.repeat = false;
        let window = self.app.window.as_ref().unwrap().id();
        for state in [ElementState::Pressed, ElementState::Released] {
            event.state = state;
            self.app.window_event(
                events,
                window,
                WindowEvent::KeyboardInput {
                    device_id: winit::event::DeviceId::dummy(),
                    event: event.clone(),
                    // Preserve the captured fresh-press classification. Winit's
                    // synthetic flag means held-key focus replay, not test provenance.
                    // This public WindowEvent dispatch remains programmatic proof.
                    is_synthetic: false,
                },
            );
        }
    }

    fn added_world_mouse(
        &mut self,
        events: &ActiveEventLoop,
        point: datum_gui_protocol::PointNm,
        button: MouseButton,
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
        for state in [ElementState::Pressed, ElementState::Released] {
            self.app.window_event(
                events,
                window,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button,
                },
            );
        }
    }
    fn added_marking_control(&mut self, events: &ActiveEventLoop, slot: &str) {
        let target = self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .presented_hits
            .regions()
            .iter()
            .find_map(|h| {
                matches!(&h.target,HitTarget::MarkingMenuItem{slot:s,..} if s==slot)
                    .then(|| h.target.clone())
            })
            .expect("actual presented marking-menu action");
        self.control(events, target);
    }
    fn added_placement(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            10 => self.added_world_mouse(
                events,
                datum_gui_protocol::PointNm { x: 0, y: 0 },
                MouseButton::Right,
            ),
            11 => self.added_marking_control(events, "MORE"),
            12 => self.added_marking_control(events, "MORE:0"),
            13 => {
                let intent = self.added_intent();
                let draft = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .gate_draft
                    .as_ref()
                    .expect("existing remaining-gate picker");
                assert_eq!(draft.options.component_instance.to_string(), CI);
                assert_eq!(draft.options.instance_path.len(), 1);
                let index = draft
                    .options
                    .choices
                    .iter()
                    .position(|c| json!(c.gate) == intent["gate"])
                    .expect("new gate is offered on old component");
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            14 => self.control(events, HitTarget::SchematicReference),
            15 => {
                self.added_key(events, Key::Character("U_AUX".into()));
                assert_eq!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .schematic_authoring
                        .reference,
                    "U_AUX"
                );
            }
            16 => self.control(events, HitTarget::SchematicPlace),
            17 | 18 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: -2_000_000,
                    y: 3_000_000,
                },
                stage == 18,
            ),
            19 => {
                let review = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .delete_review
                    .as_ref()
                    .unwrap();
                assert!(review.ready());
                assert!(review.gate_placement.is_some());
                self.added_key(events, Key::Named(NamedKey::Tab));
            }
            20 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let original: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let current = canonical(&self.root);
                let before: Value =
                    serde_json::from_slice(&original[&format!("schematic/sheets/{CHILD}.json")])
                        .unwrap();
                let after: Value =
                    serde_json::from_slice(&current[&format!("schematic/sheets/{CHILD}.json")])
                        .unwrap();
                for (id, raw) in before["symbols"].as_object().unwrap() {
                    assert_eq!(&after["symbols"][id], raw);
                }
                let fresh: Vec<_> = after["symbols"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(id, _)| before["symbols"].get(*id).is_none())
                    .collect();
                assert_eq!(fresh.len(), 1);
                let intent = self.added_intent();
                assert_eq!(fresh[0].1["gate"], intent["gate"]);
                assert_eq!(fresh[0].1["entity"], json!(ENTITY));
                let id: Uuid = fresh[0].0.parse().unwrap();
                let ci = &model.component_instances[&CI.parse::<Uuid>().unwrap()];
                assert!(ci.placed_symbol_refs.contains(&MEMBER.parse().unwrap()));
                assert!(ci.placed_symbol_refs.contains(&id));
                assert_eq!(ci.placed_symbol_refs.len(), 2);
                let path = format!(".datum/component_instances/{CI}.json");
                let old_ci: Value = serde_json::from_slice(&original[&path]).unwrap();
                let new_ci: Value = serde_json::from_slice(&current[&path]).unwrap();
                for field in [
                    "part_ref",
                    "library_bindings",
                    "placed_package_refs",
                    "placed_package_roles",
                ] {
                    assert_eq!(
                        new_ci["component_instance"][field],
                        old_ci["component_instance"][field]
                    );
                }
                assert_eq!(
                    new_ci["component_instance"]["placed_symbol_roles"][MEMBER],
                    old_ci["component_instance"]["placed_symbol_roles"][MEMBER]
                );
                for (p, bytes) in &original {
                    if p == &format!("pool/entities/{ENTITY}.json")
                        || p == &path
                        || p == &format!("schematic/sheets/{CHILD}.json")
                    {
                        continue;
                    }
                    if p.starts_with(".datum/electrical_identities/") {
                        let old: Value = serde_json::from_slice(bytes).unwrap();
                        let now: Value = serde_json::from_slice(&current[p]).unwrap();
                        if old != now {
                            let mut expected = old.clone();
                            assert_eq!(
                                old["record"]["identity"]["anchor_reason"],
                                json!("unclaimed_final_group")
                            );
                            expected["record"]["identity"]["anchor_reason"] =
                                json!("surviving_anchor");
                            expected["record"]["identity"]["predecessors"] =
                                json!([old["record"]["id"]]);
                            expected["record"]["object_revision"] =
                                json!(old["record"]["object_revision"].as_u64().unwrap() + 1);
                            assert_eq!(now, expected);
                            assert!(model.journal.iter().flat_map(|tx|&tx.operations).any(|op|matches!(op,eda_engine::substrate::Operation::SetElectricalIdentity{previous,record} if json!(previous)==old["record"]&&json!(record)==expected["record"])));
                        }
                    } else {
                        assert_eq!(&current[p], bytes);
                    }
                }
                std::fs::write(
                    self.output.join("add-gate-placed.json"),
                    serde_json::to_vec_pretty(fresh[0].1).unwrap(),
                )
                .unwrap();
            }
            21 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "View",
                "Fit to Board",
                EntrySurface::MenuPointer
            )),
            _ => panic!("unknown remaining-gate stage"),
        }
    }
}
#[test]
#[ignore = "requires real native root/child circuit and owned isolated Wayland"]
fn ordinary_saved_add_gate_cancel_history_cold_and_existing_component_placement() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let before = read(&root.join(format!("pool/entities/{ENTITY}.json")));
    std::fs::write(
        output.join("add-gate-intent.json"),
        serde_json::to_vec_pretty(&json!({"before":before})).unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::EntityAddGate(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(2),
        Step::Search("Dual".into()),
        Step::Select("Dual".into()),
        Step::Control(C::Edit),
        Step::EntityAddGate(1),
        Step::Capture("add-gate-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Control(C::AddGate),
            Step::Field("gate_name", NAME.into()),
            Step::Choice("gate_unit", "Logic".into()),
            Step::Control(C::Preview),
            Step::EntityAddGate(2),
            Step::Invariant,
        ]);
        if commit {
            steps.push(Step::Capture("add-gate-invalid"));
        }
        steps.extend([
            Step::Choice("gate_symbol", "Shared body".into()),
            Step::EntityAddGate(3),
            Step::Control(C::Gate(0)),
            Step::EntityAddGate(4),
            Step::Control(C::Preview),
            Step::EntityAddGate(5),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("add-gate-preview"),
                Step::Control(C::Commit),
                Step::EntityAddGate(6),
                Step::Capture("add-gate-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::EntityAddGate(7),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::EntityAddGate(7),
        Step::History(true),
        Step::EntityAddGate(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::EntityAddGate(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("add-gate-cold-child"),
        Step::Open,
        Step::Kind(2),
        Step::Search("Dual".into()),
        Step::Select("Dual".into()),
        Step::Control(C::Edit),
        Step::EntityAddGate(4),
        Step::EntityAddGate(8),
        Step::Capture("add-gate-cold-form"),
        Step::Control(C::Cancel),
        Step::PaneContext(Navigate(true)),
    ]);
    steps.extend((10..=13).map(Step::EntityAddGate));
    steps.push(Step::Capture("add-gate-remaining-picker"));
    steps.extend((14..=18).map(Step::EntityAddGate));
    steps.push(Step::Capture("add-gate-placement-review"));
    steps.extend([
        Step::EntityAddGate(19),
        Step::EntityAddGate(20),
        Step::EntityAddGate(21),
        Step::Capture("add-gate-placed"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityAddGate(20),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
