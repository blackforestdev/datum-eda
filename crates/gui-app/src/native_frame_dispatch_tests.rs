//! Shared scheduler dispatch order and eligibility regressions.
use super::*;

#[test]
fn delivered_tokens_have_rotating_bounded_service_independent_of_native_order() {
    let mut frames = NativeFrameCoordinator::default();
    let ids: Vec<_> = (1..=4).map(WindowId::from).collect();
    for id in &ids {
        frames.register(*id, 1, PhysicalSize::new(1280, 800), Default::default());
    }
    for round in 0..8 {
        for id in ids.iter().rev() {
            for _ in 0..20 {
                frames.redraw_received(*id);
            }
            // Later input before dispatch belongs to the current frame.
            assert!(!frames.invalidate_id(*id));
        }
        let actual = frames.ready_round(Instant::now());
        let mut expected = ids.clone();
        expected.rotate_left(round % 4);
        assert_eq!(actual, expected);
        for id in actual {
            assert!(!frames.invalidate_id(id));
            let receipt = frames.begin_frame(id).unwrap();
            assert_eq!(receipt.damage, frames.hosts[&id].damage);
            assert!(!frames.finish(receipt, true));
        }
        assert!(frames.ready_round(Instant::now()).is_empty());
    }
}

#[test]
fn idle_hosts_do_not_bias_first_service_between_active_hosts() {
    let mut frames = NativeFrameCoordinator::default();
    let ids: Vec<_> = (1..=4).map(WindowId::from).collect();
    for id in &ids {
        frames.register(*id, 1, PhysicalSize::new(1280, 800), Default::default());
    }
    let active = [ids[0], ids[3]];
    for round in 0..8 {
        for id in active.iter().rev() {
            frames.redraw_received(*id);
        }
        let actual = frames.ready_round(Instant::now());
        assert_eq!(actual, vec![active[round % 2], active[(round + 1) % 2]]);
        for id in actual {
            let receipt = frames.begin_frame(id).unwrap();
            assert!(!frames.finish(receipt, true));
        }
    }
}

#[test]
fn ready_round_skips_waiting_hidden_closed_and_replaced_tokens() {
    use crate::gui_runtime_support::native_recovery::RetryReason;
    let mut frames = NativeFrameCoordinator::default();
    for raw in 1..=5 {
        let id = WindowId::from(raw);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        frames.redraw_received(id);
    }
    let now = Instant::now();
    let waiting = WindowId::from(1);
    frames.hosts[&waiting]
        .recovery
        .borrow_mut()
        .defer(RetryReason::Queue, now);
    frames.window_event(WindowId::from(2), &WindowEvent::Occluded(true));
    frames.close(WindowId::from(3));
    frames.rebind_device(WindowId::from(4), 2, Default::default());
    assert_eq!(frames.ready_round(now), vec![WindowId::from(5)]);
    assert!(frames.ready_round(now).is_empty());
    assert_ne!(
        frames.hosts[&waiting].damage,
        frames.hosts[&waiting].presented
    );
    assert!(!frames.hosts[&waiting].pending);
    assert_eq!(frames.order.len(), 4);
    // Expiry without a new native token cannot manufacture a frame.
    assert!(
        frames
            .ready_round(now + std::time::Duration::from_millis(3))
            .is_empty()
    );
    frames.redraw_received(waiting);
    assert_eq!(
        frames.ready_round(now + std::time::Duration::from_millis(3)),
        vec![waiting]
    );
}
