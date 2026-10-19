//! Real Runtime/controller/worker controls; desktop fixture is explicitly owned.
use super::*;
use clap::Parser;
use datum_gui_protocol::{BrowserAction, NamingDraft, NamingPurpose};
use eda_engine::preferences::new_product_id;
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
fn wait(runtime: &mut Runtime, label: &str, done: impl Fn(&Runtime) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !done(runtime) {
        runtime.poll_folder_browser();
        assert!(Instant::now() < deadline, "{label}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn browse(runtime: &mut Runtime, path: &Path) {
    let state = runtime.folder_browser.state.as_mut().unwrap();
    state.expanded = true;
    state.path_input = path.display().to_string();
    runtime.read_folder_navigation(path.into(), false);
    wait(runtime, "actual directory discovery", |r| {
        r.folder_browser.state.as_ref().unwrap().ready()
    });
}
fn index(runtime: &Runtime, name: &str) -> usize {
    runtime
        .folder_browser
        .state
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.name == name)
        .unwrap()
}
fn qualified(runtime: &mut Runtime) {
    wait(runtime, "actual read-only qualification", |r| {
        r.folder_browser
            .qualification
            .as_ref()
            .is_some_and(|q| q.result_ready())
    });
}
#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn folder_actions_controller_binds_context_refuses_stale_probe_and_retains_mutation_across_close() {
    let args = GuiArgs::try_parse_from(["datum-gui", "--open-new-project"]).unwrap();
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
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let scratch = Scratch(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qa")
            .join(format!("folder-action-controller-{}", new_product_id())),
    );
    std::fs::create_dir_all(&scratch.0).unwrap();
    let root = scratch.0.canonicalize().unwrap();
    for name in ["A.txt", "B.txt"] {
        std::fs::write(root.join(name), name.as_bytes()).unwrap();
    }
    std::fs::create_dir_all(root.join("container/nested")).unwrap();
    std::fs::write(
        root.join("container/nested/project.json"),
        b"readonly classification marker",
    )
    .unwrap();
    let valid_open = root.join("valid-open-target");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    for rel in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ] {
        let path = valid_open.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(rel), path).unwrap();
    }
    browse(&mut runtime, &root);
    let a = index(&runtime, "A.txt");
    let b = index(&runtime, "B.txt");
    runtime.open_folder_actions_menu(Some(a), (200.0, 240.0));
    // Change the exact row before polling the old real worker answer. It must
    // never enable an action on B using A's completed inspection.
    {
        let state = runtime.folder_browser.state.as_mut().unwrap();
        state.selected = Some(b);
        state.context_entry = Some(state.entries[b].clone());
    }
    wait(&mut runtime, "changed selection rejects old probe", |r| {
        r.folder_browser.qualification.is_none()
    });
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
            .all(|item| item.unavailable.is_some())
    );
    runtime.folder_popup_pointer(None, true);
    runtime.folder_popup_pointer(None, false);
    runtime.open_folder_actions_menu(Some(a), (200.0, 240.0));
    qualified(&mut runtime);
    runtime.folder_popup_pointer(None, true);
    runtime.folder_popup_pointer(None, false);
    runtime.folder_browser.state.as_mut().unwrap().selected = Some(b);
    runtime.start_folder_naming(BrowserAction::Rename);
    wait(
        &mut runtime,
        "F2 route names current B, not dismissed A",
        |r| r.folder_browser.state.as_ref().unwrap().naming.is_some(),
    );
    let draft = runtime
        .folder_browser
        .state
        .as_ref()
        .unwrap()
        .naming
        .as_ref()
        .unwrap();
    assert!(matches!(&draft.purpose,NamingPurpose::Rename(entry) if entry.name=="B.txt"));
    assert_eq!(draft.editor.selected_text(&draft.value), "B.txt");
    runtime.cancel_folder_naming();
    runtime.open_folder_actions_menu(None, (200.0, 240.0));
    qualified(&mut runtime);
    assert!(
        runtime
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .menu
            .as_ref()
            .unwrap()
            .items[1]
            .unavailable
            .is_some()
    );
    runtime.folder_popup_pointer(None, true);
    runtime.folder_popup_pointer(None, false);
    let container = index(&runtime, "container");
    runtime.open_folder_actions_menu(Some(container), (200.0, 240.0));
    qualified(&mut runtime);
    assert!(
        runtime
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .menu
            .as_ref()
            .unwrap()
            .items[1]
            .unavailable
            .is_some()
    );
    assert!(root.join("container/nested/project.json").is_file());
    runtime.folder_popup_pointer(None, true);
    runtime.folder_popup_pointer(None, false);

    // An actual reservation is the existing Open worker/cleanup lifetime owner,
    // independent of whether its dialog/receiver is still present.
    let reservation = runtime
        .terminal_sessions
        .reserve_project_terminal()
        .unwrap();
    runtime.close_new_project();
    runtime.open_new_project();
    browse(&mut runtime, &root);
    let parent = runtime
        .folder_browser
        .state
        .as_ref()
        .unwrap()
        .parent_identity
        .unwrap();
    let mut draft = NamingDraft::new(
        root.display().to_string(),
        parent,
        runtime.folder_browser.scope,
        None,
    );
    draft.value = "held child".into();
    runtime.folder_browser.state.as_mut().unwrap().naming = Some(draft);
    runtime.submit_folder_naming();
    assert!(!runtime.folder_host_actions_pending() && !root.join("held child").exists());
    assert!(
        runtime
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .naming
            .as_ref()
            .unwrap()
            .refusal
            .as_ref()
            .unwrap()
            .contains("preparation")
    );
    drop(reservation);
    runtime.submit_folder_naming();
    assert!(runtime.folder_host_actions_pending());
    // Do not consume the retained receipt even if the syscall already finished.
    runtime.close_new_project();
    runtime.open_new_project();
    browse_without_poll(&mut runtime, &root);
    {
        let d = &mut runtime.session.workspace_mut().ui.new_project;
        d.project_name = "must-not-publish".into();
        d.global_units_ready = true;
    }
    runtime.refresh_folder_browser();
    {
        let mut otherwise = runtime.workspace().ui.new_project.clone();
        otherwise.host_action_pending = false;
        assert!(
            otherwise.create_enabled(),
            "all ordinary Create inputs are eligible"
        );
    }
    assert!(runtime.workspace().ui.new_project.host_action_pending);
    assert!(!runtime.workspace().ui.new_project.create_enabled());
    runtime.activate_new_project_hit(&HitTarget::NewProjectCreate);
    assert!(!root.join("must-not-publish").exists());
    assert!(
        runtime
            .workspace()
            .ui
            .new_project
            .refusal
            .as_ref()
            .unwrap()
            .contains("submitted folder action")
    );
    runtime.close_new_project();
    runtime.open_project();
    browse_without_poll(&mut runtime, &valid_open);
    {
        let mut otherwise = runtime.workspace().ui.project_open.clone();
        otherwise.host_action_pending = false;
        assert!(
            otherwise.open_enabled(),
            "valid native Project is otherwise eligible"
        );
    }
    assert!(runtime.workspace().ui.project_open.host_action_pending);
    assert!(!runtime.workspace().ui.project_open.open_enabled());
    runtime.begin_project_open();
    assert!(!runtime.workspace().ui.project_open.busy);
    assert!(!runtime.terminal_sessions.project_preparation_pending());
    assert!(
        runtime
            .workspace()
            .ui
            .project_open
            .refusal
            .as_ref()
            .unwrap()
            .contains("submitted folder action")
    );
    wait(
        &mut runtime,
        "retained operation receipt after closed originating dialog",
        |r| !r.folder_host_actions_pending(),
    );
    assert!(root.join("held child").is_dir());
    assert!(runtime.workspace().ui.project_open.open_enabled());
    runtime.begin_project_open();
    assert!(
        runtime.workspace().ui.project_open.busy,
        "same valid Open proceeds after receipt"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while runtime.workspace().ui.project_open.open {
        runtime.poll_project_open();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    runtime.open_new_project();
    browse_without_poll(&mut runtime, &root);
    runtime.session.workspace_mut().ui.new_project.project_name = "must-not-publish".into();
    runtime.refresh_folder_browser();
    assert!(runtime.workspace().ui.new_project.create_enabled());
    runtime.activate_new_project_hit(&HitTarget::NewProjectCreate);
    assert!(
        root.join("must-not-publish/project.json").is_file(),
        "same valid Create proceeds after receipt"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while runtime.workspace().ui.project_open.open {
        runtime.poll_project_open();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(root.join("held child").is_dir());
    assert_eq!(std::fs::read(root.join("A.txt")).unwrap(), b"A.txt");
    assert_eq!(std::fs::read(root.join("B.txt")).unwrap(), b"B.txt");
}
fn browse_without_poll(runtime: &mut Runtime, path: &Path) {
    runtime.folder_browser.state.as_mut().unwrap().path_input = path.display().to_string();
    runtime.refresh_folder_browser();
}
