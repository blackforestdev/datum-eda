//! Exact-byte local exchange, separate from project mutation and publication policy.
use std::{collections::BTreeMap, path::Path};

use serde_json::{Value, json};

use super::{ClosureLimits, inspect_closure, refusal, release, server_policy};
use crate::error::EngineError;

mod writer;

pub(crate) fn export_candidate(project: &Path, destination: &Path) -> Result<Value, EngineError> {
    // Refuse unsupported hosts before reading source content or creating anything.
    writer::supported()?;
    let project = project.canonicalize()?;
    if !project.is_dir() {
        return Err(refusal(
            "library export requires a source project directory",
        ));
    }
    let (catalog, release, discovery_bytes) =
        release::read_candidate_bytes(&project.join("library"))?;
    let limits = server_policy()?;
    let closure = inspect_closure(
        &release.manifest.roots,
        &release.assets,
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    let graph = closure.graph_report();
    if !graph.valid {
        return Err(refusal(format!(
            "candidate dependency graph invalid: {:?}",
            graph.diagnostics
        )));
    }
    let mut files = BTreeMap::from([
        ("library.json".to_owned(), discovery_bytes),
        (
            format!("objects/sha256/{}.json", catalog.manifest_ref.hex()),
            release.manifest_bytes,
        ),
    ]);
    for (digest, bytes) in release.objects {
        files.insert(format!("objects/sha256/{}.json", digest.hex()), bytes);
    }
    for (digest, bytes) in release.blobs {
        files.insert(format!("blobs/sha256/{}", digest.hex()), bytes);
    }
    let file_count = files.len();
    let byte_count: usize = files.values().map(Vec::len).sum();
    let mut stage = writer::Stage::new(&project, destination)?;
    stage.write_files(&files)?;
    let (destination, durability_warning) = stage.publish()?;
    Ok(json!({
        "status":"exported", "destination":destination,
        "manifest_ref":catalog.manifest_ref, "roots":release.manifest.roots,
        "file_count":file_count,"byte_count":byte_count,
        "durability_warning":durability_warning,
        "scope":"exact verified complete declared candidate inventory; source project unchanged",
        "redistribution_authority":"not_verified", "engineering_approval":"not_evaluated",
        "publication_policy":"unavailable"
    }))
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
