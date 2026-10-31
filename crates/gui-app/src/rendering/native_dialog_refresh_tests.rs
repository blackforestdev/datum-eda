//! Actual Runtime cache ownership across local native-dialog actions.
use crate::{GuiArgs, HitTarget, Runtime};
use clap::Parser;
use datum_gui_protocol::NewProjectFocus;
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
    let clean_revision = runtime.renderer.render_session().content_revision();
    assert!(runtime.renderer.render_session().prepared().is_some());

    assert!(
        runtime.activate_folder_browser_hit(&HitTarget::FolderBrowser(
            datum_gui_protocol::FolderBrowserFocus::Path
        ))
    );
    assert_eq!(
        runtime.workspace().ui.new_project.focus,
        NewProjectFocus::Navigation(datum_gui_protocol::FolderBrowserFocus::Path)
    );
    assert!(
        runtime.renderer.render_session().prepared().is_some(),
        "local focus must retain Main's preparation"
    );
    assert!(
        runtime.renderer.render_session().content_revision() == clean_revision,
        "local focus must not dirty Main"
    );
    assert!(runtime.activate_new_project_hit(&HitTarget::NewProjectUseName));
    assert!(!runtime.workspace().ui.new_project.use_project_name);
    assert!(runtime.renderer.render_session().prepared().is_some());
    assert_eq!(
        runtime.renderer.render_session().content_revision(),
        clean_revision
    );
    assert!(runtime.close_new_project());
    assert!(!runtime.workspace().ui.new_project.open);
    assert!(
        runtime.renderer.render_session().prepared().is_none(),
        "close still invalidates the owner"
    );
    assert!(runtime.renderer.render_session().content_revision() != clean_revision);

    runtime.open_global_preferences();
    let _ = runtime.prepared_scene();
    let clean_revision = runtime.renderer.render_session().content_revision();
    assert_eq!(
        runtime.activate_application_overlay_hit_target(&HitTarget::GlobalPreferencesSearch),
        Some(true)
    );
    assert!(runtime.renderer.render_session().prepared().is_some());
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    runtime
        .session
        .workspace_mut()
        .ui
        .global_preferences
        .explanation_key = Some("datum.console.feedback_duration".to_owned());
    assert!(runtime.activate_global_preference_control("datum.console.feedback_duration"));
    assert!(
        runtime
            .workspace()
            .ui
            .global_preferences
            .open_choice_key
            .is_some()
    );
    assert!(
        runtime
            .workspace()
            .ui
            .global_preferences
            .explanation_key
            .is_none()
    );
    let opened = runtime.workspace().ui.global_preferences.clone();
    assert!(!runtime.activate_global_preference_control("missing.preference"));
    assert_eq!(runtime.workspace().ui.global_preferences, opened);
    assert!(runtime.activate_global_preference_control("datum.console.feedback_duration"));
    assert!(
        runtime
            .workspace()
            .ui
            .global_preferences
            .open_choice_key
            .is_none()
    );
    assert!(runtime.renderer.render_session().prepared().is_some());
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    assert!(runtime.choose_global_preference_value("datum.units.system", "imperial"));
    assert!(
        runtime.renderer.render_session().prepared().is_some(),
        "future-project Units must retain Main preparation"
    );
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    assert!(runtime.reset_global_preference("datum.units.system"));
    assert!(
        runtime.renderer.render_session().prepared().is_some(),
        "future-project Units reset must retain Main preparation"
    );
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    assert!(
        runtime.activate_global_preference_control("datum.accessibility.high_contrast_noncolor")
    );
    assert!(
        runtime
            .workspace()
            .ui
            .global_preferences
            .high_contrast_noncolor
    );
    assert!(
        runtime.renderer.render_session().prepared().is_none(),
        "current paint change must invalidate Main"
    );
    assert!(runtime.renderer.render_session().content_revision() != clean_revision);
    runtime.close_global_preferences();
    assert!(runtime.renderer.render_session().prepared().is_none());
    assert!(runtime.renderer.render_session().content_revision() != clean_revision);

    runtime.open_project_preferences();
    assert!(runtime.workspace().ui.project_preferences.open);
    let _ = runtime.prepared_scene();
    let clean_revision = runtime.renderer.render_session().content_revision();
    assert_eq!(
        runtime.activate_project_preferences_hit_target(&HitTarget::GlobalPreferencesSearch),
        Some(true)
    );
    assert!(runtime.renderer.render_session().prepared().is_some());
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    runtime
        .session
        .workspace_mut()
        .ui
        .project_preferences
        .explanation_key = Some("datum.units.system".to_owned());
    assert!(runtime.activate_project_preference_control("datum.units.system"));
    assert!(
        runtime
            .workspace()
            .ui
            .project_preferences
            .open_choice_key
            .is_some()
    );
    assert!(
        runtime
            .workspace()
            .ui
            .project_preferences
            .explanation_key
            .is_none()
    );
    let opened = runtime.workspace().ui.project_preferences.clone();
    assert!(!runtime.activate_project_preference_control("missing.preference"));
    assert_eq!(runtime.workspace().ui.project_preferences, opened);
    assert!(runtime.activate_project_preference_control("datum.units.system"));
    assert!(
        runtime
            .workspace()
            .ui
            .project_preferences
            .open_choice_key
            .is_none()
    );
    assert!(runtime.renderer.render_session().prepared().is_some());
    assert!(runtime.renderer.render_session().content_revision() == clean_revision);
    runtime.close_project_preferences();
    assert!(runtime.renderer.render_session().prepared().is_none());
    assert!(runtime.renderer.render_session().content_revision() != clean_revision);

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

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)] // Hidden fixture; production creates windows through ActiveEventLoop.
fn new_project_modal_announces_open_and_use_name_mode_change() {
    // Regression for dat-7ynp: the New Project modal must announce to the accessibility channel
    // (previously silent). Open WITHOUT --open-new-project so open_new_project() does the opening and
    // fires the announcement (the launch-arg path would not exercise it).
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
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
    assert!(!runtime.workspace().ui.new_project.open);

    // Opening the modal announces it (polite). The announce is the last thing open_new_project does, so
    // it is the last recorded announcement.
    assert!(runtime.open_new_project());
    assert!(runtime.workspace().ui.new_project.open);
    let open = runtime
        .terminal_accessibility
        .last_announcement()
        .expect("modal open announced");
    assert_eq!(
        open.text,
        "New Project opened. Enter a project name and location, then Create."
    );
    assert_eq!(
        open.priority,
        crate::console_accessibility::AnnouncementPriority::Medium
    );

    // Toggling "use project name" changes the mode AND announces it (the exact mode string is proven
    // end-to-end by the accessibility-interaction/new-project-modes/gui row; here the regression is
    // that the toggle fires an accessibility announcement at all, which the modal never did before).
    let before = runtime.workspace().ui.new_project.use_project_name;
    assert!(runtime.activate_new_project_hit(&HitTarget::NewProjectUseName));
    assert_ne!(runtime.workspace().ui.new_project.use_project_name, before);
    assert!(
        runtime.terminal_accessibility.last_announcement().is_some(),
        "the use-project-name mode change announces to the accessibility channel"
    );
}
