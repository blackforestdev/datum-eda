use super::*;
impl Owner {
    pub(crate) fn track<T>(
        &self,
        resource: T,
        bytes: u64,
        generation: u64,
        kind: Kind,
    ) -> Tracked<T> {
        self.track_reserved(
            resource,
            generation,
            kind,
            crate::text_gpu::budget::GpuReservation::new(bytes, Vec::new())
                .expect("test GPU reservation"),
        )
    }
}

#[test]
fn texture_reservation_survives_owner_and_every_submission_hold() {
    let budget = super::super::budget::Budget::new(64);
    let owner = Owner::new();
    let texture = owner.track_reserved(
        vec![0_u8; 64],
        1,
        Kind::Texture,
        super::super::budget::GpuReservation::new(64, vec![budget.reserve(64).unwrap()]).unwrap(),
    );
    let id = texture.id();
    let first = texture.submission_ref();
    let second = texture.submission_ref();
    drop(texture);
    drop(owner);
    assert_eq!(budget.used(), 64);
    assert!(budget.reserve(1).is_err());
    let records = crate::Renderer::text_gpu_process_allocations();
    assert!(
        records
            .iter()
            .any(|record| record.id == id && record.retiring)
    );
    drop(first);
    assert_eq!(budget.used(), 64);
    drop(second);
    assert_eq!(budget.used(), 0);
    assert!(
        !crate::Renderer::text_gpu_process_allocations()
            .iter()
            .any(|record| record.id == id)
    );
}

#[test]
fn split_upload_reservation_survives_mapped_owner_and_packet_submissions() {
    use super::super::budget::{Budget, GpuReservation};
    let budget = Budget::new(96);
    let mut reservation = GpuReservation::new(96, vec![budget.reserve(96).unwrap()]).unwrap();
    assert!(reservation.split(97).is_err());
    assert_eq!(reservation.bytes(), 96);
    let owner = Owner::new();
    let mapped = owner.track_reserved((), 1, Kind::Staging, reservation.split(64).unwrap());
    let packet = owner.track_reserved((), 1, Kind::Staging, reservation.split(32).unwrap());
    assert_eq!(reservation.bytes(), 0);
    assert_eq!(owner.records().iter().map(|r| r.bytes).sum::<u64>(), 96);
    let observer = owner.observer();
    let first = packet.submission_ref();
    let second = packet.submission_ref();
    drop(reservation);
    drop(mapped);
    drop(packet);
    drop(owner);
    assert_eq!(budget.used(), 96);
    assert!(budget.reserve(1).is_err());
    assert_eq!(observer.allocations().len(), 1);
    assert_eq!(observer.allocations()[0].bytes, 32);
    assert!(observer.allocations()[0].retiring);
    drop(first);
    assert_eq!(budget.used(), 96);
    drop(second);
    assert_eq!(budget.used(), 0);
    assert!(observer.allocations().is_empty());
    assert!(budget.reserve(96).is_ok());
}

#[test]
fn submitted_allocation_stays_charged_after_cpu_replacement() {
    let owner = Owner::new();
    let observer = owner.observer();
    let first = owner.track(vec![0_u8; 64], 64, 1, Kind::Instances);
    let first_id = first.id();
    let submitted = first.submission_ref();
    let submitted_again = first.submission_ref();
    let replacement = owner.track(vec![1_u8; 64], 64, 2, Kind::Instances);
    assert_ne!(first_id, replacement.id());
    drop(first);
    let records = owner.records();
    assert_eq!(records.iter().map(|r| r.bytes).sum::<u64>(), 128);
    assert!(records.iter().find(|r| r.id == first_id).unwrap().retiring);
    drop(submitted);
    assert_eq!(
        owner.records().len(),
        2,
        "later submission still pins old allocation"
    );
    drop(submitted_again);
    assert_eq!(owner.records().len(), 1);
    drop(owner);
    assert_eq!(observer.allocations().len(), 1);
    drop(replacement);
    assert!(observer.allocations().is_empty());
}

#[test]
fn reference_roles_payload_and_final_release_survive_producer_close() {
    let owner = Owner::new();
    let observer = owner.observer();
    let resource = owner.track((), 64, 7, Kind::Vertex);
    resource.set_requested_bytes(48);
    let prepared = resource.prepared_ref();
    let submitted = resource.submission_ref();
    let snapshot = owner.records()[0];
    assert_eq!((snapshot.requested_bytes, snapshot.bytes), (48, 64));
    assert_eq!(
        (snapshot.prepared_references, snapshot.submission_references),
        (1, 1)
    );
    resource.retire(RetirementReason::Cleared);
    resource.retire(RetirementReason::Replaced);
    drop(resource);
    drop(owner);
    assert_eq!(
        observer.allocations()[0].retirement_reason,
        Some(RetirementReason::Cleared)
    );
    assert_eq!(observer.released_allocations()[2].1.allocations, 0);
    drop(prepared);
    assert_eq!(observer.allocations()[0].prepared_references, 0);
    assert_eq!(observer.allocations()[0].submission_references, 1);
    drop(submitted);
    assert!(observer.allocations().is_empty());
    assert_eq!(
        observer.released_allocations()[2].1,
        ReleasedAllocations {
            allocations: 1,
            capacity_bytes: 64
        }
    );
}

#[test]
fn directly_submitted_staging_owner_counts_once_until_completion_drop() {
    let owner = Owner::new();
    let resource = owner.track((), 32, 1, Kind::Staging);
    resource.mark_retiring();
    resource.mark_retiring();
    assert_eq!(owner.records()[0].submission_references, 1);
    let extra = resource.submission_ref();
    assert_eq!(owner.records()[0].submission_references, 2);
    drop(resource);
    assert_eq!(owner.records()[0].submission_references, 1);
    drop(extra);
    assert!(owner.records().is_empty());
    assert_eq!(
        owner.observer().released_allocations()[4].1,
        ReleasedAllocations {
            allocations: 1,
            capacity_bytes: 32
        }
    );
}

#[test]
fn accounting_receipt_does_not_keep_a_released_gpu_allocation_live() {
    let owner = Owner::new();
    let observer = owner.observer();
    let resource = owner.track((), 16, 1, Kind::Vertex);
    let receipt = resource.upload_target().receipt(4, 4);
    drop(resource);
    assert!(observer.allocations().is_empty());
    assert_eq!(
        observer.released_allocations()[0].1,
        ReleasedAllocations {
            allocations: 1,
            capacity_bytes: 16
        }
    );
    drop(receipt);
    assert_eq!(observer.released_allocations()[0].1.allocations, 1);
}
