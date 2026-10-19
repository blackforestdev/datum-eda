//! Shared gesture dispatch through real discovery, admission and publication.
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
fn browse(runtime: &mut Runtime, path: &Path) {
    let state = runtime.folder_browser.state.as_mut().unwrap();
    state.expanded = true;
    state.path_input = path.display().to_string();
    runtime.read_folder_navigation(path.into(), false);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !runtime.folder_browser.state.as_ref().unwrap().ready() {
        runtime.poll_folder_browser();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn manifest(runtime: &Runtime) -> usize {
    runtime
        .folder_browser
        .state
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.name == "project.json")
        .unwrap()
}
fn finish_open(runtime: &mut Runtime) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while runtime.workspace().ui.project_open.busy {
        runtime.poll_project_open();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn manifest_activation_resets_cross_scan_clicks_and_preserves_session_on_refusal() {
    let root = Scratch(std::env::temp_dir().join(format!(
        "datum-manifest-{}",
        eda_engine::preferences::new_product_id()
    )));
    let valid = root.0.join("valid");
    let invalid = root.0.join("invalid");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let files = [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ];
    for rel in files {
        let path = valid.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(rel), path).unwrap();
    }
    std::fs::create_dir_all(&invalid).unwrap();
    std::fs::write(invalid.join("project.json"), b"invalid native manifest").unwrap();
    let before: Vec<_> = files
        .iter()
        .map(|p| std::fs::read(valid.join(p)).unwrap())
        .collect();
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
    runtime.open_project();
    browse(&mut runtime, &valid);
    let row = manifest(&runtime);
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(!runtime.workspace().ui.project_open.busy);
    assert!(runtime.folder_browser.last_click.is_some());
    browse(&mut runtime, &valid);
    assert!(
        runtime.folder_browser.last_click.is_none(),
        "scan invalidates click pairing"
    );
    let row = manifest(&runtime);
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(
        !runtime.workspace().ui.project_open.busy,
        "first new-list click is passive"
    );
    // A genuine retained worker receipt blocks Open. Defensively preserve its
    // refusal even if a dispatch also changes the row selection.
    use crate::host_filesystem::{Command, Identity, Request, Service};
    let mut service = Service::new(|| {}).unwrap();
    service
        .submit(Request {
            scope: runtime.folder_browser.scope,
            parent: valid.clone(),
            parent_identity: Identity::of(&std::fs::metadata(&valid).unwrap()),
            command: Command::CreateDirectory {
                leaf: "invalid/leaf".into(),
            },
        })
        .unwrap();
    runtime.folder_browser.host_service = Some(service);
    runtime.folder_browser.state.as_mut().unwrap().selected = None;
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(!runtime.workspace().ui.project_open.busy);
    assert!(
        runtime
            .workspace()
            .ui
            .project_open
            .refusal
            .as_deref()
            .unwrap()
            .contains("submitted folder action")
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while runtime
        .folder_browser
        .host_service
        .as_mut()
        .unwrap()
        .receive()
        .is_none()
    {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!valid.join("invalid").exists());
    runtime.folder_browser.host_service = None;
    let folder = runtime
        .folder_browser
        .state
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.name == "board")
        .unwrap();
    runtime.folder_browser.state.as_mut().unwrap().selected = Some(folder);
    runtime.folder_browser.last_click = Some((folder, Instant::now()));
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(folder));
    assert!(
        runtime.workspace().ui.project_open.refusal.is_none(),
        "ordinary folder navigation clears obsolete Project refusal"
    );
    browse(&mut runtime, &valid);
    let row = manifest(&runtime);
    runtime.set_folder_navigation_focus(Focus::Path);
    assert!(runtime.folder_browser.last_click.is_none());
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(!runtime.workspace().ui.project_open.busy);
    runtime.open_folder_actions_menu(Some(row), (200.0, 240.0));
    assert!(runtime.folder_browser.last_click.is_none());
    runtime.folder_browser.state.as_mut().unwrap().menu = None;
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(!runtime.workspace().ui.project_open.busy);
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowserRow(row));
    assert!(runtime.workspace().ui.project_open.busy);
    assert_eq!(
        runtime.workspace().ui.project_open.directory,
        valid.display().to_string()
    );
    finish_open(&mut runtime);
    assert!(!runtime.workspace().ui.project_open.open);
    let selected = runtime.terminal_sessions.active().session_id().to_owned();
    assert_ne!(selected, original);
    assert_eq!(
        runtime
            .terminal_sessions
            .active_origin_context()
            .unwrap()
            .project_root
            .as_deref(),
        Some(valid.as_path())
    );
    for (rel, bytes) in files.iter().zip(before) {
        assert_eq!(std::fs::read(valid.join(rel)).unwrap(), bytes);
    }
    runtime.open_project();
    browse(&mut runtime, &invalid);
    runtime.folder_browser.state.as_mut().unwrap().selected = Some(manifest(&runtime));
    // The same shared dispatch is used by list Enter, without a pointer pair.
    runtime.activate_folder_browser_focus(Focus::Folders);
    assert!(runtime.workspace().ui.project_open.busy);
    finish_open(&mut runtime);
    assert!(runtime.workspace().ui.project_open.open);
    assert!(runtime.workspace().ui.project_open.refusal.is_some());
    assert_eq!(runtime.terminal_sessions.active().session_id(), selected);
    assert_eq!(
        std::fs::read(invalid.join("project.json")).unwrap(),
        b"invalid native manifest"
    );
    runtime.close_project_open();
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
