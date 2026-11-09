//! Existing saved Part defaults through the shared form and canonical history.
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
const LAND: &str = "Alternate named default land";
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
    pub(super) fn part_default_footprint(&mut self, _: &ActiveEventLoop, stage: usize) {
        let intent: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("part-default-intent.json")).unwrap(),
        )
        .unwrap();
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let entry = state
            .library
            .entries
            .iter()
            .find(|e| e.kind == "parts" && e.id.to_string() == PART)
            .unwrap();
        let expected = |which| {
            let mut raw = intent["before"].clone();
            if which == 1 {
                raw["default_footprint"] = intent["replacement"].clone();
            }
            if which == 2 {
                raw.as_object_mut().unwrap().remove("default_footprint");
            }
            raw
        };
        match stage {
            0 => {
                self.original_footprint = Some(json!(canonical(&self.root)));
                assert_eq!(entry.raw, expected(0));
            }
            1 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert!(!draft.create);
                assert_eq!(draft.fields.len(), 7);
                assert!(draft.fields[6].selector);
                assert_eq!(draft.part.original.as_ref(), Some(&entry.raw));
                assert!(
                    !draft
                        .fields
                        .iter()
                        .any(|f| matches!(f.key, "part_entity" | "part_package"))
                );
                assert!(
                    draft.fields[6]
                        .choices
                        .iter()
                        .any(|(id, name)| id == intent["replacement"].as_str().unwrap()
                            && name.starts_with(LAND))
                );
            }
            2 | 5 => {
                let draft = state.library.draft.as_ref().unwrap();
                let clear = stage == 5;
                assert_eq!(
                    draft.preview.as_ref(),
                    Some(&expected(if clear { 2 } else { 1 }))
                );
                assert_eq!(draft.request().unwrap().fields,
                    *json!({"default_footprint":if clear {Value::Null} else {intent["replacement"].clone()}}).as_object().unwrap());
            }
            3 | 4 | 6 => {
                assert_eq!(
                    entry.raw,
                    expected(match stage {
                        3 => 1,
                        6 => 2,
                        _ => 0,
                    })
                );
                let mut actual = canonical(&self.root);
                let mut original: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
                let path = format!("pool/parts/{PART}.json");
                actual.remove(&path);
                original.remove(&path);
                assert_eq!(
                    actual, original,
                    "only optional default changes; all other library, placed circuit, CI, Net and Proposal bytes remain exact"
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
                for (id, expected) in intent["net_members"].as_object().unwrap() {
                    let id: Uuid = id.parse().unwrap();
                    let eda_engine::substrate::ElectricalIdentity::Net { anchor, .. } =
                        &model.electrical_identities[&id].identity
                    else {
                        panic!("Net")
                    };
                    assert_eq!(
                        json!(snap.schematic_run(snap.revision(), anchor).unwrap().owned),
                        *expected
                    );
                }
            }
            _ => panic!("unknown default Footprint stage"),
        }
    }
}
#[test]
#[ignore = "requires saved native Part circuit and owned isolated native Wayland"]
fn ordinary_saved_part_default_footprint_change_clear_cancel_history_and_cold() {
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
    let replacement = Uuid::new_v4();
    let package: Uuid = serde_json::from_value(before["package"].clone()).unwrap();
    let mut alternate = eda_engine::api::native_write::library_footprint::pool_footprint_payload(
        replacement,
        package,
        LAND,
    );
    alternate["silkscreen"] =
        json!([{"Line":{"from":{"x":0,"y":0},"to":{"x":2000000,"y":0},"width":120000}}]);
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Named compatible alternate fixture",
        ),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "footprints", replacement),
            object: alternate,
        }],
    )
    .unwrap();
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
    std::fs::write(
        output.join("part-default-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"replacement":replacement,"net_members":net_members}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut steps = vec![
        Step::PartDefaultFootprint(0),
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
        Step::PartDefaultFootprint(1),
        Step::Capture("part-default-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.push(Step::Control(C::Edit));
        }
        steps.extend([
            Step::Choice("part_footprint", LAND.into()),
            Step::Control(C::Preview),
            Step::PartDefaultFootprint(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("part-default-preview"),
                Step::Control(C::Commit),
                Step::PartDefaultFootprint(3),
                Step::Capture("part-default-committed"),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::PartDefaultFootprint(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::PartDefaultFootprint(4),
        Step::History(true),
        Step::PartDefaultFootprint(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("part-default-frozen-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PartDefaultFootprint(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Open,
        Step::Kind(3),
        Step::Search("FIRST".into()),
        Step::Select("FIRST".into()),
        Step::Control(C::Edit),
        Step::PartDefaultFootprint(1),
        Step::Choice("part_footprint", "No default Footprint".into()),
        Step::Control(C::Preview),
        Step::PartDefaultFootprint(5),
        Step::Capture("part-default-clear-preview"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::PartDefaultFootprint(3),
        Step::Control(C::Edit),
        Step::PartDefaultFootprint(1),
        Step::Choice("part_footprint", "No default Footprint".into()),
        Step::Control(C::Preview),
        Step::PartDefaultFootprint(5),
        Step::Control(C::Commit),
        Step::PartDefaultFootprint(6),
        Step::Capture("part-default-cleared"),
        Step::History(false),
        Step::PartDefaultFootprint(3),
        Step::History(true),
        Step::PartDefaultFootprint(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PartDefaultFootprint(6),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Open,
        Step::Kind(3),
        Step::Search("FIRST".into()),
        Step::Select("FIRST".into()),
        Step::Control(C::Edit),
        Step::PartDefaultFootprint(1),
        Step::Capture("part-default-cold-cleared-form"),
        Step::KeyControl(C::Cancel),
        Step::PartDefaultFootprint(6),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
