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
    let ready = ui.new_project.global_units_ready;
    let source = coordinator.new_project_source.clone();
    let collision = root.join("occupied");
    std::fs::create_dir(&collision).unwrap();
    std::fs::write(collision.join("preserved"), b"owner bytes").unwrap();
    ui.new_project.destination = collision.display().to_string();
    ui.new_project.location_ready = true;
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
    assert_eq!(ui.new_project.global_units_ready, ready);
    assert_eq!(coordinator.new_project_source, source);
    assert!(ui.new_project.create_enabled());
    assert!(!ui.new_project.retry_global_available());
    let destination = root.join("corrected-project");
    ui.new_project.destination = destination.display().to_string();
    ui.new_project.location_ready = true;
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
    ui.new_project.location_ready = true;
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::RepositoryIo
    );
    let reason = error.details["reason"].as_str().unwrap();
    assert!(ui.new_project.refusal.as_ref().unwrap().contains(reason));
    assert!(ui.new_project.global_units_ready);
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
    ui.new_project.location_ready = true;
    let error = coordinator.create_new_project(&mut ui).unwrap_err();
    assert_eq!(
        error.code,
        eda_engine::preferences::PreferenceErrorCodeV1::StaleGeneration
    );
    assert!(!ui.new_project.global_units_ready);
    assert!(coordinator.new_project_source.is_none());
    ui.new_project.clear_creation_refusal();
    assert!(ui.new_project.refusal.is_some());
    assert!(!ui.new_project.create_enabled());
    assert!(ui.new_project.retry_global_available());
    assert!(!root.join("must-not-exist").exists());
    ui.new_project.focus = NewProjectFocus::RetryGlobal;
    coordinator.refresh_new_project_preview(&mut ui);
    assert!(ui.new_project.create_enabled());
    assert_eq!(ui.new_project.focus, NewProjectFocus::Create);
    assert!(
        !root.join("must-not-exist").exists(),
        "reread never creates automatically"
    );
    coordinator.create_new_project(&mut ui).unwrap();
    let project: Value =
        serde_json::from_slice(&std::fs::read(root.join("must-not-exist/project.json")).unwrap())
            .unwrap();
    assert_eq!(
        project["project_units_seed_receipt"]["source"]["kind"],
        "global"
    );
    let items = project["project_units_seed_receipt"]["items"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 8);
    assert_eq!(
        items
            .iter()
            .find(|i| i["key"] == "datum.units.board_length")
            .unwrap()["copied_value"],
        "mil"
    );
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
fn opening_automatically_pins_global_without_units_controls_or_repository_creation() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    assert!(ui.new_project.global_units_ready);
    assert!(matches!(
        coordinator.new_project_source,
        Some(ProjectUnitsSourceV1::Global {
            expected_generation: None
        })
    ));
    assert!(!root.join("preferences").exists());
    let nodes = ui.new_project.accessibility_nodes();
    assert!(
        !nodes
            .iter()
            .any(|n| n.id.contains("units") || n.name.contains("Factory"))
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn create_publishes_one_real_project_with_embedded_eight_item_receipt() {
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    let destination = root.join("created-project");
    ui.new_project.project_name = "created-project".to_owned();
    ui.new_project.destination = destination.to_string_lossy().into_owned();
    ui.new_project.location_ready = true;
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
    ui.new_project.location_ready = true;
    ui.new_project.focus =
        NewProjectFocus::Navigation(datum_gui_protocol::FolderBrowserFocus::Path);
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
    assert_eq!(
        ui.new_project.focus,
        NewProjectFocus::Navigation(datum_gui_protocol::FolderBrowserFocus::Path)
    );
    assert!(ui.new_project.refusal.is_some());
    assert!(!ui.new_project.create_enabled());
    assert!(!destination.exists());
    assert!(!root.join("preferences").exists());
    coordinator.refresh_new_project_preview(&mut ui);
    assert!(!ui.new_project.global_units_ready);
    assert!(!ui.new_project.create_enabled());
    assert!(coordinator.new_project_source.is_none());
    assert!(!destination.exists());
    assert!(!root.join("preferences").exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn folder_browser_adapters_create_and_open_exact_spaced_unicode_paths() {
    let (root, mut coordinator, mut ui) = fixture();
    let paths = [
        root.join(" project λ"),
        root.join("project λ"),
        root.join("project λ "),
    ];
    let mut identities = vec![];
    for (index, path) in paths.iter().enumerate() {
        coordinator.open_new_project(&mut ui, ApplicationFocus::default());
        ui.new_project.project_name = format!("Distinct Project {index}");
        ui.new_project.destination = path.to_str().unwrap().into();
        ui.new_project.location_ready = true;
        let created = coordinator.create_new_project(&mut ui).unwrap();
        assert_eq!(
            std::path::Path::new(&created.project_root_identity),
            path.canonicalize().unwrap()
        );
        identities.push(std::fs::read(path.join("project.json")).unwrap());
        let prepared =
            crate::project_open::prepare_project_directory(&ui.new_project.destination, false)
                .unwrap();
        assert_eq!(prepared.root, path.canonicalize().unwrap());
        assert_eq!(
            std::fs::read(prepared.root.join("project.json")).unwrap(),
            identities[index]
        );
    }
    assert_ne!(identities[0], identities[1]);
    assert_ne!(identities[1], identities[2]);
    ui.new_project.destination = paths[2].to_str().unwrap().into();
    ui.new_project.location_ready = true;
    assert!(coordinator.create_new_project(&mut ui).is_err());
    for (path, before) in paths.iter().zip(&identities) {
        assert_eq!(&std::fs::read(path.join("project.json")).unwrap(), before);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn folder_browser_inline_name_and_override_compose_once_for_real_genesis() {
    use datum_gui_protocol::{FolderBrowserMode, FolderBrowserState};
    let (root, mut coordinator, mut ui) = fixture();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    let source = coordinator.new_project_source.clone();
    let mut nav = FolderBrowserState::new(FolderBrowserMode::New, root.to_str().unwrap().into());
    nav.directory = nav.path_input.clone();
    nav.loading = false;
    nav.entries = vec!["highlight is not parent".into()];
    nav.selected = Some(0);
    ui.new_project.project_name = " default λ ".into();
    let mut sources = vec![];
    for (index, leaf) in [" default λ ", " override λ ", "renamed"]
        .into_iter()
        .enumerate()
    {
        if index == 1 {
            ui.new_project.set_use_project_name(false);
            ui.new_project.folder_name = leaf.into();
            ui.new_project.project_name = "renamed".into();
        }
        if index == 2 {
            ui.new_project.set_use_project_name(true);
        }
        crate::folder_browser::sync_location(&mut ui, &nav);
        let expected = root.join(leaf);
        assert_eq!(ui.new_project.destination, expected.to_str().unwrap());
        assert!(ui.new_project.create_enabled());
        let created = coordinator.create_new_project(&mut ui).unwrap();
        assert_eq!(
            std::path::Path::new(&created.project_root_identity),
            expected.canonicalize().unwrap()
        );
        sources.push((
            expected.clone(),
            std::fs::read(expected.join("project.json")).unwrap(),
        ));
        assert_eq!(coordinator.new_project_source, source);
        assert!(ui.new_project.global_units_ready);
    }
    nav.occupied.insert("renamed".into());
    crate::folder_browser::sync_location(&mut ui, &nav);
    assert!(!ui.new_project.create_enabled());
    for (path, before) in sources {
        assert_eq!(std::fs::read(path.join("project.json")).unwrap(), before);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupt_global_source_blocks_creation_until_explicit_repair_and_reread() {
    let (root, mut coordinator, mut ui) = fixture();
    std::fs::write(
        root.join("preferences"),
        b"owned non-directory repository fault",
    )
    .unwrap();
    coordinator.open_new_project(&mut ui, ApplicationFocus::default());
    ui.new_project.project_name = "preserved".into();
    ui.new_project.destination = root.join("must-stay-absent").to_str().unwrap().into();
    ui.new_project.location_ready = true;
    ui.new_project.focus =
        NewProjectFocus::Navigation(datum_gui_protocol::FolderBrowserFocus::Path);
    assert!(!ui.new_project.create_enabled());
    assert!(ui.new_project.retry_global_available());
    assert!(coordinator.new_project_source.is_none());
    assert!(
        !ui.new_project
            .refusal
            .as_ref()
            .unwrap()
            .contains("choose Factory")
    );
    ui.new_project.clear_creation_refusal();
    assert!(ui.new_project.refusal.is_some());
    std::fs::remove_file(root.join("preferences")).unwrap();
    assert!(
        !ui.new_project.create_enabled(),
        "repair alone never repins the open form"
    );
    ui.new_project.focus = NewProjectFocus::RetryGlobal;
    coordinator.refresh_new_project_preview(&mut ui);
    assert!(ui.new_project.create_enabled());
    assert_eq!(ui.new_project.focus, NewProjectFocus::Create);
    assert_eq!(ui.new_project.project_name, "preserved");
    assert!(!root.join("must-stay-absent").exists());
    assert!(!root.join("preferences").exists());
    std::fs::remove_dir_all(root).unwrap();
}
