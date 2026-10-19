//! Shared Runtime keyboard assistance with real asynchronous folder discovery.
use super::*;
use clap::Parser;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn typed(runtime: &mut Runtime, path: &Path) {
    runtime.folder_browser.state.as_mut().unwrap().path_input = path.display().to_string();
    runtime.folder_browser.state.as_mut().unwrap().path_editor =
        datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(path.to_str().unwrap());
    runtime.folder_browser.invalidate_listing();
    runtime.read_folder_request(path.into(), false, true);
    wait(runtime);
}
fn wait(runtime: &mut Runtime) {
    let until = Instant::now() + Duration::from_secs(5);
    while runtime.folder_browser.state.as_ref().unwrap().loading {
        runtime.poll_folder_browser();
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn retained_initial_answer(runtime: &mut Runtime) -> scan::ScanResult {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(answer) = runtime.folder_browser.scanner.as_ref().unwrap().receive() {
            return answer;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn reject_initial_answer(runtime: &mut Runtime, answer: scan::ScanResult) {
    assert!(!discovery::admit_answer(
        &mut runtime.folder_browser.state,
        &mut runtime.folder_browser.pending,
        answer
    ));
}
#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn path_assistance_shared_runtime_preserves_candidates_drafts_and_exact_directory_guards() {
    let root = Scratch(std::env::temp_dir().join(format!(
        "datum-path-assist-{}",
        eda_engine::preferences::new_product_id()
    )));
    for name in ["Documents", "Documents-old", "Downloads", "space λ "] {
        std::fs::create_dir_all(root.0.join(name)).unwrap();
    }
    let events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = events.create_proxy();
    let window = Arc::new(
        events
            .create_window(
                Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let args = GuiArgs::try_parse_from(["datum-gui"]).unwrap();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let original = runtime.terminal_sessions.active().session_id().to_owned();
    for mode in [Mode::New, Mode::Open] {
        if mode == Mode::New {
            runtime.open_new_project();
        } else {
            runtime.open_project();
        }
        let home = initial_location("").unwrap();
        let initial_focus = runtime.folder_navigation_focus();
        assert!(runtime.folder_browser.state.as_ref().unwrap().expanded);
        assert!(runtime.folder_browser.state.as_ref().unwrap().loading);
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .selected
                .is_none()
        );
        wait(&mut runtime);
        let initial = runtime.folder_browser.state.as_ref().unwrap();
        assert!(initial.ready() && initial.suggestions.is_none());
        assert_eq!(initial.directory, home.display().to_string());
        assert!(initial.parent_identity.is_some() && initial.selected.is_none());
        assert_eq!(runtime.folder_navigation_focus(), initial_focus);
        if mode == Mode::New {
            assert_eq!(
                runtime.workspace().ui.new_project.focus,
                datum_gui_protocol::NewProjectFocus::ProjectName
            );
        } else {
            assert_eq!(initial_focus, Some(Focus::Path));
        }
        assert!(!runtime.workspace().ui.project_open.busy);
        assert!(!runtime.terminal_sessions.project_preparation_pending());
        assert_eq!(runtime.terminal_sessions.active().session_id(), original);
        runtime.session.workspace_mut().ui.new_project.project_name = "kept draft".into();
        runtime
            .session
            .workspace_mut()
            .ui
            .new_project
            .set_use_project_name(false);
        runtime.session.workspace_mut().ui.new_project.folder_name = " custom λ ".into();
        let defaults = runtime.workspace().ui.new_project.global_units_ready;
        let query = root.0.join("Doc");
        typed(&mut runtime, &query);
        assert!(runtime.folder_browser.state.as_ref().unwrap().expanded);
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .suggestions_ready()
        );
        assert!(!runtime.folder_browser.state.as_ref().unwrap().ready());
        runtime.set_folder_navigation_focus(Focus::Path);
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        let query = root.0.join("Documents");
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .suggestions_ready()
        );
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            query.display().to_string()
        );
        assert!(
            !runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)),
            "unchanged common start traverses"
        );
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowDown)));
        assert_eq!(runtime.folder_navigation_focus(), Some(Focus::Path));
        assert_eq!(
            runtime
                .terminal_accessibility
                .last_announcement()
                .unwrap()
                .text,
            "Documents, folder suggestion, 1 of 2, selected"
        );
        runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowDown));
        assert_eq!(
            runtime
                .terminal_accessibility
                .last_announcement()
                .unwrap()
                .text,
            "Documents-old, folder suggestion, 2 of 2, selected"
        );
        runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowUp));
        assert_eq!(
            runtime
                .terminal_accessibility
                .last_announcement()
                .unwrap()
                .text,
            "Documents, folder suggestion, 1 of 2, selected"
        );
        runtime.refresh_folder_browser();
        let preview = runtime
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .preview((mode == Mode::New).then_some(" custom λ "))
            .unwrap();
        assert_eq!(
            PathBuf::from(&preview),
            if mode == Mode::New {
                query.join(" custom λ ")
            } else {
                query.clone()
            }
        );
        assert!(
            !runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .opens_selected_manifest()
        );
        runtime.open_folder_actions_menu(Some(0), (200.0, 240.0));
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .menu
                .as_ref()
                .unwrap()
                .items
                .iter()
                .all(|i| i.unavailable.is_some())
        );
        runtime.folder_browser.state.as_mut().unwrap().menu = None;
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Escape)));
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            query.display().to_string()
        );
        assert!(!runtime.handle_path_assistance_key(&Key::Named(NamedKey::Escape)));
        assert!(
            !runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)),
            "list focus retains ordinary Tab traversal"
        );
        runtime.set_folder_navigation_focus(Focus::Path);
        runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowDown));
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        assert_eq!(
            runtime
                .terminal_accessibility
                .last_announcement()
                .unwrap()
                .text,
            format!("Location {}/", root.0.join("Documents").display())
        );
        wait(&mut runtime);
        assert_eq!(
            PathBuf::from(&runtime.folder_browser.state.as_ref().unwrap().path_input),
            root.0.join("Documents")
        );
        assert!(runtime.folder_browser.state.as_ref().unwrap().ready());
        let documents = root.0.join("Documents");
        for name in ["datum-eda", "datum-eda-backups", "datum-eda-workspaces"] {
            std::fs::create_dir_all(documents.join(name)).unwrap();
        }
        let datum = documents.join("datum-eda");
        std::fs::create_dir_all(datum.join("design-targets")).unwrap();
        typed(&mut runtime, &documents.join("datum-ed"));
        runtime.set_folder_navigation_focus(Focus::Path);
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            datum.display().to_string()
        );
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .suggestions_ready()
        );
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().entries.len(),
            3
        );
        assert!(!runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowDown));
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        assert!(runtime.folder_browser.state.as_ref().unwrap().ready());
        typed(&mut runtime, &datum.join("design"));
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        assert_eq!(
            PathBuf::from(&runtime.folder_browser.state.as_ref().unwrap().path_input),
            datum.join("design-targets")
        );
        let state = runtime.folder_browser.state.as_mut().unwrap();
        state.path_input = datum.display().to_string();
        state.path_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&state.path_input);
        runtime.read_folder_navigation(datum.clone(), false);
        wait(&mut runtime);
        runtime.folder_browser.state.as_mut().unwrap().selected = Some(0);
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            format!("{}/", datum.display())
        );
        assert!(!runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        let prefix_root = root.0.join("prefix-case");
        for name in [
            "datum-eda",
            "datum-eda-backups",
            "datum-eda-workspaces",
            "datum_projects",
            "plugdata",
        ] {
            std::fs::create_dir_all(prefix_root.join(name)).unwrap();
        }
        typed(&mut runtime, &prefix_root.join("dat"));
        runtime.set_folder_navigation_focus(Focus::Path);
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().entries.len(),
            5
        );
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        wait(&mut runtime);
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            prefix_root.join("datum").display().to_string()
        );
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().entries.len(),
            4
        );
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .suggestions_ready()
        );
        assert_eq!(runtime.folder_navigation_focus(), Some(Focus::Path));
        let vanishing = root.0.join("Vanishing folder");
        std::fs::create_dir(&vanishing).unwrap();
        typed(&mut runtime, &root.0.join("Vanish"));
        runtime.set_folder_navigation_focus(Focus::Path);
        runtime.handle_path_assistance_key(&Key::Named(NamedKey::ArrowDown));
        std::fs::remove_dir(&vanishing).unwrap();
        if mode == Mode::Open {
            std::fs::write(&vanishing, b"replaced by file").unwrap();
        }
        assert!(runtime.handle_path_assistance_key(&Key::Named(NamedKey::Enter)));
        wait(&mut runtime);
        assert_eq!(
            runtime.folder_browser.state.as_ref().unwrap().path_input,
            format!("{}/", vanishing.display())
        );
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .error
                .as_deref()
                .unwrap()
                .contains("Cannot read folder")
        );
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .suggestions
                .is_none(),
            "accepted missing/replaced folder is explicit navigation, never another neutral query"
        );
        if mode == Mode::Open {
            std::fs::remove_file(&vanishing).unwrap();
        }
        typed(&mut runtime, &root.0.join("missing"));
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .error
                .is_none()
        );
        assert!(!runtime.handle_path_assistance_key(&Key::Named(NamedKey::Tab)));
        runtime.read_folder_navigation(root.0.join("missing"), false);
        wait(&mut runtime);
        assert!(
            runtime
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .error
                .as_deref()
                .unwrap()
                .contains("Cannot read folder")
        );
        assert_eq!(runtime.terminal_sessions.active().session_id(), original);
        assert_eq!(
            runtime.workspace().ui.new_project.global_units_ready,
            defaults
        );
        assert_eq!(runtime.workspace().ui.new_project.folder_name, " custom λ ");
        if mode == Mode::New {
            runtime.close_new_project();
        } else {
            runtime.close_project_open();
        }
    }
    // Hold a real initial worker reply, then change input before admitting it.
    runtime.open_project();
    let old_home = retained_initial_answer(&mut runtime);
    let edited = root.0.join("Doc");
    let state = runtime.folder_browser.state.as_mut().unwrap();
    state.path_input = edited.display().to_string();
    state.path_editor =
        datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&state.path_input);
    runtime.folder_browser.invalidate_listing();
    runtime.read_folder_request(edited.clone(), false, true);
    reject_initial_answer(&mut runtime, old_home);
    wait(&mut runtime);
    assert_eq!(
        runtime.folder_browser.state.as_ref().unwrap().path_input,
        edited.display().to_string()
    );
    runtime.close_project_open();
    runtime.open_new_project();
    let old_home = retained_initial_answer(&mut runtime);
    runtime.close_new_project();
    runtime.open_project();
    reject_initial_answer(&mut runtime, old_home);
    wait(&mut runtime);
    assert!(runtime.folder_browser.state.as_ref().unwrap().ready());
    assert_eq!(
        runtime.folder_browser.state.as_ref().unwrap().directory,
        initial_location("").unwrap().display().to_string()
    );
    // The created-root handoff must supersede its default Home read.
    runtime.open_project();
    let old_home = retained_initial_answer(&mut runtime);
    let created = root.0.join("created-root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let files = [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ];
    for rel in files {
        let path = created.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(rel), path).unwrap();
    }
    let before: Vec<_> = files
        .iter()
        .map(|rel| std::fs::read(created.join(rel)).unwrap())
        .collect();
    runtime.open_created_project(created.clone());
    reject_initial_answer(&mut runtime, old_home);
    wait(&mut runtime);
    assert!(runtime.folder_browser.state.as_ref().unwrap().ready());
    assert_eq!(
        runtime.folder_browser.state.as_ref().unwrap().directory,
        created.display().to_string()
    );
    assert!(
        runtime
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .any(|entry| entry.name == "project.json")
    );
    let until = Instant::now() + Duration::from_secs(10);
    while runtime.workspace().ui.project_open.busy {
        runtime.poll_project_open();
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!runtime.workspace().ui.project_open.open);
    assert_eq!(
        runtime
            .terminal_sessions
            .active_origin_context()
            .unwrap()
            .project_root
            .as_deref(),
        Some(created.as_path())
    );
    for (rel, bytes) in files.iter().zip(before) {
        assert_eq!(std::fs::read(created.join(rel)).unwrap(), bytes);
    }
    runtime.activate_terminal_session(&original);
    assert_eq!(runtime.terminal_sessions.active().session_id(), original);
    let until =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    runtime.terminal_sessions.terminate_all_by(until);
    while !runtime.terminal_sessions.all_sessions_closed() && Instant::now() < until {
        runtime
            .terminal_sessions
            .drain_all(&mut runtime.session.workspace_mut().ui.terminal);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(runtime.terminal_sessions.all_sessions_closed());
}
