//! One immutable authority's completed physical Run query, shared by acquisition and inspection.
use super::{BoardRunMembership, Class, Point, RunSelectionIdentity, SelectionResolutionError};
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct RunQueryCache {
    completed: Mutex<Option<(RunSelectionIdentity, Option<Point>, BoardRunMembership)>>,
    #[cfg(test)]
    acquisitions: std::sync::atomic::AtomicUsize,
}

impl RunQueryCache {
    pub(super) fn acquire(
        &self,
        origin: RunSelectionIdentity,
        hit: Option<Point>,
        query: impl FnOnce(
            RunSelectionIdentity,
        ) -> Result<
            (RunSelectionIdentity, BoardRunMembership),
            SelectionResolutionError,
        >,
    ) -> Result<RunSelectionIdentity, SelectionResolutionError> {
        // Same immutable source basis, origin and hit reproduce the completed Run.
        // A hitless inspection entry cannot infer a Zone's region for acquisition.
        if let Ok(completed) = self.completed.lock()
            && let Some((run, cached_hit, _)) = completed.as_ref()
            && run.origin == origin.origin
            && *cached_hit == hit
            && (origin.origin.class != Class::Zone || hit.is_some())
        {
            return Ok(run.clone());
        }
        #[cfg(test)]
        self.acquisitions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (run, result) = query(origin)?;
        self.remember(run.clone(), hit, result);
        Ok(run)
    }

    pub(super) fn resolve(
        &self,
        run: &RunSelectionIdentity,
        query: impl FnOnce() -> Result<BoardRunMembership, SelectionResolutionError>,
    ) -> Result<BoardRunMembership, SelectionResolutionError> {
        if let Ok(completed) = self.completed.lock()
            && let Some((key, _, value)) = completed.as_ref()
            && key == run
        {
            // Consumption preserves the acquisition hit. Clearing it here would force
            // every subsequent menu selection to repeat the expensive graph query.
            return Ok(value.clone());
        }
        let result = query()?;
        self.remember(run.clone(), None, result.clone());
        Ok(result)
    }

    fn remember(&self, run: RunSelectionIdentity, hit: Option<Point>, result: BoardRunMembership) {
        if let Ok(mut completed) = self.completed.lock() {
            *completed = Some((run, hit, result));
        }
    }

    #[cfg(test)]
    pub(super) fn acquisitions(&self) -> usize {
        self.acquisitions.load(std::sync::atomic::Ordering::Relaxed)
    }
}
