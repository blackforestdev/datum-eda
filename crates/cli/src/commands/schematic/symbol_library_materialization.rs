//! Compatibility CLI boundary over engine-owned qualified library materialization.
use super::*;
use eda_engine::api::native_write::schematic_library as library;
use eda_engine::substrate::ProjectResolver;
pub(crate) use library::{PoolSymbolBindingResolution, PoolSymbolComponentBinding};

pub(crate) fn materialize_pool_symbol_pins(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<Vec<SymbolPin>> {
    if lib_id.and_then(|id| Uuid::parse_str(id).ok()).is_none() {
        return Ok(Vec::new());
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    Ok(library::materialize_pool_symbol_pins(&model, lib_id)?)
}
pub(crate) fn resolve_pool_symbol_component_binding(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<PoolSymbolBindingResolution> {
    let Some(id) = lib_id else {
        return Ok(PoolSymbolBindingResolution {
            binding: None,
            status: "no_lib_id",
            diagnostics: Vec::new(),
        });
    };
    if Uuid::parse_str(id).is_err() {
        return Ok(PoolSymbolBindingResolution {
            binding: None,
            status: "compatibility_lib_id",
            diagnostics: vec![format!(
                "lib_id {id} is not a pool symbol UUID; placed as compatibility identifier"
            )],
        });
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    Ok(library::resolve_pool_symbol_component_binding(
        &model, lib_id,
    )?)
}
