//! Native surfaces exercising the same pointer dispatcher as all three hosts.
use super::*;
use clap::Parser;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::platform::x11::EventLoopBuilderExtX11;

fn host(index: usize) -> OwnedHost {
    match index {
        0 => OwnedHost::Global,
        1 => OwnedHost::Project,
        _ => OwnedHost::New,
    }
}

fn surface(app: &App, index: usize) -> &GlobalPreferencesWindowSurface {
    match index {
        0 => app.global_preferences_surface.as_ref().unwrap(),
        1 => app.project_preferences_surface.as_ref().unwrap(),
        _ => app.new_project_surface.as_ref().unwrap(),
    }
}

fn primary(state: ElementState) -> WindowEvent {
    WindowEvent::MouseInput {
        device_id: winit::event::DeviceId::dummy(),
        state,
        button: MouseButton::Left,
    }
}

fn motion(x: f32, y: f32) -> WindowEvent {
    WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position: winit::dpi::PhysicalPosition::new(x as f64, y as f64),
    }
}

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_dialog_scroll_release_cannot_click_through_to_controls() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
    let events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
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
    app.window = Some(window);
    app.runtime = Some(runtime);
    let mut failures = Vec::new();
    for (scale, supported_minimum) in [(1.0, false), (1.0, true), (1.5, true), (2.0, true)] {
        for index in 0..3 {
            let runtime = app.runtime.as_mut().unwrap();
            let target = match index {
                0 => {
                    runtime.close_global_preferences();
                    runtime.open_global_preferences();
                    runtime.activate_application_overlay_hit_target(
                        &HitTarget::GlobalPreferencesSection("units".into()),
                    );
                    HitTarget::GlobalPreferencesControl("datum.units.system".into())
                }
                1 => {
                    runtime.close_project_preferences();
                    runtime.open_project_preferences();
                    HitTarget::GlobalPreferencesControl("datum.units.system".into())
                }
                _ => {
                    runtime.close_new_project();
                    runtime.open_new_project();
                    runtime.activate_new_project_hit(&HitTarget::NewProjectDestination);
                    HitTarget::NewProjectName
                }
            };
            let logical = if !supported_minimum {
                (960.0, 480.0)
            } else if index == 2 {
                (620.0, 620.0)
            } else {
                (700.0, 540.0)
            };
            let physical = winit::dpi::PhysicalSize::new(
                (logical.0 * scale) as u32,
                (logical.1 * scale) as u32,
            );
            let window = Arc::new(
                events
                    .create_window(
                        Window::default_attributes()
                            .with_visible(false)
                            .with_inner_size(physical),
                    )
                    .unwrap(),
            );
            let mut owned =
                GlobalPreferencesWindowSurface::new(runtime, window.clone(), Some(scale)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !owned.render(runtime, index == 1, index == 2).unwrap() {
                assert!(Instant::now() < deadline, "native presentation timeout");
                runtime.device.poll(wgpu::PollType::Poll).unwrap();
                std::thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(window.inner_size(), physical);
            let thumb = owned.scroll.thumb();
            if !supported_minimum {
                assert!(thumb.is_some(), "overflow fixture");
            }
            let rect = owned
                .presented_hits
                .regions()
                .iter()
                .find(|hit| hit.target == target)
                .expect("visible target")
                .rect;
            app.frames.register(
                window.id(),
                owned.measurements.epoch(),
                window.inner_size(),
                owned.surface_transaction.recovery.clone(),
            );
            match index {
                0 => {
                    app.global_preferences_window = Some(window);
                    app.global_preferences_surface = Some(owned);
                }
                1 => {
                    app.project_preferences_window = Some(window);
                    app.project_preferences_surface = Some(owned);
                }
                _ => {
                    app.new_project_window = Some(window);
                    app.new_project_surface = Some(owned);
                }
            }
            let focused = |app: &App| {
                let ui = &app.runtime.as_ref().unwrap().workspace().ui;
                match index {
                    0 => {
                        ui.global_preferences.open_choice_key.as_deref()
                            == Some("datum.units.system")
                    }
                    1 => {
                        ui.project_preferences.open_choice_key.as_deref()
                            == Some("datum.units.system")
                    }
                    _ => ui.new_project.focus == datum_gui_protocol::NewProjectFocus::ProjectName,
                }
            };
            assert!(!focused(&app), "target must start unfocused");
            let point = motion(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            if let Some(thumb) = thumb {
                assert!(app.handle_owned_dialog_pointer(
                    host(index),
                    &motion((thumb.x + 1.0) * scale, (thumb.y + 4.0) * scale)
                ));
                assert!(
                    app.handle_owned_dialog_pointer(host(index), &primary(ElementState::Pressed))
                );
                assert!(surface(&app, index).scrollbar_pressed);
                app.handle_owned_dialog_pointer(host(index), &point);
                app.handle_owned_dialog_pointer(host(index), &primary(ElementState::Released));
                if focused(&app) {
                    failures.push((index, scale, supported_minimum));
                }
                assert!(!surface(&app, index).scrollbar_pressed);
            } else {
                app.handle_owned_dialog_pointer(host(index), &point);
            }
            // A fresh ordinary click must still activate the same presented target.
            let was_open = focused(&app);
            app.handle_owned_dialog_pointer(host(index), &primary(ElementState::Pressed));
            app.handle_owned_dialog_pointer(host(index), &primary(ElementState::Released));
            assert!(
                (if index == 2 {
                    focused(&app)
                } else {
                    focused(&app) != was_open
                }),
                "ordinary click must reach control for host {index}"
            );
            assert!(!app.handle_owned_dialog_pointer(host(index), &WindowEvent::Focused(true)));
            eprintln!(
                "host={index} scale={scale} supported_minimum={supported_minimum} physical={physical:?} scrollbar={} release_suppressed={:?} fresh_control_click=true",
                thumb.is_some(),
                thumb.map(|_| !failures.contains(&(index, scale, supported_minimum)))
            );
            let id = surface(&app, index).window_id();
            app.frames.close(id);
            match index {
                0 => {
                    app.global_preferences_surface = None;
                    app.global_preferences_window = None;
                }
                1 => {
                    app.project_preferences_surface = None;
                    app.project_preferences_window = None;
                }
                _ => {
                    app.new_project_surface = None;
                    app.new_project_window = None;
                }
            }
        }
    }
    let runtime = app.runtime.as_mut().unwrap();
    runtime.begin_application_terminal_shutdown();
    let deadline = Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        runtime.poll_terminal_output();
        assert!(Instant::now() < deadline, "owned terminal shutdown");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        failures.is_empty(),
        "scrollbar release clicked through for hosts {failures:?}"
    );
}
