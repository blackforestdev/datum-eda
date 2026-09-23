//! Explicit measurement admission; normal renderers allocate no query resources.
use super::{
    GpuFrameSample, Renderer,
    gpu_measurements::{FrameQueries, GpuMeasurements},
};
use std::time::Instant;

impl Renderer {
    pub fn enable_gpu_measurements(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        host: u64,
        epoch: u64,
        cancellation_observer: super::GpuCancellationObserver,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.measurements.is_none(),
            "GPU measurements already enabled"
        );
        self.measurements = Some(GpuMeasurements::new(
            device,
            queue,
            host,
            epoch,
            cancellation_observer,
        )?);
        Ok(())
    }

    pub fn poll_gpu_measurements(
        &mut self,
        device: &wgpu::Device,
    ) -> anyhow::Result<Vec<GpuFrameSample>> {
        self.measurements
            .as_mut()
            .map(|m| m.poll(device))
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    pub fn gpu_measurement_poll_deadline(&self) -> Option<Instant> {
        self.measurements
            .as_ref()
            .and_then(GpuMeasurements::next_poll)
    }

    pub fn set_gpu_measurement_visibility(
        &mut self,
        drawable: bool,
        occluded: bool,
        suspended: bool,
    ) {
        if let Some(m) = self.measurements.as_mut() {
            m.set_drawable(drawable);
            m.set_occluded(occluded);
            m.set_suspended(suspended);
        }
    }

    pub(super) fn begin_gpu_measurement(&mut self) -> anyhow::Result<Option<FrameQueries>> {
        if self.cold_world.measurement_frame.is_some() {
            return Ok(None);
        }
        self.measurements
            .as_mut()
            .map(GpuMeasurements::begin)
            .transpose()
    }

    pub(super) fn resolve_gpu_measurement(
        &self,
        frame: &mut Option<FrameQueries>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> anyhow::Result<()> {
        if let (Some(m), Some(frame)) = (&self.measurements, frame) {
            m.resolve(frame, encoder)?;
        }
        Ok(())
    }

    pub(super) fn submit_gpu_measurement(
        &mut self,
        frame: Option<FrameQueries>,
    ) -> anyhow::Result<()> {
        if let Some(cold_frame) = self.cold_world.measurement_frame.take()
            && let Some(m) = &mut self.measurements
        {
            m.incomplete_upload_submission(Some(cold_frame))?;
        }
        if let (Some(m), Some(frame)) = (&mut self.measurements, frame) {
            m.submitted(frame)?;
        }
        Ok(())
    }
}
