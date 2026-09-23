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

/// One shared render entry attempt, including upload-only continuation or failure.
/// `rendered` is None on error; true means encoded/submitted, not presented.
/// The owner/attempt pair is unique across renderer replacement. Observation is
/// bounded to the latest attempt; callers archive receipts when needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UploadFrame {
    pub owner: u64,
    pub attempt: u64,
    pub rendered: Option<bool>,
    pub totals: UploadTotals,
}

#[derive(Default)]
pub(super) struct Accounting {
    pub total: UploadTotals,
    latest: Option<UploadFrame>,
    active: bool,
}

impl Accounting {
    pub fn begin(&mut self, owner: u64) {
        assert!(!self.active, "nested renderer upload attempt");
        let attempt = self.latest.map_or(1, |last| {
            last.attempt
                .checked_add(1)
                .expect("upload attempt exhausted")
        });
        self.latest = Some(UploadFrame {
            owner,
            attempt,
            rendered: None,
            totals: UploadTotals::default(),
        });
        self.active = true;
    }

    pub fn record(&mut self, totals: UploadTotals) {
        self.total.add(totals);
        if self.active {
            self.latest
                .as_mut()
                .expect("active upload attempt")
                .totals
                .add(totals);
        }
    }

    pub fn finish(&mut self, rendered: Option<bool>) {
        assert!(self.active, "finish active upload attempt");
        self.latest
            .as_mut()
            .expect("active upload attempt")
            .rendered = rendered;
        self.active = false;
    }

    pub fn latest(&self) -> Option<UploadFrame> {
        self.latest
    }
}

impl crate::Renderer {
    /// Submitted uploads from the latest render attempt, also available on error
    /// and upload-only yield. Includes glyph/world/terminal continuation batches.
    /// Offscreen callers use the same path; capture readback is excluded.
    pub fn last_upload_frame(&self) -> Option<UploadFrame> {
        self.atlas.owner.last_upload_frame()
    }
}
