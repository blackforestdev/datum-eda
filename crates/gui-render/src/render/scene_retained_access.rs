use super::*;

pub(crate) fn finish_retained_stroke_batch(batches: &mut Vec<RetainedStrokeBatch>,
    layer_id: Option<String>, start: usize, end: usize) {
    if end > start { batches.push(RetainedStrokeBatch { layer_id, start: start as u32, len: (end - start) as u32 }); }
}

pub(crate) fn finish_retained_quad_batch(batches: &mut Vec<RetainedWorldBatch>,
    layer_id: Option<String>, start: usize, end: usize) {
    if end > start { batches.push(RetainedWorldBatch {
        layer_id, start: (start * 6) as u32, len: ((end - start) * 6) as u32 }); }
}

impl RetainedScene {
    pub fn world_vertices(&self) -> &[Vertex] { &self.world_vertices }
    pub(crate) fn world_strokes(&self) -> &[WorldStrokeInstance] { &self.world_strokes }

    pub fn all_world_ranges(&self) -> Vec<Range<u32>> {
        self.world_batches.iter().map(|b| b.start..b.start + b.len).collect()
    }

    pub(crate) fn all_world_stroke_ranges(&self) -> Vec<Range<u32>> {
        self.world_stroke_batches.iter().map(|b| b.start..b.start + b.len).collect()
    }

    pub(crate) fn visible_world_stroke_ranges(
        &self,
        state: &ReviewWorkspaceState,
    ) -> Vec<Range<u32>> {
        if !authored_visible(state) { return Vec::new(); }
        self.world_stroke_batches.iter()
            .filter(|b| b.layer_id.as_deref().is_none_or(|id| layer_visible(state, id)))
            .map(|b| b.start..b.start + b.len).collect()
    }

    pub(crate) fn visible_world_ranges(
        &self,
        state: &ReviewWorkspaceState,
    ) -> Vec<Range<u32>> {
        if !authored_visible(state) { return Vec::new(); }
        self.world_batches.iter()
            .filter(|b| b.layer_id.as_deref().is_none_or(|id| layer_visible(state, id)))
            .map(|b| b.start..b.start + b.len).collect()
    }
}
