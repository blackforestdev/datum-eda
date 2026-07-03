//! Thin shim over the engine-owned guard insertion in
//! `eda_engine::api::native_write::guards`. The logic that used to live here
//! moved into the engine so every surface (CLI, daemon, GUI, MCP) shares one
//! guard-insertion path; these wrappers only preserve the CLI-local
//! `anyhow::Result` signatures the existing callsites use.

use anyhow::Result;

use eda_engine::api::native_write::guards;
use eda_engine::substrate::{DesignModel, OperationBatch};

pub(crate) fn guarded_operation_batch(
    model: &DesignModel,
    batch: OperationBatch,
) -> Result<OperationBatch> {
    Ok(guards::guarded_operation_batch(model, batch)?)
}
