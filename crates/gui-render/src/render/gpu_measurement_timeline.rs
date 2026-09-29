//! Fixed-capacity submission lineage; no missing upload can become a zero sample.
const SUBMISSIONS: usize = 6;

/// Session owner, attempt, revision, preparation, host, device and configuration.
pub(crate) type Attempt = [u64; 7];
#[derive(Clone, Copy, Debug)]
pub(crate) struct Boundary {
    pub id: u64,
    pub kind: &'static str,
    pub attempt: Attempt,
    pub workload: [u64; 8],
    pub first: u32,
    pub last: u32,
    pub transfer_first: u32,
    pub transfer_last: u32,
    pub submitted: bool,
}
#[derive(Clone, Default)]
pub(crate) struct Timeline {
    entries: [Option<Boundary>; SUBMISSIONS],
    count: usize,
    final_started: bool,
}
impl Timeline {
    pub(crate) fn entries(&self) -> impl Iterator<Item = &Boundary> {
        self.entries[..self.count].iter().flatten()
    }
    pub(crate) fn begin(
        &mut self,
        id: u64,
        kind: &'static str,
        attempt: Attempt,
        first: u32,
        final_submission: bool,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(!self.final_started, "submission after final boundary");
        anyhow::ensure!(
            self.count
                < if final_submission {
                    SUBMISSIONS
                } else {
                    SUBMISSIONS - 1
                },
            "GPU continuation limit exceeded"
        );
        if let Some(previous) = self.entries().last() {
            anyhow::ensure!(
                previous.submitted
                    && previous.id < id
                    && previous.last.checked_add(1) == Some(first),
                "missing, duplicate or overlapping GPU submission boundary"
            );
            // Preparation can advance between upload-only turns. Earlier work
            // stays in this transaction with its original immutable identity.
            // The fixed-target observer still rejects foreign owners or targets.
            for field in [0, 4, 5, 6] {
                anyhow::ensure!(
                    previous.attempt[field] == attempt[field],
                    "GPU transaction owner or target lineage changed at field {field}: previous={:?} current={attempt:?}",
                    previous.attempt
                );
            }
            for field in [1, 2, 3] {
                anyhow::ensure!(
                    previous.attempt[field] <= attempt[field],
                    "GPU transaction lineage reversed at field {field}: previous={:?} current={attempt:?}",
                    previous.attempt
                );
            }
        }
        self.entries[self.count] = Some(Boundary {
            id,
            kind,
            attempt,
            workload: [0; 8],
            first,
            last: first,
            transfer_first: first + 1,
            transfer_last: first + 1,
            submitted: false,
        });
        self.count += 1;
        self.final_started = final_submission;
        Ok(())
    }
    pub(crate) fn label(&mut self, tag: [u64; 8]) -> anyhow::Result<()> {
        anyhow::ensure!(self.count != 0, "workload label without submission");
        if self.count > 1 {
            anyhow::ensure!(
                self.entries[self.count - 2].as_ref().unwrap().workload[0] == tag[0],
                "GPU submission workload epoch changed"
            );
        }
        self.entries[self.count - 1].as_mut().unwrap().workload = tag;
        Ok(())
    }
    pub(crate) fn end(&mut self, last: u32, transfer_last: u32) -> anyhow::Result<()> {
        let current = self.entries[..self.count]
            .last_mut()
            .and_then(Option::as_mut)
            .ok_or_else(|| anyhow::anyhow!("GPU submission missing leading boundary"))?;
        anyhow::ensure!(
            !current.submitted
                && last > current.first
                && transfer_last > current.transfer_first
                && transfer_last <= last,
            "GPU submission missing upload or trailing boundary"
        );
        current.last = last;
        current.transfer_last = transfer_last;
        Ok(())
    }
    pub(crate) fn submitted(&mut self) -> anyhow::Result<()> {
        let current = self.entries[..self.count]
            .last_mut()
            .and_then(Option::as_mut)
            .ok_or_else(|| anyhow::anyhow!("unmeasured GPU submission"))?;
        anyhow::ensure!(
            !current.submitted
                && current.last > current.first
                && current.transfer_last > current.transfer_first,
            "duplicate or incomplete GPU submission"
        );
        current.submitted = true;
        Ok(())
    }
    pub(crate) fn complete(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.final_started && self.count != 0 && self.entries().all(|e| e.submitted),
            "incomplete GPU frame submission manifest"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ATTEMPT: Attempt = [1, 2, 3, 4, 5, 6, 7];
    #[test]
    fn all_continuations_and_final_upload_require_boundaries() {
        let mut timeline = Timeline::default();
        for i in 0..5 {
            timeline
                .begin(i + 1, "world", ATTEMPT, i as u32 * 4, false)
                .unwrap();
            assert!(timeline.submitted().is_err());
            timeline.end(i as u32 * 4 + 3, i as u32 * 4 + 2).unwrap();
            timeline.submitted().unwrap();
            assert!(timeline.submitted().is_err());
        }
        assert!(timeline.begin(6, "glyph", ATTEMPT, 20, false).is_err());
        timeline.begin(6, "final", ATTEMPT, 20, true).unwrap();
        assert!(
            timeline.end(25, 21).is_err(),
            "omitted final upload interval"
        );
        assert!(timeline.complete().is_err());
        timeline.end(25, 22).unwrap();
        timeline.submitted().unwrap();
        timeline.complete().unwrap();
        assert!(timeline.begin(7, "final", ATTEMPT, 26, true).is_err());
    }
    #[test]
    fn preparation_advances_without_losing_submitted_or_foreign_lineage() {
        let mut timeline = Timeline::default();
        timeline.begin(1, "terminal", ATTEMPT, 0, false).unwrap();
        assert!(timeline.begin(2, "final", ATTEMPT, 4, true).is_err());
        timeline.end(3, 2).unwrap();
        timeline.submitted().unwrap();
        for field in [0, 4, 5, 6] {
            let mut wrong = ATTEMPT;
            wrong[field] += 1;
            assert!(timeline.clone().begin(2, "final", wrong, 4, true).is_err());
        }
        for field in [1, 2, 3] {
            let mut reversed = ATTEMPT;
            reversed[field] -= 1;
            assert!(
                timeline
                    .clone()
                    .begin(2, "final", reversed, 4, true)
                    .is_err()
            );
        }
        assert!(
            timeline
                .clone()
                .begin(1, "final", ATTEMPT, 4, true)
                .is_err()
        );
        assert!(
            timeline
                .clone()
                .begin(2, "final", ATTEMPT, 3, true)
                .is_err()
        );
        let mut next = ATTEMPT;
        next[1] += 1;
        next[2] += 1;
        next[3] += 1;
        timeline.begin(2, "final", next, 4, true).unwrap();
        let attempts: Vec<_> = timeline.entries().map(|entry| entry.attempt).collect();
        assert_eq!(attempts, [ATTEMPT, next]);
        timeline.end(7, 6).unwrap();
        timeline.submitted().unwrap();
        timeline.complete().unwrap();
    }
}
