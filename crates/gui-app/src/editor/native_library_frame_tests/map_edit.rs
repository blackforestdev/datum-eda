//! Ordinary saved assignment revision leaves existing circuitry frozen.
use super::*;
use serde_json::{Value, json};
const MAP: &str = "0308a65b-681c-479b-99b7-c38d27df35b8";
const FIRST: &str = "00000000-0000-0000-0000-000000000001";
const SECOND: &str = "00000000-0000-0000-0000-000000000002";
const GATE_B: &str = "df2605d7-853b-40b0-ba7e-4afff49a249e";
impl Fixture {
    fn map_edit_expected(&self, edited: bool) -> Value {
        let mut original = self.edited_footprint.clone().unwrap();
        if edited {
            original["mappings"].as_object_mut().unwrap().remove(SECOND);
            original["mappings"][FIRST]["gate"] = json!(GATE_B);
        }
        original
    }
    pub(super) fn check_map_edit(&mut self, stage: usize) {
        if stage >= 5 {
            self.check_entity_edit(stage);
            return;
        }
        let model = eda_engine::substrate::ProjectResolver::new(&self.root)
            .resolve()
            .unwrap();
        assert!(!model.diagnostics.iter().any(|d| matches!(
            d.code.as_str(),
            "invalid_component_instance_shard" | "missing_component_instance_shard"
        )));
        let s = self.app.runtime.as_ref().unwrap().workspace();
        let entry = s
            .library
            .entries
            .iter()
            .find(|e| e.kind == "pin_pad_maps" && e.id.to_string() == MAP)
            .unwrap();
        if stage == 0 {
            assert_eq!(entry.raw["mappings"].as_object().unwrap().len(), 2);
            assert_eq!(model.component_instances.len(), 2);
            self.edited_footprint = Some(entry.raw.clone());
            self.original_footprint = Some(json!(
                source_bytes(&self.root)
                    .into_iter()
                    .filter(|(p, _)| !p.starts_with(".datum/"))
                    .collect::<BTreeMap<_, _>>()
            ));
            std::fs::write(
                self.output.join("map-edit-ci-before.json"),
                serde_json::to_vec(&model.component_instances).unwrap(),
            )
            .unwrap();
            return;
        }
        if stage == 1 || stage == 2 {
            let d = s.library.draft.as_ref().unwrap();
            assert!(!d.create);
            assert_eq!(d.id.to_string(), MAP);
            assert_eq!(
                d.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
                ["map_pad", "map_gate", "map_pin"]
            );
            assert_eq!(
                d.map_source("map_part"),
                entry.raw["part"].as_str().unwrap()
            );
            assert_eq!(
                d.map_source("map_footprint"),
                entry.raw["footprint"].as_str().unwrap()
            );
            if stage == 2 {
                assert_eq!(d.preview.as_ref().unwrap(), &self.map_edit_expected(true));
            }
            return;
        }
        assert_eq!(entry.raw, self.map_edit_expected(stage == 3));
        let original_ci: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("map-edit-ci-before.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&model.component_instances).unwrap(),
            original_ci
        );
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
        let mut actual: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| !p.starts_with(".datum/"))
            .collect();
        let path = format!("pool/pin_pad_maps/{MAP}.json");
        before.remove(&path);
        actual.remove(&path);
        assert_eq!(
            actual, before,
            "Part/default, linked sources and all existing authored circuits remain byte exact"
        );
    }
}

#[test]
#[ignore = "requires saved Footprint-bound map and root/child circuit in isolated compositor"]
fn ordinary_saved_mapping_assignments_cancel_history_save_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![
        Step::MapEdit(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::MapEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::MapEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(7),
        Step::Search("SECOND".into()),
        Step::SelectMapping,
        Step::Control(C::Edit),
        Step::MapEdit(1),
        Step::Capture("map-edit-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.extend([Step::Control(C::Edit), Step::MapEdit(1)]);
        }
        steps.extend([
            Step::Choice("map_pad", "2".into()),
            Step::Control(C::UnmapPad),
            Step::Choice("map_pad", "1".into()),
            Step::Choice("map_gate", "B".into()),
            Step::Choice("map_pin", "Input".into()),
            Step::Control(C::Preview),
            Step::MapEdit(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("map-edit-preview"),
                Step::Control(C::Commit),
                Step::MapEdit(3),
                Step::Capture("map-edit-committed"),
            ]);
        } else {
            steps.extend([Step::Control(C::Cancel), Step::Invariant, Step::MapEdit(4)]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::MapEdit(4),
        Step::History(true),
        Step::MapEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::MapEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::MapEdit(6),
        Step::Capture("map-edit-placed-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::MapEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::MapEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::MapEdit(6),
        Step::Capture("map-edit-cold-child"),
        Step::Open,
        Step::Kind(7),
        Step::Search("SECOND".into()),
        Step::SelectMapping,
        Step::Control(C::Edit),
        Step::MapEdit(1),
        Step::Capture("map-edit-cold-assignments"),
        Step::Control(C::Cancel),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
