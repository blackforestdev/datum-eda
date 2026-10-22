//! Supplied key phases exercise production dispatch on real Wayland hosts.
//! Ordinary owned-Open QA separately establishes OS delivery and text ownership.
use super::*;
use clap::Parser;
use datum_gui_protocol::SelectionTarget;
use std::{process::Command, sync::Arc, time::Duration};
use winit::{
    event::{DeviceId, Event},
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
};

// KeyEvent contains backend-private fields. Capture a genuine event once, then
// clone it and change only public phase/key fields; never fabricate its payload.
fn observe_template_event(event: &Event<()>, main: WindowId, focused: &mut bool) {
    if let Event::WindowEvent { window_id, event } = event {
        match event {
            WindowEvent::Focused(value) => {
                eprintln!("template native host={window_id:?} focused={value}");
                if *window_id == main {
                    *focused = *value;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                eprintln!(
                    "template native host={window_id:?} physical={:?} logical={:?} state={:?}",
                    event.physical_key, event.logical_key, event.state
                );
            }
            _ => {}
        }
    }
}

#[allow(deprecated)]
fn capture_template(events: &mut EventLoop<()>, app: &mut App) -> KeyEvent {
    let host = std::env::var("DATUM_NATIVE_TEST_INPUT_WINDOW").expect("owned Weston XID");
    let display = std::env::var("DATUM_NATIVE_TEST_INPUT_DISPLAY").expect("outer X11 display");
    assert!(host.bytes().all(|b| b.is_ascii_digit()) && !host.is_empty());
    let main = app.window.as_ref().unwrap().id();
    app.window.as_ref().unwrap().set_visible(true);
    let mut presented = false;
    let mut focused = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !presented && std::time::Instant::now() < deadline {
        events.pump_events(Some(Duration::from_millis(20)), |event, _| {
            observe_template_event(&event, main, &mut focused);
        });
        presented = app.runtime.as_mut().unwrap().render().unwrap();
    }
    assert!(presented, "native Main presented before template input");
    let run = |args: &[&str]| {
        assert!(
            Command::new("xdotool")
                .env("DISPLAY", &display)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    run(&["windowactivate", "--sync", &host]);
    // The existing StartupQA envelope setup enters/clicks the owned host. No
    // captured pointer event is supplied to Runtime and no selection exists yet.
    run(&["mousemove", "--window", &host, "600", "400", "click", "1"]);
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        events.pump_events(Some(Duration::from_millis(20)), |event, _| {
            observe_template_event(&event, main, &mut focused);
        });
        if focused || std::time::Instant::now() >= deadline {
            break;
        }
    }
    assert!(
        focused,
        "actual native Main focus acknowledged before template key"
    );
    // F6 is already mapped in the outer X11 keymap; no dynamic remap is needed.
    run(&["key", "--clearmodifiers", "F6"]);
    let mut template = None;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while template.is_none() && std::time::Instant::now() < deadline {
        events.pump_events(Some(Duration::from_millis(20)), |event, _| {
            observe_template_event(&event, main, &mut focused);
            if let Event::WindowEvent {
                window_id,
                event: WindowEvent::KeyboardInput { event, .. },
            } = event
                && window_id == main
                && focused
                && event.logical_key == Key::Named(NamedKey::F6)
            {
                template = Some(event);
            }
        });
    }
    template.expect("genuine native Wayland KeyEvent template")
}

#[allow(deprecated)]
fn deliver(
    events: &mut EventLoop<()>,
    app: &mut App,
    id: WindowId,
    template: &KeyEvent,
    key: NamedKey,
    state: ElementState,
    repeat: bool,
) {
    let mut key_event = template.clone();
    key_event.logical_key = Key::Named(key);
    key_event.physical_key = PhysicalKey::Code(if key == NamedKey::Escape {
        KeyCode::Escape
    } else {
        KeyCode::F24
    });
    key_event.text = None;
    key_event.state = state;
    key_event.repeat = repeat;
    let mut event = Some(WindowEvent::KeyboardInput {
        device_id: DeviceId::dummy(),
        event: key_event,
        is_synthetic: false,
    });
    events.pump_events(Some(Duration::ZERO), |_, active| {
        if let Some(event) = event.take() {
            app.handle_native_window_event(active, id, event);
        }
    });
    assert!(event.is_none(), "production native entry exercised");
}

#[allow(deprecated)]
fn sync_owned(events: &mut EventLoop<()>, app: &mut App) {
    let mut done = false;
    events.pump_events(Some(Duration::ZERO), |_, active| {
        if !done {
            app.sync_owned_product_windows(active).unwrap();
            done = true;
        }
    });
    assert!(done);
}

#[test]
#[ignore = "requires private Wayland/Vulkan, existing native project, pinned CLI and verified owned Weston input XID; run serially"]
#[allow(deprecated)]
fn native_owned_escape_release_survives_host_retirement_and_renderer_filtering() {
    let project =
        std::env::var("DATUM_NATIVE_TEST_PROJECT").expect("existing frozen native project");
    let args = GuiArgs::try_parse_from(["datum-gui", "--project-root", &project]).unwrap();
    let mut builder = EventLoop::<()>::with_user_event();
    builder.with_wayland().with_any_thread(true);
    let mut events = builder.build().unwrap();
    let wake = events.create_proxy();
    let window = Arc::new(
        events
            .create_window(
                Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(960, 720)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let runtime = pollster::block_on(Runtime::new(
        window.clone(),
        launch,
        Some(1.0),
        wake.clone(),
    ))
    .unwrap();
    let mut app = App::new(args, wake);
    app.frames.register(
        window.id(),
        runtime.measurements.epoch(),
        window.inner_size(),
        runtime.surface_transaction.recovery.clone(),
    );
    app.window = Some(window);
    app.runtime = Some(runtime);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let template = capture_template(&mut events, &mut app);
        let main = app.window.as_ref().unwrap().id();
        let unrelated = events
            .create_window(Window::default_attributes().with_visible(false))
            .unwrap();
        for destination in ["active", "retired", "main", "failed"] {
            let runtime = app.runtime.as_mut().unwrap();
            assert!(runtime.native_select_all_on(false));
            let prior = runtime.workspace().selection.clone();
            assert_ne!(prior, SelectionTarget::None);
            assert!(runtime.open_project());
            sync_owned(&mut events, &mut app);
            let owned = app.new_project_window.as_ref().unwrap().id();
            deliver(
                &mut events,
                &mut app,
                owned,
                &template,
                NamedKey::Escape,
                ElementState::Pressed,
                false,
            );
            assert!(
                !app.runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .ui
                    .project_open
                    .open
            );
            assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);

            // Non-Escape input on an unrelated host must not discharge the cycle.
            deliver(
                &mut events,
                &mut app,
                unrelated.id(),
                &template,
                NamedKey::F24,
                ElementState::Released,
                false,
            );
            for repeat in [false, true] {
                deliver(
                    &mut events,
                    &mut app,
                    main,
                    &template,
                    NamedKey::Escape,
                    ElementState::Pressed,
                    repeat,
                );
                assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);
            }
            if destination == "retired" {
                sync_owned(&mut events, &mut app);
                assert!(app.new_project_window.is_none());
            } else if destination == "failed" {
                app.frames
                    .render_failed(owned, &"injected test renderer failure");
                assert!(app.frames.rendering_failed(owned));
            }
            let release_host = if destination == "main" { main } else { owned };
            deliver(
                &mut events,
                &mut app,
                release_host,
                &template,
                NamedKey::Escape,
                ElementState::Released,
                false,
            );
            assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);
            sync_owned(&mut events, &mut app);

            // No intervening key phase may drain a stranded dismissal cycle. Main
            // clears on release, so test both phases of the very next fresh cycle.
            deliver(
                &mut events,
                &mut app,
                main,
                &template,
                NamedKey::Escape,
                ElementState::Pressed,
                false,
            );
            assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);
            deliver(
                &mut events,
                &mut app,
                main,
                &template,
                NamedKey::Escape,
                ElementState::Released,
                false,
            );
            assert_eq!(
                app.runtime.as_ref().unwrap().workspace().selection,
                SelectionTarget::None,
                "fresh Main Escape release after {destination} dismissal release"
            );
            // Extra nonpending releases are controls only after the decisive cycle.
            // A Main Escape release can ordinarily clear an existing selection.
            for host in [unrelated.id(), owned, main] {
                deliver(
                    &mut events,
                    &mut app,
                    host,
                    &template,
                    NamedKey::Escape,
                    ElementState::Released,
                    false,
                );
                assert_eq!(
                    app.runtime.as_ref().unwrap().workspace().selection,
                    SelectionTarget::None
                );
            }
            eprintln!(
                "native Wayland dispatch: {destination} release preserved prior selection; fresh Main Escape cleared; non-Escape/nonpending/repeat controls passed"
            );
        }

        // Global Preferences must own the same closing Escape cycle. This is
        // the Main-release case after host retirement, using the captured genuine KeyEvent
        // and actual production dispatch rather than another host matrix.
        let runtime = app.runtime.as_mut().unwrap();
        assert!(runtime.native_select_all_on(false));
        let prior = runtime.workspace().selection.clone();
        assert_ne!(prior, SelectionTarget::None);
        assert!(runtime.open_global_preferences());
        sync_owned(&mut events, &mut app);
        let owned = app.global_preferences_window.as_ref().unwrap().id();
        deliver(
            &mut events,
            &mut app,
            owned,
            &template,
            NamedKey::Escape,
            ElementState::Pressed,
            false,
        );
        assert!(
            !app.runtime
                .as_ref()
                .unwrap()
                .workspace()
                .ui
                .global_preferences
                .open
        );
        assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);
        sync_owned(&mut events, &mut app);
        assert!(app.global_preferences_window.is_none());
        deliver(
            &mut events,
            &mut app,
            main,
            &template,
            NamedKey::Escape,
            ElementState::Released,
            false,
        );
        assert_eq!(
            app.runtime.as_ref().unwrap().workspace().selection,
            prior,
            "Global Preferences closing release must preserve selection after host retirement"
        );
        deliver(
            &mut events,
            &mut app,
            main,
            &template,
            NamedKey::Escape,
            ElementState::Pressed,
            false,
        );
        assert_eq!(app.runtime.as_ref().unwrap().workspace().selection, prior);
        deliver(
            &mut events,
            &mut app,
            main,
            &template,
            NamedKey::Escape,
            ElementState::Released,
            false,
        );
        assert_eq!(
            app.runtime.as_ref().unwrap().workspace().selection,
            SelectionTarget::None,
            "the next fresh idle Main Escape must clear after the single owned release"
        );
        eprintln!(
            "native Global Preferences closing press/release preserves selection; next fresh Main cycle clears"
        );
    }));
    // Explicit test housekeeping on success and assertion failure. This is not
    // evidence of an ordinary user Close or application shutdown workflow.
    let runtime = app.runtime.as_mut().unwrap();
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
    assert!(runtime.application_terminal_shutdown_complete());
    if let Err(failure) = outcome {
        std::panic::resume_unwind(failure);
    }
}
