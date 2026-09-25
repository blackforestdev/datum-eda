use super::*;
use crate::resource_consumers::Consumer;
use crate::text_gpu::{
    allocation_host::Host,
    budget::{Budget, GpuReservation},
    lifetime::{Kind, Owner, RetirementReason},
};

#[test]
#[ignore = "exclusive process-wide GPU observer; run serially"]
fn gpu_history_preserves_shared_holds_replacement_and_loss() {
    assert!(start(0).is_err());
    assert!(start(65_537).is_err());
    let id = start(64).unwrap();
    assert!(start(64).is_err());
    assert!(drain(id + 1).is_err());
    let host = Host::new();
    let _host = host.enter_for(Consumer::Main.into());
    let owner = Owner::new();
    let observed_owner = owner.observer();
    let allocation = owner.track(vec![1_u8; 64], 64, 1, Kind::Vertex);
    let first_id = allocation.id();
    allocation.set_requested_bytes(32);
    let prepared = allocation.prepared_ref();
    let submitted = allocation.submission_ref();
    allocation.set_consumers(Consumer::Menu.into());
    allocation
        .upload_target()
        .receipt(32, 64)
        .commit(Some((owner.id(), 1)));
    allocation.retire(RetirementReason::Replaced);
    let replacement = owner.track(vec![2_u8; 64], 64, 2, Kind::Vertex);
    let second_id = replacement.id();
    assert_ne!(first_id, second_id);
    drop(allocation);
    drop(owner);
    assert_eq!(observed_owner.allocations().len(), 2);
    assert!(stop(id).is_err());
    let first = drain(id).unwrap();
    assert_eq!(first.active_allocations, 2);
    assert_eq!(first.tracked_capacity_bytes, 128);
    assert_eq!(first.tracked_capacity_peak_bytes, 128);
    let record = first
        .events
        .iter()
        .rev()
        .find(|e| e.allocation.id == first_id)
        .unwrap()
        .allocation;
    assert_eq!(record.renderer_id, Some(host.id()));
    assert_eq!(record.requested_bytes, 32);
    assert_eq!(record.prepared_references, 1);
    assert_eq!(record.submission_references, 1);
    assert_eq!(record.prepared_consumers, Consumer::Main.into());
    assert_eq!(record.submitted_consumers, Consumer::Main.into());
    assert_eq!(record.consumers, Consumer::Menu.into());
    assert_eq!(record.retirement_reason, Some(RetirementReason::Replaced));
    assert_eq!(
        (
            record.submitted_source_bytes,
            record.submitted_transfer_bytes
        ),
        (32, 64)
    );
    drop(prepared);
    assert_eq!(observed_owner.allocations().len(), 2);
    drop(submitted);
    assert_eq!(observed_owner.allocations().len(), 1);
    replacement.retire(RetirementReason::Cleared);
    drop(replacement);
    assert!(observed_owner.allocations().is_empty());
    let last = stop(id).unwrap();
    assert_eq!(last.active_allocations, 0);
    assert_eq!(last.tracked_capacity_bytes, 0);
    assert_eq!(last.tracked_capacity_peak_bytes, 128);
    assert!(!last.invalid_accounting);
    assert_eq!(last.dropped_events, 0);
    let mut events = first.events;
    events.extend(last.events);
    for (index, event) in events.iter().enumerate() {
        assert_eq!(event.sequence, index as u64 + 1);
    }
    let releases: Vec<_> = events
        .iter()
        .filter(|e| e.transition == Transition::Released)
        .collect();
    assert_eq!(releases.len(), 2);
    assert_eq!(releases[0].allocation.id, first_id);
    assert_eq!(
        releases[0].allocation.retirement_reason,
        Some(RetirementReason::Replaced)
    );
    assert_eq!(
        releases[1].allocation.retirement_reason,
        Some(RetirementReason::Cleared)
    );
    assert!(drain(id).is_err());

    let loss = start(1).unwrap();
    assert_ne!(loss, id);
    let allocation = Owner::new().track((), 4, 1, Kind::Texture);
    drop(allocation);
    let lost = drain(loss).unwrap();
    assert_eq!(lost.events.len(), 1);
    assert_eq!(lost.events[0].transition, Transition::Registered);
    assert_eq!(lost.dropped_events, 2);
    assert_eq!(lost.active_allocations, 0);
    assert_eq!(stop(loss).unwrap().dropped_events, 2);
    // A late observer cannot pretend a live allocation was born in its interval.
    let existing = Owner::new().track((), 4, 1, Kind::Texture);
    assert!(start(16).is_err());
    drop(existing);
}

#[test]
#[ignore = "exclusive process-wide GPU observer; run serially"]
fn gpu_history_serializes_concurrent_references_and_reservation_peaks() {
    let id = start(128).unwrap();
    let owner = Owner::new();
    let allocation = std::sync::Arc::new(owner.track((), 64, 1, Kind::Vertex));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(5));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let allocation = allocation.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let reference = allocation.submission_ref();
                barrier.wait();
                barrier.wait();
                drop(reference);
            })
        })
        .collect();
    barrier.wait();
    let first = drain(id).unwrap();
    assert_eq!(
        first
            .events
            .last()
            .unwrap()
            .allocation
            .submission_references,
        4
    );
    assert_eq!(first.tracked_capacity_peak_bytes, 64);
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    drop(allocation);
    let last = stop(id).unwrap();
    let counts: Vec<_> = last
        .events
        .iter()
        .filter(|e| e.transition == Transition::ReferenceReleased)
        .map(|e| e.allocation.submission_references)
        .collect();
    assert_eq!(counts, vec![3, 2, 1, 0, 0]);
    assert_eq!(last.active_allocations, 0);
    assert_eq!(last.dropped_events, 0);

    // The admission peak covers capacity reserved before any API registration;
    // shared batch splits do not multiply it or release it prematurely.
    let budget = Budget::new(1024);
    let mut reservation = GpuReservation::new(128, vec![budget.reserve(128).unwrap()]).unwrap();
    let split = reservation.split(64).unwrap();
    assert_eq!(budget.peak(), 128);
    assert_eq!(budget.used(), 128);
    drop(reservation);
    assert_eq!(budget.used(), 128);
    drop(split);
    assert_eq!(budget.used(), 0);
    assert_eq!(budget.peak(), 128);
    assert!(budget.reserve(1025).is_err());
    assert_eq!(budget.peak(), 128);
}
