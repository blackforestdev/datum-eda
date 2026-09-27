//! Real allocation windows, including loss and overlapping completion order.
use super::*;
use crate::cpu_alloc::calls::{Call, Scope, observation};
use crate::text_gpu::budget::Budget;

fn allocation(record: &Record) -> &AllocationRecord {
    match record {
        Record::Allocation(event) => event,
        Record::Call(_) => panic!("expected allocation"),
    }
}

fn summary_record(record: &Record) -> &Event {
    match record {
        Record::Call(event) => event,
        Record::Allocation(_) => panic!("expected call summary"),
    }
}

#[test]
#[ignore = "exclusive process-wide private-call observer; run serially"]
fn private_call_delivery_preserves_transient_peaks_and_rejects_loss() {
    assert!(start(0).is_err());
    assert!(start(65_537).is_err());
    let scope = Scope::new("observation-conformance");
    let host = Budget::new(512);
    let process = Budget::new(1024);
    let active = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    assert!(start(16).is_err());
    drop(active);
    let id = start(16).unwrap();
    assert!(start(16).is_err());
    assert!(drain(id + 1).is_err());
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    assert!(stop(id).is_err());
    drop(scope.with(|| vec![0_u8; 600]));
    assert!(call.finish(0, None, None).is_err());
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    drop(scope.with(|| vec![0_u8; 100]));
    let (_, report) = call.finish(0, None, None).unwrap();
    assert!(
        report.peak_bytes < 512,
        "old lifetime peak is not this call's peak"
    );
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    drop(call);
    let first = drain(id).unwrap();
    assert_eq!(first.events.len(), 6);
    assert_eq!(first.dropped_events, 0);
    assert_eq!(first.total_events, 6);
    assert_eq!(first.active_calls, 0);
    assert_eq!(summary_record(&first.events[1]).phase, Phase::Finished);
    assert!(summary_record(&first.events[1]).report.exceeded);
    assert!(summary_record(&first.events[1]).report.peak_bytes >= 600);
    assert_eq!(summary_record(&first.events[1]).report.final_bytes, 0);
    assert!(!summary_record(&first.events[3]).report.exceeded);
    assert_eq!(
        summary_record(&first.events[3]).report.peak_bytes,
        report.peak_bytes
    );
    assert_eq!(summary_record(&first.events[5]).phase, Phase::Abandoned);
    assert_eq!(
        summary_record(&first.events[5]).owner_label,
        "observation-conformance"
    );
    for (index, event) in first.events.iter().enumerate() {
        assert_eq!(event.sequence(), index as u64 + 1);
    }
    assert!(!has_pending(id).unwrap());
    assert_eq!(stop(id).unwrap().total_events, 6);
    assert!(drain(id).is_err());

    // The completed event is lost with a one-slot observer; never overwrite the
    // begin event or reset the loss count after a successful drain.
    let lost = start(1).unwrap();
    assert_ne!(lost, id);
    let call = Call::begin(&scope, host, process, 0, 0).unwrap();
    call.finish(0, None, None).unwrap();
    let batch = drain(lost).unwrap();
    assert_eq!(batch.events.len(), 1);
    assert_eq!(summary_record(&batch.events[0]).phase, Phase::Begin);
    assert_eq!(batch.total_events, 2);
    assert_eq!(batch.dropped_events, 1);
    assert_eq!(stop(lost).unwrap().dropped_events, 1);
}

#[test]
#[ignore = "exclusive process-wide private-call observer; run serially"]
fn private_call_delivery_records_concurrent_host_incidence() {
    let id = observation::start(16).unwrap();
    let process = Budget::new(1000);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let process = process.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let scope = Scope::new("observed-concurrent-host");
                let call = Call::begin(&scope, Budget::new(1000), process, 0, 0).unwrap();
                let payload = scope.with(|| vec![0_u8; 600]);
                barrier.wait();
                barrier.wait();
                drop(payload);
                assert!(call.finish(0, None, None).is_err());
            })
        })
        .collect();
    barrier.wait();
    let beginnings = drain(id).unwrap();
    assert_eq!(beginnings.active_calls, 2);
    assert_eq!(beginnings.events.len(), 2);
    assert_ne!(
        summary_record(&beginnings.events[0]).report.owner_id,
        summary_record(&beginnings.events[1]).report.owner_id
    );
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    let endings = stop(id).unwrap();
    assert_eq!(endings.events.len(), 2);
    assert_eq!(endings.total_events, 4);
    assert_eq!(endings.dropped_events, 0);
    for record in endings.events {
        let event = summary_record(&record);
        assert_eq!(event.phase, Phase::Finished);
        assert!(event.report.process_peak_bytes >= 1200);
        assert!(event.report.host_peak_bytes < 1000);
        assert!(
            beginnings
                .events
                .iter()
                .any(|e| summary_record(e).report.call_id == event.report.call_id)
        );
    }
}

#[test]
#[ignore = "exclusive process-wide private-call observer; run serially"]
fn allocation_lifetimes_include_input_release_realloc_overlap_and_retained_output() {
    let scope = Scope::new("private-allocation-lifetimes");
    let input = scope.with(|| vec![1_u8; 32]);
    let input_bytes = scope.usage().payload_bytes + scope.usage().tracking_bytes;
    let id = start_with_allocations(32).unwrap();
    let call = Call::begin(&scope, Budget::new(4096), Budget::new(4096), 0, 0).unwrap();
    let mut output = scope.with(|| vec![9_u8; 64]);
    let first_output = scope.usage().payload_bytes + scope.usage().tracking_bytes - input_bytes;
    output.reserve_exact(64);
    drop(input);
    let retained = scope.usage().payload_bytes + scope.usage().tracking_bytes;
    let (permits, report) = call.finish(retained, None, None).unwrap();
    let batch = stop(id).unwrap();
    assert_eq!(batch.dropped_events, 0);
    assert_eq!(batch.events.len(), 6);
    assert_eq!(summary_record(&batch.events[0]).phase, Phase::Begin);
    assert_eq!(
        summary_record(&batch.events[0]).scope_live_bytes,
        input_bytes
    );
    let events = &batch.events[1..5];
    let expected = [
        (true, first_output, input_bytes + first_output),
        (true, retained, input_bytes + first_output + retained),
        (false, first_output, input_bytes + retained),
        (false, input_bytes, retained),
    ];
    for (record, (allocated, bytes, live)) in events.iter().zip(expected) {
        let Record::Allocation(event) = record else {
            panic!("expected allocation")
        };
        assert_eq!(event.call_id, report.call_id);
        assert_eq!(event.owner_id, scope.usage().owner_id);
        let allocation = event.allocation;
        assert_eq!(allocation.allocated, allocated);
        assert_eq!(allocation.bytes, bytes);
        assert_eq!(event.scope_live_bytes, live);
    }
    assert_eq!(
        allocation(&events[0]).allocation.address,
        allocation(&events[2]).allocation.address
    );
    assert_ne!(
        allocation(&events[0]).allocation.address,
        allocation(&events[1]).allocation.address
    );
    assert_eq!(summary_record(&batch.events[5]).phase, Phase::Finished);
    assert_eq!(summary_record(&batch.events[5]).scope_live_bytes, retained);
    assert_eq!(report.peak_bytes, input_bytes + first_output + retained);
    for (index, event) in batch.events.iter().enumerate() {
        assert_eq!(event.sequence(), index as u64 + 1);
    }
    drop((output, permits));
    assert_eq!(scope.usage().allocations, 0);

    // Allocation transitions share the same sticky loss budget as call records.
    let id = start_with_allocations(1).unwrap();
    let call = Call::begin(&scope, Budget::new(4096), Budget::new(4096), 0, 0).unwrap();
    drop(scope.with(|| vec![0_u8; 16]));
    call.finish(0, None, None).unwrap();
    let lost = stop(id).unwrap();
    assert_eq!(lost.total_events, 4);
    assert_eq!(lost.dropped_events, 3);
}

#[test]
#[ignore = "exclusive process-wide private-call observer; run serially"]
fn observed_startup_burst_fits_compact_storage_and_byte_exhaustion_stays_fatal() {
    assert!(start_with_allocations(131_073).is_err());
    let scope = Scope::new("native-sized-observer-burst");
    let id = start_with_allocations(131_072).unwrap();
    let active = Call::begin(&scope, Budget::new(4096), Budget::new(4096), 0, 0).unwrap();
    // Reproduce the demonstrated123064-event burst through real allocator hooks.
    for _ in 0..61_531 {
        let value = scope.with(|| Box::new(7_u8));
        std::hint::black_box(&value);
        drop(value);
    }
    active.finish(0, None, None).unwrap();
    let batch = stop(id).unwrap();
    assert_eq!(batch.events.len(), 123_064);
    assert_eq!(batch.total_events, 123_064);
    assert_eq!(batch.dropped_events, 0);
    assert!(batch.buffer_capacity_bytes <= MAX_STORAGE_BYTES);
    drop(batch);

    // Full call reports consume bytes as well as slots. Exceed bytes while
    // remaining below the declared event count, and retain sticky failure.
    let id = start_with_allocations(131_072).unwrap();
    let pairs = (MAX_STORAGE_BYTES - vector_bytes(131_072)) / call_bytes() / 2 + 2;
    assert!(pairs * 2 < 131_072);
    for _ in 0..pairs {
        Call::begin(&scope, Budget::new(4096), Budget::new(4096), 0, 0)
            .unwrap()
            .finish(0, None, None)
            .unwrap();
    }
    let batch = drain(id).unwrap();
    assert!(batch.dropped_events > 0);
    assert!(batch.buffer_capacity_bytes <= MAX_STORAGE_BYTES);
    assert_eq!(stop(id).unwrap().dropped_events, batch.dropped_events);
}
