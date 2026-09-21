use super::NativeFrameCoordinator;
use std::time::{Duration, Instant};
use winit::event_loop::ControlFlow;

#[test]
fn mixed_deadlines_cannot_postpone_recovery_or_survive_cancellation() {
    let now = Instant::now();
    let surface_retry = now + Duration::from_millis(16);
    let terminal_refresh = now + Duration::from_millis(500);
    let device_progress = now + Duration::from_millis(2);
    let shutdown = now + Duration::from_secs(6);
    let mut coordinator = NativeFrameCoordinator::default();
    assert_eq!(coordinator.take_control_flow(), ControlFlow::Wait);
    for deadlines in [
        [surface_retry, terminal_refresh, device_progress, shutdown],
        [shutdown, device_progress, terminal_refresh, surface_retry],
    ] {
        for deadline in deadlines {
            coordinator.wake_at(Some(deadline));
        }
        coordinator.wake_at(None); // An idle host cannot erase another timer.
        assert_eq!(
            coordinator.take_control_flow(),
            ControlFlow::WaitUntil(device_progress)
        );
        // The serviced/canceled producers contribute no work in the next round.
        assert_eq!(coordinator.take_control_flow(), ControlFlow::Wait);
    }
    coordinator.wake_at(Some(terminal_refresh));
    assert_eq!(
        coordinator.take_control_flow(),
        ControlFlow::WaitUntil(terminal_refresh)
    );
}

#[test]
fn native_extent_bursts_keep_latest_size_and_one_restore_per_host() {
    use winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowId};
    let mut coordinator = NativeFrameCoordinator::default();
    for id in (1..=4).map(WindowId::from) {
        coordinator.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        let initial = coordinator.begin_frame(id).unwrap();
        assert!(!coordinator.finish(initial, true));
        let initial_damage = coordinator.hosts[&id].damage;
        for _ in 0..20 {
            coordinator.window_event(id, &WindowEvent::Resized(PhysicalSize::new(1280, 800)));
        }
        assert_eq!(coordinator.hosts[&id].damage, initial_damage);
        assert!(!coordinator.take_restore_request(id));
        for width in [1400, 1400, 1500, 1500, 1440, 1440] {
            coordinator.window_event(id, &WindowEvent::Resized(PhysicalSize::new(width, 900)));
        }
        assert_eq!(coordinator.hosts[&id].extent, PhysicalSize::new(1440, 900));
        assert_eq!(coordinator.hosts[&id].damage, initial_damage + 3);
        assert!(coordinator.take_restore_request(id));
        assert!(!coordinator.take_restore_request(id));
        let final_frame = coordinator.begin_frame(id).unwrap();
        assert!(!coordinator.finish(final_frame, true));
        assert_eq!(
            coordinator.hosts[&id].presented,
            coordinator.hosts[&id].damage
        );
    }
}
