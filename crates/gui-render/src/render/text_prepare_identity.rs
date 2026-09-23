//! Placement and paint identity, separate from shaping/layout ownership.
use super::*;

pub(super) fn text_prepare_signature(
    indices: &[usize],
    runs: &[TextRun],
    width: u32,
    height: u32,
) -> TextPrepareSignature {
    let mut span_colors = Vec::with_capacity(runs.iter().map(|run| run.rich_spans.len()).sum());
    span_colors.extend(runs.iter().enumerate().flat_map(|(index, run)| {
        run.rich_spans
            .iter()
            .map(move |span| (index, span.color.map(f32::to_bits)))
    }));
    TextPrepareSignature {
        span_colors,
        width,
        height,
        runs: indices
            .iter()
            .zip(runs.iter())
            .map(|(index, run)| TextPrepareRunKey {
                buffer_index: *index,
                x_bits: run.x.to_bits(),
                y_bits: run.y.to_bits(),
                color_bits: run.color.map(f32::to_bits),
                clip_bounds: run.clip_bounds.map(|rect| RectBits {
                    x_bits: rect.x.to_bits(),
                    y_bits: rect.y.to_bits(),
                    width_bits: rect.width.to_bits(),
                    height_bits: rect.height.to_bits(),
                }),
            })
            .collect(),
    }
}

/// Signature storage stays charged until reuse identity retires, including old/new overlap.
pub(super) struct AdmittedSignature {
    pub(super) value: TextPrepareSignature,
    _permits: [crate::text_gpu::budget::Permit; 2],
    pub(super) bytes: u64,
}

pub(super) fn admitted_signature(
    indices: &[usize],
    runs: &[TextRun],
    width: u32,
    height: u32,
    host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
) -> anyhow::Result<AdmittedSignature> {
    use crate::text_gpu::{budget::staging_process, staging_vec::StagingVec};
    let spans = runs
        .iter()
        .try_fold(0usize, |count, run| count.checked_add(run.rich_spans.len()))
        .ok_or_else(|| anyhow::anyhow!("text span count overflow"))?;
    let bytes = StagingVec::<(usize, [u32; 3])>::capacity_bytes(spans)?
        .checked_add(StagingVec::<TextPrepareRunKey>::capacity_bytes(
            indices.len().min(runs.len()),
        )?)
        .ok_or_else(|| anyhow::anyhow!("text signature storage overflow"))?;
    let permits = [host.reserve(bytes)?, staging_process().reserve(bytes)?];
    let value = super::text_prepare_signature(indices, runs, width, height);
    anyhow::ensure!(
        value.span_colors.capacity() == spans
            && value.runs.capacity() == indices.len().min(runs.len()),
        "text signature capacity differs from admission"
    );
    Ok(AdmittedSignature {
        value,
        _permits: permits,
        bytes,
    })
}
