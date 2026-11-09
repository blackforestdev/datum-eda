//! Saved metadata correction retains raw gate links and existing placed circuitry.
use super::*;
use serde_json::{Value, json};
const ENTITY: &str = "6be4c754-4199-4d0d-b848-25ff4b3931c0";
const NAME: &str = "Revised dual gate";

impl Fixture {
    fn entity_edit_expected(&self, edited: bool) -> Value {
        let mut raw: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("entity-edit-before.json")).unwrap(),
        )
        .unwrap();
        if edited {
            raw["name"] = json!(NAME);
            raw["prefix"] = json!("IC");
            raw["manufacturer"] = json!("Designer corrected");
            raw["tags"] = json!(["analog", "reviewed"]);
        }
        raw
    }
    pub(super) fn check_entity_edit(&mut self, stage: usize) {
        let s = self.app.runtime.as_ref().unwrap().workspace();
        if stage >= 5 {
            let pane = s.ui.layout.focused;
            let key = &s.schematic_documents.panes[&pane];
            assert_eq!(
                s.schematic_documents.row(key).unwrap().parent.is_some(),
                stage == 6
            );
            let sheet = s.schematic_documents.source_sheet(key).unwrap();
            assert!(
                !sheet.symbols.is_empty(),
                "actual placed symbols are required"
            );
            let scene = datum_gui_protocol::camera_scene_for_pane(s, pane)
                .expect("ordinary focused schematic is visible");
            assert_eq!(scene.schematic_document.as_ref(), Some(key));
            for id in sheet.symbols.keys() {
                assert!(
                    scene
                        .board_graphics
                        .iter()
                        .any(|g| g.object_id.starts_with(&format!("schematic-symbol:{id}"))),
                    "each actual placed Symbol has a rendered body"
                );
            }
            let mut visible = json!({"components":scene.components,"texts":scene.component_texts,
                "graphics":scene.component_graphics,"board_texts":scene.board_texts,"text_geometry":scene.board_text_geometries,
                "board_graphics":scene.board_graphics,"tracks":scene.tracks,"nets":scene.net_display});
            // As in the existing frozen-library proof, compare collections by
            // primitive identity, retaining every value and inner geometry order.
            for values in visible.as_object_mut().unwrap().values_mut() {
                values
                    .as_array_mut()
                    .unwrap()
                    .sort_by_cached_key(Value::to_string);
            }
            let path = self.output.join(format!("entity-edit-scene-{stage}.json"));
            if path.exists() {
                let before: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
                assert_eq!(
                    visible, before,
                    "placed IDs, pose, reference annotations and rendered geometry remain exact"
                );
            } else {
                std::fs::write(path, serde_json::to_vec_pretty(&visible).unwrap()).unwrap();
            }
            return;
        }
        let entry = s
            .library
            .entries
            .iter()
            .find(|e| e.kind == "entities" && e.id.to_string() == ENTITY)
            .unwrap();
        if stage == 0 {
            assert_eq!(entry.name, "Dual");
            assert_eq!(entry.raw["gates"].as_object().unwrap().len(), 2);
            assert!(
                s.library
                    .entries
                    .iter()
                    .any(|e| e.kind == "parts" && e.raw["entity"] == json!(ENTITY))
            );
            std::fs::write(
                self.output.join("entity-edit-before.json"),
                serde_json::to_vec_pretty(&entry.raw).unwrap(),
            )
            .unwrap();
            self.original_footprint = Some(json!(
                source_bytes(&self.root)
                    .into_iter()
                    .filter(|(p, _)| !p.starts_with(".datum/"))
                    .collect::<BTreeMap<_, _>>()
            ));
            return;
        }
        if stage == 1 || stage == 2 {
            let draft = s.library.draft.as_ref().unwrap();
            assert!(!draft.create);
            assert_eq!(
                draft.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
                [
                    "name",
                    "prefix",
                    "manufacturer",
                    "tags",
                    "gate_name",
                    "gate_unit",
                    "gate_symbol"
                ]
            );
            assert_eq!(draft.entity.gates, *entry.raw["gates"].as_object().unwrap());
            assert!(draft.entity.gate_id.is_none());
            assert!(
                !self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .presented_hits
                    .regions()
                    .iter()
                    .any(|h| matches!(h.target, HitTarget::Library(C::AddGate | C::RemoveGate)))
            );
            assert!(draft.fields.iter().any(|f| f.key == "gate_unit"));
            assert_eq!(
                self.app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .presented_hits
                    .regions()
                    .iter()
                    .filter(|h| matches!(h.target, HitTarget::Library(C::Gate(_))))
                    .count(),
                2
            );
            assert!(
                draft
                    .fields
                    .iter()
                    .find(|f| f.key == "gate_symbol")
                    .unwrap()
                    .choices
                    .is_empty()
            );
            if stage == 2 {
                assert_eq!(
                    draft.preview.as_ref().unwrap(),
                    &self.entity_edit_expected(true)
                );
            }
            return;
        }
        assert_eq!(entry.raw, self.entity_edit_expected(stage == 3));
        let mut actual: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| !p.starts_with(".datum/"))
            .collect();
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
        let path = format!("pool/entities/{ENTITY}.json");
        actual.remove(&path);
        before.remove(&path);
        assert_eq!(
            actual, before,
            "all other authored files including linked definitions, Part, placed circuit and annotations remain byte exact"
        );
    }
}

#[test]
#[ignore = "requires saved named multi-gate Entity with placed Part circuit and isolated native compositor"]
fn ordinary_saved_entity_metadata_cancel_history_save_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![
        Step::EntityEdit(0),
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
        Step::EntityEdit(1),
        Step::Capture("entity-edit-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.extend([Step::Control(C::Edit), Step::EntityEdit(1)]);
        }
        steps.extend([
            Step::Field("name", NAME.into()),
            Step::Field("prefix", "IC".into()),
            Step::Field("manufacturer", "Designer corrected".into()),
            Step::Field("tags", "reviewed, analog".into()),
            Step::Control(C::Preview),
            Step::EntityEdit(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("entity-edit-preview"),
                Step::Control(C::Commit),
                Step::EntityEdit(3),
                Step::Capture("entity-edit-committed"),
            ]);
        } else {
            steps.extend([
                Step::Control(C::Cancel),
                Step::Invariant,
                Step::EntityEdit(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::EntityEdit(4),
        Step::History(true),
        Step::EntityEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("entity-edit-placed-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::EntityEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("entity-edit-cold-child"),
        Step::Open,
        Step::Kind(2),
        Step::Search(NAME.into()),
        Step::Select(NAME.into()),
        Step::Control(C::Edit),
        Step::EntityEdit(1),
        Step::Capture("entity-edit-cold-metadata"),
        Step::Control(C::Cancel),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
