//! Maintenance conversion of one library, with stable shipped-source identities.
use super::{ConvertedSymbol, Library, Result, convert, refusal};
use std::path::{Path, PathBuf};

pub struct SymbolConversion {
    pub name: String,
    pub description: Option<String>,
    pub result: Result<ConvertedSymbol>,
}

pub struct LibraryConversion {
    pub source: PathBuf,
    pub source_hash: String,
    pub format_version: Option<String>,
    pub symbols: Vec<SymbolConversion>,
}

/// Parse the source once. Every name has a conversion outcome, including
/// unsupported symbols; a caller must never mistake the successes for coverage.
pub fn convert_library(path: &Path) -> Result<LibraryConversion> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refusal("library filename must be UTF-8"))?;
    let mut library = Library::read(path)?;
    // Release/version and physical installation location are provenance, not
    // identity. A later bundle release preserves IDs for the same library/name.
    library.path = PathBuf::from("kicad").join(name);
    Ok(convert_parsed(&library))
}

fn convert_parsed(library: &Library) -> LibraryConversion {
    LibraryConversion {
        source: library.path.clone(),
        source_hash: library.hash.clone(),
        format_version: library.format_version.clone(),
        symbols: library
            .symbols
            .iter()
            .map(|(name, node)| SymbolConversion {
                name: name.clone(),
                description: node
                    .children("property")
                    .find(|property| property.atom(1) == Some("Description"))
                    .and_then(|property| property.atom(2))
                    .map(str::to_owned),
                result: convert::convert(library, name),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batch_preserves_successes_refusals_and_content_independent_identity() {
        let source = r#"(kicad_symbol_lib (version 20241209)
          (symbol "R" (property "Description" "Resistor")
            (symbol "R_0_1" (rectangle (start -1 -2) (end 1 2)
              (stroke (width 0.2) (type default)) (fill (type none))))
            (symbol "R_1_1" (pin passive line (at -3 0 0) (length 2)
              (name "A") (number "1"))))
          (symbol "Unsupported" (power)))"#;
        let first =
            convert_parsed(&Library::parse("kicad/Device.kicad_sym".into(), source).unwrap());
        assert_eq!(first.symbols.len(), 2);
        assert_eq!(first.format_version.as_deref(), Some("20241209"));
        let original = first.symbols[0].result.as_ref().unwrap();
        assert!(
            first.symbols[1]
                .result
                .as_ref()
                .unwrap_err()
                .to_string()
                .contains("power symbols")
        );
        let edited = source.replace("Resistor", "Updated resistor description");
        let second =
            convert_parsed(&Library::parse("kicad/Device.kicad_sym".into(), &edited).unwrap());
        let changed = second.symbols[0].result.as_ref().unwrap();
        assert_eq!(original.symbol.uuid, changed.symbol.uuid);
        assert_eq!(original.unit.uuid, changed.unit.uuid);
        assert_ne!(first.source_hash, second.source_hash);
        assert_eq!(original.source_path, Path::new("kicad/Device.kicad_sym"));
    }
}
