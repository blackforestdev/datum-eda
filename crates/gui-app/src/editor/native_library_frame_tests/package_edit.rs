//! Saved Package metadata correction preserves raw topology and placed circuitry.
use super::*;
use serde_json::{Value, json};
const PACKAGE: &str = "f5c904cc-9a70-4325-ab82-c42415e6f895";
const NAME: &str = "Revised physical body";

impl Fixture {
    fn package_edit_expected(&self, edited: bool) -> Value {
        let mut raw: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("package-edit-before.json")).unwrap(),
        )
        .unwrap();
        if edited {
            raw["name"] = json!(NAME);
            raw["package_family"] = json!("Designer corrected family");
            raw["package_code"] = json!("Corrected code");
            raw["mounting_type"] = json!("Designer mounting");
        }
        raw
    }
    pub(super) fn check_package_edit(&mut self, stage: usize) {
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
            let path = self.output.join(format!("package-edit-scene-{stage}.json"));
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
            .find(|e| e.kind == "packages" && e.id.to_string() == PACKAGE)
            .unwrap();
        if stage == 0 {
            assert_eq!(entry.name, "Body");
            assert!(
                s.library
                    .entries
                    .iter()
                    .any(|e| e.kind == "parts" && e.raw["package"] == json!(PACKAGE))
            );
            std::fs::write(
                self.output.join("package-edit-before.json"),
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
                ["name", "package_family", "package_code", "mounting_type"]
            );
            assert!(draft.terminals.is_empty());
            assert!(draft.terminal_id.is_none());
            if stage == 1 {
                for key in ["name", "package_family", "package_code", "mounting_type"] {
                    assert_eq!(draft.value(key), entry.raw[key].as_str().unwrap_or(""));
                }
            }
            assert!(
                !self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .presented_hits
                    .regions()
                    .iter()
                    .any(|h| matches!(
                        h.target,
                        HitTarget::Library(C::Terminal(_) | C::AddTerminal | C::RemoveTerminal)
                    ))
            );
            if stage == 2 {
                assert_eq!(
                    draft.preview.as_ref().unwrap(),
                    &self.package_edit_expected(true)
                );
            }
            return;
        }
        assert_eq!(entry.raw, self.package_edit_expected(stage == 3));
        let mut actual: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| !p.starts_with(".datum/"))
            .collect();
        let mut before: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(self.original_footprint.clone().unwrap()).unwrap();
        let path = format!("pool/packages/{PACKAGE}.json");
        actual.remove(&path);
        before.remove(&path);
        assert_eq!(
            actual, before,
            "all other authored files including linked definitions, Part, placed circuit and annotations remain byte exact"
        );
    }
}

#[test]
#[ignore = "requires saved Package with root/child Part circuitry and isolated native compositor"]
fn ordinary_saved_package_metadata_cancel_history_save_and_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![
        Step::PackageEdit(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::PackageEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::PackageEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(4),
        Step::Search("Body".into()),
        Step::Select("Body".into()),
        Step::Control(C::Edit),
        Step::PackageEdit(1),
        Step::Capture("package-edit-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.extend([Step::Control(C::Edit), Step::PackageEdit(1)]);
        }
        steps.extend([
            Step::Field("name", NAME.into()),
            Step::Field("package_family", "Designer corrected family".into()),
            Step::Field("package_code", "Corrected code".into()),
            Step::Field("mounting_type", "Designer mounting".into()),
            Step::Control(C::Preview),
            Step::PackageEdit(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("package-edit-preview"),
                Step::Control(C::Commit),
                Step::PackageEdit(3),
                Step::Capture("package-edit-committed"),
            ]);
        } else {
            steps.extend([
                Step::Control(C::Cancel),
                Step::Invariant,
                Step::PackageEdit(4),
            ]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::PackageEdit(4),
        Step::History(true),
        Step::PackageEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::PackageEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::PackageEdit(6),
        Step::Capture("package-edit-placed-child"),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PackageEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::PackageEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::PackageEdit(6),
        Step::Capture("package-edit-cold-child"),
        Step::Open,
        Step::Kind(4),
        Step::Search(NAME.into()),
        Step::Select(NAME.into()),
        Step::Control(C::Edit),
        Step::PackageEdit(1),
        Step::Capture("package-edit-cold-metadata"),
        Step::Control(C::Cancel),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
