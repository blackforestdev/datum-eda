use crate::error::EngineError;
use crate::import::{ImportKind, ImportReport};
use crate::pool::Pool;

mod parser;
mod pool_builder;
mod xml_helpers;

use parser::parse_library;
use pool_builder::pool_counts;
use xml_helpers::extract_library_name;

/// Import a standalone Eagle library (`.lbr`) into pool objects.
///
/// M0 scope only:
/// - symbols -> Unit + Symbol
/// - packages -> Package + Padstack + Pad
/// - devicesets/gates -> Entity + Gate
/// - devices/connects -> Part + pad_map
pub fn import_library_str(xml: &str) -> Result<Pool, EngineError> {
    parse_library(xml)
}

pub fn import_library_file(path: &std::path::Path) -> Result<(Pool, ImportReport), EngineError> {
    let xml = std::fs::read_to_string(path)?;
    let library_name = extract_library_name(&xml)?;
    let pool = import_library_str(&xml)?;
    let report = ImportReport::new(ImportKind::EagleLibrary, path, pool_counts(&pool))
        .with_metadata("library_name", library_name);
    Ok((pool, report))
}

pub fn import_board_file(path: &std::path::Path) -> Result<ImportReport, EngineError> {
    Err(EngineError::Import(format!(
        "Eagle board import is not implemented yet; Eagle design import is secondary in M1: {}",
        path.display()
    )))
}

pub fn import_schematic_file(path: &std::path::Path) -> Result<ImportReport, EngineError> {
    Err(EngineError::Import(format!(
        "Eagle schematic import is not implemented yet; Eagle design import is secondary in M1: {}",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    pub(super) const SIMPLE_EAGLE_LIBRARY: &str =
        include_str!("../../../testdata/import/eagle/simple-opamp.lbr");
    pub(super) const DUAL_GATE_EAGLE_LIBRARY: &str =
        include_str!("../../../testdata/import/eagle/dual-nand.lbr");

    pub(super) fn fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("testdata/import/eagle");
        path.push(name);
        path
    }

    pub(super) fn corpus_fixture_paths() -> Vec<PathBuf> {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("testdata/import/eagle");

        let mut paths: Vec<_> = std::fs::read_dir(&root)
            .expect("fixture directory should exist")
            .map(|entry| entry.expect("fixture entry should read").path())
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("lbr"))
            .collect();
        paths.sort();
        paths
    }

    pub(super) fn golden_path_for_fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/golden/eagle")
            .join(format!("{name}.json"))
    }

    #[path = "mod_tests_import_eagle.rs"]
    mod import_eagle;
}
