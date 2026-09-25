//! GUI-owned retention policy around the authoritative engine read resolver.
//! Allocation ownership stays at the executable boundary, not in the engine.
use datum_gui_render::cpu_alloc::{Scope, Usage};
use eda_engine::error::EngineError;
use eda_engine::substrate::{DesignModel, ProjectReadCache};
use std::{path::Path, sync::Arc};

// A local retention ceiling, not a replacement for aggregate PM045 budgets.
// Optional input encoding is separately limited to 8 MiB by the engine.
const RETAINED_BYTES: u64 = 16 * 1024 * 1024;

pub(super) struct ReadCache {
    cache: ProjectReadCache,
    owner: Scope,
}

impl Default for ReadCache {
    fn default() -> Self {
        Self {
            cache: ProjectReadCache::default(),
            owner: Scope::new("project-preferences-resolver"),
        }
    }
}

impl ReadCache {
    pub(super) fn resolve(&mut self, root: &Path) -> Result<Arc<DesignModel>, EngineError> {
        self.resolve_with_limit(root, RETAINED_BYTES)
    }

    fn resolve_with_limit(
        &mut self,
        root: &Path,
        limit: u64,
    ) -> Result<Arc<DesignModel>, EngineError> {
        // Resolution and scanning are synchronous. This captures all Rust heap
        // keys, model payload/capacity, IO/parser scratch and tracking prefixes.
        // Returned Arcs keep their attribution until the final reader drops them.
        let result = self.owner.with(|| self.cache.resolve(root));
        let usage = self.owner.usage();
        let retained = accounted_bytes(&usage);
        let evicted = !usage.allocator_installed || retained > limit;
        if evicted {
            self.cache = ProjectReadCache::default();
        }
        crate::append_gui_verbose_diagnostic_line(|| {
            format!(
                "project preferences resolver ownership retained_before_policy={retained} limit={limit} evicted={evicted} usage={usage:?} after_policy={:?}",
                self.owner.usage()
            )
        });
        // An over-limit output remains available to this read, but never becomes
        // retained cache state. Eviction doesn't change authority or diagnostics.
        result
    }
}

fn accounted_bytes(usage: &Usage) -> u64 {
    usage
        .payload_bytes
        .saturating_add(usage.tracking_bytes)
        .saturating_add(usage.owner_metadata_bytes)
        .saturating_add(std::mem::size_of::<ReadCache>() as u64)
}

#[cfg(test)]
#[path = "project_preferences_read_cache_tests.rs"]
mod tests;
