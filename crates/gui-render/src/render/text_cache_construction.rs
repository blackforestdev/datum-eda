//! Admission for key copies and cache index/entry allocations before construction.
use super::*;

pub(super) struct PreparedKey {
    pub key: TextBufferKey,
    pub _construction: Option<budget::Construction>,
}

fn copy_text(text: &str) -> anyhow::Result<String> {
    let mut result = String::new();
    result.try_reserve_exact(text.len())?;
    anyhow::ensure!(
        result.capacity() == text.len(),
        "text key capacity differs from admission"
    );
    result.push_str(text);
    Ok(result)
}

pub(super) fn key(
    run: &TextRun,
    width: u32,
    height: u32,
    owner: Option<&budget::Owner>,
) -> anyhow::Result<PreparedKey> {
    let capacity = crate::text_gpu::staging_vec::StagingVec::<TextBufferSpanKey>::capacity_bytes(
        run.rich_spans.len(),
    )?;
    let bytes = run.rich_spans.iter().try_fold(
        (capacity as usize)
            .checked_add(capacity_bytes::<u8>(shaping_text(run).len()))
            .ok_or_else(|| anyhow::anyhow!("text key byte overflow"))?,
        |sum, span| {
            sum.checked_add(capacity_bytes::<u8>(span.text.len()))
                .ok_or_else(|| anyhow::anyhow!("text key byte overflow"))
        },
    )?;
    let construction = owner.map(|owner| owner.reserve(bytes)).transpose()?;
    let mut spans = Vec::new();
    spans.try_reserve_exact(run.rich_spans.len())?;
    anyhow::ensure!(
        spans.capacity() == run.rich_spans.len(),
        "rich key capacity differs from admission"
    );
    for span in &run.rich_spans {
        spans.push(TextBufferSpanKey {
            text: copy_text(&span.text)?,
            bold: span.bold,
            italic: span.italic,
        });
    }
    let (width_px, height_px) = text_buffer_extent(run, width, height);
    Ok(PreparedKey {
        key: TextBufferKey {
            text: copy_text(shaping_text(run))?,
            rich_spans: spans,
            size_bits: run.size.to_bits(),
            face: run.face,
            width_px,
            height_px,
        },
        _construction: construction,
    })
}

pub(super) fn reserve_slot<T>(
    values: &mut Vec<T>,
    lease: &mut Option<budget::Construction>,
    owner: Option<&budget::Owner>,
) -> anyhow::Result<()> {
    if values.len() < values.capacity() {
        return Ok(());
    }
    let needed = values
        .len()
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("text cache index overflow"))?;
    let allocate = |count| -> anyhow::Result<(Vec<T>, Option<budget::Construction>)> {
        let bytes = crate::text_gpu::staging_vec::StagingVec::<T>::capacity_bytes(count)?;
        let reservation = owner.map(|o| o.reserve(bytes as usize)).transpose()?;
        let mut storage = Vec::new();
        storage.try_reserve_exact(count)?;
        anyhow::ensure!(
            storage.capacity() == count,
            "text cache metadata differs from admission"
        );
        Ok((storage, reservation))
    };
    let grown = needed.max(values.capacity().saturating_mul(2));
    let (mut replacement, reservation) = allocate(grown).or_else(|error| {
        if grown == needed {
            Err(error)
        } else {
            allocate(needed)
        }
    })?;
    replacement.append(values);
    *values = replacement;
    *lease = reservation;
    Ok(())
}
