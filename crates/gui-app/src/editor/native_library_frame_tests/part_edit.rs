//! Saved Part revision plus a new placement; old instances retain exact sources.
use super::*;
use eda_engine::substrate::{ElectricalSelectionSnapshot, LibraryBindingRole};
use serde_json::{Value, json};
use uuid::Uuid;
const PART: &str = "88175c8c-8479-4291-8ea2-3ccac699cc60";
const ROOT: &str = "77ac3f79-5121-4dfe-82e5-2f390da0c99c";
const GATE: &str = "df2605d7-853b-40b0-ba7e-4afff49a249e";
const VALUE: &str = "Revised second value";
const MPN: &str = "REVISED SECOND";

impl Fixture {
    fn part_edit_expected(&self, corrected: bool) -> Value {
        let mut v: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("part-edit-before.json")).unwrap(),
        )
        .unwrap();
        if corrected {
            for (k, value) in [
                ("manufacturer", json!("Designer corrected")),
                ("mpn", json!(MPN)),
                ("value", json!(VALUE)),
                ("description", json!("Corrected orderable metadata")),
                ("datasheet", json!("revised-reference.pdf")),
                ("tags", json!(["analog", "revised"])),
            ] {
                v[k] = value;
            }
        }
        v
    }
    fn part_edit_choice(&self) -> usize {
        self.app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring
            .choices
            .iter()
            .position(|c| {
                c.explicit_part
                    .as_ref()
                    .is_some_and(|p| p.part.to_string() == PART && p.gate.to_string() == GATE)
            })
            .unwrap()
    }
    fn part_edit_rendered_old_and_new(&self) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let scene =
            datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused).unwrap();
        assert_eq!(
            scene
                .schematic_document
                .as_ref()
                .unwrap()
                .source_sheet
                .to_string(),
            ROOT
        );
        let old: Value = serde_json::from_slice(
            &std::fs::read(self.output.join("entity-edit-scene-5.json")).unwrap(),
        )
        .unwrap();
        let mut actual = json!({"components":scene.components,"texts":scene.component_texts,
            "graphics":scene.component_graphics,"board_texts":scene.board_texts,"text_geometry":scene.board_text_geometries,
            "board_graphics":scene.board_graphics,"tracks":scene.tracks,"nets":scene.net_display});
        for (key, entries) in old.as_object().unwrap() {
            let values = actual[key].as_array_mut().unwrap();
            for expected in entries.as_array().unwrap() {
                let index=values.iter().position(|v|v==expected).expect("every old rendered primitive, including duplicate values and annotations, survives new placement");
                values.remove(index);
            }
        }
        let id = self.edited_footprint.as_ref().unwrap()["symbol"]["uuid"]
            .as_str()
            .unwrap();
        assert!(
            scene
                .board_graphics
                .iter()
                .any(|g| g.object_id.contains(id))
        );
        assert!(scene.board_texts.iter().any(|t| t.text == VALUE));
    }
    fn part_edit_check(&mut self, corrected: bool, new: bool, record: bool) {
        let state = self.app.runtime.as_ref().unwrap().workspace();
        let model = state.library.basis.as_ref().unwrap();
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_")),
            "{:?}",
            model.diagnostics
        );
        let part = state
            .library
            .entries
            .iter()
            .find(|e| e.id.to_string() == PART && e.kind == "parts")
            .unwrap();
        assert_eq!(part.raw, self.part_edit_expected(corrected));
        let old = self.original_footprint.as_ref().unwrap();
        let old_cis: BTreeMap<Uuid, Value> = serde_json::from_value(old["cis"].clone()).unwrap();
        for (id, raw) in &old_cis {
            let ci = model
                .component_instances
                .get(id)
                .expect("existing declared CI must still resolve");
            assert_eq!(serde_json::to_value(ci).unwrap(), *raw);
            for b in ci.library_bindings.values().filter(|b| {
                b.binding_role == LibraryBindingRole::Part && b.target_object_id.to_string() == PART
            }) {
                assert_eq!(
                    model
                        .materialized_pool_object_revision(
                            b.target_object_id,
                            "parts",
                            b.pinned_object_revision
                        )
                        .unwrap(),
                    self.part_edit_expected(false)
                );
            }
        }
        assert_eq!(
            model.component_instances.len(),
            old_cis.len() + usize::from(new)
        );
        let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
        let source = snap.schematic_source().unwrap();
        let mut sheets = serde_json::to_value(&source.sheets).unwrap();
        let original_sheets = old["sheets"].as_object().unwrap();
        let root = source.sheets.get(&ROOT.parse().unwrap()).unwrap();
        let original_root = &original_sheets[ROOT];
        let added: Vec<_> = root
            .symbols
            .iter()
            .filter(|(id, _)| original_root["symbols"].get(id.to_string()).is_none())
            .collect();
        if new {
            assert_eq!(added.len(), 1);
            let (id, placed) = added[0];
            assert_eq!(placed.value, VALUE);
            assert!(
                !original_root["symbols"]
                    .as_object()
                    .unwrap()
                    .values()
                    .any(|s| s["reference"] == json!(placed.reference))
            );
            let ci = model
                .component_instances
                .values()
                .find(|ci| ci.placed_symbol_refs.contains(id))
                .unwrap();
            let binding = ci
                .library_bindings
                .values()
                .find(|b| b.binding_role == LibraryBindingRole::Part)
                .unwrap();
            assert_eq!(binding.target_object_id.to_string(), PART);
            let choice: Value = serde_json::from_slice(
                &std::fs::read(self.output.join("part-edit-new-choice.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(json!(binding.pinned_object_revision.0), choice["revision"]);
            assert_eq!(
                model
                    .materialized_pool_object_revision(
                        binding.target_object_id,
                        "parts",
                        binding.pinned_object_revision
                    )
                    .unwrap(),
                self.part_edit_expected(true)
            );
            let current = json!({"symbol":placed,"ci":ci});
            if record {
                self.edited_footprint = Some(current);
            } else {
                assert_eq!(Some(&current), self.edited_footprint.as_ref());
            }
            sheets[ROOT]["symbols"]
                .as_object_mut()
                .unwrap()
                .remove(&id.to_string());
        } else {
            assert!(added.is_empty());
        }
        assert_eq!(
            sheets, old["sheets"],
            "all old symbols/pins/values/references/geometry and sheet metadata remain exact"
        );
        let mut files: BTreeMap<String, Vec<u8>> =
            serde_json::from_value(old["files"].clone()).unwrap();
        let mut actual = source_bytes(&self.root);
        let part_path = format!("pool/parts/{PART}.json");
        files.remove(&part_path);
        if new {
            let path = format!("schematic/sheets/{ROOT}.json");
            let previous: Value = serde_json::from_slice(&files.remove(&path).unwrap()).unwrap();
            let mut current: Value =
                serde_json::from_slice(&actual.remove(&path).unwrap()).unwrap();
            current["symbols"]
                .as_object_mut()
                .unwrap()
                .remove(&added[0].0.to_string());
            assert_eq!(
                current, previous,
                "raw old Root sheet payload and unknown metadata remain exact"
            );
        }
        for (path, bytes) in files {
            assert_eq!(
                actual.remove(&path).as_ref(),
                Some(&bytes),
                "unchanged source {path}"
            );
        }
    }
    pub(super) fn part_edit_stage(&mut self, events: &ActiveEventLoop, stage: usize) {
        match stage {
            0 => {
                let state = self.app.runtime.as_ref().unwrap().workspace();
                let model = state.library.basis.as_ref().unwrap();
                let part = state
                    .library
                    .entries
                    .iter()
                    .find(|e| e.id.to_string() == PART && e.kind == "parts")
                    .unwrap();
                assert_eq!(part.raw["mpn"], json!("SECOND"));
                assert_eq!(model.objects[&part.id].object_revision.0, 1);
                assert!(!part.raw["default_pin_pad_map"].is_null());
                let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
                let files: BTreeMap<_, _> = source_bytes(&self.root)
                    .into_iter()
                    .filter(|(p, _)| {
                        !p.starts_with(".datum/") || p.starts_with(".datum/component_instances/")
                    })
                    .collect();
                self.original_footprint = Some(
                    json!({"cis":model.component_instances,"sheets":snap.schematic_source().unwrap().sheets,"files":files}),
                );
                std::fs::write(
                    self.output.join("part-edit-before.json"),
                    serde_json::to_vec_pretty(&part.raw).unwrap(),
                )
                .unwrap();
                assert!(model.component_instances.values().any(|ci| {
                    ci.library_bindings.values().any(|b| {
                        b.binding_role == LibraryBindingRole::Part
                            && b.target_object_id == part.id
                            && b.pinned_object_revision.0 == 1
                    })
                }));
            }
            1 | 2 => {
                let r = self.app.runtime.as_ref().unwrap();
                let d = r.workspace().library.draft.as_ref().unwrap();
                assert!(!d.create);
                assert_eq!(
                    d.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
                    [
                        "manufacturer",
                        "mpn",
                        "value",
                        "description",
                        "datasheet",
                        "tags",
                        "part_entity",
                        "part_package",
                        "part_footprint",
                        "part_map"
                    ]
                );
                assert!(d.fields.iter().take(6).all(|f| !f.selector));
                assert!(d.fields[8].selector);
                assert!(d.fields[9].selector);
                assert_eq!(
                    d.part.original.as_ref().unwrap()["entity"],
                    self.part_edit_expected(false)["entity"]
                );
                assert_eq!(
                    d.part.original.as_ref().unwrap()["default_pin_pad_map"],
                    self.part_edit_expected(false)["default_pin_pad_map"]
                );
                if stage == 2 {
                    assert_eq!(d.preview.as_ref().unwrap(), &self.part_edit_expected(true));
                }
            }
            3 => self.part_edit_check(true, false, false),
            4 => self.part_edit_check(false, false, false),
            5 => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Place",
                    "Symbol",
                    EntrySurface::MenuPointer
                ));
            }
            6 => {
                if self.part_edit_choice() >= 4 {
                    self.control(events, HitTarget::SchematicLibraryPage(true));
                }
            }
            7 => {
                let index = self.part_edit_choice();
                let r = self.app.runtime.as_ref().unwrap();
                let s = r.workspace();
                let choice = &s.schematic_authoring.choices[index];
                assert_eq!(choice.value, VALUE);
                let model = s.library.basis.as_ref().unwrap();
                let id: Uuid = PART.parse().unwrap();
                let revision = model.objects[&id].object_revision.0;
                assert!(revision > 1);
                std::fs::write(
                    self.output.join("part-edit-new-choice.json"),
                    serde_json::to_vec_pretty(&json!({"revision":revision,"value":choice.value}))
                        .unwrap(),
                )
                .unwrap();
                self.control(events, HitTarget::SchematicLibraryChoice(index));
            }
            8 => self.control(events, HitTarget::SchematicPlace),
            9 | 10 => self.placement_mouse(
                events,
                datum_gui_protocol::PointNm {
                    x: -2_000_000,
                    y: 3_000_000,
                },
                stage == 10,
            ),
            11 => self.control(events, HitTarget::SchematicCancel),
            12 => self.part_edit_check(true, true, true),
            13 => self.part_edit_check(true, true, false),
            14 => self.part_edit_rendered_old_and_new(),
            _ => panic!("unknown saved Part stage{stage}"),
        }
    }
}

#[test]
#[ignore = "requires saved Part/root-child users and owned isolated compositor"]
fn ordinary_saved_part_metadata_pinned_old_and_new_current_placement_history_cold() {
    use pane_context::Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![
        Step::PartEdit(0),
        Step::Snapshot,
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::PaneContext(Navigate(false)),
        Step::Open,
        Step::Kind(3),
        Step::Search("SECOND".into()),
        Step::Select("SECOND".into()),
        Step::Control(C::Edit),
        Step::PartEdit(1),
        Step::Capture("part-edit-before"),
    ];
    for commit in [false, true] {
        if commit {
            steps.extend([Step::Control(C::Edit), Step::PartEdit(1)]);
        }
        steps.extend([
            Step::Field("manufacturer", "Designer corrected".into()),
            Step::Field("mpn", MPN.into()),
            Step::Field("value", VALUE.into()),
            Step::Field("description", "Corrected orderable metadata".into()),
            Step::Field("datasheet", "revised-reference.pdf".into()),
            Step::Field("tags", "revised, analog".into()),
            Step::Control(C::Preview),
            Step::PartEdit(2),
            Step::Invariant,
        ]);
        if commit {
            steps.extend([
                Step::Capture("part-edit-preview"),
                Step::Control(C::Commit),
                Step::PartEdit(3),
                Step::Capture("part-edit-committed"),
            ]);
        } else {
            steps.extend([Step::Control(C::Cancel), Step::Invariant, Step::PartEdit(4)]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::PartEdit(4),
        Step::History(true),
        Step::PartEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PartEdit(3),
        Step::PaneContext(Navigate(false)),
        Step::EntityEdit(5),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::EntityEdit(6),
        Step::Capture("part-edit-cold-old-bindings"),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::Snapshot,
    ]);
    for commit in [false, true] {
        steps.extend([
            Step::PartEdit(5),
            Step::Control(C::ProjectSymbols),
            Step::PartEdit(6),
            Step::PartEdit(7),
        ]);
        if commit {
            steps.push(Step::Capture("part-edit-new-current-choice"));
        }
        steps.extend([Step::PartEdit(8), Step::PartEdit(9)]);
        if commit {
            steps.extend([
                Step::Capture("part-edit-new-ghost"),
                Step::PartEdit(10),
                Step::PartEdit(12),
                Step::PartEdit(14),
                Step::Capture("part-edit-new-placed"),
            ]);
        } else {
            steps.extend([Step::PartEdit(11), Step::Invariant, Step::PartEdit(3)]);
        }
    }
    steps.extend([
        Step::History(false),
        Step::PartEdit(3),
        Step::History(true),
        Step::PartEdit(13),
        Step::Save,
        Step::Snapshot,
        Step::Reopen,
        Step::Invariant,
        Step::PartEdit(13),
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Fit),
        Step::PartEdit(14),
        Step::Capture("part-edit-cold-old-and-new"),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
