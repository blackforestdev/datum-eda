//! Actual Runtime cache ownership across local native-dialog actions.
use crate::{GuiArgs, HitTarget, Runtime};
use clap::Parser;
use datum_gui_protocol::{NewProjectFocus, NewProjectUnitsChoice};
use std::{sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)] // Hidden fixture; production creates windows through ActiveEventLoop.
fn native_dialog_actions_preserve_main_preparation_until_close() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--board", &board, "--open-new-project"]).unwrap();
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = event_loop.create_proxy();
    let window = Arc::new(
        event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    assert!(runtime.workspace().ui.new_project.open);
    let _ = runtime.prepared_scene();
    runtime.scene_dirty = false;
    assert!(runtime.prepared_scene.is_some());

    assert!(runtime.activate_new_project_hit(&HitTarget::NewProjectDestination));
    assert_eq!(
        runtime.workspace().ui.new_project.focus,
        NewProjectFocus::Destination
    );
    assert!(
        runtime.prepared_scene.is_some(),
        "local focus must retain Main's preparation"
    );
    assert!(!runtime.scene_dirty, "local focus must not dirty Main");
    assert!(
        runtime.activate_new_project_hit(&HitTarget::NewProjectUnitsChoice(
            NewProjectUnitsChoice::Factory
        ))
    );
    assert_eq!(
        runtime.workspace().ui.new_project.units_choice,
        NewProjectUnitsChoice::Factory
    );
    assert!(
        runtime.prepared_scene.is_some(),
        "preview choice must retain Main's preparation"
    );
    assert!(!runtime.scene_dirty, "preview choice must not dirty Main");
    assert!(runtime.close_new_project());
    assert!(!runtime.workspace().ui.new_project.open);
    assert!(
        runtime.prepared_scene.is_none(),
        "close still invalidates the owner"
    );
    assert!(runtime.scene_dirty);

    runtime.open_global_preferences();
    let _ = runtime.prepared_scene();
    runtime.scene_dirty = false;
    assert_eq!(
        runtime.activate_application_overlay_hit_target(&HitTarget::GlobalPreferencesSearch),
        Some(true)
    );
    assert!(runtime.prepared_scene.is_some());
    assert!(!runtime.scene_dirty);
    assert!(runtime.activate_global_preference_control("datum.console.feedback_duration"));
    assert!(
        runtime
            .workspace()
            .ui
            .global_preferences
            .open_choice_key
            .is_some()
    );
    assert!(runtime.prepared_scene.is_some());
    assert!(!runtime.scene_dirty);
    runtime.close_global_preferences();
    assert!(runtime.prepared_scene.is_none());
    assert!(runtime.scene_dirty);

    runtime.open_project_preferences();
    assert!(runtime.workspace().ui.project_preferences.open);
    let _ = runtime.prepared_scene();
    runtime.scene_dirty = false;
    assert_eq!(
        runtime.activate_project_preferences_hit_target(&HitTarget::GlobalPreferencesSearch),
        Some(true)
    );
    assert!(runtime.prepared_scene.is_some());
    assert!(!runtime.scene_dirty);
    assert!(runtime.activate_project_preference_control("datum.units.system"));
    assert!(
        runtime
            .workspace()
            .ui
            .project_preferences
            .open_choice_key
            .is_some()
    );
    assert!(runtime.prepared_scene.is_some());
    assert!(!runtime.scene_dirty);
    runtime.close_project_preferences();
    assert!(runtime.prepared_scene.is_none());
    assert!(runtime.scene_dirty);

    runtime.begin_application_terminal_shutdown();
    let deadline = std::time::Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        runtime.poll_terminal_output();
        assert!(
            std::time::Instant::now() < deadline,
            "owned terminal shutdown"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
