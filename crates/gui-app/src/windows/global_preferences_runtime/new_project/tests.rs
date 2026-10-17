use super::*;
use eda_engine::preferences::{FixedPreferenceLocationProvider, PreferenceLocations};

#[test]
fn home_shorthand_is_input_only_and_keeps_the_whole_destination() {
    let home = std::ffi::OsStr::new("/owned/home");
    for (input, expected) in [
        ("~/projects/new", "/owned/home/projects/new"),
        ("~//projects/new", "/owned/home/projects/new"),
        ("~", "/owned/home"),
        ("/absolute/new", "/absolute/new"),
        ("relative/new", "relative/new"),
        ("$HOME/new", "$HOME/new"),
    ] {
        assert_eq!(
            destination_from_input(input, Some(home)).unwrap(),
            std::path::PathBuf::from(expected)
        );
    }
    assert!(destination_from_input("~/new", None).is_err());
    assert!(destination_from_input("~/new", Some(std::ffi::OsStr::new("relative"))).is_err());
}

#[test]
fn destination_failure_keeps_units_and_correction_retries_without_rereading_global() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    ui.new_project.project_name = "Display name".into();
    let rows = ui.new_project.units_summary.clone();
    let source = coordinator.new_project_source.clone();
    let source_detail = ui.new_project.source_detail.clone();
    let collision = root.join("occupied");
    std::fs::create_dir(&collision).unwrap();
    std::fs::write(collision.join("preserved"), b"owner bytes").unwrap();
    ui.new_project.destination = collision.display().to_string();
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::RepositoryIo
    );
    // Existing non-Project directories fail the engine's receipt read, before
    // the different-Project request check. Preserve that engine classification.
    let reason = error.details["reason"].as_str().unwrap();
    assert!(ui.new_project.refusal.as_ref().unwrap().contains(reason));
    assert!(
        ui.new_project
            .refusal
            .as_ref()
            .unwrap()
            .contains("Location exists. Use a new folder.")
    );
    assert_eq!(
        std::fs::read(collision.join("preserved")).unwrap(),
        b"owner bytes"
    );
    assert!(
        !collision.join("Display name").exists(),
        "destination is not a parent plus name"
    );
    assert_eq!(ui.new_project.units_summary, rows);
    assert_eq!(coordinator.new_project_source, source);
    assert_eq!(ui.new_project.source_detail, source_detail);
    assert!(ui.new_project.create_enabled());
    assert!(!ui.new_project.retry_global_available());
    let destination = root.join("corrected-project");
    ui.new_project.destination = destination.display().to_string();
    ui.new_project.clear_creation_refusal();
    assert!(ui.new_project.refusal.is_none());
    coordinator.create_new_project(&mut ui).unwrap();
    let raw: Value =
        serde_json::from_slice(&std::fs::read(destination.join("project.json")).unwrap()).unwrap();
    assert_eq!(raw["name"], "Display name");
    assert_eq!(
        raw["project_units_seed_receipt"]["items"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_destination_parent_keeps_units_and_exposes_io_reason() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    ui.new_project.project_name = "IO refusal".into();
    ui.new_project.destination = root.join("missing-parent/project").display().to_string();
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::RepositoryIo
    );
    let reason = error.details["reason"].as_str().unwrap();
    assert!(ui.new_project.refusal.as_ref().unwrap().contains(reason));
    assert_eq!(ui.new_project.units_summary.len(), 8);
    assert!(coordinator.new_project_source.is_some());
    assert!(ui.new_project.create_enabled());
    assert!(!root.join("missing-parent").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_global_snapshot_still_invalidates_units_and_requires_explicit_reread() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    let mut writer = GlobalPreferencesProductService::open(
        &FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: root.clone(),
            repository_root: root.join("preferences"),
            legacy_console_path: root.join("legacy.json"),
        }),
        "separate-writer",
    )
    .unwrap();
    writer
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.units.board_length".into(),
                value: Value::String("mil".into()),
                expected: head_expectation(writer.status()),
                request_id: new_product_id(),
                reason: "Explicit defaults change".into(),
            },
            &human_gui_actor(),
        )
        .unwrap();
    ui.new_project.project_name = "Stale seed".into();
    ui.new_project.destination = root.join("must-not-exist").display().to_string();
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::StaleGeneration
    );
    assert!(ui.new_project.units_summary.is_empty());
    assert!(coordinator.new_project_source.is_none());
    ui.new_project.clear_creation_refusal();
    assert!(ui.new_project.refusal.is_some());
    assert!(!ui.new_project.create_enabled());
    assert!(ui.new_project.retry_global_available());
    assert!(!root.join("must-not-exist").exists());
    coordinator.refresh_new_project_preview(&mut ui);
    assert!(ui.new_project.create_enabled());
    std::fs::remove_dir_all(root).unwrap();
}

fn fixture() -> (
    std::path::PathBuf,
    GlobalPreferencesCoordinator,
    WorkspaceUiState,
) {
    let root = std::env::temp_dir().join(format!(
        "datum-new-project-gui-{}-{}",
        std::process::id(),
        new_product_id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let service = GlobalPreferencesProductService::open(
        &FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: root.clone(),
            repository_root: root.join("preferences"),
            legacy_console_path: root.join("legacy.json"),
        }),
        "new-project-gui-test",
    )
    .unwrap();
    let coordinator = GlobalPreferencesCoordinator {
        service,
        return_focus: ApplicationFocus::default(),
        new_project_source: None,
        terminal_theme_before_high_contrast: None,
    };
    let filters = datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        excluded_selection_classes: Default::default(),
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    };
    (root, coordinator, WorkspaceUiState::new(filters))
}

#[test]
fn opening_defaults_to_global_and_projects_exactly_eight_resolved_values() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    assert_eq!(ui.new_project.units_choice, NewProjectUnitsChoice::Global);
    assert_eq!(ui.new_project.units_summary.len(), 8);
    assert_eq!(
        ui.new_project
            .units_summary
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        ACTIVE_UNITS_KEYS
    );
    assert_eq!(
        ui.new_project.source_detail,
        "factory defaults; no preference file yet"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn factory_requires_explicit_choice_and_never_creates_a_preference_repository() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    ui.new_project.units_choice = NewProjectUnitsChoice::Factory;
    coordinator.refresh_new_project_preview(&mut ui);
    assert!(
        ui.new_project
            .source_detail
            .contains("datum.units.factory.v1")
    );
    assert!(!root.join("preferences").exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn create_publishes_one_real_project_with_embedded_eight_item_receipt() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    let destination = root.join("created-project");
    ui.new_project.project_name = "created-project".to_owned();
    ui.new_project.destination = destination.to_string_lossy().into_owned();
    coordinator.create_new_project(&mut ui).unwrap();
    let project: Value =
        serde_json::from_slice(&std::fs::read(destination.join("project.json")).unwrap()).unwrap();
    assert_eq!(
        project["project_units_seed_receipt"]["items"]
            .as_array()
            .map(Vec::len),
        Some(8)
    );
    assert!(!project.to_string().contains("revision"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unresolved_global_reader_preserves_draft_without_genesis_or_implicit_factory() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.service =
        GlobalPreferencesProductService::factory_only("unresolved-global-test").unwrap();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    ui.new_project.project_name = "Preserved name".into();
    let destination = root.join("must-stay-absent");
    ui.new_project.destination = destination.display().to_string();
    ui.new_project.focus = NewProjectFocus::Destination;
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::SeedSourceUnavailable
    );
    assert_eq!(ui.new_project.project_name, "Preserved name");
    assert_eq!(
        ui.new_project.destination,
        destination.display().to_string()
    );
    assert_eq!(ui.new_project.focus, NewProjectFocus::Destination);
    assert_eq!(ui.new_project.units_choice, NewProjectUnitsChoice::Global);
    assert!(ui.new_project.refusal.is_some());
    assert!(!ui.new_project.create_enabled());
    assert!(!destination.exists());
    assert!(!root.join("preferences").exists());
    ui.new_project.units_choice = NewProjectUnitsChoice::Factory;
    coordinator.refresh_new_project_preview(&mut ui);
    assert_eq!(ui.new_project.units_summary.len(), 8);
    assert!(ui.new_project.create_enabled());
    assert!(!destination.exists());
    assert!(!root.join("preferences").exists());
    let _ = std::fs::remove_dir_all(root);
}
