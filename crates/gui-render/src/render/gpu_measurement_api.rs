//! Explicit measurement admission; normal renderers allocate no query resources.
use super::{
    GpuFrameSample, Renderer,
    gpu_measurements::{FrameQueries, GpuMeasurements},
};
use std::time::Instant;

impl Renderer {
    /// Seals the full frame range after the existing controlled queue drain.
    pub fn gpu_measurement_drained_manifest(&self) -> anyhow::Result<Option<[u64; 3]>> {
        self.measurements
            .as_ref()
            .map(GpuMeasurements::drained_manifest)
            .transpose()
    }

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
        let _resource_scope = self.resource_host.enter();
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
        if let Some(frame) = self.pending_measurement.take() {
            return Ok(Some(frame));
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
            // This marker follows the graph, including all presentation copies.
            // Resolve queries only after its end timestamp has been encoded.
            m.final_marker(frame, encoder)?;
            m.resolve(frame, encoder)?;
        }
        Ok(())
    }

    pub(super) fn submit_gpu_measurement(
        &mut self,
        queue: &wgpu::Queue,
        frame: Option<FrameQueries>,
    ) -> anyhow::Result<()> {
        if let (Some(m), Some(frame)) = (&mut self.measurements, frame) {
            m.submitted(queue, frame)?;
        }
        Ok(())
    }
}

impl Renderer {
    pub(crate) fn begin_upload_measurement(
        &mut self,
        device: &wgpu::Device,
        kind: &'static str,
    ) -> anyhow::Result<Option<(wgpu::CommandBuffer, wgpu::CommandBuffer)>> {
        let Some(mut frame) = self.begin_gpu_measurement()? else {
            return Ok(None);
        };
        let measurements = self.measurements.as_ref().expect("reserved measurement");
        let before = measurements.leading(
            device,
            &mut frame,
            kind,
            self.measurement_attempt,
            self.measurement_workload,
            false,
        )?;
        let after = measurements.trailing(device, &mut frame)?;
        self.pending_measurement = Some(frame);
        Ok(Some((before, after)))
    }
    pub(crate) fn finish_upload_measurement(&mut self, queue: &wgpu::Queue) -> anyhow::Result<()> {
        if let Some(measurements) = &mut self.measurements {
            let frame = self
                .pending_measurement
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("upload submitted without GPU boundary"))?;
            measurements.continued(queue, frame)?;
        }
        Ok(())
    }
    pub(super) fn final_measurement_leading(
        &self,
        device: &wgpu::Device,
        frame: &mut Option<FrameQueries>,
    ) -> anyhow::Result<Option<wgpu::CommandBuffer>> {
        if let (Some(measurements), Some(frame)) = (&self.measurements, frame) {
            Ok(Some(measurements.leading(
                device,
                frame,
                "final",
                self.measurement_attempt,
                self.measurement_workload,
                true,
            )?))
        } else {
            Ok(None)
        }
    }
}
