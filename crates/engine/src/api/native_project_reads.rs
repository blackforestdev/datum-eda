//! Native project reads share existing bounded exact-input evidence per engine.
use super::Engine;
use crate::{error::EngineError, substrate::DesignModel};
use std::{path::Path, sync::Arc};

impl Engine {
    pub fn read_native_project(&mut self, root: &Path) -> Result<Arc<DesignModel>, EngineError> {
        self.native_project_reads.resolve(root)
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
