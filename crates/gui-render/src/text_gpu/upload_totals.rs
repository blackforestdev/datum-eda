//! Submission-boundary totals for the common mapped upload path.
/// Cumulative submitted work, not GPU execution timing or live memory usage.
/// Texture source bytes exclude row padding; scatter metadata is separate from
/// changed buffer payload. Staging capacity includes both mapped and packet buffers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UploadTotals {
    pub batches: u64,
    pub buffer_payload_bytes: u64,
    pub texture_source_bytes: u64,
    pub texture_padding_bytes: u64,
    pub scatter_index_bytes: u64,
    pub staging_capacity_bytes: u64,
    pub buffer_copy_bytes: u64,
    pub buffer_copies: u64,
    pub texture_copies: u64,
    pub scatter_dispatches: u64,
}

impl UploadTotals {
    pub(crate) fn add(&mut self, other: Self) {
        macro_rules! add {
            ($($field:ident),+ $(,)?) => {$(self.$field = self.$field.saturating_add(other.$field);)+};
        }
        add!(
            batches,
            buffer_payload_bytes,
            texture_source_bytes,
            texture_padding_bytes,
            scatter_index_bytes,
            staging_capacity_bytes,
            buffer_copy_bytes,
            buffer_copies,
            texture_copies,
            scatter_dispatches
        );
    }
}

impl crate::Renderer {
    /// This renderer's submitted shared uploads, including cold continuation
    /// chunks. Planning, refusal and cancelled batches do not increment totals.
    /// GPU completion, readback copies and device-internal work are not measured.
    pub fn submitted_upload_totals(&self) -> UploadTotals {
        self.atlas.owner.observer().submitted_upload_totals()
    }
}
