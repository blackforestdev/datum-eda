//! Ordinary Entity form, explicit named gates, canonical history and cold reopen.
use super::*;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, registry::NativeWriteContext,
};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
use uuid::Uuid;
impl Fixture {
    pub(super) fn check_entity(&mut self, stage: usize) {
        let bytes: BTreeMap<_, _> = source_bytes(&self.root)
            .into_iter()
            .filter(|(p, _)| {
                ["pool/units/", "pool/symbols/", "schematic/", "board/"]
                    .iter()
                    .any(|prefix| p.starts_with(prefix))
            })
            .collect();
        let baseline = json!(bytes);
        if stage == 0 {
            self.original_footprint = Some(baseline);
            return;
        }
        assert_eq!(
            Some(&baseline),
            self.original_footprint.as_ref(),
            "existing definitions and placed circuit bytes remain unchanged"
        );
        if stage == 1 {
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
            let raw = d.preview.as_ref().expect("validated Entity preview");
            assert_eq!(raw["uuid"], json!(d.id));
            assert_eq!(raw["name"], json!("Designer Entity"));
            assert_eq!(raw["prefix"], json!("U"));
            assert_eq!(raw["manufacturer"], json!("Designer"));
            assert_eq!(raw["tags"], json!(["logic", "test"]));
            let browser = &self.app.runtime.as_ref().unwrap().workspace().library;
            let unit = browser
                .entries
                .iter()
                .find(|e| e.kind == "units" && e.name == "Logical unit")
                .unwrap()
                .id;
            let symbol = browser
                .entries
                .iter()
                .find(|e| e.kind == "symbols" && e.name == "Circuit symbol")
                .unwrap()
                .id;
            let names: std::collections::BTreeSet<_> = raw["gates"]
                .as_object()
                .unwrap()
                .values()
                .map(|g| g["name"].as_str().unwrap())
                .collect();
            assert_eq!(names, std::collections::BTreeSet::from(["A", "B"]));
            assert_eq!(raw["gates"].as_object().unwrap().len(), 2);
            for (key, gate) in raw["gates"].as_object().unwrap() {
                assert_eq!(key, gate["uuid"].as_str().unwrap());
                assert_eq!(gate["unit"], json!(unit));
                assert_eq!(gate["symbol"], json!(symbol));
            }
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
fn ordinary_entity_named_gates_preview_cancel_history_save_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_ENTITY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(!root.exists());
    std::fs::create_dir_all(&output).unwrap();
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Entity gate authoring".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, sheet) = (
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
            WriteProvenance::new("test", CommitSource::Manual, "Prepare existing circuit"),
            params,
        )
        .unwrap();
        native::commit_prepared(&mut model, &root, p).unwrap();
    }
    let mut steps = vec![
        Step::Entity(0),
        Step::Open,
        Step::Kind(2),
        Step::Control(C::New),
        Step::Snapshot,
        Step::Field("name", "Cancelled Entity".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("entity-empty-refusal"),
        Step::Invariant,
        Step::Field("gate_name", "Cancelled".into()),
        Step::Choice("gate_unit", "Logical unit".into()),
        Step::Choice("gate_symbol", "Circuit symbol".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("entity-cancel-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::KeyControl(C::New),
        Step::Field("name", "Designer Entity".into()),
        Step::Field("prefix", "U".into()),
        Step::Field("manufacturer", "Designer".into()),
        Step::Field("tags", "logic, test".into()),
        Step::Field("gate_name", "A".into()),
        Step::Choice("gate_unit", "Logical unit".into()),
        Step::Choice("gate_symbol", "Circuit symbol".into()),
        Step::KeyControl(C::AddGate),
        Step::Field("gate_name", "Temporary".into()),
        Step::KeyControl(C::RemoveGate),
        Step::KeyControl(C::AddGate),
        Step::Field("gate_name", "B".into()),
        Step::Choice("gate_unit", "Logical unit".into()),
        Step::Choice("gate_symbol", "Circuit symbol".into()),
        Step::KeyControl(C::Gate(0)),
        Step::Field("gate_name", "A".into()),
        Step::KeyControl(C::Preview),
        Step::Entity(1),
        Step::Capture("entity-two-gate-preview"),
        Step::Focus(C::Gate(0)),
        Step::Capture("entity-gate-table"),
        Step::KeyControl(C::Commit),
        Step::Entity(2),
        Step::History(false),
        Step::Entity(3),
        Step::History(true),
        Step::Entity(2),
        Step::Save,
        Step::Entity(2),
        Step::Reopen,
        Step::Entity(2),
    ];
    steps.extend([
        Step::Open,
        Step::Kind(2),
        Step::First,
        Step::Capture("entity-reopened"),
    ]);
    scenario::run(root, output, steps, Uuid::nil());
}
