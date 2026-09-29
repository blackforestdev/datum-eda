//! One query reservation spans bounded upload continuations and final rendering.
use super::*;
pub(crate) struct FrameQueries {
    pub(super) timeline: timeline::Timeline,
    pub(super) pass_indices: Vec<[u32; 2]>,
    pub(super) scene_offset: Option<u32>,
    pub(super) final_transfer_end: Option<u32>,
    pub(super) slot: usize,
    pub(super) epoch: u64,
    pub(super) frame: u64,
    pub(super) submission: u64,
    pub(super) queries: wgpu::QuerySet,
    pub(super) resources: Vec<SubmissionRef>,
    pub(super) passes: Vec<&'static str>,
    pub(super) query_count: u32,
    pub(super) marker_count: u32,
    pub(super) scene_markers_enabled: bool,
    pub(super) signal: Arc<AtomicU8>,
    pub(super) resolved: bool,
    pub(super) submitted: bool,
}

impl FrameQueries {
    pub(crate) fn pass(
        &mut self,
        name: &'static str,
    ) -> anyhow::Result<wgpu::RenderPassTimestampWrites<'_>> {
        anyhow::ensure!(
            !self.resolved,
            "GPU measurement pass after query resolution"
        );
        anyhow::ensure!(
            self.query_count + 2 <= QUERIES,
            "GPU measurement pass capacity exceeded"
        );
        let index = self.query_count;
        self.query_count += 2;
        self.passes.push(name);
        self.pass_indices.push([index, index + 1]);
        Ok(wgpu::RenderPassTimestampWrites {
            query_set: &self.queries,
            beginning_of_pass_write_index: Some(index),
            end_of_pass_write_index: Some(index + 1),
        })
    }

    pub(crate) fn mark_scene(
        &mut self,
        pass: &mut wgpu::RenderPass<'_>,
        marker: u32,
    ) -> anyhow::Result<()> {
        if !self.scene_markers_enabled {
            return Ok(());
        }
        anyhow::ensure!(
            !self.resolved
                && self.passes.last() == Some(&"frame")
                && marker == self.marker_count
                && marker < 3
                && self.query_count < QUERIES,
            "invalid GPU scene marker order or capacity"
        );
        if marker == 0 {
            self.scene_offset = Some(self.query_count);
        }
        pass.write_timestamp(&self.queries, self.query_count);
        self.query_count += 1;
        self.marker_count += 1;
        Ok(())
    }
}

impl Drop for FrameQueries {
    fn drop(&mut self) {
        if !self.submitted {
            self.signal.store(ABORTED, Ordering::Release);
        }
    }
}

#[derive(Debug)]
pub struct SubmissionSample {
    pub submission: u64,
    pub kind: &'static str,
    pub attempt: [u64; 7],
    pub workload: [u64; 8],
    pub first_tick: u64,
    pub last_tick: u64,
    pub transfer_first_tick: u64,
    pub transfer_last_tick: u64,
    pub span_ns: f64,
    pub transfer_interval_ns: f64,
}
impl FrameQueries {
    pub(super) fn leading(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        kind: &'static str,
        attempt: timeline::Attempt,
        workload: [u64; 8],
        final_submission: bool,
    ) -> anyhow::Result<wgpu::CommandBuffer> {
        if self.timeline.entries().next().is_some() {
            self.submission = next_submission_id()?;
        }
        self.timeline.begin(
            self.submission,
            kind,
            attempt,
            self.query_count,
            final_submission,
        )?;
        self.timeline.label(workload)?;
        let command = self.marker(device, view, "upload-leading")?;
        if final_submission {
            self.final_transfer_end = Some(self.query_count);
        }
        Ok(command)
    }
    pub(super) fn trailing(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> anyhow::Result<wgpu::CommandBuffer> {
        let first = self.query_count;
        let command = self.marker(device, view, "upload-trailing")?;
        self.timeline.end(first + 1, first)?;
        Ok(command)
    }
    fn marker(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        name: &'static str,
    ) -> anyhow::Result<wgpu::CommandBuffer> {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(name) });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                timestamp_writes: Some(self.pass(name)?),
                ..Default::default()
            });
        }
        Ok(encoder.finish())
    }
    pub(super) fn finish_boundaries(&mut self) -> anyhow::Result<()> {
        if let Some(transfer_end) = self.final_transfer_end {
            let last = self
                .pass_indices
                .last()
                .ok_or_else(|| anyhow::anyhow!("missing final pass"))?[1];
            self.timeline.end(last, transfer_end)?;
        } else {
            anyhow::ensure!(
                self.timeline.entries().next().is_none(),
                "upload transaction missing final submission"
            );
        }
        Ok(())
    }
    pub(super) fn finish_submission(&mut self) -> anyhow::Result<()> {
        if self.timeline.entries().next().is_some() {
            self.timeline.submitted()?;
            if self.final_transfer_end.is_some() {
                self.timeline.complete()?;
            }
        }
        Ok(())
    }
}
pub(super) fn decode_boundaries(
    timeline: &timeline::Timeline,
    ticks: &[u64],
    period: f64,
) -> anyhow::Result<Vec<SubmissionSample>> {
    if timeline.entries().next().is_none() {
        return Ok(Vec::new());
    }
    timeline.complete()?;
    let mut result = Vec::new();
    let mut previous = None;
    for boundary in timeline.entries() {
        let get = |i: u32| {
            ticks
                .get(i as usize)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("missing submission timestamp"))
        };
        let first = get(boundary.first)?;
        let last = get(boundary.last)?;
        let transfer_first = get(boundary.transfer_first)?;
        let transfer_last = get(boundary.transfer_last)?;
        anyhow::ensure!(
            first <= transfer_first
                && transfer_first <= transfer_last
                && transfer_last <= last
                && previous.is_none_or(|p| p <= first),
            "reversed or overlapping GPU submission timestamps"
        );
        previous = Some(last);
        result.push(SubmissionSample {
            submission: boundary.id,
            kind: boundary.kind,
            attempt: boundary.attempt,
            workload: boundary.workload,
            first_tick: first,
            last_tick: last,
            transfer_first_tick: transfer_first,
            transfer_last_tick: transfer_last,
            span_ns: (last - first) as f64 * period,
            transfer_interval_ns: (transfer_last - transfer_first) as f64 * period,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn five_uploads_final_mixed_upload_and_nested_markers_decode_one_span() {
        let mut timeline = timeline::Timeline::default();
        for i in 0..5 {
            timeline
                .begin(i + 1, "world", [1; 7], i as u32 * 4, false)
                .unwrap();
            timeline.end(i as u32 * 4 + 3, i as u32 * 4 + 2).unwrap();
            timeline.submitted().unwrap();
        }
        timeline.begin(6, "final", [1; 7], 20, true).unwrap();
        timeline.end(28, 22).unwrap();
        timeline.submitted().unwrap();
        let mut ticks: Vec<u64> = (0..29).map(|i| i * 10).collect();
        ticks[23] = 260;
        ticks[24] = 230;
        ticks[25] = 240;
        ticks[26] = 250;
        let records = decode_boundaries(&timeline, &ticks, 2.0).unwrap();
        assert_eq!(records.len(), 6);
        assert_eq!(records[0].span_ns, 60.0);
        assert_eq!(records[5].span_ns, 160.0);
        assert_eq!(records[5].transfer_interval_ns, 20.0);
        let markers = extract_scene_markers_at(&mut ticks, 3, Some(24)).unwrap();
        assert_eq!(markers, Some([230, 240, 250]));
        let names = [
            "leading", "trailing", "leading", "trailing", "leading", "trailing", "leading",
            "trailing", "leading", "trailing", "leading", "frame", "suffix",
        ];
        let (_, sum, span) = decode(&names, &ticks, 2.0).unwrap();
        assert_eq!(span, 560.0);
        assert!(
            sum < span,
            "transfer intervals and queue gaps are not pass durations"
        );
    }
    #[test]
    fn truncated_reversed_and_missing_submission_timestamps_fail() {
        let mut timeline = timeline::Timeline::default();
        timeline.begin(1, "final", [1; 7], 0, true).unwrap();
        timeline.end(5, 2).unwrap();
        timeline.submitted().unwrap();
        for ticks in [
            &[0, 1][..],
            &[0, 4, 2, 3, 4, 5][..],
            &[6, 7, 8, 9, 10, 5][..],
        ] {
            assert!(decode_boundaries(&timeline, ticks, 1.0).is_err());
        }
    }
}
