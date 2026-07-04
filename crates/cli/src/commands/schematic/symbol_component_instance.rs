use super::symbol_library_materialization::PoolSymbolComponentBinding;
use eda_engine::api::native_write::schematic_symbols::PlacedSymbolPartBinding;

/// Convert a resolved pool-symbol component binding into the engine facade's
/// placed-symbol part binding (present only when the binding resolved to a
/// pool part). The engine's `build_place_schematic_symbol` authors the
/// component-instance operation from this.
pub(crate) fn part_binding_for_pool_symbol(
    binding: &PoolSymbolComponentBinding,
) -> Option<PlacedSymbolPartBinding> {
    binding.part.as_ref().map(|part| PlacedSymbolPartBinding {
        pool_symbol_id: binding.symbol_id,
        part_id: part.part_id,
    })
}
