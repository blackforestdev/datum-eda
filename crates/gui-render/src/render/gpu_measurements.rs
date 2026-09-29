//! Opt-in GPU-01..03 pass measurements. Never a presentation clock or scheduler.
//! Three slots own their query/resolve/readback resources until map completion.
use crate::text_gpu::lifetime::{SubmissionRef, Tracked};
#[path = "gpu_measurement_frame.rs"]
mod frame;
#[path = "gpu_measurement_resources.rs"]
mod resources;
pub(crate) use frame::FrameQueries;
#[path = "gpu_measurement_timeline.rs"]
pub(crate) mod timeline;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
use std::time::{Duration, Instant};

const SLOTS: usize = 3;
const QUERIES: u32 = 32;
const BYTES: u64 = QUERIES as u64 * 8;
const DEADLINE: Duration = Duration::from_secs(2);
const ENCODING: u8 = 0;
const MAPPING: u8 = 1;
const READY: u8 = 2;
const MAP_FAILED: u8 = 3;
const ABORTED: u8 = 4;

#[derive(Debug)]
pub struct GpuFrameSample {
    pub submission_manifest: Vec<frame::SubmissionSample>,
    pub host: u64,
    pub device_epoch: u64,
    pub frame: u64,
    pub submission: u64,
    pub period_ns: f64,
    pub raw_ticks: Vec<u64>,
    /// Raw queries 2/3/4 inside the first scene pass: before grid, after grid,
    /// after composed world bundles. These nested points are not extra passes.
    pub scene_marker_ticks: Option<[u64; 3]>,
    pub passes_ns: Vec<(&'static str, f64)>,
    pub own_pass_sum_ns: f64,
    pub frame_span_ns: f64,
}

#[derive(Debug, Clone)]
pub struct GpuMeasurementCancellation {
    pub submitted_lineage: Vec<(u64, &'static str, [u64; 7], [u64; 8])>,
    pub reason: &'static str,
    pub host: u64,
    pub device_epoch: u64,
    pub frame: u64,
    pub submission: Option<u64>,
}

pub type GpuCancellationObserver = Box<dyn Fn(GpuMeasurementCancellation) + Send + Sync>;

struct Pending {
    frame: u64,
    submission: Option<u64>,
    passes: Vec<&'static str>,
    marker_count: u32,
    scene_offset: Option<u32>,
    timeline: timeline::Timeline,
    signal: Arc<AtomicU8>,
    active_start: Duration,
}

struct Slot {
    queries: Tracked<wgpu::QuerySet>,
    resolve: Tracked<wgpu::Buffer>,
    readback: Tracked<wgpu::Buffer>,
    pending: Option<Pending>,
}

/// Active time is paused by native drawable/occlusion/suspension notifications.
struct ActiveClock {
    previous: Instant,
    elapsed: Duration,
    drawable: bool,
    occluded: bool,
    suspended: bool,
}

impl ActiveClock {
    fn advance(&mut self, now: Instant) {
        if self.active() {
            self.elapsed += now.saturating_duration_since(self.previous);
        }
        self.previous = now;
    }

    fn active(&self) -> bool {
        self.drawable && !self.occluded && !self.suspended
    }
}

pub(crate) struct GpuMeasurements {
    host: u64,
    epoch: u64,
    period_ns: f64,
    scene_markers_enabled: bool,
    next_frame: u64,
    slots: Vec<Slot>,
    marker: Tracked<resources::Marker>,
    clock: ActiveClock,
    cancelled: bool,
    cancellation_observer: GpuCancellationObserver,
}

impl GpuMeasurements {
    pub(crate) fn drained_manifest(&self) -> anyhow::Result<[u64; 3]> {
        anyhow::ensure!(
            !self.cancelled && self.slots.iter().all(|s| s.pending.is_none()),
            "GPU frame manifest is not drained"
        );
        Ok([self.host, self.epoch, self.next_frame])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        host: u64,
        epoch: u64,
        cancellation_observer: GpuCancellationObserver,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            device.features().contains(wgpu::Features::TIMESTAMP_QUERY),
            "GPU measurement requires enabled TIMESTAMP_QUERY"
        );
        let period_ns = f64::from(queue.get_timestamp_period());
        anyhow::ensure!(
            period_ns.is_finite() && period_ns > 0.0,
            "GPU measurement timestamp period unavailable"
        );
        anyhow::ensure!(epoch != 0, "GPU measurement requires a device epoch");
        let owner = crate::text_gpu::lifetime::Owner::new();
        let slots = (0..SLOTS)
            .map(|_| Slot::new(device, &owner, epoch))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self {
            host,
            epoch,
            period_ns,
            scene_markers_enabled: device
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
            next_frame: 0,
            slots,
            marker: resources::Marker::new(device, &owner, epoch)?,
            clock: ActiveClock {
                previous: Instant::now(),
                elapsed: Duration::ZERO,
                drawable: true,
                occluded: false,
                suspended: false,
            },
            cancelled: false,
            cancellation_observer,
        })
    }

    pub(crate) fn begin(&mut self) -> anyhow::Result<FrameQueries> {
        anyhow::ensure!(!self.cancelled, "GPU measurement device epoch cancelled");
        self.clock.advance(Instant::now());
        let (index, slot) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.pending.is_none())
            .ok_or_else(|| {
                anyhow::anyhow!("GPU measurement ring exhausted; timed trial incomplete")
            })?;
        self.next_frame = self
            .next_frame
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("GPU measurement frame ID exhausted"))?;
        let submission = next_submission_id()?;
        let consumers = crate::text_gpu::allocation_host::consumers();
        slot.queries.set_consumers(consumers);
        slot.resolve.set_consumers(consumers);
        slot.readback.set_consumers(consumers);
        let signal = Arc::new(AtomicU8::new(ENCODING));
        slot.pending = Some(Pending {
            frame: self.next_frame,
            submission: None,
            passes: Vec::new(),
            marker_count: 0,
            scene_offset: None,
            timeline: Default::default(),
            signal: signal.clone(),
            active_start: self.clock.elapsed,
        });
        Ok(FrameQueries {
            timeline: Default::default(),
            pass_indices: Vec::new(),
            scene_offset: None,
            final_transfer_end: None,
            slot: index,
            epoch: self.epoch,
            frame: self.next_frame,
            submission,
            queries: (*slot.queries).clone(),
            resources: slot
                .submission_refs()
                .into_iter()
                .chain([self.marker.submission_ref()])
                .collect(),
            passes: Vec::new(),
            query_count: 0,
            marker_count: 0,
            scene_markers_enabled: self.scene_markers_enabled,
            signal,
            resolved: false,
            submitted: false,
        })
    }

    fn validate(&self, frame: &FrameQueries) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.cancelled && frame.epoch == self.epoch,
            "GPU measurement stale device epoch"
        );
        anyhow::ensure!(
            self.slots[frame.slot]
                .pending
                .as_ref()
                .is_some_and(|p| p.frame == frame.frame && Arc::ptr_eq(&p.signal, &frame.signal)),
            "GPU measurement stale frame reservation"
        );
        Ok(())
    }

    pub(crate) fn resolve(
        &self,
        frame: &mut FrameQueries,
        encoder: &mut wgpu::CommandEncoder,
    ) -> anyhow::Result<()> {
        self.validate(frame)?;
        anyhow::ensure!(
            !frame.resolved && !frame.passes.is_empty(),
            "GPU measurement missing or duplicate pass resolution"
        );
        let slot = &self.slots[frame.slot];
        anyhow::ensure!(
            matches!(frame.marker_count, 0 | 3),
            "incomplete GPU scene markers"
        );
        frame.finish_boundaries()?;
        let count = frame.query_count;
        encoder.resolve_query_set(&slot.queries, 0..count, &slot.resolve, 0);
        encoder.copy_buffer_to_buffer(&slot.resolve, 0, &slot.readback, 0, u64::from(count) * 8);
        frame.resolved = true;
        Ok(())
    }

    /// Called only after the production queue submission; mapping never blocks.
    pub(crate) fn submitted(
        &mut self,
        queue: &wgpu::Queue,
        mut frame: FrameQueries,
    ) -> anyhow::Result<()> {
        // The caller already submitted. Even a stale/invalid receipt must retain
        // encoded resource accounting through that submission's completion.
        crate::text_gpu::hold_until_done(queue, std::mem::take(&mut frame.resources));
        self.validate(&frame)?;
        anyhow::ensure!(
            frame.resolved,
            "GPU measurement submission without query resolution"
        );
        let slot = &mut self.slots[frame.slot];
        let pending = slot.pending.as_mut().expect("validated reservation");
        frame.finish_submission()?;
        pending.timeline = frame.timeline.clone();
        pending.scene_offset = frame.scene_offset;
        pending.passes = std::mem::take(&mut frame.passes);
        pending.marker_count = frame.marker_count;
        pending.submission = Some(frame.submission);
        frame.signal.store(MAPPING, Ordering::Release);
        let signal = frame.signal.clone();
        slot.readback
            .slice(..(pending.passes.len() as u64 * 2 + u64::from(pending.marker_count)) * 8)
            .map_async(wgpu::MapMode::Read, move |result| {
                signal.store(
                    if result.is_ok() { READY } else { MAP_FAILED },
                    Ordering::Release,
                );
            });
        frame.submitted = true;
        Ok(())
    }

    pub(crate) fn set_drawable(&mut self, drawable: bool) {
        self.clock.advance(Instant::now());
        self.clock.drawable = drawable;
    }
    pub(crate) fn set_occluded(&mut self, occluded: bool) {
        self.clock.advance(Instant::now());
        self.clock.occluded = occluded;
    }
    pub(crate) fn set_suspended(&mut self, suspended: bool) {
        self.clock.advance(Instant::now());
        self.clock.suspended = suspended;
    }

    pub(crate) fn next_poll(&self) -> Option<Instant> {
        if self.cancelled || !self.clock.active() {
            return None;
        }
        let now = Instant::now();
        let elapsed = self.clock.elapsed + now.saturating_duration_since(self.clock.previous);
        self.slots
            .iter()
            .filter_map(|s| s.pending.as_ref())
            .map(|p| {
                now + DEADLINE
                    .saturating_sub(elapsed.saturating_sub(p.active_start))
                    .min(Duration::from_millis(10))
            })
            .min()
    }

    pub(crate) fn poll(&mut self, device: &wgpu::Device) -> anyhow::Result<Vec<GpuFrameSample>> {
        anyhow::ensure!(!self.cancelled, "GPU measurement device epoch cancelled");
        self.clock.advance(Instant::now());
        if !self.slots.iter().any(|s| s.pending.is_some()) {
            return Ok(Vec::new());
        }
        device.poll(wgpu::PollType::Poll)?;
        let mut samples = Vec::new();
        for slot in &mut self.slots {
            let Some(pending) = slot.pending.as_ref() else {
                continue;
            };
            anyhow::ensure!(
                self.clock.elapsed.saturating_sub(pending.active_start) < DEADLINE,
                "GPU measurement exceeded two seconds active time; sample unavailable"
            );
            match pending.signal.load(Ordering::Acquire) {
                READY => {
                    let view = slot
                        .readback
                        .slice(
                            ..(pending.passes.len() as u64 * 2 + u64::from(pending.marker_count))
                                * 8,
                        )
                        .get_mapped_range();
                    let raw_queries: Vec<u64> = view
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|v| u64::from_ne_bytes(*v))
                        .collect();
                    drop(view);
                    slot.readback.unmap();
                    let pending = slot.pending.take().expect("ready frame");
                    let mut raw = raw_queries.clone();
                    let scene_marker_ticks = extract_scene_markers_at(
                        &mut raw,
                        pending.marker_count,
                        pending.scene_offset,
                    )?;
                    let submission_manifest =
                        frame::decode_boundaries(&pending.timeline, &raw_queries, self.period_ns)?;
                    let (passes_ns, own_pass_sum_ns, frame_span_ns) =
                        decode(&pending.passes, &raw, self.period_ns)?;
                    samples.push(GpuFrameSample {
                        submission_manifest,
                        host: self.host,
                        device_epoch: self.epoch,
                        frame: pending.frame,
                        submission: pending.submission.expect("mapped only after submit"),
                        period_ns: self.period_ns,
                        raw_ticks: raw,
                        scene_marker_ticks,
                        passes_ns,
                        own_pass_sum_ns,
                        frame_span_ns,
                    });
                }
                MAP_FAILED => anyhow::bail!(
                    "GPU measurement map failed for host={} frame={}",
                    self.host,
                    pending.frame
                ),
                ABORTED => {
                    let aborted = slot.pending.take().expect("observed aborted frame");
                    (self.cancellation_observer)(GpuMeasurementCancellation {
                        submitted_lineage: aborted
                            .timeline
                            .entries()
                            .filter(|e| e.submitted)
                            .map(|e| (e.id, e.kind, e.attempt, e.workload))
                            .collect(),
                        reason: "encoding_aborted",
                        host: self.host,
                        device_epoch: self.epoch,
                        frame: aborted.frame,
                        submission: aborted.submission,
                    });
                    anyhow::bail!(
                        "GPU measurement encoding aborted for host={} frame={}; sample unavailable",
                        self.host,
                        aborted.frame
                    );
                }
                _ => anyhow::ensure!(
                    self.clock.elapsed.saturating_sub(pending.active_start) < DEADLINE,
                    "GPU measurement exceeded two seconds active time; sample unavailable"
                ),
            }
        }
        Ok(samples)
    }

    pub(crate) fn cancel(&mut self) {
        self.cancelled = true;
        for slot in &mut self.slots {
            if let Some(pending) = slot.pending.take() {
                (self.cancellation_observer)(GpuMeasurementCancellation {
                    submitted_lineage: pending
                        .timeline
                        .entries()
                        .filter(|e| e.submitted)
                        .map(|e| (e.id, e.kind, e.attempt, e.workload))
                        .collect(),
                    reason: "host_or_device_closed_before_collection",
                    host: self.host,
                    device_epoch: self.epoch,
                    frame: pending.frame,
                    submission: pending.submission,
                });
                let state = pending.signal.swap(ABORTED, Ordering::AcqRel);
                if matches!(state, MAPPING | READY) {
                    slot.readback.unmap();
                }
            }
        }
    }
}

impl Drop for GpuMeasurements {
    fn drop(&mut self) {
        self.cancel();
    }
}

// Keep the ordinary pass-pair representation intact for existing consumers.
// Nested markers never contribute a second time to own_pass_sum_ns.
#[cfg(test)]
fn extract_scene_markers(raw: &mut Vec<u64>, count: u32) -> anyhow::Result<Option<[u64; 3]>> {
    extract_scene_markers_at(raw, count, (count != 0).then_some(2))
}
fn extract_scene_markers_at(
    raw: &mut Vec<u64>,
    count: u32,
    offset: Option<u32>,
) -> anyhow::Result<Option<[u64; 3]>> {
    if count == 0 {
        return Ok(None);
    }
    let offset = offset.ok_or_else(|| anyhow::anyhow!("missing scene marker offset"))? as usize;
    anyhow::ensure!(
        count == 3 && offset >= 2 && raw.len() >= offset + 3,
        "missing GPU scene markers"
    );
    let markers = [raw[offset], raw[offset + 1], raw[offset + 2]];
    anyhow::ensure!(
        raw[offset - 2] <= markers[0] && markers.is_sorted() && markers[2] <= raw[offset - 1],
        "GPU scene markers outside ordered scene interval"
    );
    raw.drain(offset..offset + 3);
    Ok(Some(markers))
}

type Decoded = (Vec<(&'static str, f64)>, f64, f64);
fn decode(names: &[&'static str], ticks: &[u64], period: f64) -> anyhow::Result<Decoded> {
    anyhow::ensure!(
        period.is_finite() && period > 0.0,
        "invalid GPU timestamp period"
    );
    anyhow::ensure!(
        !names.is_empty() && ticks.len() == names.len() * 2,
        "missing GPU timestamp pairs"
    );
    let mut passes = Vec::with_capacity(names.len());
    for (name, pair) in names.iter().zip(ticks.as_chunks::<2>().0.iter()) {
        let delta = pair[1]
            .checked_sub(pair[0])
            .ok_or_else(|| anyhow::anyhow!("reversed/wrapped GPU timestamp pair"))?;
        let ns = delta as f64 * period;
        anyhow::ensure!(
            ns.is_finite() && ns < DEADLINE.as_nanos() as f64,
            "GPU timestamp span exceeds bounded measurement interval"
        );
        passes.push((*name, ns));
    }
    let span = ticks[ticks.len() - 1]
        .checked_sub(ticks[0])
        .ok_or_else(|| anyhow::anyhow!("reversed GPU frame span"))? as f64
        * period;
    anyhow::ensure!(
        span.is_finite() && span < DEADLINE.as_nanos() as f64,
        "GPU frame span exceeds bounded measurement interval"
    );
    let sum = passes.iter().map(|(_, ns)| *ns).sum();
    Ok((passes, sum, span))
}

#[cfg(test)]
#[path = "gpu_measurements_tests.rs"]
mod tests;

fn next_submission_id() -> anyhow::Result<u64> {
    static SUBMISSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    SUBMISSION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
        .map_err(|_| anyhow::anyhow!("GPU submission identity exhausted"))
}

impl GpuMeasurements {
    pub(crate) fn leading(
        &self,
        device: &wgpu::Device,
        frame: &mut FrameQueries,
        kind: &'static str,
        attempt: timeline::Attempt,
        workload: [u64; 8],
        final_submission: bool,
    ) -> anyhow::Result<wgpu::CommandBuffer> {
        self.validate(frame)?;
        self.marker
            .set_consumers(crate::text_gpu::allocation_host::consumers());
        frame.leading(
            device,
            &self.marker.view,
            kind,
            attempt,
            workload,
            final_submission,
        )
    }
    pub(crate) fn trailing(
        &self,
        device: &wgpu::Device,
        frame: &mut FrameQueries,
    ) -> anyhow::Result<wgpu::CommandBuffer> {
        self.validate(frame)?;
        frame.trailing(device, &self.marker.view)
    }
    pub(crate) fn continued(
        &mut self,
        queue: &wgpu::Queue,
        frame: &mut FrameQueries,
    ) -> anyhow::Result<()> {
        // The query/marker holds follow each real queue submission, even if the
        // logical frame or whole host is abandoned before final rendering.
        let resources = self.slots[frame.slot]
            .submission_refs()
            .into_iter()
            .chain([self.marker.submission_ref()])
            .collect();
        crate::text_gpu::hold_until_done(queue, resources);
        self.validate(frame)?;
        frame.finish_submission()?;
        let pending = self.slots[frame.slot].pending.as_mut().unwrap();
        pending.submission = Some(frame.submission);
        pending.timeline = frame.timeline.clone();
        Ok(())
    }
}
