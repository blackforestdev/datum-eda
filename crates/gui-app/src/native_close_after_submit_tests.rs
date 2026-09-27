//! Real X11 close delivery with backend-delivered production callbacks held.
use super::*;
use crate::{App, GuiArgs, global_preferences_window::GlobalPreferencesWindowSurface};
use clap::Parser;
use std::{sync::Arc, time::Duration};
use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

fn surface(app: &App, host: usize) -> Option<&GlobalPreferencesWindowSurface> {
    match host {
        0 => app.global_preferences_surface.as_ref(),
        1 => app.project_preferences_surface.as_ref(),
        _ => app.new_project_surface.as_ref(),
    }
}

fn host_window(app: &App, host: usize) -> &Arc<winit::window::Window> {
    match host {
        0 => app.global_preferences_window.as_ref(),
        1 => app.project_preferences_window.as_ref(),
        _ => app.new_project_window.as_ref(),
    }
    .unwrap()
}

fn pump_until(events: &mut EventLoop<()>, app: &mut App, done: impl Fn(&App) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !done(app) {
        assert!(
            Instant::now() < deadline,
            "native lifecycle predicate timed out"
        );
        events.pump_app_events(Some(Duration::from_millis(10)), app);
        assert_healthy(app);
    }
}

fn assert_healthy(app: &App) {
    if let Some(runtime) = &app.runtime {
        runtime
            .surface_transaction
            .queue_owner()
            .assert_healthy_for_test();
        assert!(!runtime.device_health.failed());
    }
    for window in [
        app.window.as_ref(),
        app.global_preferences_window.as_ref(),
        app.project_preferences_window.as_ref(),
        app.new_project_window.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        assert!(
            !app.frames.rendering_failed(window.id()),
            "coordinator failed during lifetime proof"
        );
    }
}

fn close_native(window: &winit::window::Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let id = match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window,
        RawWindowHandle::Xcb(handle) => u64::from(handle.window.get()),
        _ => panic!("this test requires native X11"),
    };
    let helper = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reviews/gui-performance/implementation/S5/resource-snapshots/tools");
    let mut child = std::process::Command::new("python3")
        .args(["-c", "import sys; sys.path.insert(0,sys.argv[1]); from pm045_wm_close import close_window; close_window(int(sys.argv[2]))"])
        .arg(helper).arg(id.to_string()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("native close helper timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn counts(counts: &FrameCounts) -> [u64; 5] {
    [
        counts.configure_attempts.get(),
        counts.acquire_attempts.get(),
        counts.acquired.get(),
        counts.frame_submissions.get(),
        counts.presented.get(),
    ]
}

#[test]
#[ignore = "requires unlocked X11 desktop, native GPU and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn auxiliary_close_after_own_submit_releases_host_before_completion_and_detects_capture() {
    use winit::platform::x11::EventLoopBuilderExtX11;
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
    let mut builder = EventLoop::<()>::with_user_event();
    builder.with_x11().with_any_thread(true);
    let mut events = builder.build().unwrap();
    let mut app = App::new(args, events.create_proxy());
    pump_until(&mut events, &mut app, |app| {
        app.runtime
            .as_ref()
            .is_some_and(|runtime| runtime.surface_transaction.has_presented())
    });

    for negative in [false, true] {
        for host in 0..3 {
            let runtime = app.runtime.as_mut().unwrap();
            match host {
                0 => runtime.open_global_preferences(),
                1 => runtime.open_project_preferences(),
                _ => runtime.open_new_project(),
            };
            // The first pump performs owned-window synchronization.
            events.pump_app_events(Some(Duration::ZERO), &mut app);
            pump_until(&mut events, &mut app, |app| {
                surface(app, host)
                    .is_some_and(|surface| surface.surface_transaction.has_presented())
            });
            let runtime = app.runtime.as_ref().unwrap();
            runtime
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(5)),
                })
                .unwrap();
            let owned = surface(&app, host).unwrap();
            let transaction = &owned.surface_transaction;
            let owner = transaction.queue_owner.clone();
            assert_eq!(owner.snapshot().1, owner.snapshot().2);
            let queue_host = transaction.queue_host;
            let epoch = owner.snapshot().0;
            let window_id = owned.window_id();
            let frame_counts = transaction.texture_active.clone();
            let before_presented = frame_counts.presented.get();
            let window = host_window(&app, host).clone();
            let weak = Arc::downgrade(&window);
            let hold =
                crate::gui_runtime_support::native_queue_owner::callback_hold::CallbackHold::new(
                    negative.then(|| weak.clone()),
                );
            // Every later receipt in this epoch is held, so a newer Main
            // completion cannot advance the maximum watermark past this host.
            owner.hold_callbacks_for_test(Some(hold.clone()));
            app.frames.invalidate(&window);
            drop(window);
            pump_until(&mut events, &mut app, |_| {
                frame_counts.presented.get() > before_presented
            });
            let owned = surface(&app, host).unwrap();
            let serial = owned.surface_transaction.in_flight;
            assert_eq!(owned.surface_transaction.last_present_receipt, serial);
            assert!(
                serial > owner.snapshot().2,
                "own frame completion must remain unpublished"
            );
            assert!(frame_counts.frame_submissions.get() > 0);
            close_native(host_window(&app, host));
            pump_until(&mut events, &mut app, |app| surface(app, host).is_none());
            // Identical oracle in positive and deliberate strong-capture cases.
            assert!(!app.frames.contains_host(window_id));
            assert_healthy(&app);
            assert_eq!(owner.snapshot().0, epoch);
            let host_released = weak.upgrade().is_none();
            assert_eq!(host_released, !negative, "closed host retention oracle");
            let closed_counts = counts(&frame_counts);
            let pending = owner.attachment_snapshot_for_test();
            assert!(
                pending["allocations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|allocation| allocation["host"].as_u64() == Some(queue_host)
                        && allocation["last_submission"].as_u64() == Some(serial)
                        && allocation["release_reason"].as_str() == Some("host_closed"))
            );
            app.runtime
                .as_ref()
                .unwrap()
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(5)),
                })
                .unwrap();
            assert!(owner.snapshot().2 < serial);
            owner.hold_callbacks_for_test(None);
            let released = hold.release();
            assert!(
                released > 0,
                "release only actual backend-delivered callbacks"
            );
            assert!(
                weak.upgrade().is_none(),
                "negative capture must release too"
            );
            assert!(owner.snapshot().2 >= serial);
            let deadline = Instant::now() + Duration::from_millis(250);
            while Instant::now() < deadline {
                events.pump_app_events(Some(Duration::from_millis(10)), &mut app);
                assert_healthy(&app);
                assert_eq!(
                    app.runtime
                        .as_ref()
                        .unwrap()
                        .surface_transaction
                        .queue_owner()
                        .snapshot()
                        .0,
                    epoch
                );
                assert!(
                    !app.frames.contains_host(window_id),
                    "closed host re-registered"
                );
                assert_eq!(counts(&frame_counts), closed_counts);
            }
            assert_eq!(
                counts(&frame_counts),
                closed_counts,
                "closed host must never redraw/reconfigure"
            );
            let retired = owner.attachment_snapshot_for_test();
            assert!(
                !retired["allocations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|allocation| allocation["host"].as_u64() == Some(queue_host))
            );
            eprintln!(
                "close-after-submit host={host} negative={negative} serial={serial} host_released_before_callback={host_released} callbacks={released} no_late_work=true retired=true"
            );
        }
    }
    close_native(app.window.as_ref().unwrap());
    pump_until(&mut events, &mut app, |app| {
        app.runtime
            .as_ref()
            .unwrap()
            .application_terminal_shutdown_complete()
    });
}
