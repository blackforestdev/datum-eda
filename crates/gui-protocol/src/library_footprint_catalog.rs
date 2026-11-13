//! Shallow local KiCad footprint-library discovery; conversion happens on selection.
use super::{LibrarySourceSymbol, SourceAvailability, SourceCatalog};
use std::path::{Path, PathBuf};

fn files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = std::fs::read_dir(directory)
        .map_err(|e| e.to_string())?
        .map(|entry| entry.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    Ok(paths)
}

pub(super) fn discover(source: &Path) -> Result<SourceCatalog, String> {
    let mut result = SourceCatalog::default();
    let mut paths = Vec::new();
    if source.is_dir() {
        for path in files(source)? {
            if path.extension().is_some_and(|ext| ext == "kicad_mod") {
                paths.push(path);
            } else if path.is_dir() && path.extension().is_some_and(|ext| ext == "pretty") {
                match files(&path) {
                    Ok(children) => paths.extend(
                        children
                            .into_iter()
                            .filter(|p| p.extension().is_some_and(|ext| ext == "kicad_mod")),
                    ),
                    Err(reason) => result.notices.push(format!("{}: {reason}", path.display())),
                }
            }
        }
    } else {
        paths.push(source.to_owned());
    }
    for path in paths {
        let read = (|| {
            let path = path.canonicalize().map_err(|e| e.to_string())?;
            let summary = eda_engine::import::kicad_footprint_library::catalog(&path)
                .map_err(|e| e.to_string())?;
            let library = path
                .parent()
                .and_then(Path::file_stem)
                .and_then(|name| name.to_str())
                .unwrap_or("Footprints");
            Ok::<_, String>(LibrarySourceSymbol {
                label: format!("{library}:{}", summary.name),
                name: summary.name,
                description: summary.description.unwrap_or_default(),
                path,
                availability: SourceAvailability::Unchecked,
            })
        })();
        match read {
            Ok(footprint) => result.symbols.push(footprint),
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
    fn actual_footprint_directory_and_single_file_keep_the_same_identity() {
        let root = Path::new("/usr/share/kicad/footprints/Resistor_SMD.pretty");
        if !root.is_dir() {
            eprintln!("Installed KiCad footprints unavailable; source test skipped");
            return;
        }
        let all = discover(root).unwrap();
        let file = discover(&root.join("R_0603_1608Metric.kicad_mod")).unwrap();
        assert_eq!(file.symbols.len(), 1);
        let source = &file.symbols[0];
        assert_eq!(source.label, "Resistor_SMD:R_0603_1608Metric");
        assert!(all.symbols.contains(source));
        assert_eq!(source.availability, SourceAvailability::Unchecked);
    }
}
