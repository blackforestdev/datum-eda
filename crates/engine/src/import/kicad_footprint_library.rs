//! Read-only source-library acquisition, independent of board/project import.
use std::path::{Path, PathBuf};

use super::{
    ids_sidecar::compute_source_hash_bytes,
    kicad_library::sexpr::{self, Node},
};
use crate::{
    error::EngineError,
    pool::{Footprint, Package, Padstack},
};

mod convert;
mod drawings;
mod pads;
#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, EngineError>;
fn refusal(reason: impl Into<String>) -> EngineError {
    EngineError::Import(format!("KiCad footprint library: {}", reason.into()))
}

#[derive(Debug, Clone)]
pub struct FootprintSummary {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConvertedFootprint {
    pub package: Package,
    pub footprint: Footprint,
    pub padstacks: Vec<Padstack>,
    pub source_path: PathBuf,
    pub source_hash: String,
    pub warnings: Vec<String>,
    /// Retained as provenance, never executed or treated as a native model.
    pub source_document: String,
}

fn read(path: &Path) -> Result<(PathBuf, String, Node)> {
    let path = path.canonicalize()?;
    if path.extension().and_then(|s| s.to_str()) != Some("kicad_mod") {
        return Err(refusal("choose a .kicad_mod library source"));
    }
    let text = std::fs::read_to_string(&path)?;
    let node = sexpr::parse(&text)?;
    if !matches!(node.tag(), Some("footprint" | "module")) {
        return Err(refusal("expected footprint or module root"));
    }
    Ok((path, text, node))
}

pub fn catalog(path: &Path) -> Result<FootprintSummary> {
    let (_, _, node) = read(path)?;
    Ok(FootprintSummary {
        name: node.required_atom(1)?.into(),
        description: node
            .child("descr")
            .and_then(|n| n.atom(1))
            .map(str::to_owned),
    })
}

/// Conversion never writes or silently substitutes an unsupported pad shape.
pub fn convert(path: &Path) -> Result<ConvertedFootprint> {
    let (path, text, node) = read(path)?;
    let source_hash = compute_source_hash_bytes(text.as_bytes());
    convert::convert(path, text, node, source_hash)
}

fn length(node: &Node, index: usize) -> Result<i64> {
    crate::ir::units::parse_fixed_length(
        node.required_atom(index)?,
        crate::ir::units::LengthUnit::Millimeter,
    )
    .map(|v| v.get())
    .map_err(|e| refusal(format!("invalid source length: {e:?}")))
}

fn point(node: &Node) -> Result<crate::ir::geometry::Point> {
    Ok(crate::ir::geometry::Point::new(
        length(node, 1)?,
        length(node, 2)?,
    ))
}

fn identity(source: &str, kind: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("datum:kicad-footprint:{source}:{kind}").as_bytes(),
    )
}
