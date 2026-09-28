//! Producer damage stays leased until its native composition is presented.
//! This owns only retry metadata; TerminalCore remains the terminal authority.
use crate::{TerminalPaneRenderState, cpu_alloc};
use datum_terminal_core::Damage;

type Panes = Vec<(String, Vec<Damage>)>;

pub(super) struct PendingTerminalDamage {
    revision: u64,
    panes: Panes,
    storage: cpu_alloc::Scope,
}

impl Default for PendingTerminalDamage {
    fn default() -> Self {
        Self {
            revision: 0,
            panes: Vec::new(),
            storage: cpu_alloc::Scope::new("render-terminal-damage-lease"),
        }
    }
}

impl PendingTerminalDamage {
    pub(super) fn retain(&mut self, revision: u64, panes: &[TerminalPaneRenderState<'_>]) {
        assert!(
            self.panes.is_empty(),
            "restore prior producer lease before replacing it"
        );
        self.revision = revision;
        // All retained requested capacities, strings, and allocation headers
        // belong to this scope, including metadata returned for producer merge.
        self.panes = self.storage.with(|| {
            panes
                .iter()
                .filter(|pane| !pane.damage.is_empty())
                .map(|pane| (pane.session_id.clone(), pane.damage.clone()))
                .collect()
        });
    }

    pub(super) fn restore(&mut self) -> Panes {
        std::mem::take(&mut self.panes)
    }

    pub(super) fn presented(&mut self, revision: u64) {
        if self.revision <= revision {
            self.panes = Vec::new();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_presentation_cannot_retire_newer_terminal_damage() {
        let mut lease = PendingTerminalDamage {
            revision: 7,
            ..Default::default()
        };
        lease.panes = lease
            .storage
            .with(|| vec![("pane".into(), vec![Damage::Full])]);
        lease.presented(6);
        let retry = lease.restore();
        assert_eq!(retry, vec![("pane".into(), vec![Damage::Full])]);
        assert!(lease.storage.usage().payload_bytes > 0);
        assert!(lease.restore().is_empty());
        drop(retry);
        assert_eq!(lease.storage.usage().payload_bytes, 0);
        lease.panes = lease
            .storage
            .with(|| vec![("pane".into(), vec![Damage::Full])]);
        lease.presented(7);
        assert!(lease.restore().is_empty());
        assert_eq!(lease.storage.usage().payload_bytes, 0);
    }
}
