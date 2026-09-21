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
