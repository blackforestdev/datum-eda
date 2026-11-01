//! Reusable source discovery, never canonical library storage or write authority.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LibrarySourceSymbol {
    pub path: PathBuf,
    pub name: String,
    pub label: String,
    pub description: String,
}

#[derive(Clone)]
struct CachedFile {
    stamp: (u64, SystemTime),
    symbols: Vec<LibrarySourceSymbol>,
}

#[derive(Default)]
pub struct SourceCatalog {
    pub symbols: Vec<LibrarySourceSymbol>,
    pub notices: Vec<String>,
}

/// The release contains these native records; this never reads a KiCad install.
pub fn bundled() -> Result<SourceCatalog, String> {
    use eda_engine::import::kicad_library::bundle_format::BundleEntryOutcome;
    let manifest = eda_engine::import::library_bundle::catalog().map_err(|e| e.to_string())?;
    let mut result = SourceCatalog::default();
    result.notices.push(format!(
        "{} available symbols · {} not yet supported",
        manifest.converted, manifest.refused
    ));
    for library in &manifest.libraries {
        let prefix = library
            .source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Library");
        for entry in &library.symbols {
            let suffix = if matches!(entry.outcome, BundleEntryOutcome::Refused { .. }) {
                " · unavailable"
            } else {
                ""
            };
            result.symbols.push(LibrarySourceSymbol {
                path: library.source.clone(),
                name: entry.name.clone(),
                label: format!("{prefix}:{}{suffix}", entry.name),
                description: entry.description.clone().unwrap_or_default(),
            });
        }
    }
    Ok(result)
}

type Cache = BTreeMap<PathBuf, CachedFile>;
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn stamp(path: &Path) -> Result<(u64, SystemTime), String> {
    let metadata = path.metadata().map_err(|e| e.to_string())?;
    Ok((
        metadata.len(),
        metadata.modified().map_err(|e| e.to_string())?,
    ))
}

fn file_symbols(path: &Path) -> Result<Vec<LibrarySourceSymbol>, String> {
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    let before = stamp(&path)?;
    let cache = CACHE.get_or_init(Default::default);
    if let Some(entry) = cache.lock().map_err(|e| e.to_string())?.get(&path)
        && entry.stamp == before
    {
        return Ok(entry.symbols.clone());
    }
    let library = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Library");
    let symbols: Vec<_> = eda_engine::import::kicad_library::catalog(&path)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|entry| LibrarySourceSymbol {
            path: path.clone(),
            label: format!("{library}:{}", entry.name),
            name: entry.name,
            description: entry.description.unwrap_or_default(),
        })
        .collect();
    if stamp(&path)? != before {
        return Err("Library changed during discovery; refresh the source".into());
    }
    cache.lock().map_err(|e| e.to_string())?.insert(
        path,
        CachedFile {
            stamp: before,
            symbols: symbols.clone(),
        },
    );
    Ok(symbols)
}

/// Directory discovery is shallow: installed KiCad libraries are named files.
/// A bad file is reported without making every other library inaccessible.
pub fn discover(source: &Path) -> Result<SourceCatalog, String> {
    let paths = if source.is_dir() {
        let mut paths = std::fs::read_dir(source)
            .map_err(|e| e.to_string())?
            .map(|entry| entry.map(|entry| entry.path()).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|path| path.extension().is_some_and(|ext| ext == "kicad_sym"));
        paths.sort();
        paths
    } else {
        vec![source.to_owned()]
    };
    let mut result = SourceCatalog::default();
    for path in paths {
        match file_symbols(&path) {
            Ok(symbols) => result.symbols.extend(symbols),
            Err(reason) => result.notices.push(format!("{}: {reason}", path.display())),
        }
    }
    if result.symbols.is_empty() && !result.notices.is_empty() {
        return Err(result.notices.join("\n"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_symbol_catalog_preserves_source_identity_across_libraries() {
        let root = Path::new("/usr/share/kicad/symbols");
        if !root.is_dir() {
            eprintln!("Installed KiCad catalog unavailable; actual-source test skipped");
            return;
        }
        let catalog = discover(root).unwrap();
        assert!(catalog.notices.is_empty(), "{:?}", catalog.notices);
        eprintln!(
            "Discovered {} installed KiCad symbols",
            catalog.symbols.len()
        );
        for (library, name) in [
            ("Device", "R"),
            ("Device", "C"),
            ("Connector_Generic", "Conn_01x02"),
            ("Amplifier_Operational", "LM741"),
        ] {
            let expected = root
                .join(format!("{library}.kicad_sym"))
                .canonicalize()
                .unwrap();
            let entry = catalog
                .symbols
                .iter()
                .find(|entry| entry.label == format!("{library}:{name}"))
                .unwrap_or_else(|| panic!("Missing installed symbol {library}:{name}"));
            assert_eq!(entry.name, name);
            assert_eq!(entry.path, expected);
        }
        assert_eq!(discover(root).unwrap().symbols, catalog.symbols);
    }

    #[test]
    fn catalog_reuses_sources_but_refreshes_changed_and_removed_files() {
        let root =
            std::env::temp_dir().join(format!("datum-source-catalog-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("Device.kicad_sym");
        std::fs::write(
            &path,
            "(kicad_symbol_lib (symbol \"R\" (property \"Description\" \"Resistor\")))",
        )
        .unwrap();
        let first = discover(&root).unwrap();
        assert_eq!(first.symbols[0].label, "Device:R");
        assert_eq!(first.symbols[0].description, "Resistor");
        assert_eq!(discover(&root).unwrap().symbols, first.symbols);
        std::fs::write(&path, "(kicad_symbol_lib (symbol \"Capacitor\"))").unwrap();
        assert_eq!(discover(&root).unwrap().symbols[0].name, "Capacitor");
        std::fs::remove_file(path).unwrap();
        assert!(discover(&root).unwrap().symbols.is_empty());
        std::fs::remove_dir(root).unwrap();
    }
}
