use super::*;
use std::time::Duration;

fn at(start: Instant, millis: u64) -> Instant {
    start + Duration::from_millis(millis)
}

#[test]
fn idle_completion_reaps_closed_host_without_another_submission() {
    let owner = QueueOwner::default();
    let start = Instant::now();
    assert_eq!(
        owner.progress(start, true, || panic!("idle poll")).unwrap(),
        (None, false)
    );
    let host = owner.register();
    let (serial, completion) = owner.submission_receipt();
    let callback = owner.completion_callback(serial, completion);
    owner.observe_attachment(host, 1, 1, Some(4096), serial);
    owner.close_host(host);
    assert_eq!(
        owner.progress(start, true, || Ok(())).unwrap(),
        (Some(at(start, 2)), false)
    );
    assert_eq!(
        owner
            .progress(at(start, 1), true, || panic!("early poll"))
            .unwrap(),
        (Some(at(start, 2)), false)
    );
    assert_eq!(
        owner
            .progress(at(start, 2), true, || {
                callback();
                Ok(())
            })
            .unwrap(),
        (None, false)
    );
    assert_eq!(owner.snapshot().1, serial);
    assert_eq!(owner.snapshot().2, serial);
    let state = owner.0.borrow();
    let snapshot = state.attachments.lock().unwrap().snapshot(serial);
    assert_eq!(snapshot.completed_retirements, 1);
    assert_eq!(snapshot.retiring_payload_bytes, 0);
    drop(state);
    assert_eq!(
        owner
            .progress(at(start, 2000), true, || panic!("completed poll"))
            .unwrap(),
        (None, false)
    );
}

#[test]
fn newer_submissions_do_not_extend_stalled_watermark_and_retry_is_explicit() {
    let owner = QueueOwner::default();
    let host = owner.register();
    let start = Instant::now();
    let (first, completion) = owner.submission_receipt();
    owner.progress(start, true, || Ok(())).unwrap();
    let (second, _) = owner.submission_receipt();
    assert_eq!(
        owner.progress(at(start, 1999), true, || Ok(())).unwrap(),
        (Some(at(start, 2000)), false)
    );
    assert_eq!(
        owner
            .progress(at(start, 2000), true, || panic!("expired poll"))
            .unwrap(),
        (None, true)
    );
    completion.store(first, Ordering::Release);
    assert_eq!(
        owner
            .progress(at(start, 2001), true, || panic!("failed poll"))
            .unwrap(),
        (None, false)
    );
    assert_eq!(owner.admit(host, false, second), Admission::Wait);
    assert!(owner.retry_progress());
    assert!(!owner.retry_progress());
    assert_eq!(
        owner.progress(at(start, 2002), true, || Ok(())).unwrap(),
        (Some(at(start, 2004)), false)
    );
    assert_eq!(owner.admit(host, false, second), Admission::Wait);
    completion.store(second, Ordering::Release);
    assert_eq!(
        owner
            .progress(at(start, 2003), true, || panic!("already completed"))
            .unwrap(),
        (None, false)
    );
    assert_eq!(owner.admit(host, false, second), Admission::Frame);
}

#[test]
fn external_completion_renews_budget_and_reaps_before_poll_deadline() {
    let owner = QueueOwner::default();
    let start = Instant::now();
    let host = owner.register();
    let (first, completion) = owner.submission_receipt();
    owner.observe_attachment(host, 1, 1, Some(32), first);
    owner.close_host(host);
    let (second, _) = owner.submission_receipt();
    owner.progress(start, true, || Ok(())).unwrap();
    completion.store(first, Ordering::Release);
    // A callback delivered by another frame's poll is valid progress, even at
    // the previous deadline. It cannot retire the still-pending second receipt.
    assert_eq!(
        owner.progress(at(start, 2000), true, || Ok(())).unwrap(),
        (Some(at(start, 2002)), false)
    );
    assert_eq!(
        owner
            .with_attachments(|ledger, n| ledger.snapshot(n))
            .completed_retirements,
        1
    );
    assert_eq!(owner.snapshot().2, first);
    assert_eq!(owner.snapshot().1, second);
    assert_eq!(
        owner.progress(at(start, 3999), true, || Ok(())).unwrap(),
        (Some(at(start, 4000)), false)
    );
    assert_eq!(
        owner
            .progress(at(start, 4000), true, || panic!("expired"))
            .unwrap(),
        (None, true)
    );
}

#[test]
fn all_hosts_hidden_pause_polling_and_preserve_active_budget() {
    let owner = QueueOwner::default();
    let start = Instant::now();
    owner.submission_receipt();
    owner.progress(start, true, || Ok(())).unwrap();
    assert_eq!(
        owner
            .progress(at(start, 900), false, || panic!("hidden poll"))
            .unwrap(),
        (None, false)
    );
    assert_eq!(
        owner
            .progress(at(start, 10000), false, || panic!("hidden poll"))
            .unwrap(),
        (None, false)
    );
    assert_eq!(
        owner
            .progress(at(start, 20000), true, || panic!("restore backoff"))
            .unwrap(),
        (Some(at(start, 20016)), false)
    );
    assert_eq!(
        owner.progress(at(start, 21099), true, || Ok(())).unwrap(),
        (Some(at(start, 21100)), false)
    );
    assert_eq!(
        owner
            .progress(at(start, 21100), true, || panic!("expired"))
            .unwrap(),
        (None, true)
    );
}

#[test]
fn poll_error_latches_failure_without_fabricating_completion() {
    let owner = QueueOwner::default();
    let start = Instant::now();
    owner.submission_receipt();
    assert!(
        owner
            .progress(start, true, || anyhow::bail!("device poll failed"))
            .is_err()
    );
    assert_eq!(owner.snapshot().2, 0);
    assert_eq!(
        owner
            .progress(at(start, 2), true, || panic!("failed"))
            .unwrap(),
        (None, false)
    );
    assert!(owner.retry_progress());
}
