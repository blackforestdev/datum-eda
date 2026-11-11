//! Author a component, then place, connect and correct it on Root and Child.
use super::*;
use datum_gui_protocol::PointNm;
use eda_engine::substrate::{ElectricalOccurrence, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;

const UNIT: &str = "Teru logic";
const SYMBOL: &str = "Teru component";
const ENTITY: &str = "Teru entity";
const PART: &str = "TERU-1";
const INPUT: &str = "TERU_IN";
const OUTPUT: &str = "TERU_OUT";

fn point(x: i64, y: i64) -> PointNm {
    PointNm {
        x: x * 1_000_000,
        y: y * 1_000_000,
    }
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn entry(state: &datum_gui_protocol::ReviewWorkspaceState, kind: &str, name: &str) -> Uuid {
    state
        .library
        .entries
        .iter()
        .find(|e| e.kind == kind && (e.name == name || e.raw["mpn"] == name))
        .unwrap()
        .id
}

impl Fixture {
    fn circuit_key(&mut self, events: &ActiveEventLoop, key: Key) {
        let mut event = self.key_template.as_ref().unwrap().clone();
        event.text = match &key {
            Key::Character(text) => Some(text.clone()),
            _ => None,
        };
        event.logical_key = key;
        event.physical_key = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F24);
        event.state = ElementState::Pressed;
        event.repeat = false;
        let id = self.app.window.as_ref().unwrap().id();
        for state in [ElementState::Pressed, ElementState::Released] {
            event.state = state;
            self.app.window_event(
                events,
                id,
                WindowEvent::KeyboardInput {
                    device_id: winit::event::DeviceId::dummy(),
                    event: event.clone(),
                    // Fresh press, not held-key focus replay. Dispatch remains
                    // programmatic proof, as in the existing Add Gate scenario.
                    is_synthetic: false,
                },
            );
        }
    }

    fn circuit_mouse(
        &mut self,
        events: &ActiveEventLoop,
        at: PointNm,
        button: Option<ElementState>,
    ) {
        let runtime = self.app.runtime.as_mut().unwrap();
        let layout = runtime.workspace().ui.layout.clone();
        let prepared = runtime.prepared_scene().unwrap();
        let pane = prepared
            .layout
            .viewport_panes(&layout)
            .panes
            .into_iter()
            .find(|pane| pane.id == layout.focused && pane.content == PaneContent::Schematic)
            .unwrap();
        let (x, y) = (pane.rect.scene.x + 20., pane.rect.scene.y + 20.);
        let a = prepared
            .world_point_for_pane_unclipped(pane.id, x, y)
            .unwrap();
        let b = prepared
            .world_point_for_pane_unclipped(pane.id, x + 100., y + 100.)
            .unwrap();
        let px = x + (at.x - a.x) as f32 * 100. / (b.x - a.x) as f32;
        let py = y + (at.y - a.y) as f32 * 100. / (b.y - a.y) as f32;
        assert!(
            pane.rect.scene.contains(px, py),
            "circuit point {at:?} inside presented pane"
        );
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(px as f64, py as f64),
            },
        );
        if let Some(state) = button {
            self.app.window_event(
                events,
                id,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Left,
                },
            );
        }
    }

    fn circuit_click(&mut self, events: &ActiveEventLoop, at: PointNm) {
        self.circuit_mouse(events, at, Some(ElementState::Pressed));
        self.circuit_mouse(events, at, Some(ElementState::Released));
    }

    fn circuit_menu(&mut self, label: &str) {
        assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
            "Place",
            label,
            EntrySurface::MenuPointer
        ));
    }

    fn circuit_choice(&self) -> usize {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let part = entry(state, "parts", PART);
        state
            .schematic_authoring
            .choices
            .iter()
            .position(|c| {
                c.explicit_part
                    .as_ref()
                    .is_some_and(|choice| choice.part == part)
            })
            .unwrap()
    }

    fn circuit_sheet(&self, which: usize) -> (Uuid, Vec<Uuid>) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let row = state
            .schematic_documents
            .rows
            .iter()
            .find(|row| row.parent.is_some() == (which == 1))
            .unwrap();
        (row.key.source_sheet, row.key.instance_path.clone())
    }

    fn circuit_check(&self, which: usize, connected: bool) {
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
        let source = snap.schematic_source().unwrap();
        let (sheet_id, path) = self.circuit_sheet(which);
        let sheet = &source.sheets[&sheet_id];
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let part = entry(state, "parts", PART);
        let unit = entry(state, "units", UNIT);
        let symbol = entry(state, "symbols", SYMBOL);
        let logical = state.library.entries.iter().find(|e| e.id == unit).unwrap();
        let placed = sheet
            .symbols
            .values()
            .find(|p| p.part == Some(part))
            .unwrap();
        assert_eq!(placed.lib_id, Some(symbol.to_string()));
        assert_eq!(placed.position.x, -10_000_000);
        assert_eq!(placed.pins.len(), 2);
        let mut nets = Vec::new();
        let mut records = Vec::new();
        for (name, position) in [(INPUT, point(-12, 0)), (OUTPUT, point(-8, 0))] {
            let pin = placed.pins.iter().find(|p| p.name == name).unwrap();
            let original = logical.raw["pins"]
                .as_object()
                .unwrap()
                .iter()
                .find(|(_, pin)| pin["name"] == name)
                .unwrap()
                .0;
            assert_eq!(pin.library_pin, Some(original.parse().unwrap()));
            assert_eq!((pin.position.x, pin.position.y), (position.x, position.y));
            let origin = ElectricalOccurrence {
                class: "pins".into(),
                source_id: pin.uuid,
                instance_path: path.clone(),
            };
            let run = snap.schematic_run(snap.revision(), &origin).unwrap();
            assert!(run.owned.contains(&origin));
            if connected {
                assert!(
                    run.owned
                        .iter()
                        .any(|o| o.class == "wires" && o.instance_path == path)
                );
                if name == INPUT {
                    let other = sheet
                        .symbols
                        .values()
                        .find(|p| p.reference == if which == 0 { "OLD1" } else { "U2" })
                        .unwrap();
                    assert!(run.owned.contains(&ElectricalOccurrence {
                        class: "pins".into(),
                        source_id: other.pins[0].uuid,
                        instance_path: path.clone(),
                    }));
                } else {
                    let label = sheet
                        .labels
                        .values()
                        .find(|label| label.name == format!("TERU_{}_RETURN", which))
                        .unwrap();
                    assert!(run.owned.contains(&ElectricalOccurrence {
                        class: "labels".into(),
                        source_id: label.uuid,
                        instance_path: path.clone(),
                    }));
                }
            }
            nets.push(run.subject_id);
            records.push(json!({"name":name,"pin":pin,"net":run.subject_id,"members":run.owned}));
        }
        assert_ne!(nets[0], nets[1], "the component connects two distinct nets");
        let ci = model
            .component_instances
            .values()
            .find(|ci| ci.placed_symbol_refs.contains(&placed.uuid))
            .unwrap();
        assert_eq!(ci.part_ref, Some(part));
        assert!(
            ci.library_bindings
                .iter()
                .any(|(_, binding)| binding.target_object_id == part)
        );
        let record = json!({"sheet":sheet_id,"path":path,"placed":placed,"ci":ci,"pins":records});
        let file = self.output.join(format!(
            "circuit-{which}-{}.json",
            if connected { "connected" } else { "placed" }
        ));
        if file.exists() {
            assert_eq!(
                read(&file),
                record,
                "component IDs, source pin mapping and net identity persist"
            );
        } else {
            std::fs::write(file, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
        }
    }

    pub(super) fn authored_circuit(&mut self, events: &ActiveEventLoop, stage: usize) {
        let (which, action) = (stage / 100, stage % 100);
        let old_x = if which == 0 { -6 } else { -2 };
        match action {
            0 => {
                if stage == 0 {
                    std::fs::write(
                        self.output.join("circuit-baseline.json"),
                        serde_json::to_vec_pretty(&source_bytes(&self.root)).unwrap(),
                    )
                    .unwrap();
                }
            }
            1 => {
                let state = self.app.runtime.as_ref().unwrap().workspace();
                let ids = json!({"unit":entry(state,"units",UNIT),"symbol":entry(state,"symbols",SYMBOL),
                    "entity":entry(state,"entities",ENTITY),"part":entry(state,"parts",PART)});
                std::fs::write(
                    self.output.join("circuit-authored.json"),
                    serde_json::to_vec_pretty(&ids).unwrap(),
                )
                .unwrap();
            }
            2 => {
                let layout = &self.app.runtime.as_ref().unwrap().workspace().ui.layout;
                if layout.zoomed != Some(layout.focused) {
                    assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                        "View",
                        "Maximize Pane",
                        EntrySurface::MenuPointer
                    ));
                }
            }
            3 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                "View",
                "Zoom Out",
                EntrySurface::MenuPointer
            )),
            10 => self.circuit_menu("Symbol"),
            11 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
            12 => {
                let index = self.circuit_choice();
                let offset = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring
                    .choice_offset;
                if !(offset..offset + 4).contains(&index) {
                    self.steps
                        .insert(self.stage + 1, Step::AuthoredCircuit(stage));
                    self.control(events, HitTarget::SchematicLibraryPage(index >= offset + 4));
                }
            }
            13 => {
                self.control(
                    events,
                    HitTarget::SchematicLibraryChoice(self.circuit_choice()),
                );
                let expected = self.circuit_sheet(which);
                let editor = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                assert_eq!(editor.placement_sheet(), Some(expected.0));
                assert_eq!(editor.placement_path(), expected.1);
            }
            14 => self.control(events, HitTarget::SchematicPlace),
            15 => self.circuit_mouse(events, point(-10, 0), None),
            16 => self.circuit_click(events, point(-10, 0)),
            17 => self.circuit_check(which, false),
            18 => self.circuit_key(events, Key::Named(NamedKey::Escape)),
            20 | 28 => self.circuit_menu("Wire"),
            21 => self.circuit_click(events, point(-12, 0)),
            22 => self.circuit_click(events, point(-12, -3)),
            23 => self.circuit_click(events, point(old_x, -3)),
            24 => self.circuit_click(events, point(old_x, 0)),
            25 => self.control(events, HitTarget::SchematicFinishWire),
            26 => self.circuit_menu("Label"),
            41 => {
                self.control(events, HitTarget::SchematicLabelName);
                self.circuit_key(
                    events,
                    Key::Character(format!("TERU_{}_RETURN", which).into()),
                );
            }
            27 => self.control(events, HitTarget::SchematicLabelPreview),
            42 => {
                let editor = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                assert!(
                    !editor.picker,
                    "label preview must leave the picker; refusal={:?}",
                    editor.refusal
                );
                assert!(
                    editor.refusal.is_none(),
                    "label preview refusal: {:?}",
                    editor.refusal
                );
                assert_eq!(
                    editor.label.as_ref().unwrap().name,
                    format!("TERU_{}_RETURN", which)
                );
                self.circuit_mouse(events, point(-8, 3), None);
            }
            29 => self.circuit_click(events, point(-8, 3)),
            43 => {
                let (sheet, _) = self.circuit_sheet(which);
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
                let labels = &snapshot.schematic_source().unwrap().sheets[&sheet].labels;
                assert!(
                    labels
                        .values()
                        .any(|label| label.name == format!("TERU_{}_RETURN", which)),
                    "label must be committed before starting Wire"
                );
            }
            30 => self.circuit_click(events, point(-8, 0)),
            31 => self.circuit_click(events, point(-8, 3)),
            32 => {
                self.circuit_check(which, true);
                let (sheet, _) = self.circuit_sheet(which);
                let file = self.root.join(format!("schematic/sheets/{sheet}.json"));
                std::fs::copy(
                    file,
                    self.output
                        .join(format!("circuit-{which}-wire-before.json")),
                )
                .unwrap();
            }
            33 => self.circuit_click(events, point(-12, -1)),
            34 => self.circuit_mouse(events, point(-12, -3), Some(ElementState::Pressed)),
            35 => self.circuit_mouse(events, point(-13, -3), None),
            36 => self.circuit_mouse(events, point(-13, -3), Some(ElementState::Released)),
            37 | 38 | 39 => {
                self.circuit_check(which, true);
                let (sheet, _) = self.circuit_sheet(which);
                let current =
                    std::fs::read(self.root.join(format!("schematic/sheets/{sheet}.json")))
                        .unwrap();
                let old = std::fs::read(
                    self.output
                        .join(format!("circuit-{which}-wire-before.json")),
                )
                .unwrap();
                let after = self.output.join(format!("circuit-{which}-wire-after.json"));
                if action == 37 {
                    assert_ne!(current, old, "connected endpoint drag must commit");
                    let sheet: Value = serde_json::from_slice(&current).unwrap();
                    let ends = sheet["wires"]
                        .as_object()
                        .unwrap()
                        .values()
                        .filter(|wire| {
                            wire["from"] == json!({"x":-13000000,"y":-3000000})
                                || wire["to"] == json!({"x":-13000000,"y":-3000000})
                        })
                        .count();
                    assert_eq!(
                        ends, 2,
                        "both incident wire endpoints follow the connected node"
                    );
                    std::fs::write(after, current).unwrap();
                } else {
                    assert_eq!(
                        current,
                        if action == 38 {
                            old
                        } else {
                            std::fs::read(after).unwrap()
                        }
                    );
                }
            }
            40 => self.circuit_check(which, true),
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires native Root/Child fixture and owned native key template"]
fn ordinary_authored_component_root_child_wiring_history_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let f = |key, value: &str| Step::Field(key, value.into());
    let c = Step::KeyControl;
    let a = Step::AuthoredCircuit;
    let mut steps = vec![
        a(0),
        Step::Open,
        Step::Kind(0),
        c(C::New),
        f("name", UNIT),
        f("pin_name", INPUT),
        Step::Choice("pin_type", "Passive".into()),
        c(C::AddPin),
        f("pin_name", OUTPUT),
        Step::Choice("pin_type", "Passive".into()),
        c(C::Preview),
        Step::Capture("teru-unit-preview"),
        c(C::Commit),
        Step::Kind(1),
        c(C::New),
        f("name", SYMBOL),
        Step::Choice("unit", UNIT.into()),
        f("rect_min_x", "-1mm"),
        f("rect_min_y", "-0.5mm"),
        f("rect_max_x", "1mm"),
        f("rect_max_y", "0.5mm"),
        f("rect_width", "0.15mm"),
        Step::Choice("anchor_pin", INPUT.into()),
        f("anchor_x", "-2mm"),
        f("anchor_y", "0"),
        Step::Choice("anchor_orientation", "Right".into()),
        f("anchor_length", "1mm"),
        c(C::AddAnchor),
        Step::Choice("anchor_pin", OUTPUT.into()),
        f("anchor_x", "2mm"),
        f("anchor_y", "0"),
        Step::Choice("anchor_orientation", "Left".into()),
        f("anchor_length", "1mm"),
        c(C::Preview),
        Step::Capture("teru-symbol-preview"),
        c(C::Commit),
        Step::Kind(2),
        c(C::New),
        f("name", ENTITY),
        f("prefix", "TC"),
        f("gate_name", "A"),
        Step::Choice("gate_unit", UNIT.into()),
        Step::Choice("gate_symbol", SYMBOL.into()),
        c(C::Preview),
        c(C::Commit),
        Step::Kind(3),
        c(C::New),
        f("manufacturer", "Datum workflow"),
        f("mpn", PART),
        f("value", "1k"),
        Step::Choice("part_entity", ENTITY.into()),
        Step::Choice("part_package", "Body".into()),
        c(C::Preview),
        Step::Capture("teru-part-preview"),
        c(C::Commit),
        a(1),
    ];
    for which in 0usize..2 {
        let s = |command: usize| a(which * 100 + command);
        steps.extend([
            Step::PaneContext(Expand),
            Step::PaneContext(Navigate(which == 1)),
            s(2),
            Step::PaneContext(Fit),
            s(3),
            s(3),
            s(3),
            s(10),
            s(11),
            s(12),
            s(13),
            s(14),
            s(15),
            s(16),
            s(18),
            s(17),
            Step::PaneContext(Fit),
            s(20),
            s(21),
            s(22),
            s(23),
            s(24),
            s(25),
            s(18),
            s(26),
            s(41),
            s(27),
            s(42),
            s(29),
            s(43),
            s(18),
            s(28),
            s(30),
            s(31),
            s(25),
            s(18),
            s(32),
            Step::Capture(if which == 0 {
                "teru-root-connected"
            } else {
                "teru-child-connected"
            }),
            s(33),
            s(34),
            s(35),
            Step::Capture(if which == 0 {
                "teru-root-correction-ghost"
            } else {
                "teru-child-correction-ghost"
            }),
            s(36),
            s(37),
            Step::History(false),
            s(38),
            Step::History(true),
            s(39),
        ]);
    }
    steps.extend([
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        a(2),
        Step::PaneContext(Fit),
        a(40),
        Step::Capture("teru-root-cold"),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        a(102),
        Step::PaneContext(Fit),
        a(140),
        Step::Capture("teru-child-cold"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
