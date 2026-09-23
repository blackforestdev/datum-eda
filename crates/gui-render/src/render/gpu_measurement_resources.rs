//! Existing timestamp resources participate in shared GPU admission and retirement.
use super::{BYTES, QUERIES, Slot};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef};

impl Slot {
    pub(super) fn new(device: &wgpu::Device, owner: &Owner, epoch: u64) -> anyhow::Result<Self> {
        let budget = crate::text_gpu::budget::gpu_process();
        // All three reservations precede API allocation; failure rolls them back.
        let queries = budget.reserve(BYTES)?;
        let resolve = budget.reserve(BYTES)?;
        let readback = budget.reserve(BYTES)?;
        Ok(Self {
            queries: owner.track_with_permits(
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("datum-measurement-pass-queries"),
                    ty: wgpu::QueryType::Timestamp,
                    count: QUERIES,
                }),
                // Public logical timestamp capacity (u64 per result). Backend
                // query-pool metadata/residency is not exposed by this API.
                BYTES,
                epoch,
                Kind::Query,
                vec![queries],
            ),
            resolve: owner.track_with_permits(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-measurement-query-resolve"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                BYTES,
                epoch,
                Kind::QueryResolve,
                vec![resolve],
            ),
            readback: owner.track_with_permits(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-measurement-query-readback"),
                    size: BYTES,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                BYTES,
                epoch,
                Kind::Readback,
                vec![readback],
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
