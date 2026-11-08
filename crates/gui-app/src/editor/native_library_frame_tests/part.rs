//! Ordinary Part metadata and optional compatible defaults through the real App.
use super::*;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, registry::NativeWriteContext,
};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
use uuid::Uuid;
impl Fixture {
    pub(super) fn check_part(&mut self, stage: usize, default: bool) {
        let bytes: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| {
                ["pool/", "schematic/", "board/"]
                    .iter()
                    .any(|prefix| p.starts_with(prefix))
            })
            .collect();
        let now = json!(bytes);
        if stage == 0 || stage == 4 {
            self.original_footprint = Some(now);
            return;
        }
        for (path, expected) in self
            .original_footprint
            .as_ref()
            .unwrap()
            .as_object()
            .unwrap()
        {
            assert_eq!(
                &now[path], expected,
                "existing definition/placed circuit {path} stays byte-exact"
            );
        }
        let browser = &self.app.runtime.as_ref().unwrap().workspace().library;
        if stage == 5 {
            let d = browser.draft.as_ref().unwrap();
            assert_eq!(d.value("part_footprint"), "");
            assert!(d.preview.is_none());
            let choices = &d
                .fields
                .iter()
                .find(|f| f.key == "part_footprint")
                .unwrap()
                .choices;
            assert_eq!(choices.len(), 2);
            assert!(choices[0].0.is_empty());
            assert!(choices[1].1.starts_with("Footprint B"));
            return;
        }
        if stage == 1 {
            let d = browser.draft.as_ref().unwrap();
            let raw = d.preview.as_ref().unwrap();
            assert_eq!(raw["uuid"], json!(d.id));
            assert_eq!(raw["manufacturer"], json!("Designer"));
            assert_eq!(
                raw["mpn"],
                json!(if default { "PN-DEFAULT" } else { "PN-NONE" })
            );
            for (key, value) in [
                ("value", "As entered"),
                ("description", "Designer part"),
                ("datasheet", "reference.pdf"),
                ("lifecycle", "Unknown"),
            ] {
                assert_eq!(raw[key], json!(value));
            }
            assert_eq!(raw["tags"], json!(["logic", "test"]));
            assert!(raw.get("name").is_none());
            assert!(raw["default_pin_pad_map"].is_null());
            assert_eq!(raw["pad_map"], json!({}));
            assert!(
                !d.fields
                    .iter()
                    .any(|f| matches!(f.key, "name" | "lifecycle"))
            );
            let id = |kind: &str, name: &str| {
                browser
                    .entries
                    .iter()
                    .find(|e| e.kind == kind && e.name == name)
                    .unwrap()
                    .id
            };
            assert_eq!(raw["entity"], json!(id("entities", "Logical entity")));
            assert_eq!(
                raw["package"],
                json!(id(
                    "packages",
                    if default { "Package B" } else { "Package A" }
                ))
            );
            assert_eq!(
                raw["default_footprint"],
                if default {
                    json!(id("footprints", "Footprint B"))
                } else {
                    json!(null)
                }
            );
            self.edited_footprint = Some(raw.clone());
            return;
        }
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let b = datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(
            model,
        ));
        let expected = self.edited_footprint.as_ref().unwrap();
        let found = b
            .entries
            .iter()
            .find(|e| e.id.to_string() == expected["uuid"].as_str().unwrap());
        if stage == 3 {
            assert!(found.is_none());
        } else {
            assert_eq!(&found.unwrap().raw, expected);
        }
    }
}
#[test]
#[ignore = "requires fresh owned project/output and isolated native Wayland with matching companions"]
fn ordinary_part_optional_defaults_preview_cancel_history_save_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_PART_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(!root.exists());
    std::fs::create_dir_all(&output).unwrap();
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Part authoring".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, sheet, entity, gate, package, other, footprint, other_footprint) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (verb, params) in [
        (
            "datum.library.create",
            json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"name":"Logical unit","pins":{pin.to_string():{"name":"Input","electrical_type":"Input"}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"name":"Circuit symbol","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"name":"Logical entity","prefix":"U","manufacturer":"","gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"packages","object_id":package,"pool_path":"pool","fields":{"name":"Package A"}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"packages","object_id":other,"pool_path":"pool","fields":{"name":"Package B"}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"footprints","object_id":footprint,"pool_path":"pool","fields":{"name":"Footprint A","package":package,"pads":{}}}),
        ),
        (
            "datum.library.create",
            json!({"kind":"footprints","object_id":other_footprint,"pool_path":"pool","fields":{"name":"Footprint B","package":other,"pads":{}}}),
        ),
        (
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":"Root"}),
        ),
        (
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Original","x_nm":0,"y_nm":0}),
        ),
    ] {
        let p = (native::registry::find_native_write_verb(verb)
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            WriteProvenance::new(
                "test",
                CommitSource::Manual,
                "Prepare existing definitions/circuit",
            ),
            params,
        )
        .unwrap();
        native::commit_prepared(&mut model, &root, p).unwrap();
    }
    let mut steps = vec![
        Step::Part(0, false),
        Step::Open,
        Step::Kind(3),
        Step::Control(C::New),
        Step::Snapshot,
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("part-empty-refusal"),
        Step::Invariant,
        Step::Field("manufacturer", "Designer".into()),
        Step::Field("mpn", "Cancelled".into()),
        Step::Choice("part_entity", "Logical entity".into()),
        Step::Choice("part_package", "Package A".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("part-cancel-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ];
    for default in [false, true] {
        if default {
            steps.extend([Step::Part(4, false), Step::Open, Step::Kind(3)]);
        }
        steps.extend([
            Step::KeyControl(C::New),
            Step::Field("manufacturer", "Designer".into()),
            Step::Field("mpn", if default { "PN-DEFAULT" } else { "PN-NONE" }.into()),
            Step::Field("value", "As entered".into()),
            Step::Field("description", "Designer part".into()),
            Step::Field("datasheet", "reference.pdf".into()),
            Step::Field("tags", "logic,test".into()),
            Step::Choice("part_entity", "Logical entity".into()),
            Step::Choice("part_package", "Package A".into()),
        ]);
        if default {
            steps.extend([
                Step::Choice("part_footprint", "Footprint A".into()),
                Step::Choice("part_package", "Package B".into()),
                Step::Part(5, true),
                Step::Capture("part-package-changed"),
                Step::Choice("part_footprint", "Footprint B".into()),
            ]);
        }
        steps.extend([
            Step::KeyControl(C::Preview),
            Step::Part(1, default),
            Step::Capture(if default {
                "part-default-preview"
            } else {
                "part-no-default-preview"
            }),
            Step::KeyControl(C::Commit),
            Step::Part(2, default),
            Step::History(false),
            Step::Part(3, default),
            Step::History(true),
            Step::Part(2, default),
            Step::Save,
            Step::Part(2, default),
            Step::Reopen,
            Step::Part(2, default),
        ]);
    }
    steps.extend([
        Step::Open,
        Step::Kind(3),
        Step::Search("PN-DEFAULT".into()),
        Step::Select("PN-DEFAULT".into()),
        Step::Capture("part-reopened"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
