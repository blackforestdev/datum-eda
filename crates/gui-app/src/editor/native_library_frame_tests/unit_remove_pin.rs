//! Ordinary saved Unit pin removal with actual used refusal and preserved circuits.
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    },
    substrate::{CommitSource, ProjectResolver},
};
use serde_json::{Value, json};
use uuid::Uuid;
const UNIT: &str = "f69c04fd-a070-4985-b5ed-c42426f4f259";
const USED: &str = "35c36e99-78c5-4bb5-a50e-19a2bf0d47ed";
const NAME: &str = "Unused diagnostic";
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn save(path: &Path, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
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
impl Fixture {
    fn removed_unit_pin_sources(&self, removed: bool) {
        let data = read(&self.output.join("remove-unit-pin-intent.json"));
        let mut expected = data["before"].clone();
        let pin = data["pin"].as_str().unwrap();
        if removed {
            expected["pins"].as_object_mut().unwrap().remove(pin);
        }
        assert_eq!(
            read(&self.root.join(format!("pool/units/{UNIT}.json"))),
            expected
        );
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        assert_eq!(model.objects.contains_key(&pin.parse().unwrap()), !removed);
        let old = self.original_footprint.as_ref().unwrap();
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        let mut now = canonical(&self.root);
        let path = format!("pool/units/{UNIT}.json");
        before.remove(&path);
        now.remove(&path);
        assert_eq!(
            now, before,
            "all other authored sources, CI, mappings and root/child circuits stay byte-exact"
        );
        assert_eq!(json!(model.component_instances), old["ci"]);
        assert_eq!(json!(model.electrical_identities), old["ei"]);
    }
    pub(super) fn unit_remove_pin(&mut self, events: &ActiveEventLoop, stage: usize) {
        let data = read(&self.output.join("remove-unit-pin-intent.json"));
        let pin = data["pin"].as_str().unwrap();
        match stage {
            0 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                self.original_footprint = Some(
                    json!({"files":canonical(&self.root),"ci":model.component_instances,"ei":model.electrical_identities}),
                );
                self.removed_unit_pin_sources(false);
            }
            1 | 2 => {
                let key = if stage == 1 { USED } else { pin };
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
                let index = d.electrical.pins.keys().position(|k| k == key).unwrap();
                self.control(events, HitTarget::Library(C::Pin(index)));
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
                assert!(d.preview.is_none());
                assert!(
                    d.refusal
                        .as_ref()
                        .is_some_and(|s| s.contains("Symbol anchor")),
                    "{:?}",
                    d.refusal
                );
                self.removed_unit_pin_sources(false);
            }
            4 | 5 => {
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
                    json!({"pin_removals":[pin]})
                );
                assert!(d.electrical.pin_id.is_none());
                assert!(!d.electrical.pins.contains_key(pin));
                if stage == 5 {
                    let mut expected = data["before"].clone();
                    expected["pins"].as_object_mut().unwrap().remove(pin);
                    assert_eq!(d.preview, Some(expected));
                }
            }
            6 | 7 => self.removed_unit_pin_sources(stage == 6),
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
                assert!(!d.create);
                assert!(!d.electrical.pins.contains_key(pin));
                assert!(d.electrical.pins.contains_key(USED));
                assert!(d.request().unwrap().fields.is_empty());
            }
            9 => {
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
                assert!(d.create);
                let choices = &d
                    .fields
                    .iter()
                    .find(|f| f.key == "anchor_pin")
                    .unwrap()
                    .choices;
                assert!(!choices.iter().any(|(id, _)| id == pin));
                for (id, row) in data["original"]["pins"].as_object().unwrap() {
                    assert!(choices.contains(&(id.clone(), row["name"].as_str().unwrap().into())));
                }
            }
            _ => panic!("unknown saved Unit pin removal stage"),
        }
    }
}
#[test]
#[ignore = "requires real native root/child circuits and owned isolated Wayland"]
fn ordinary_saved_unit_remove_pin_refusal_cancel_history_cold_and_symbol_choices() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let pin = Uuid::new_v4();
    let path = format!("pool/units/{UNIT}.json");
    let original = read(&root.join(&path));
    let before = canonical(&root);
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    save(&output.join("setup-before-inventory.json"), &before);
    std::fs::copy(&journal, output.join("setup-before-journal.jsonl")).unwrap();
    // Only setup mutation: canonical AddPin supplies a genuinely unused pin.
    let setup=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},WriteProvenance::new("test",CommitSource::Manual,"Unused logical pin fixture"),
        json!({"kind":"units","object_id":UNIT,"pool_path":"pool","fields":{"pin_additions":{pin.to_string():{"name":NAME,"electrical_type":"Passive"}}}})).unwrap();
    commit_prepared(&mut model, &root, setup).unwrap();
    let after = canonical(&root);
    assert_eq!(before.len(), after.len());
    for (p, bytes) in &before {
        if *p != path {
            assert_eq!(after[p], *bytes);
        }
    }
    save(&output.join("setup-after-inventory.json"), &after);
    std::fs::copy(&journal, output.join("setup-after-journal.jsonl")).unwrap();
    save(
        &output.join("remove-unit-pin-intent.json"),
        &json!({"pin":pin,"before":read(&root.join(&path)),"original":original}),
    );
    let mut steps = vec![
        Step::UnitRemovePin(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(0),
        Step::Search("Logic".into()),
        Step::Select("Logic".into()),
        Step::Control(C::Edit),
        Step::UnitRemovePin(1),
        Step::Focus(C::RemovePin),
        Step::Capture("remove-unit-pin-used-before"),
        Step::Control(C::RemovePin),
        Step::Control(C::Preview),
        Step::UnitRemovePin(3),
        Step::Invariant,
        Step::Capture("remove-unit-pin-used-refusal"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ];
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::UnitRemovePin(2),
            Step::Focus(C::RemovePin),
        ]);
        if commit {
            steps.push(Step::Capture("remove-unit-pin-unused-before"));
        }
        steps.extend([
            Step::Field("pin_name", "".into()),
            Step::Control(C::RemovePin),
            Step::UnitRemovePin(4),
            Step::Control(C::Preview),
            Step::UnitRemovePin(5),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("remove-unit-pin-preview"),
                Step::Control(C::Commit),
                Step::UnitRemovePin(6),
                Step::Capture("remove-unit-pin-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::UnitRemovePin(7),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::UnitRemovePin(7),
        Step::History(true),
        Step::UnitRemovePin(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::UnitRemovePin(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::Capture("remove-unit-pin-cold-root"),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Open,
        Step::Kind(0),
        Step::Search("Logic".into()),
        Step::Select("Logic".into()),
        Step::Control(C::Edit),
        Step::UnitRemovePin(8),
        Step::Capture("remove-unit-pin-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::Kind(1),
        Step::Control(C::New),
        Step::Field("name", "Retired pin choices".into()),
        Step::Choice("unit", "Logic".into()),
        Step::UnitRemovePin(9),
        Step::Capture("remove-unit-pin-symbol-choices"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::UnitRemovePin(6),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
