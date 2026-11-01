//! Read-only local KiCad symbol acquisition. Persistence belongs to native writes.
pub mod batch;
pub mod bundle_format;
mod convert;
mod geometry;
mod pins;
mod resolve;
mod sexpr;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::EngineError;
use crate::pool::{Symbol, Unit};
use sexpr::Node;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSummary {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConvertedSymbol {
    pub unit: Unit,
    pub symbol: Symbol,
    pub warnings: Vec<String>,
    pub source_hash: String,
    pub source_path: PathBuf,
    pub source_symbol: String,
}

type Result<T> = std::result::Result<T, EngineError>;

fn refusal(message: impl Into<String>) -> EngineError {
    EngineError::Import(format!("KiCad symbol library: {}", message.into()))
}

struct Library {
    path: PathBuf,
    hash: String,
    format_version: Option<String>,
    symbols: BTreeMap<String, Node>,
}

impl Library {
    fn read(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        if path.to_str().is_none() {
            return Err(refusal(
                "source path must be UTF-8 for stable source identity",
            ));
        }
        let bytes = std::fs::read(&path)?;
        let source = std::str::from_utf8(&bytes).map_err(|_| refusal("source is not UTF-8"))?;
        Self::parse(path, source)
    }

    fn parse(path: PathBuf, source: &str) -> Result<Self> {
        let root = sexpr::parse(source)?;
        if root.tag() != Some("kicad_symbol_lib") {
            return Err(refusal("expected kicad_symbol_lib root"));
        }
        let mut symbols = BTreeMap::new();
        for symbol in root.children("symbol") {
            let name = symbol.required_atom(1)?.to_owned();
            if symbols.insert(name.clone(), symbol.clone()).is_some() {
                return Err(refusal(format!("duplicate symbol {name}")));
            }
        }
        Ok(Self {
            path,
            hash: super::ids_sidecar::compute_source_hash_bytes(source.as_bytes()),
            format_version: root
                .child("version")
                .and_then(|version| version.atom(1))
                .map(str::to_owned),
            symbols,
        })
    }
}

/// List source names without acquiring objects or changing the source/project.
pub fn catalog(path: &Path) -> Result<Vec<SymbolSummary>> {
    let library = Library::read(path)?;
    Ok(library
        .symbols
        .iter()
        .map(|(name, node)| SymbolSummary {
            name: name.clone(),
            description: node
                .children("property")
                .find(|p| p.atom(1) == Some("Description"))
                .and_then(|p| p.atom(2))
                .map(str::to_owned),
        })
        .collect())
}

/// Resolve a single source symbol into typed native objects. Never writes files.
pub fn convert(path: &Path, name: &str) -> Result<ConvertedSymbol> {
    let library = Library::read(path)?;
    convert::convert(&library, name)
}

#[cfg(test)]
mod tests;
