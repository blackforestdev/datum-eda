//! Native surfaces exercising the same pointer dispatcher as all three hosts.
use super::*;
use clap::Parser;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::platform::pump_events::EventLoopExtPumpEvents;

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

// Compare every delivered delta with an independent logical-offset oracle.
fn verify_wheel_offsets(
    events: &mut winit::event_loop::EventLoop<()>,
    app: &mut App,
    index: usize,
    scale: f32,
) {
    let owned = surface(app, index);
    let id = owned.window_id();
    let original = owned.scroll.offset();
    let viewport = owned.scroll.viewport;
    let maximum = owned.scroll.maximum();
    deliver(
        events,
        app,
        id,
        motion(
            (viewport.x + viewport.width * 0.5) * scale,
            (viewport.y + viewport.height * 0.5) * scale,
        ),
    );
    let mut wheel = |delta, expected: f32| {
        deliver(
            events,
            app,
            id,
            WindowEvent::MouseWheel {
                device_id: winit::event::DeviceId::dummy(),
                delta,
                phase: winit::event::TouchPhase::Moved,
            },
        );
        let actual = surface(app, index).scroll.offset();
        assert!(
            (actual - expected).abs() < 0.0001,
            "host={index} scale={scale} wheel={delta:?}: expected={expected}, actual={actual}"
        );
    };
    let pixel = |y| MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, y));
    for pixels in [true, false] {
        wheel(pixel(4096.0), 0.0);
        let mut expected = 0.0;
        let step = if pixels { 0.25 / scale } else { 10.0 };
        for direction in [-1.0, 1.0] {
            for _ in 0..16 {
                expected = (expected - direction * step).clamp(0.0, maximum);
                let delta = if pixels {
                    pixel(f64::from(direction) * 0.25)
                } else {
                    MouseScrollDelta::LineDelta(0.0, direction * 0.25)
                };
                wheel(delta, expected);
            }
        }
        wheel(pixel(-4096.0), maximum);
        for direction in [-1.0, -1.0, 1.0] {
            let delta = if pixels {
                pixel(f64::from(direction) * 0.25)
            } else {
                MouseScrollDelta::LineDelta(0.0, direction * 0.25)
            };
            wheel(
                delta,
                if direction < 0.0 {
                    maximum
                } else {
                    (maximum - step).max(0.0)
                },
            );
        }
        wheel(pixel(4096.0), 0.0);
        wheel(
            MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(40.0, 0.0)),
            0.0,
        );
    }
    wheel(pixel(-f64::from(original * scale)), original);
    eprintln!(
        "host={index} scale={scale} native_entry_fractional_pixels_lines_boundaries=true maximum={maximum}"
    );
}

// Supply events to the production native entry point with a real active loop.
// This intentionally does not claim that the OS generated or delivered them.
#[allow(deprecated)]
fn deliver(
    events: &mut winit::event_loop::EventLoop<()>,
    app: &mut App,
    id: WindowId,
    event: WindowEvent,
) {
    let mut pending = Some(event);
    events.pump_events(Some(Duration::ZERO), |_, active| {
        if let Some(event) = pending.take() {
            app.handle_native_window_event(active, id, event);
        }
    });
    assert!(pending.is_none(), "native entry point was exercised");
}

#[test]
#[ignore = "requires native Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_dialog_scroll_release_cannot_click_through_to_controls() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
    let backend = std::env::var("DATUM_NATIVE_TEST_BACKEND").unwrap_or_else(|_| "x11".into());
    let mut builder = winit::event_loop::EventLoop::<()>::with_user_event();
    match backend.as_str() {
        "x11" => {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_x11().with_any_thread(true);
        }
        "wayland" => {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            builder.with_wayland().with_any_thread(true);
        }
        _ => panic!("unsupported native test backend: {backend}"),
    }
    let mut events = builder.build().unwrap();
    eprintln!("Native dialog input backend: {backend}; events supplied to production entry");
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
            let requested_physical = physical;
            let physical = window.inner_size();
            assert!(physical.width > 0 && physical.height > 0);
            assert_eq!(
                (owned.config.width, owned.config.height),
                (physical.width, physical.height)
            );
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
            verify_wheel_offsets(&mut events, &mut app, index, scale);
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
            if let Some(thumb) = surface(&app, index).scroll.thumb() {
                let id = surface(&app, index).window_id();
                let x = (thumb.x + 1.0) * scale;
                let y = (thumb.y + 4.0) * scale;
                deliver(&mut events, &mut app, id, motion(x, y));
                deliver(&mut events, &mut app, id, primary(ElementState::Pressed));
                assert!(surface(&app, index).scrollbar_pressed);
                let before = surface(&app, index).scroll.offset();
                deliver(&mut events, &mut app, id, motion(x, y + 20.0 * scale));
                let dragged = surface(&app, index).scroll.offset();
                assert_ne!(dragged, before, "establish an active drag for host {index}");
                let was_open = focused(&app);
                deliver(&mut events, &mut app, id, WindowEvent::Focused(false));
                assert!(
                    !surface(&app, index).scrollbar_pressed,
                    "focus loss must clear capture ownership for host {index}"
                );
                deliver(
                    &mut events,
                    &mut app,
                    id,
                    motion(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5),
                );
                assert_eq!(
                    surface(&app, index).scroll.offset(),
                    dragged,
                    "focus loss must cancel the grab for host {index}"
                );
                deliver(&mut events, &mut app, id, primary(ElementState::Released));
                assert_eq!(
                    focused(&app),
                    was_open,
                    "cancelled release must not activate the target for host {index}"
                );
            }
            // A fresh ordinary click must still activate the same presented target.
            let was_open = focused(&app);
            let id = surface(&app, index).window_id();
            deliver(&mut events, &mut app, id, primary(ElementState::Pressed));
            deliver(&mut events, &mut app, id, primary(ElementState::Released));
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
                "host={index} scale={scale} requested_supported_minimum={supported_minimum} requested={requested_physical:?} accepted={physical:?} scrollbar={} release_suppressed={:?} fresh_control_click=true",
                thumb.is_some(),
                thumb.map(|_| !failures.contains(&(index, scale, supported_minimum)))
            );
            let id = surface(&app, index).window_id();
            if let Some(thumb) = surface(&app, index).scroll.thumb() {
                deliver(
                    &mut events,
                    &mut app,
                    id,
                    motion((thumb.x + 1.0) * scale, (thumb.y + 4.0) * scale),
                );
                deliver(&mut events, &mut app, id, primary(ElementState::Pressed));
                assert!(
                    surface(&app, index).scrollbar_pressed,
                    "close must exercise active capture for host {index}"
                );
            }
            deliver(&mut events, &mut app, id, WindowEvent::CloseRequested);
            let (surface_closed, window_closed, model_closed) = match index {
                0 => (
                    app.global_preferences_surface.is_none(),
                    app.global_preferences_window.is_none(),
                    !app.runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .ui
                        .global_preferences
                        .open,
                ),
                1 => (
                    app.project_preferences_surface.is_none(),
                    app.project_preferences_window.is_none(),
                    !app.runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .ui
                        .project_preferences
                        .open,
                ),
                _ => (
                    app.new_project_surface.is_none(),
                    app.new_project_window.is_none(),
                    !app.runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .ui
                        .new_project
                        .open,
                ),
            };
            assert!(
                surface_closed && window_closed && model_closed,
                "native close must retire host {index} and its model state"
            );
            eprintln!(
                "host={index} scale={scale} native_entry_focus_cancel={:?} native_entry_close=true",
                thumb.map(|_| true)
            );
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
