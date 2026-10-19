use super::discovery::admit_answer;
use super::*;
fn state(mode: Mode, path: &str) -> FolderBrowserState {
    let mut state = FolderBrowserState::new(mode, path.into());
    state.directory = path.into();
    state.expanded = true;
    state.loading = false;
    state
}
#[test]
fn inline_navigation_keeps_exact_paths_and_home_shorthand() {
    for input in [
        "/parent/ project λ",
        "/parent/project λ ",
        " leading space ",
    ] {
        let expected = if Path::new(input).is_absolute() {
            PathBuf::from(input)
        } else {
            std::env::current_dir().unwrap().join(input)
        };
        assert_eq!(entered_path(input).unwrap(), expected);
        assert_eq!(initial_location(input).unwrap(), expected);
    }
    if let Some(home) = std::env::var_os("HOME") {
        assert_eq!(
            entered_path("~//folder").unwrap(),
            PathBuf::from(home).join("folder")
        );
    }
    assert!(entered_path("").is_err());
}
#[test]
fn inline_cancel_reopen_and_mode_change_reject_old_tickets() {
    let answer = |ticket| scan::ScanResult {
        ticket,
        input: None,
        directory: PathBuf::from("/new"),
        result: Ok(scan::Listing {
            entries: vec!["late".into()],
            parent_identity: Some((0, 0)),
            suggestions: None,
            occupied: Default::default(),
            notice: None,
        }),
    };
    let mut state = None;
    let mut pending = None;
    assert!(!admit_answer(&mut state, &mut pending, answer(1)));
    for mode in [Mode::New, Mode::Open] {
        state = Some(FolderBrowserState::new(mode, "/new".into()));
        pending = Some(2);
        assert!(!admit_answer(&mut state, &mut pending, answer(1)));
        assert_eq!(state.as_ref().unwrap().path_input, "/new");
        assert_eq!(pending, Some(2));
        assert!(admit_answer(&mut state, &mut pending, answer(2)));
    }
}
#[test]
fn inline_path_projection_preserves_units_override_and_terminal_state() {
    use datum_gui_protocol::*;
    let mut ui = load_fixture_workspace_state().ui;
    ui.new_project.open = true;
    ui.new_project.project_name = "Display name".into();
    ui.new_project.set_use_project_name(false);
    ui.new_project.folder_name = " exact λ ".into();
    ui.new_project.global_units_ready = true;
    let terminal = ui.terminal.clone();
    let mut navigation = state(Mode::New, "/parent");
    navigation.entries = vec!["highlight is not parent".into()];
    navigation.selected = Some(0);
    sync_location(&mut ui, &navigation);
    assert_eq!(ui.new_project.destination, "/parent/ exact λ ");
    assert!(ui.new_project.create_enabled());
    navigation.loading = true;
    sync_location(&mut ui, &navigation);
    assert!(ui.new_project.create_enabled());
    assert_eq!(ui.new_project.folder_name, " exact λ ");
    assert!(!ui.new_project.use_project_name);
    assert!(ui.new_project.global_units_ready);
    assert_eq!(ui.terminal, terminal);
    ui.new_project.open = false;
    let closed = ui.new_project.clone();
    sync_location(&mut ui, &navigation);
    assert_eq!(ui.new_project, closed);
    ui.project_open.open = true;
    ui.project_open.refusal = Some("Actual Open failure".into());
    let mut open = state(Mode::Open, "/open parent");
    open.entries = vec!["chosen ".into()];
    open.selected = Some(0);
    sync_location(&mut ui, &open);
    assert_eq!(ui.project_open.directory, "/open parent/chosen ");
    assert_eq!(
        ui.project_open.refusal.as_deref(),
        Some("Actual Open failure")
    );
    ui.project_open.busy = true;
    open.selected = None;
    sync_location(&mut ui, &open);
    assert_eq!(ui.project_open.directory, "/open parent/chosen ");
}
#[test]
fn inline_navigation_correction_cannot_clear_a_seed_refusal() {
    let mut ui = datum_gui_protocol::load_fixture_workspace_state().ui;
    ui.new_project.open = true;
    ui.new_project.project_name = "new".into();
    ui.new_project.refusal = Some("Global source unavailable".into());
    ui.new_project.global_units_ready = false;
    sync_location(&mut ui, &state(Mode::New, "/parent"));
    assert_eq!(
        ui.new_project.refusal.as_deref(),
        Some("Global source unavailable")
    );
    assert!(!ui.new_project.create_enabled());
}

#[test]
fn folder_browser_typed_open_and_listing_failure_bind_current_candidate() {
    let mut ui = datum_gui_protocol::load_fixture_workspace_state().ui;
    ui.project_open.open = true;
    let mut navigation = FolderBrowserState::new(Mode::Open, "/typed exact λ ".into());
    sync_location(&mut ui, &navigation);
    assert_eq!(ui.project_open.directory, "/typed exact λ ");
    navigation.expanded = true;
    navigation.directory = "/old".into();
    navigation.entries = vec!["stale".into()];
    navigation.selected = Some(0);
    navigation.error = Some("Listing denied".into());
    sync_location(&mut ui, &navigation);
    assert_eq!(ui.project_open.directory, "/typed exact λ ");
    assert!(ui.project_open.open_enabled());
}

#[test]
fn folder_browser_edited_location_rejects_late_reply_and_accepts_new_listing() {
    let answer = |ticket, path: &str, child: &str| scan::ScanResult {
        ticket,
        input: None,
        directory: PathBuf::from(path),
        result: Ok(scan::Listing {
            entries: vec![child.into()],
            parent_identity: Some((0, 0)),
            suggestions: None,
            occupied: Default::default(),
            notice: None,
        }),
    };
    let mut state = Some(FolderBrowserState::new(Mode::Open, "/new".into()));
    let mut pending = Some(2);
    assert!(!admit_answer(
        &mut state,
        &mut pending,
        answer(1, "/old", "stale")
    ));
    assert!(!admit_answer(
        &mut state,
        &mut pending,
        answer(2, "/old", "wrong path")
    ));
    assert!(state.as_ref().unwrap().entries.is_empty());
    assert!(admit_answer(
        &mut state,
        &mut pending,
        answer(2, "/new", "current")
    ));
    assert_eq!(
        state
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        vec!["current"]
    );
    assert_eq!(state.as_ref().unwrap().directory, "/new");
    assert_eq!(state.as_ref().unwrap().path_input, "/new");
}

#[test]
fn folder_browser_shorthand_row_navigation_reads_actual_child_and_admits_listing() {
    let cwd = std::env::current_dir().unwrap();
    let root = cwd
        .join(".datum/native-project-startup-repair/c09-tests")
        .join(format!("row-{}", std::process::id()));
    let child = root.join(" exact child λ ");
    std::fs::create_dir_all(child.join("actual grandchild")).unwrap();
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let spellings = [
        format!("~/{}", root.strip_prefix(home).unwrap().display()),
        root.strip_prefix(&cwd).unwrap().display().to_string(),
    ];
    let scanner = scan::Scanner::new(|| {}).unwrap();
    for spelling in spellings {
        let mut navigation = state(Mode::Open, &spelling);
        navigation.entries = vec![" exact child λ ".into()];
        navigation.selected = Some(0);
        let entered = input::selected_folder_path(&navigation).unwrap().unwrap();
        assert_eq!(entered, child);
        let ticket = scanner.request(entered.clone()).unwrap();
        navigation.path_input = entered.display().to_string();
        navigation.loading = true;
        let mut state = Some(navigation);
        let mut pending = Some(ticket);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(answer) = scanner.receive() {
                assert!(admit_answer(&mut state, &mut pending, answer));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let current = state.unwrap();
        assert!(current.ready());
        assert_eq!(
            current
                .entries
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>(),
            vec!["actual grandchild"]
        );
        assert_eq!(current.path_input, child.display().to_string());
    }
    std::fs::remove_dir_all(root).unwrap();
}
