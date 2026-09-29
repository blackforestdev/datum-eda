//! Opt-in causal workload tags. Log arrival time never determines GPU ownership.
//! Epoch, pending phase bits, then the latest demand sequence for each phase.
pub(crate) type Tag = [u64; 8];
#[derive(Default)]
pub(super) struct Workload {
    context: [u64; 3],
    scoped: bool,
    previous_context: [u64; 3],
    invalid: bool,
    phases: [[u64; 2]; 6],
}
impl Workload {
    pub(super) fn context(&mut self, context: [u64; 3]) -> anyhow::Result<()> {
        anyhow::ensure!(
            context[0] != 0 && (1..=6).contains(&context[1]),
            "invalid workload epoch or phase"
        );
        anyhow::ensure!(
            self.context[0] == 0 || self.context[0] == context[0],
            "workload epoch changed within render session"
        );
        anyhow::ensure!(
            context[2] >= self.context[2],
            "workload demand sequence reversed"
        );
        self.previous_context = self.context;
        self.context = context;
        self.scoped = true;
        Ok(())
    }
    pub(super) fn end_context(&mut self, retain: bool) {
        if !retain {
            self.context = self.previous_context;
        }
        self.scoped = false;
    }
    pub(super) fn changed(&mut self, revision: u64) {
        if self.context[0] != 0 && !self.scoped {
            self.invalid = true;
        }
        if self.context[0] != 0 {
            self.phases[self.context[1] as usize - 1] = [revision, self.context[2]];
        }
    }
    pub(super) fn pending(&self, acknowledged: u64) -> Tag {
        let mut tag = [0; 8];
        if self.invalid {
            return tag;
        }
        tag[0] = self.context[0];
        for (phase, &[revision, demand]) in self.phases.iter().enumerate() {
            if revision > acknowledged {
                tag[1] |= 1 << phase;
                tag[2 + phase] = demand;
            }
        }
        // Exposure-only demand belongs to its actual native event, while pending
        // active work remains active through still/drain/close dispatch.
        if tag[1] == 0 && tag[0] != 0 {
            let phase = self.context[1] as usize - 1;
            tag[1] = 1 << phase;
            tag[2 + phase] = self.context[2];
        }
        tag
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_demand_survives_late_drain_and_close_until_acknowledged() {
        let mut workload = Workload::default();
        workload.context([7, 3, 20]).unwrap();
        workload.changed(2);
        let active = workload.pending(1);
        workload.context([7, 5, 21]).unwrap();
        assert_eq!(
            workload.pending(1),
            active,
            "late active receipt is not drain work"
        );
        workload.context([7, 6, 22]).unwrap();
        workload.changed(3);
        assert_eq!(workload.pending(1)[1], (1 << 2) | (1 << 5));
        assert_eq!(
            workload.pending(2)[1],
            1 << 5,
            "only acknowledged active work can leave close-only demand"
        );
        assert_eq!(
            workload.pending(3)[1],
            1 << 5,
            "exposure records explicit current phase"
        );
    }
    #[test]
    fn unchanged_native_round_restores_context_but_changed_round_owns_demand() {
        let mut workload = Workload::default();
        workload.context([7, 3, 1]).unwrap();
        workload.changed(2);
        workload.end_context(true);
        workload.context([7, 4, 2]).unwrap();
        workload.end_context(false);
        assert_eq!(workload.pending(2)[4], 1);
        workload.context([7, 4, 2]).unwrap();
        workload.changed(3);
        workload.end_context(true);
        assert_eq!(workload.pending(1)[1], 12);
        assert_eq!(workload.pending(2)[5], 2);
    }
    #[test]
    fn unscoped_background_change_cannot_inherit_a_native_demand() {
        let mut workload = Workload::default();
        workload.context([7, 3, 20]).unwrap();
        workload.changed(2);
        workload.end_context(true);
        assert_eq!(workload.pending(1)[1], 4);
        workload.changed(3);
        assert_eq!(workload.pending(1), [0; 8]);
        workload.context([7, 6, 21]).unwrap();
        assert_eq!(
            workload.pending(1),
            [0; 8],
            "later demand cannot bless unattributed work"
        );
    }
    #[test]
    fn missing_reversed_or_changed_epoch_cannot_be_attributed() {
        let mut workload = Workload::default();
        assert_eq!(workload.pending(0), [0; 8]);
        assert!(workload.context([0, 3, 1]).is_err());
        assert!(workload.context([1, 7, 1]).is_err());
        workload.context([1, 2, 2]).unwrap();
        assert!(workload.context([2, 3, 3]).is_err());
        assert!(workload.context([1, 3, 1]).is_err());
    }
}
