//! Saved Unit pin alternate names and swap group through the ordinary App.
use super::*;
use serde_json::{Value, json};

const UNIT_SOURCE: &str = "pool/units/f69c04fd-a070-4985-b5ed-c42426f4f259.json";
const PIN: &str = "f22cf09d-67d1-4b71-abae-1129a8e8af8e";

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn canonical_siblings(root: &Path) -> BTreeMap<String, Vec<u8>> {
    source_bytes(root)
        .into_iter()
        .filter(|(path, _)| {
            path != UNIT_SOURCE
                && (!path.starts_with(".datum/")
                    || [
                        ".datum/component_instances/",
                        ".datum/electrical_identities/",
                        ".datum/proposals/",
                    ]
                    .iter()
                    .any(|prefix| path.starts_with(prefix)))
        })
        .collect()
}

fn expected(mut unit: Value, phase: usize) -> Value {
    let pin = &mut unit["pins"][PIN];
    match phase {
        0 => {}
        1 => {
            pin["swap_group"] = json!(7);
            pin["alternates"] =
                json!([{"name":"AUX","kind":"signal"},{"name":"REMOVE","kind":"alias"}]);
        }
        2 => {
            pin["swap_group"] = json!(12);
            pin["alternates"] = json!([{"name":" AUX READY ","kind":" alternate "}]);
        }
        _ => unreachable!(),
    }
    unit
}

impl Fixture {
    pub(super) fn unit_pin_metadata(&self, stage: usize) {
        let record = read(&self.output.join("unit-pin-metadata-intent.json"));
        let state = self.app.runtime.as_ref().unwrap().workspace();
        match stage {
            0..=2 => {
                assert_eq!(
                    read(&self.root.join(UNIT_SOURCE)),
                    expected(record["before"].clone(), stage)
                );
                let siblings: BTreeMap<String, Vec<u8>> =
                    serde_json::from_value(record["siblings"].clone()).unwrap();
                assert_eq!(canonical_siblings(&self.root), siblings);
            }
            11 | 12 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert!(draft.refusal.is_none());
                assert_eq!(
                    draft.preview,
                    Some(expected(record["before"].clone(), stage - 10))
                );
            }
            13 => {
                let draft = state.library.draft.as_ref().unwrap();
                assert_eq!(draft.electrical.pin_id.unwrap().to_string(), PIN);
                assert_eq!(draft.value("pin_swap_group"), "12");
                assert_eq!(draft.value("pin_alternate_name"), " AUX READY ");
                assert_eq!(draft.value("pin_alternate_kind"), " alternate ");
                assert!(draft.request().unwrap().fields.is_empty());
            }
            _ => unreachable!(),
        }
    }
}

#[test]
#[ignore = "requires a native project and owned isolated Wayland"]
fn ordinary_saved_unit_pin_alternates_swap_group_preview_cancel_history_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let before = read(&root.join(UNIT_SOURCE));
    assert_eq!(before["pins"][PIN]["name"], "Auxiliary supply");
    assert_eq!(before["pins"][PIN]["swap_group"], 0);
    assert_eq!(before["pins"][PIN]["alternates"], json!([]));
    let mut keys: Vec<_> = before["pins"].as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys.len(), 2);
    let pin = keys.iter().position(|key| key.as_str() == PIN).unwrap();
    std::fs::write(
        output.join("unit-pin-metadata-intent.json"),
        serde_json::to_vec_pretty(
            &json!({"before":before,"siblings":canonical_siblings(&root),"setup_transactions":0}),
        )
        .unwrap(),
    )
    .unwrap();

    let field = |key, value: &str| Step::Field(key, value.into());
    let open = || {
        [
            Step::Open,
            Step::Kind(0),
            Step::Search("Logic".into()),
            Step::Select("Logic".into()),
        ]
    };
    let mut steps = vec![Step::UnitPinMetadata(0), Step::Snapshot];
    steps.extend(open());
    for commit in [false, true] {
        steps.extend([
            Step::Control(C::Edit),
            Step::KeyControl(C::Pin(pin)),
            field("pin_swap_group", "7"),
            Step::KeyControl(C::AddPinAlternate),
            field("pin_alternate_name", "AUX"),
            field("pin_alternate_kind", "signal"),
            Step::KeyControl(C::AddPinAlternate),
            field("pin_alternate_name", "REMOVE"),
            field("pin_alternate_kind", "alias"),
            Step::KeyControl(C::Pin(1 - pin)),
            Step::KeyControl(C::Pin(pin)),
            Step::KeyControl(C::Preview),
            Step::UnitPinMetadata(11),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("unit-pin-alternates-add-preview"),
                Step::KeyControl(C::Commit),
                Step::UnitPinMetadata(1),
            ]);
        } else {
            steps.extend([
                Step::KeyControl(C::Cancel),
                Step::Invariant,
                Step::UnitPinMetadata(0),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::UnitPinMetadata(0),
        Step::History(true),
        Step::UnitPinMetadata(1),
        Step::Snapshot,
        Step::Control(C::Edit),
        Step::KeyControl(C::Pin(pin)),
        Step::KeyControl(C::PinAlternate(0)),
        field("pin_alternate_name", " AUX READY "),
        field("pin_alternate_kind", " alternate "),
        Step::KeyControl(C::PinAlternate(1)),
        Step::KeyControl(C::RemovePinAlternate),
        field("pin_swap_group", "12"),
        Step::KeyControl(C::Preview),
        Step::UnitPinMetadata(12),
        Step::Invariant,
        Step::Capture("unit-pin-alternates-edit-preview"),
        Step::KeyControl(C::Commit),
        Step::UnitPinMetadata(2),
        Step::History(false),
        Step::UnitPinMetadata(1),
        Step::History(true),
        Step::UnitPinMetadata(2),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::UnitPinMetadata(2),
    ]);
    steps.extend(open());
    steps.extend([
        Step::Control(C::Edit),
        Step::KeyControl(C::Pin(pin)),
        Step::KeyControl(C::PinAlternate(0)),
        Step::UnitPinMetadata(13),
        Step::Capture("unit-pin-alternates-cold-form"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
