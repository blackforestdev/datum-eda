//! Existing timestamp resources participate in shared GPU admission and retirement.
use super::{BYTES, QUERIES, Slot};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef};

impl Slot {
    pub(super) fn new(device: &wgpu::Device, owner: &Owner, epoch: u64) -> anyhow::Result<Self> {
        use crate::text_gpu::budget::GpuReservation;
        // All three reservations precede API allocation; failure rolls them back.
        let queries = GpuReservation::new(BYTES, Vec::new())?;
        let resolve = GpuReservation::new(BYTES, Vec::new())?;
        let readback = GpuReservation::new(BYTES, Vec::new())?;
        Ok(Self {
            queries: owner.track_reserved(
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("datum-measurement-pass-queries"),
                    ty: wgpu::QueryType::Timestamp,
                    count: QUERIES,
                }),
                // Public logical timestamp capacity (u64 per result). Backend
                // query-pool metadata/residency is not exposed by this API.
                epoch,
                Kind::Query,
                queries,
            ),
            resolve: owner.track_reserved(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-measurement-query-resolve"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                epoch,
                Kind::QueryResolve,
                resolve,
            ),
            readback: owner.track_reserved(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-measurement-query-readback"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                epoch,
                Kind::Readback,
                readback,
            ),
            pending: None,
        })
    }

    pub(super) fn submission_refs(&self) -> Vec<SubmissionRef> {
        vec![
            self.queries.submission_ref(),
            self.resolve.submission_ref(),
            self.readback.submission_ref(),
        ]
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "gpu_measurement_resource_tests.rs"]
mod tests;
