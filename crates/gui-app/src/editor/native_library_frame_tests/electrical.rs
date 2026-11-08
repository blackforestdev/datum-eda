//! Saved electrical forms through presented controls, real worker and cold reopen.
use super::*;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, registry::NativeWriteContext,
};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;

fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) {
    let context = NativeWriteContext {
        model,
        project_root: root,
    };
    let prepared = (native::registry::find_native_write_verb(verb)
        .unwrap()
        .build)(
        &context,
        WriteProvenance::new(
            "library-form-proof",
            CommitSource::Manual,
            "Author native library",
        ),
        params,
    )
    .unwrap();
    native::commit_prepared(model, root, prepared).unwrap();
}
impl Fixture {
    pub(super) fn saved_tab_cycle(&mut self) {
        assert!(
            !self
                .app
                .runtime
                .as_ref()
                .unwrap()
                .workspace()
                .library
                .draft
                .as_ref()
                .unwrap()
                .create
        );
        for _ in 0..80 {
            self.key(Key::Named(NamedKey::Tab));
            let focus = &self.app.runtime.as_ref().unwrap().workspace().library.focus;
            assert!(
                !matches!(
                    focus,
                    Some(C::AddPin | C::RemovePin | C::AddAnchor | C::RemoveAnchor)
                ),
                "hidden topology controls must not receive Tab"
            );
        }
    }
    pub(super) fn check_electrical(&mut self, stage: usize) {
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let b = datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(
            model.clone(),
        ));
        let unit = b.entries.iter().find(|e| e.kind == "units").unwrap();
        let symbol = b.entries.iter().find(|e| e.kind == "symbols").unwrap();
        let snapshot = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
        let scene = datum_gui_protocol::project_native_schematic_scene(&model, &self.root)
            .unwrap()
            .unwrap();
        let mut now = json!({"unit":unit.raw,"symbol":symbol.raw,"sheets":snapshot.schematic_source().unwrap().sheets,
            "instances":model.component_instances,"graphics":scene.board_graphics,"texts":scene.board_texts,"text_geometry":scene.board_text_geometries});
        for key in ["graphics", "texts", "text_geometry"] {
            now[key]
                .as_array_mut()
                .unwrap()
                .sort_by_cached_key(Value::to_string);
        }
        if stage == 0 {
            self.original_footprint = Some(now);
            return;
        }
        let mut expected = self.original_footprint.clone().unwrap();
        if stage >= 2 {
            expected["unit"]["name"] = json!("Edited logical unit");
            let pin = expected["unit"]["pins"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap();
            pin["name"] = json!("Edited input");
            pin["direction"] = json!("Passive");
            pin["electrical_type"] = json!("Passive");
        }
        if stage >= 3 {
            expected["symbol"]["name"] = json!("Edited symbol");
            expected["symbol"]["pin_anchors"][0]["position"]["x"] = json!(-2500000);
            expected["symbol"]["pin_anchors"][0]["style"]["orientation"] = json!("Up");
            expected["symbol"]["pin_anchors"][0]["style"]["length_nm"] = json!(1500000);
        }
        assert_eq!(
            now, expected,
            "saved electrical stage {stage}: raw siblings and placed rendering stay exact"
        );
    }
}
#[test]
#[ignore = "requires owned fresh Project/output paths, isolated native Wayland and matching companions"]
fn ordinary_saved_electrical_preview_cancel_history_save_and_cold() {
    let root = PathBuf::from(std::env::var("DATUM_ELECTRICAL_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(!root.exists());
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Saved library forms".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    std::fs::create_dir_all(&output).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    write(
        &mut model,
        &root,
        "datum.library.create",
        json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"name":"Logical unit","pins":{pin.to_string():{"name":"Input","electrical_type":"Input"}}}}),
    );
    write(
        &mut model,
        &root,
        "datum.library.create",
        json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"name":"Circuit symbol","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
    );
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    write(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Original","x_nm":0,"y_nm":0}),
    );
    let steps = vec![
        Step::Electrical(0),
        Step::Open,
        Step::Kind(0),
        Step::First,
        Step::Control(C::Edit),
        Step::Snapshot,
        Step::SavedTabCycle,
        Step::KeyControl(C::Pin(0)),
        Step::Field("pin_name", "Cancelled input".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("unit-cancel-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::Electrical(1),
        Step::KeyControl(C::Edit),
        Step::Field("name", "Edited logical unit".into()),
        Step::KeyControl(C::Pin(0)),
        Step::Field("pin_name", "Edited input".into()),
        Step::Choice("pin_type", "Passive".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("unit-edit-preview"),
        Step::KeyControl(C::Commit),
        Step::Electrical(2),
        Step::History(false),
        Step::Electrical(1),
        Step::History(true),
        Step::Electrical(2),
        Step::Kind(1),
        Step::First,
        Step::KeyControl(C::Edit),
        Step::Snapshot,
        Step::SavedTabCycle,
        Step::KeyControl(C::Anchor(0)),
        Step::Field("anchor_length", "invalid".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("symbol-invalid-length"),
        Step::Field("anchor_length", "1500000".into()),
        Step::Choice("anchor_orientation", "Up".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("symbol-cancel-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::KeyControl(C::Edit),
        Step::Field("name", "Edited symbol".into()),
        Step::KeyControl(C::Anchor(0)),
        Step::Field("anchor_x", "-2500000".into()),
        Step::Field("anchor_length", "1500000".into()),
        Step::Choice("anchor_orientation", "Up".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("symbol-edit-preview"),
        Step::KeyControl(C::Commit),
        Step::Electrical(3),
        Step::History(false),
        Step::Electrical(2),
        Step::History(true),
        Step::Electrical(3),
        Step::Save,
        Step::Reopen,
        Step::Open,
        Step::Kind(1),
        Step::First,
        Step::Electrical(3),
        Step::Control(C::Edit),
        Step::Capture("symbol-reopened"),
        Step::KeyControl(C::Cancel),
    ];
    scenario::run(root, output, steps, Uuid::nil());
}
