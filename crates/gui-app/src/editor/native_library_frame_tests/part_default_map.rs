//! Saved default-map choice, composed defaults and explicit clear in ordinary App.
use super::*;
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
const LAND: &str = "Composed map alternate land";
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
fn expected(intent: &Value, which: usize) -> Value {
    let mut raw = intent["before"].clone();
    if which == 1 {
        raw["default_pin_pad_map"] = intent["map_a"].clone();
    }
    if which >= 2 {
        raw["default_footprint"] = intent["footprint"].clone();
        raw["default_pin_pad_map"] = intent["map_b"].clone();
    }
    if which == 3 {
        raw.as_object_mut().unwrap().remove("default_pin_pad_map");
    }
    raw
}
impl Fixture {
    pub(super) fn part_default_map(&mut self, _: &ActiveEventLoop, stage: usize) {
        let intent: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("part-map-intent.json")).unwrap(),
        )
        .unwrap();
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "parts" && e.id.to_string() == PART)
            .unwrap();
        if stage == 0 {
            self.original_footprint = Some(json!(canonical(&self.root)));
        }
        if stage == 1 {
            let d = state.library.draft.as_ref().unwrap();
            assert!(!d.create);
            assert_eq!(d.fields.len(), 10);
            assert!(d.fields[9].selector);
            assert_eq!(d.part.original.as_ref(), Some(&entry.raw));
            assert!(d.fields[9].choices.iter().any(|(id, label)| (id
                == intent["map_a"].as_str().unwrap()
                || id == intent["map_b"].as_str().unwrap())
                && label.starts_with("Part: ")
                && label.contains(" · pool/pin_pad_maps/")));
            return;
        }
        let preview = matches!(stage, 2 | 4 | 6);
        let which = match stage {
            0 | 8 => 0,
            2 | 3 => 1,
            4 | 5 => 2,
            6 | 7 => 3,
            _ => panic!("unknown map stage"),
        };
        if preview {
            let d = state.library.draft.as_ref().unwrap();
            assert_eq!(d.preview.as_ref(), Some(&expected(&intent, which)));
            let fields = d.request().unwrap().fields;
            let wanted = match which {
                1 => json!({"default_pin_pad_map":intent["map_a"]}),
                2 => {
                    json!({"default_footprint":intent["footprint"],"default_pin_pad_map":intent["map_b"]})
                }
                _ => json!({"default_pin_pad_map":null}),
            };
            assert_eq!(fields, *wanted.as_object().unwrap());
            return;
        }
        assert_eq!(entry.raw, expected(&intent, which));
        let mut actual = canonical(&self.root);
        let mut original: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
        let path = format!("pool/parts/{PART}.json");
        actual.remove(&path);
        original.remove(&path);
        assert_eq!(
            actual, original,
            "only explicit Part defaults change; all Map payloads and old circuit/source/CI/Net/Proposal bytes stay exact"
        );
        let model = state.library.basis.as_ref().unwrap();
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_")),
            "{:?}",
            model.diagnostics
        );
        let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
        for (id, members) in intent["net_members"].as_object().unwrap() {
            let id: Uuid = id.parse().unwrap();
            let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                panic!("Net")
            };
            assert_eq!(
                json!(snap.schematic_run(snap.revision(), anchor).unwrap().owned),
                *members
            );
        }
    }
}
#[test]
#[ignore = "requires saved native Part circuit and owned isolated native Wayland"]
fn ordinary_saved_part_default_map_composed_clear_cancel_history_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let part: Uuid = PART.parse().unwrap();
    let before =
        library::pool_library_object_payload(&model, &format!("pool/parts/{PART}.json"), part)
            .unwrap();
    assert!(before["default_pin_pad_map"].is_null());
    let footprint = Uuid::new_v4();
    let map_a = Uuid::new_v4();
    let map_b = Uuid::new_v4();
    let package = serde_json::from_value(before["package"].clone()).unwrap();
    let alternate = eda_engine::api::native_write::library_footprint::pool_footprint_payload(
        footprint, package, LAND,
    );
    let p = library::build_pool_library_write(&model,
        WriteProvenance::new("test", CommitSource::Manual, "Existing named compatible Map fixtures"), Some("pool"), vec![
            Spec::Create {target:Target::new("pool","footprints",footprint),object:alternate},
            Spec::Create {target:Target::new("pool","pin_pad_maps",map_a),object:json!({"uuid":map_a,"part":part,"footprint":before["default_footprint"],"schema_version":1,"mappings":{},"tags":["retained", "ordered"]})},
            Spec::Create {target:Target::new("pool","pin_pad_maps",map_b),object:json!({"uuid":map_b,"part":part,"footprint":footprint,"schema_version":1,"mappings":{}})},
        ]).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let net_members: BTreeMap<_, _> = snap
        .net_ids()
        .into_iter()
        .map(|id| {
            let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                unreachable!()
            };
            (
                id,
                snap.schematic_run(snap.revision(), anchor).unwrap().owned,
            )
        })
        .collect();
    std::fs::write(output.join("part-map-intent.json"),serde_json::to_vec_pretty(&json!({"before":before,"map_a":map_a,"map_b":map_b,"footprint":footprint,"net_members":net_members})).unwrap()).unwrap();
    let mut steps = vec![
        Step::PartDefaultMap(0),
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
        Step::PartDefaultMap(1),
        Step::Capture("part-map-before"),
    ];
    // Each episode previews/cancels first, then explicitly commits the same choice.
    let map_a_label = format!(
        "Part: {} · Footprint: ",
        before["manufacturer"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|m| format!("{m} {}", before["mpn"].as_str().unwrap()))
            .unwrap_or_else(|| before["mpn"].as_str().unwrap().to_string())
    );
    // Exact source-path disambiguates compatible Maps without adding a naming policy.
    let original_foot: Uuid = serde_json::from_value(before["default_footprint"].clone()).unwrap();
    let original_foot_name = library::pool_library_object_payload(
        &model,
        &format!("pool/footprints/{original_foot}.json"),
        original_foot,
    )
    .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();
    let map_a_label = format!("{map_a_label}{original_foot_name} · pool/pin_pad_maps/{map_a}.json");
    let map_b_label = map_a_label
        .split(" · Footprint: ")
        .next()
        .unwrap()
        .to_string()
        + " · Footprint: "
        + LAND;
    for (episode, current, prior) in [(0, 3, 8), (1, 5, 3), (2, 7, 5)] {
        for commit in [false, true] {
            if episode != 0 || commit {
                steps.push(Step::Control(C::Edit));
            }
            if episode == 1 {
                steps.push(Step::Choice("part_footprint", LAND.into()));
            }
            steps.push(Step::Choice(
                "part_map",
                match episode {
                    0 => map_a_label.clone(),
                    1 => map_b_label.clone(),
                    _ => "No default PinPadMap".into(),
                },
            ));
            steps.extend([
                Step::Control(C::Preview),
                Step::PartDefaultMap(current - 1),
                Step::Invariant,
            ]);
            if commit {
                steps.extend([
                    Step::Capture(match episode {
                        0 => "part-map-preview",
                        1 => "part-map-composed-preview",
                        _ => "part-map-clear-preview",
                    }),
                    Step::Control(C::Commit),
                    Step::PartDefaultMap(current),
                ]);
            } else {
                steps.extend([
                    Step::KeyControl(C::Cancel),
                    Step::Invariant,
                    Step::PartDefaultMap(prior),
                ]);
            }
        }
        steps.extend([
            Step::History(false),
            Step::PartDefaultMap(prior),
            Step::History(true),
            Step::PartDefaultMap(current),
            Step::Save,
            Step::Snapshot,
            Step::Reopen,
            Step::Invariant,
            Step::PartDefaultMap(current),
            Step::PaneContext(Navigate(false)),
            Step::EntityEdit(5),
            Step::PaneContext(Expand),
            Step::PaneContext(Navigate(true)),
            Step::EntityEdit(6),
            Step::Open,
            Step::Kind(3),
            Step::Search("FIRST".into()),
            Step::Select("FIRST".into()),
        ]);
    }
    steps.extend([
        Step::Control(C::Edit),
        Step::PartDefaultMap(1),
        Step::Capture("part-map-cold-cleared-form"),
        Step::KeyControl(C::Cancel),
        Step::PartDefaultMap(7),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
