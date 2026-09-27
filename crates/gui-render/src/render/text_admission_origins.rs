//! Per-origin counts from the production shaped buffers, without reshaping.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextOriginAdmission {
    pub origin: TextOrigin,
    pub overlay: bool,
    pub counts: TextAdmissionGroup,
}

pub(super) fn collect(
    entries: &[text_buffer_cache::CachedTextBuffer],
    groups: [(&[usize], &[TextRun]); 2],
    keys: &mut StagingVec<CacheKey>,
    budget: &std::sync::Arc<crate::text_gpu::budget::Budget>,
) -> anyhow::Result<StagingVec<TextOriginAdmission>> {
    let capacity = groups[0]
        .1
        .len()
        .checked_add(groups[1].1.len())
        .ok_or_else(|| anyhow::anyhow!("text origin count overflow"))?;
    let mut origins: StagingVec<TextOriginAdmission> = StagingVec::new(capacity, budget)?;
    for (overlay, (indices, runs)) in groups.iter().enumerate() {
        for (index, run) in runs.iter().enumerate() {
            let counts = group(
                entries,
                &indices[index..index + 1],
                std::slice::from_ref(run),
            )?;
            let position = origins
                .iter()
                .position(|o| o.origin == run.origin && o.overlay == (overlay == 1));
            let position = position.unwrap_or_else(|| {
                let position = origins.len();
                origins.push(TextOriginAdmission {
                    origin: run.origin,
                    overlay: overlay == 1,
                    counts: TextAdmissionGroup::default(),
                });
                position
            });
            let total = &mut origins[position].counts;
            total.runs += counts.runs;
            total.layout_rows = total
                .layout_rows
                .checked_add(counts.layout_rows)
                .ok_or_else(|| anyhow::anyhow!("text row count overflow"))?;
            total.shaped_instances = total
                .shaped_instances
                .checked_add(counts.shaped_instances)
                .ok_or_else(|| anyhow::anyhow!("text glyph count overflow"))?;
        }
    }
    for origin in origins.iter_mut() {
        keys.clear();
        let (indices, runs) = groups[usize::from(origin.overlay)];
        for (index, run) in runs
            .iter()
            .enumerate()
            .filter(|(_, run)| run.origin == origin.origin)
        {
            for area in text_buffer_cache::build_text_areas(
                entries,
                &indices[index..index + 1],
                std::slice::from_ref(run),
            ) {
                for row in area.rows {
                    for glyph in row.glyphs {
                        keys.push(glyph.physical((area.left, area.top), area.scale).cache_key);
                    }
                }
            }
        }
        origin.counts.unique_raster_keys = unique(keys);
    }
    Ok(origins)
}
