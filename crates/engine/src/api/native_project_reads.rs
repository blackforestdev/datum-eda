//! Native project reads share existing bounded exact-input evidence per engine.
use super::Engine;
use crate::{error::EngineError, substrate::DesignModel};
use std::{path::Path, sync::Arc};

impl Engine {
    pub fn read_native_project(&mut self, root: &Path) -> Result<Arc<DesignModel>, EngineError> {
        self.native_project_reads.resolve(root)
    }

    /// Execute the prepared native batch through the existing canonical writer,
    /// qualifying any reused preimage under its newly acquired writer lease.
    pub fn commit_native_prepared(
        &mut self,
        model: &mut DesignModel,
        root: &Path,
        prepared: super::native_write::PreparedWrite,
    ) -> Result<crate::substrate::CommitReport, EngineError> {
        model.commit_journaled_using_reads(root, prepared.batch, &mut self.native_project_reads)
    }

    pub fn confirm_native_project_durable(
        &mut self,
        root: &Path,
    ) -> Result<Arc<DesignModel>, EngineError> {
        super::native_write::durability::resolve_durable_cached(
            root,
            &mut self.native_project_reads,
        )
    }
}
