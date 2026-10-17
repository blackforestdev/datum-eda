//! Empty native Project -> GUI Package -> named Footprint, through ordinary App.
use super::*;

impl Fixture {
    pub(super) fn check_package(&mut self, present: bool) {
        let browser = &self.app.runtime.as_ref().unwrap().workspace().library;
        let entry = browser
            .entries
            .iter()
            .find(|e| e.kind == "packages" && e.name == "GUI physical body");
        if !present {
            assert!(entry.is_none(), "Undo removes the created Package");
            return;
        }
        let entry = entry.expect("Package appears in ordinary named results");
        if self.expected_package.is_nil() {
            self.expected_package = entry.id;
        }
        assert_eq!(entry.id, self.expected_package);
        let raw = &entry.raw;
        assert_eq!(raw["package_family"], "Designer family");
        assert_eq!(raw["package_code"], "Designer code");
        assert_eq!(raw["mounting_type"], "Designer mounting");
        assert_eq!(
            raw["body_dimensions"],
            serde_json::json!({"x_nm":5000000,"y_nm":3000000})
        );
        assert_eq!(raw["body_height_nm"], 1500000);
        assert!(raw["body_height_mounted_nm"].is_null());
        let terminals = raw["terminals"].as_object().unwrap();
        assert_eq!(terminals.len(), 2);
        for (id, terminal) in terminals {
            assert_eq!(terminal["uuid"].as_str(), Some(id.as_str()));
            match terminal["name"].as_str().unwrap() {
                "1" => assert_eq!(terminal["role"], "power"),
                "2" => assert!(terminal.get("role").is_none()),
                other => panic!("unexpected terminal {other}"),
            }
        }
        if let Some(expected) = &self.expected_terminals {
            assert_eq!(
                &raw["terminals"], expected,
                "terminal IDs and optional absence survive history/reopen"
            );
        } else {
            self.expected_terminals = Some(raw["terminals"].clone());
        }
        assert!(raw["pads"].as_object().unwrap().is_empty());
        assert!(
            !entry.editable(),
            "saved Package editing remains unavailable"
        );
        assert!(raw.get("approval_status").is_none());
        assert!(raw.get("standards_basis").is_none());
    }
}

#[test]
#[ignore = "requires owned fresh Project/output paths, native Wayland and matching daemon"]
fn ordinary_package_creation_to_named_footprint_history_and_reopen() {
    let root = PathBuf::from(std::env::var("DATUM_PACKAGE_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    assert!(
        !root.exists(),
        "retain prior attempts; use a fresh owned Project path"
    );
    eda_engine::api::native_write::genesis::bootstrap_native_project(
        &root,
        eda_engine::api::native_write::genesis::GenesisSpec {
            project_name: "GUI Package creation".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    std::fs::create_dir_all(&output).unwrap();
    let initial = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    assert!(
        datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(initial))
            .entries
            .is_empty()
    );
    let steps = vec![
        Step::Open,
        Step::Kind(4),
        Step::Snapshot,
        Step::Control(C::New),
        Step::Field("name", "Cancelled body".into()),
        Step::Field("body_x", "4 mm".into()),
        Step::Field("body_y", "2 mm".into()),
        Step::Field("terminal_name", "cancelled terminal".into()),
        Step::Control(C::Preview),
        Step::Capture("package-cancel-preview"),
        Step::Invariant,
        Step::Control(C::Cancel),
        Step::Invariant,
        Step::Control(C::New),
        Step::Field("name", "GUI physical body".into()),
        Step::Field("package_family", "Designer family".into()),
        Step::Field("package_code", "Designer code".into()),
        Step::Field("mounting_type", "Designer mounting".into()),
        Step::Field("body_x", "5 mm".into()),
        Step::Field("body_y", "3 mm".into()),
        Step::Field("body_height_nm", "1.5 mm".into()),
        Step::Field("terminal_name", "1".into()),
        Step::Field("terminal_role", "power".into()),
        Step::Control(C::AddTerminal),
        Step::Field("terminal_name", "2".into()),
        Step::Control(C::AddTerminal),
        Step::Field("terminal_name", "discard".into()),
        Step::Control(C::RemoveTerminal),
        Step::Field("body_x", "invalid".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("package-malformed-refusal"),
        Step::Field("body_x", "5 mm".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("package-body-preview"),
        Step::Invariant,
        Step::KeyControl(C::Commit),
        Step::CheckPackage(true),
        Step::History(false),
        Step::CheckPackage(false),
        Step::History(true),
        Step::CheckPackage(true),
        Step::Save,
        Step::Reopen,
        Step::Open,
        Step::Kind(4),
        Step::Select("GUI physical body".into()),
        Step::CheckPackage(true),
        Step::Capture("package-reopened"),
        Step::Kind(6),
        Step::Control(C::New),
        Step::Field("name", "GUI resistor copper".into()),
        Step::Choice("shape", "Rect".into()),
        Step::Field("width", "1000000".into()),
        Step::Field("height", "500000".into()),
        Step::Field("drill", "200000".into()),
        Step::Choice("mask", "Exposed".into()),
        Step::Choice("paste", "Copper aperture".into()),
        Step::Control(C::Preview),
        Step::Control(C::Commit),
        Step::Kind(5),
        Step::Control(C::New),
        Step::Field("name", "GUI two pad resistor".into()),
        Step::Choice("package", "GUI physical body".into()),
        Step::Field("pad_name", "1".into()),
        Step::Choice("padstack", "GUI resistor copper".into()),
        Step::Field("x", "-1000000".into()),
        Step::Field("y", "0".into()),
        Step::Field("layer", "1".into()),
        Step::Control(C::AddPad),
        Step::Field("pad_name", "2".into()),
        Step::Choice("padstack", "GUI resistor copper".into()),
        Step::Field("x", "1000000".into()),
        Step::Field("y", "0".into()),
        Step::Field("layer", "1".into()),
        Step::Control(C::Preview),
        Step::Capture("package-linked-footprint-preview"),
        Step::Control(C::Commit),
        Step::CheckFootprint(2),
        Step::RecordFootprint,
        Step::Save,
        Step::CheckFrozen,
        Step::Kind(4),
        Step::Narrow,
        Step::CheckNarrow,
        Step::Snapshot,
        Step::KeyControl(C::New),
        Step::Field("name", "Narrow draft".into()),
        Step::Field("body_x", "invalid".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("package-narrow-refusal"),
        Step::Field("body_x", "2 mm".into()),
        Step::Field("body_y", "1 mm".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("package-narrow-preview"),
        Step::Invariant,
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::Reopen,
        Step::Open,
        Step::Kind(4),
        Step::Select("GUI physical body".into()),
        Step::CheckPackage(true),
        Step::Kind(5),
        Step::Select("GUI two pad resistor".into()),
        Step::CheckFootprint(2),
        Step::VerifyReopen,
        Step::Capture("package-footprint-reopened"),
    ];
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
