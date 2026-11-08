//! Existing Symbol correction through shared field paint/pointer/keyboard ownership.
use super::*;
use serde_json::{Value, json};
use winit::keyboard::ModifiersState;

const SYMBOL: &str = "da837e11-8aa9-4f65-8a25-ce253fb11c5c";
const NAME: &str = "Edited μ symbol";

impl Fixture {
    fn caret_key(&mut self, events: &ActiveEventLoop, key: Key, modifiers: ModifiersState) {
        let id = self.app.window.as_ref().unwrap().id();
        self.app
            .window_event(events, id, WindowEvent::ModifiersChanged(modifiers.into()));
        self.key(key);
        self.app.window_event(
            events,
            id,
            WindowEvent::ModifiersChanged(ModifiersState::empty().into()),
        );
    }
    fn field_start(&mut self, events: &ActiveEventLoop, key: &str) {
        let r = self.app.runtime.as_ref().unwrap();
        let d = r.workspace().library.draft.as_ref().unwrap();
        let index = d.fields.iter().position(|f| f.key == key).unwrap();
        assert_eq!(r.workspace().library.focus, Some(C::Field(index)));
        let target = HitTarget::Library(C::Field(index));
        let rect = r
            .presented_hits
            .regions()
            .iter()
            .find(|hit| hit.target == target)
            .expect("focused field must be presented")
            .rect;
        let (x, y) = (rect.x + 1., rect.y + rect.height / 2.);
        assert_eq!(r.presented_hits.hit_test(x, y), Some(&target));
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(x as f64, y as f64),
            },
        );
        for state in [ElementState::Pressed, ElementState::Released] {
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
            d.fields[index].editor.caret, 0,
            "pointer uses actual painted value rectangle"
        );
        assert_eq!(d.fields[index].editor.anchor, 0);
    }
    fn caret_source(&self, edited: bool) {
        let mut expected: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("caret-source-before.json")).unwrap(),
        )
        .unwrap();
        if edited {
            expected["name"] = json!(NAME);
            expected["pin_anchors"][0]["position"]["x"] = json!(-2600000);
        }
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.id.to_string() == SYMBOL && e.kind == "symbols")
            .unwrap();
        assert_eq!(
            entry.raw, expected,
            "exact source payload, logical pins, other anchors, geometry and unknown metadata survive correction/history/cold"
        );
        for (path, before) in &self.snapshot {
            if (path.starts_with("pool/") && path != &format!("pool/symbols/{SYMBOL}.json"))
                || path.starts_with("board/")
                || path.starts_with("schematic/")
                || path.starts_with("components/")
            {
                assert_eq!(
                    std::fs::read(self.root.join(path)).unwrap(),
                    *before,
                    "unrelated authored source {path}"
                );
            }
        }
    }
    pub(super) fn text_fields_stage(&mut self, events: &ActiveEventLoop, stage: usize) {
        assert!(
            self.key_template.is_some(),
            "caret proof requires genuine native backend template"
        );
        match stage {
            0 => {
                let state = self.app.runtime.as_ref().unwrap().workspace();
                let e = state
                    .library
                    .entries
                    .iter()
                    .find(|e| e.id.to_string() == SYMBOL && e.kind == "symbols")
                    .unwrap();
                assert_eq!(e.raw["name"], json!("Edited symbol"));
                assert_eq!(e.raw["pin_anchors"].as_array().unwrap().len(), 1);
                assert_eq!(e.raw["pin_anchors"][0]["position"]["x"], json!(-2500000));
                std::fs::write(
                    self.output.join("caret-source-before.json"),
                    serde_json::to_vec_pretty(&e.raw).unwrap(),
                )
                .unwrap();
            }
            1 => self.focus_field("name"),
            2 => {
                self.field_start(events, "name");
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
                assert!(d.dirty.is_empty());
                assert!(d.preview.is_some());
            }
            3 => {
                self.caret_key(events, Key::Character("a".into()), ModifiersState::CONTROL);
                self.caret_key(events, Key::Character("c".into()), ModifiersState::CONTROL);
            }
            4 => {
                self.caret_key(events, Key::Character("v".into()), ModifiersState::CONTROL);
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
                assert_eq!(d.value("name"), "Edited symbol");
                assert!(d.dirty.is_empty());
                assert!(d.preview.is_some());
                self.insert_caret_name(events);
            }
            5 => self.focus_field("anchor_x"),
            6 => {
                assert_eq!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .as_ref()
                        .unwrap()
                        .value("anchor_x"),
                    "-2500000",
                    "select the existing anchor before editing its numeric value"
                );
                self.field_start(events, "anchor_x");
                self.key(Key::Named(NamedKey::ArrowRight));
                self.key(Key::Named(NamedKey::ArrowRight));
                self.caret_key(
                    events,
                    Key::Named(NamedKey::ArrowRight),
                    ModifiersState::SHIFT,
                );
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
                    d.fields[d.focused]
                        .editor
                        .selected_text(d.value("anchor_x")),
                    "5"
                );
            }
            7 => {
                self.key(Key::Character("6".into()));
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
                assert_eq!(d.value("anchor_x"), "-2600000");
                assert!(d.preview.is_none());
                assert!(d.dirty.contains("anchor_x"));
            }
            8 => self.focus_field("anchor_orientation"),
            9 => {
                let before = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap()
                    .value("anchor_orientation")
                    .to_owned();
                self.key(Key::Named(NamedKey::ArrowRight));
                assert_ne!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .as_ref()
                        .unwrap()
                        .value("anchor_orientation"),
                    before
                );
                self.key(Key::Named(NamedKey::ArrowLeft));
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
                assert_eq!(d.value("anchor_orientation"), before);
                assert_eq!(d.value("name"), NAME);
                assert_eq!(d.value("anchor_x"), "-2600000");
            }
            10 => {
                assert!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .is_none()
                );
                self.caret_source(false);
            }
            11 => {
                self.field_start(events, "name");
                self.insert_caret_name(events);
            }
            12 => {
                self.text_fields_stage(events, 6);
                self.text_fields_stage(events, 7);
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
                assert!(d.preview.is_some());
                assert!(d.refusal.is_none());
                assert_eq!(d.value("name"), NAME);
                assert_eq!(d.value("anchor_x"), "-2600000");
            }
            14 => self.caret_source(true),
            15 => self.caret_source(false),
            _ => unreachable!(),
        }
    }
    fn insert_caret_name(&mut self, events: &ActiveEventLoop) {
        self.key(Key::Named(NamedKey::Home));
        for _ in 0..7 {
            self.key(Key::Named(NamedKey::ArrowRight));
        }
        self.caret_key(events, Key::Character("μ ".into()), ModifiersState::empty());
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
            d.value("name"),
            NAME,
            "middle insertion preserves the existing suffix"
        );
        assert!(d.preview.is_none());
        assert!(d.dirty.contains("name"));
    }
}

#[test]
#[ignore = "requires saved ordinary Symbol project and owned native keyboard-template transport"]
fn ordinary_shared_library_caret_name_numeric_selector_cancel_commit_history_save_cold() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let steps = vec![
        Step::TextFields(0),
        Step::Snapshot,
        Step::Open,
        Step::Kind(1),
        Step::Search("Edited symbol".into()),
        Step::Select("Edited symbol".into()),
        Step::Control(C::Edit),
        Step::Control(C::Preview),
        Step::TextFields(1),
        Step::TextFields(2),
        Step::Capture("library-name-caret"),
        Step::TextFields(3),
        Step::Capture("library-name-selection"),
        Step::TextFields(4),
        Step::Capture("library-name-corrected"),
        Step::KeyControl(C::Anchor(0)),
        Step::TextFields(5),
        Step::TextFields(6),
        Step::Capture("library-number-selection"),
        Step::TextFields(7),
        Step::TextFields(8),
        Step::TextFields(9),
        Step::Control(C::Preview),
        Step::TextFields(13),
        Step::Capture("library-corrected-preview"),
        Step::Invariant,
        Step::Control(C::Cancel),
        Step::Invariant,
        Step::TextFields(10),
        Step::Control(C::Edit),
        Step::TextFields(1),
        Step::TextFields(11),
        Step::KeyControl(C::Anchor(0)),
        Step::TextFields(5),
        Step::TextFields(12),
        Step::TextFields(8),
        Step::TextFields(9),
        Step::Control(C::Preview),
        Step::TextFields(13),
        Step::Control(C::Commit),
        Step::TextFields(14),
        Step::Capture("library-committed"),
        Step::History(false),
        Step::TextFields(15),
        Step::History(true),
        Step::TextFields(14),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::TextFields(14),
        Step::Open,
        Step::Kind(1),
        Step::Search(NAME.into()),
        Step::Select(NAME.into()),
        Step::Capture("library-cold-corrected"),
        Step::Invariant,
    ];
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
