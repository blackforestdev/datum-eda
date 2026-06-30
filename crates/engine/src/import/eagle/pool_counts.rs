use crate::import::ImportObjectCounts;
use crate::pool::Pool;

pub(super) fn pool_counts(pool: &Pool) -> ImportObjectCounts {
    ImportObjectCounts {
        units: pool.units.len(),
        symbols: pool.symbols.len(),
        entities: pool.entities.len(),
        padstacks: pool.padstacks.len(),
        packages: pool.packages.len(),
        parts: pool.parts.len(),
    }
}
