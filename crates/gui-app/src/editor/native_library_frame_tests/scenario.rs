//! Finite designer sequence and canonical project / history assertions.
use super::*;

#[test]
#[ignore = "requires owned canonical eight-kind project, existing placed users and native Wayland"]
fn ordinary_library_browse_author_preview_cancel_history_save_reopen() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let browser =
        datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model));
    for (kind, _) in KINDS {
        assert!(
            browser.entries.iter().any(|e| e.kind == kind),
            "ordinary project contains {kind}"
        );
    }
    let package = browser
        .entries
        .iter()
        .find(|e| e.kind == "packages" && e.warning.is_none())
        .unwrap()
        .name
        .clone();
    let expected_package = browser
        .entries
        .iter()
        .find(|e| e.kind == "packages" && e.warning.is_none() && e.name == package)
        .unwrap()
        .id;
    let placed_footprint = browser
        .entries
        .iter()
        .find(|e| e.kind == "footprints" && e.name == "Native process footprint")
        .expect("source of the existing placed user");
    let used_stack = placed_footprint.raw["pads"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["padstack"]
        .as_str()
        .unwrap();
    let used_stack_name = browser
        .entries
        .iter()
        .find(|e| e.kind == "padstacks" && e.id.to_string() == used_stack)
        .unwrap()
        .name
        .clone();
    let placed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    assert!(
        !placed["packages"].as_object().unwrap().is_empty(),
        "existing placed user required"
    );
    let mut steps = vec![Step::Open, Step::Snapshot];
    for i in 0..8 {
        steps.extend([
            Step::Kind(i),
            Step::First,
            Step::Capture(match i {
                0 => "unit",
                1 => "symbol",
                2 => "entity",
                3 => "part",
                4 => "package",
                5 => "footprint",
                6 => "padstack",
                _ => "pin-pad-map",
            }),
        ]);
    }
    steps.extend([
        Step::Kind(5),
        Step::Search("Alternate footprint".into()),
        Step::Select("Alternate footprint".into()),
        Step::Capture("missing-geometry-reason"),
        Step::Control(C::Notice),
        Step::MissingGeometry,
        Step::Capture("missing-geometry-console-reason"),
        Step::Invariant,
        Step::Kind(6),
        Step::Control(C::New),
        Step::Field("name", "Cancelled copper".into()),
        Step::Choice("shape", "Circle".into()),
        Step::Field("width", "1000000".into()),
        Step::Control(C::Preview),
        Step::Capture("cancel-preview"),
        Step::Invariant,
        Step::Control(C::Cancel),
        Step::Invariant,
        Step::Control(C::New),
        Step::Field("name", "GUI resistor copper".into()),
        Step::Choice("shape", "Rect".into()),
        Step::Field("width", "1000000".into()),
        Step::Field("height", "500000".into()),
        Step::Field("drill", "200000".into()),
        Step::Choice("mask", "Exposed".into()),
        Step::Choice("paste", "Copper aperture".into()),
        Step::Control(C::Preview),
        Step::Capture("padstack-preview"),
        Step::Control(C::Commit),
        Step::Kind(5),
        Step::Control(C::New),
        Step::Field("name", "GUI two pad resistor".into()),
        Step::Choice("package", package),
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
        Step::Capture("footprint-preview"),
        Step::Control(C::Commit),
        Step::CheckFootprint(2),
        Step::RecordFootprint,
        Step::Snapshot,
        Step::Control(C::Edit),
        Step::Control(C::RemovePad),
        Step::Control(C::Preview),
        Step::CheckRemovedPreview,
        Step::Capture("remove-pad-preview"),
        Step::Control(C::Cancel),
        Step::Invariant,
        Step::Control(C::Edit),
        Step::Field("x", "-1500000".into()),
        Step::Control(C::Preview),
        Step::Capture("footprint-edit-preview"),
        Step::Control(C::Commit),
        Step::CheckFootprintValue(true),
        Step::History(false),
        Step::CheckFootprintValue(false),
        Step::History(true),
        Step::CheckFootprintValue(true),
        Step::Save,
        Step::CheckFrozen,
        Step::Kind(5),
        Step::Search("Native process footprint".into()),
        Step::Select("Native process footprint".into()),
        Step::Control(C::PreviewPad(0)),
        Step::CheckProcess(0),
        Step::Capture("process-pad-zero"),
        Step::Control(C::PreviewPad(1)),
        Step::CheckProcess(1),
        Step::Capture("process-pad-one"),
        Step::RecordPlaced(false),
        Step::Control(C::Edit),
        Step::Field("x", "-2000000".into()),
        Step::Control(C::Preview),
        Step::Capture("existing-source-preview"),
        Step::Control(C::Commit),
        Step::CheckPlaced(false),
        Step::CheckFrozen,
        Step::Kind(6),
        Step::Search(used_stack_name.clone()),
        Step::Select(used_stack_name),
        Step::RecordPlaced(true),
        Step::Control(C::Edit),
        Step::Field("width", "1200000".into()),
        Step::Control(C::Preview),
        Step::Control(C::Commit),
        Step::CheckPlaced(true),
        Step::CheckFrozen,
        Step::Save,
        Step::Kind(5),
        Step::Search("GUI two pad resistor".into()),
        Step::Select("GUI two pad resistor".into()),
        Step::Resize,
        Step::Capture("narrow-library"),
        Step::Relationship,
        Step::Search("GUI two pad resistor".into()),
        Step::Select("GUI two pad resistor".into()),
        Step::Tile,
        Step::Focus(C::Edit),
        Step::Capture("short-tiled-library"),
        Step::Control(C::Edit),
        Step::Capture("short-tiled-draft"),
        Step::Snapshot,
        Step::Field("layer", "0".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::Capture("narrow-refusal"),
        Step::Field("layer", "1".into()),
        Step::Field("x", "-1800000".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("narrow-edit-preview"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::VerifyReopen,
        Step::Reopen,
        Step::Open,
        Step::Kind(5),
        Step::Search("GUI two pad resistor".into()),
        Step::Select("GUI two pad resistor".into()),
        Step::CheckFootprintValue(true),
        Step::Capture("ordinary-gui-reopen"),
    ]);
    run(root, output, steps, expected_package);
}

pub(super) fn run(root: PathBuf, output: PathBuf, steps: Vec<Step>, expected_package: uuid::Uuid) {
    let args = <GuiArgs as clap::Parser>::try_parse_from([
        "datum-gui",
        "--project-root",
        root.to_str().unwrap(),
    ])
    .unwrap();
    let events = EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut fixture = Fixture {
        app: App::new(args, events.create_proxy()),
        stage: 0,
        since: Instant::now(),
        snapshot: BTreeMap::new(),
        board: std::fs::read(root.join("board/board.json")).unwrap(),
        root,
        output,
        failure: None,
        proxy: events.create_proxy(),
        original_footprint: None,
        edited_footprint: None,
        original_revision: None,
        expected_package,
        expected_terminals: None,
        placed_source: None,
        normal_pane_width: None,
        steps,
    };
    events.run_app(&mut fixture).unwrap();
    assert!(fixture.failure.is_none(), "{:?}", fixture.failure);
    assert_eq!(fixture.stage, fixture.steps.len());
}

#[test]
#[ignore = "requires saved ordinary library project and native Wayland"]
fn ordinary_library_actual_narrow_pane_preview_refusal_cancel() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let steps = vec![
        Step::Open,
        Step::Kind(5),
        Step::Search("GUI two pad resistor".into()),
        Step::Select("GUI two pad resistor".into()),
        Step::Narrow,
        Step::CheckNarrow,
        Step::Focus(C::Edit),
        Step::Capture("actual-narrow-library"),
        Step::Relationship,
        Step::Search("GUI two pad resistor".into()),
        Step::Select("GUI two pad resistor".into()),
        Step::Control(C::Edit),
        Step::Snapshot,
        Step::Field("layer", "0".into()),
        Step::KeyControl(C::Preview),
        Step::Refusal,
        Step::KeyControl(C::Notice),
        Step::RefusalNotice,
        Step::Capture("actual-narrow-refusal"),
        Step::Field("layer", "1".into()),
        Step::Field("x", "-1800000".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("actual-narrow-preview"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
        Step::Tile,
        Step::CheckNarrow,
        Step::Focus(C::Edit),
        Step::Capture("actual-narrow-short-library"),
        Step::Control(C::Edit),
        Step::Field("x", "-1800000".into()),
        Step::KeyControl(C::Preview),
        Step::Capture("actual-narrow-short-preview"),
        Step::KeyControl(C::Cancel),
        Step::Invariant,
    ];
    run(root, output, steps, uuid::Uuid::nil());
}
