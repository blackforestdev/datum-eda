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
    /// Prepared producers; upload-only/error attempts do not establish presentation.
    pub consumers: crate::resource_consumers::Consumers,
    pub owner: u64,
    pub attempt: u64,
    pub rendered: Option<bool>,
    pub totals: UploadTotals,
}

/// Latest submitted contribution by one allocation to a renderer attempt.
/// Match owner/attempt to UploadFrame; a different key means zero for that frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationUploadFrame {
    /// Captured shared-stream incidence. Count this allocation's bytes once.
    pub consumers: crate::resource_consumers::Consumers,
    pub owner: u64,
    pub attempt: u64,
    pub source_bytes: u64,
    pub transfer_bytes: u64,
}

#[derive(Default)]
pub(super) struct AllocationUploads {
    pub source_bytes: u64,
    pub transfer_bytes: u64,
    pub latest: Option<AllocationUploadFrame>,
}
impl AllocationUploads {
    #[cfg(test)]
    pub fn record(&mut self, attempt: Option<(u64, u64)>, source: u64, transfer: u64) {
        self.record_consumers(attempt, source, transfer, Default::default());
    }
    pub fn record_consumers(
        &mut self,
        attempt: Option<(u64, u64)>,
        source: u64,
        transfer: u64,
        consumers: crate::resource_consumers::Consumers,
    ) {
        self.source_bytes = self.source_bytes.saturating_add(source);
        self.transfer_bytes = self.transfer_bytes.saturating_add(transfer);
        if let Some((owner, attempt)) = attempt {
            if !self
                .latest
                .is_some_and(|last| last.owner == owner && last.attempt == attempt)
            {
                self.latest = Some(AllocationUploadFrame {
                    consumers: Default::default(),
                    owner,
                    attempt,
                    source_bytes: 0,
                    transfer_bytes: 0,
                });
            }
            let last = self
                .latest
                .as_mut()
                .expect("initialized allocation attempt");
            last.consumers = last.consumers.union(consumers);
            last.source_bytes = last.source_bytes.saturating_add(source);
            last.transfer_bytes = last.transfer_bytes.saturating_add(transfer);
        }
    }
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
            consumers: super::allocation_host::consumers(),
            owner,
            attempt,
            rendered: None,
            totals: UploadTotals::default(),
        });
        self.active = true;
    }

    pub fn record(&mut self, totals: UploadTotals) -> Option<(u64, u64)> {
        self.total.add(totals);
        if self.active {
            self.latest
                .as_mut()
                .expect("active upload attempt")
                .totals
                .add(totals);
            self.latest.map(|frame| (frame.owner, frame.attempt))
        } else {
            None
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocation_uploads_accumulate_chunks_and_distinguish_recovered_attempts() {
        let mut uploads = AllocationUploads::default();
        uploads.record(Some((1, 1)), 4, 8);
        uploads.record(Some((1, 1)), 6, 256);
        assert_eq!(uploads.latest.unwrap().source_bytes, 10);
        assert_eq!(uploads.latest.unwrap().transfer_bytes, 264);
        uploads.record(Some((1, 2)), 3, 256);
        assert_eq!(uploads.latest.unwrap().source_bytes, 3);
        uploads.record(Some((2, 2)), 5, 8);
        assert_eq!(
            uploads.latest.unwrap(),
            AllocationUploadFrame {
                consumers: Default::default(),
                owner: 2,
                attempt: 2,
                source_bytes: 5,
                transfer_bytes: 8
            }
        );
        assert_eq!((uploads.source_bytes, uploads.transfer_bytes), (18, 528));
    }
}
