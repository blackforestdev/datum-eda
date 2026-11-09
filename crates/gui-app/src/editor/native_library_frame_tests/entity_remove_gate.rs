//! Ordinary shared form removal, atomic used-gate refusal and canonical history.
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
const ENTITY: &str = "6be4c754-4199-4d0d-b848-25ff4b3931c0";
const USED: &str = "027727c3-7f8d-41d1-a699-3de03187f825";
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
fn save(p: &Path, value: &impl serde::Serialize) {
    std::fs::write(p, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
impl Fixture {
    pub(super) fn entity_remove_gate(&mut self, events: &ActiveEventLoop, stage: usize) {
        let intent = read(&self.output.join("remove-gate-intent.json"));
        let id = intent["gate"].as_str().unwrap();
        let expected = |removed: bool| {
            let mut v = intent["before"].clone();
            if removed {
                v["gates"].as_object_mut().unwrap().remove(id);
            }
            v
        };
        match stage {
            0 => {
                self.original_footprint = Some(json!(canonical(&self.root)));
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
                assert_eq!(d.entity.gates.len(), 3);
                assert!(d.request().unwrap().fields.is_empty());
            }
            2 | 7 => {
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
                let key = if stage == 2 { id } else { USED };
                let index = d.entity.gates.keys().position(|k| k == key).unwrap();
                self.control(events, HitTarget::Library(C::Gate(index)));
            }
            3 | 4 => {
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
                    json!({"gate_removals":[id]})
                );
                assert_eq!(json!(d.entity.gates), expected(true)["gates"]);
                assert_eq!(json!(d.entity.saved_gates), intent["before"]["gates"]);
                assert!(d.entity.gate_id.is_none());
                if stage == 4 {
                    assert_eq!(d.preview, Some(expected(true)));
                }
            }
            5 | 6 => {
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                let raw = eda_engine::api::native_write::library::pool_library_object_payload(
                    &model,
                    &format!("pool/entities/{ENTITY}.json"),
                    ENTITY.parse().unwrap(),
                )
                .unwrap();
                assert_eq!(raw, expected(stage == 5));
                assert_eq!(model.objects.contains_key(&id.parse().unwrap()), stage == 6);
                let mut before: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let mut now = canonical(&self.root);
                let path = format!("pool/entities/{ENTITY}.json");
                before.remove(&path);
                now.remove(&path);
                assert_eq!(
                    now, before,
                    "all other authored source, CI, pin, Map and Net bytes stay exact"
                );
            }
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
                assert!(d.preview.is_none());
                assert!(
                    d.refusal
                        .as_ref()
                        .is_some_and(|e| e.contains("schematic placement")),
                    "{:?}",
                    d.refusal
                );
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
                assert_eq!(d.entity.gates.len(), 2);
                assert!(!d.entity.gates.contains_key(id));
                assert!(d.request().unwrap().fields.is_empty());
            }
            _ => panic!("unknown remove stage"),
        }
    }
}
#[test]
#[ignore = "requires real native circuit and owned isolated Wayland"]
fn ordinary_saved_remove_gate_cancel_refusal_history_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let entity: Uuid = ENTITY.parse().unwrap();
    let gate = Uuid::new_v4();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let path = format!("pool/entities/{ENTITY}.json");
    let original = read(&root.join(&path));
    let old = original["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    let before = canonical(&root);
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    save(&output.join("setup-before-inventory.json"), &before);
    std::fs::copy(&journal, output.join("setup-before-journal.jsonl")).unwrap();
    // One canonical AddGate transaction supplies an unused gate; existing A/B
    // remain mapped and cannot serve as legal positive removal targets.
    let setup=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},
        WriteProvenance::new("test",CommitSource::Manual,"Unused Gate fixture"),
        json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_additions":{gate.to_string():{"uuid":gate,"name":"Unused auxiliary","unit":old["unit"],"symbol":old["symbol"]}}}})).unwrap();
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
        &output.join("remove-gate-intent.json"),
        &json!({"gate":gate,"before":read(&root.join(&path)),"original":original}),
    );
    let mut steps = vec![
        Step::EntityRemoveGate(0),
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
        Step::EntityRemoveGate(1),
        Step::EntityRemoveGate(7),
        Step::Control(C::RemoveGate),
        Step::Control(C::Preview),
        Step::EntityRemoveGate(8),
        Step::Invariant,
        Step::Capture("remove-gate-used-refusal"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ];
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::EntityRemoveGate(2),
            Step::Capture(if commit {
                "remove-gate-before"
            } else {
                "remove-gate-cancel-before"
            }),
            Step::Control(C::RemoveGate),
            Step::EntityRemoveGate(3),
            Step::Control(C::Preview),
            Step::EntityRemoveGate(4),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("remove-gate-preview"),
                Step::Control(C::Commit),
                Step::EntityRemoveGate(5),
                Step::Capture("remove-gate-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::EntityRemoveGate(6),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::EntityRemoveGate(6),
        Step::History(true),
        Step::EntityRemoveGate(5),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::EntityRemoveGate(5),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("remove-gate-cold-child"),
        Step::Open,
        Step::Kind(2),
        Step::Search("Dual".into()),
        Step::Select("Dual".into()),
        Step::Control(C::Edit),
        Step::EntityRemoveGate(9),
        Step::Capture("remove-gate-cold-form"),
        Step::Control(C::Cancel),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
