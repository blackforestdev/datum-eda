//! Ordinary saved gate-name editing, row navigation and unchanged placed authority.
use super::*;
use eda_engine::substrate::{ElectricalIdentity, ElectricalSelectionSnapshot, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;
const ENTITY: &str = "6be4c754-4199-4d0d-b848-25ff4b3931c0";
const NAMES: [&str; 2] = ["Amplifier input", "Amplifier output"];
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
fn members(model: &eda_engine::substrate::DesignModel) -> BTreeMap<Uuid, Value> {
    let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
    snap.net_ids()
        .into_iter()
        .map(|id| {
            let ElectricalIdentity::Net { anchor, .. } = &model.electrical_identities[&id].identity
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
    fn gate_name_intent(&self) -> Value {
        read(&self.output.join("gate-name-intent.json"))
    }
    fn gate_name_expected(&self, edited: bool) -> Value {
        let intent = self.gate_name_intent();
        let mut raw = intent["before"].clone();
        if edited {
            for (i, key) in intent["keys"].as_array().unwrap().iter().enumerate() {
                raw["gates"][key.as_str().unwrap()]["name"] = json!(NAMES[i]);
            }
        }
        raw
    }
    pub(super) fn entity_gate_name(&mut self, events: &ActiveEventLoop, stage: usize) {
        if stage >= 10 {
            match stage {
                10 => assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Place",
                    "Symbol",
                    EntrySurface::MenuPointer
                )),
                11 => self.control(events, HitTarget::Library(C::ProjectSymbols)),
                12 => {
                    let intent = self.gate_name_intent();
                    let choices = &self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .schematic_authoring
                        .choices;
                    for (i, key) in intent["keys"].as_array().unwrap().iter().enumerate() {
                        let gate: Uuid = key.as_str().unwrap().parse().unwrap();
                        let matching: Vec<_> = choices
                            .iter()
                            .filter(|c| {
                                c.explicit_part.as_ref().is_some_and(|p| {
                                    p.entity == ENTITY.parse::<Uuid>().unwrap() && p.gate == gate
                                })
                            })
                            .collect();
                        assert!(!matching.is_empty());
                        assert!(matching.iter().all(|c| c.name.contains(NAMES[i])));
                    }
                }
                13 => self.control(events, HitTarget::SchematicCancel),
                _ => panic!("unknown choice stage"),
            }
            return;
        }
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "entities" && e.id.to_string() == ENTITY)
            .unwrap();
        let intent = self.gate_name_intent();
        match stage {
            0 => {
                assert_eq!(entry.raw, self.gate_name_expected(false));
                self.original_footprint = Some(json!(canonical(&self.root)));
            }
            1 => {
                let d = state.library.draft.as_ref().unwrap();
                assert!(!d.create);
                assert!(d.fields.iter().any(|f| f.key == "gate_name" && !f.selector));
                assert!(!d.fields.iter().any(|f| f.key == "gate_unit"));
                assert!(d.request().unwrap().fields.is_empty());
            }
            2 | 3 => {
                let d = state.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_name"), NAMES[0]);
                let changes: serde_json::Map<_, _> = intent["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, key)| (key.as_str().unwrap().to_string(), json!(NAMES[i])))
                    .collect();
                assert_eq!(
                    d.request().unwrap().fields,
                    *json!({"gate_names":changes}).as_object().unwrap()
                );
                assert_eq!(
                    json!(d.entity_gates().unwrap()),
                    self.gate_name_expected(true)["gates"]
                );
                if stage == 3 {
                    assert_eq!(d.preview, Some(self.gate_name_expected(true)));
                }
                assert_eq!(json!(d.entity.saved_gates), intent["before"]["gates"]);
            }
            4 | 5 => {
                assert_eq!(entry.raw, self.gate_name_expected(stage == 4));
                let mut now = canonical(&self.root);
                let mut before: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let path = format!("pool/entities/{ENTITY}.json");
                now.remove(&path);
                before.remove(&path);
                assert_eq!(
                    now, before,
                    "all root/child placed sources, CIs, mappings, references and electrical source bytes remain exact"
                );
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                assert_eq!(json!(members(&model)), intent["net_members"]);
            }
            6 | 7 => {
                let d = state.library.draft.as_ref().unwrap();
                assert_eq!(d.value("gate_name"), NAMES[stage - 6]);
                assert!(
                    d.request().unwrap().fields.is_empty(),
                    "cold selection cannot author a rename"
                );
            }
            _ => panic!("unknown gate-name stage"),
        }
    }
}
#[test]
#[ignore = "requires saved root/child native Part circuit and owned isolated native Wayland"]
fn ordinary_saved_gate_names_row_navigation_cancel_history_cold_and_current_captions() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let before = eda_engine::api::native_write::library::pool_library_object_payload(
        &model,
        &format!("pool/entities/{ENTITY}.json"),
        ENTITY.parse().unwrap(),
    )
    .unwrap();
    let keys: Vec<_> = before["gates"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(keys.len(), 2);
    std::fs::write(
        output.join("gate-name-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"keys":keys,"net_members":members(&model)}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::EntityGateName(0),
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
        Step::EntityGateName(1),
        Step::Capture("gate-name-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Control(C::Gate(0)),
            Step::Field("gate_name", NAMES[0].into()),
            Step::Control(C::Gate(1)),
            Step::Field("gate_name", NAMES[1].into()),
            Step::Control(C::Gate(0)),
            Step::EntityGateName(2),
            Step::Control(C::Preview),
            Step::EntityGateName(3),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("gate-name-preview"),
                Step::Control(C::Commit),
                Step::EntityGateName(4),
                Step::Capture("gate-name-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::EntityGateName(5),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::EntityGateName(5),
        Step::History(true),
        Step::EntityGateName(4),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("gate-name-old-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::EntityGateName(4),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("gate-name-cold-child"),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(2),
        Step::Search("Dual".into()),
        Step::Select("Dual".into()),
        Step::Control(C::Edit),
        Step::Control(C::Gate(0)),
        Step::EntityGateName(6),
        Step::Control(C::Gate(1)),
        Step::EntityGateName(7),
        Step::Capture("gate-name-cold-form"),
        Step::Control(C::Cancel),
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::EntityGateName(10),
        Step::EntityGateName(11),
        Step::EntityGateName(12),
        Step::Capture("gate-name-current-choices"),
        Step::EntityGateName(13),
        Step::Invariant,
        Step::EntityGateName(4),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
