//! Guarded acknowledgement of the existing canonical durable state, including
//! an uncertain write receipt. No second serialization or design mutation.
use crate::{
    error::EngineError,
    revision::ProjectWriteLease,
    substrate::{DesignModel, ProjectReadCache, ProjectResolver},
};
use std::{fs::File, path::Path, sync::Arc};

pub fn resolve_durable(root: &Path) -> Result<DesignModel, EngineError> {
    let root = root.canonicalize()?;
    let lease = ProjectWriteLease::acquire(&root)?;
    let mut model = ProjectResolver::new(&root).resolve_with_lease(&lease)?;
    sync_durable_model(&root, &model, &lease)?;
    model
        .diagnostics
        .retain(|d| d.code != "journal_cursor_behind");
    Ok(model)
}

/// Existing exact-input reuse, still qualified under current writer exclusion.
/// Cached bytes do not replace file/directory synchronization or recovery.
pub(crate) fn resolve_durable_cached(
    root: &Path,
    cache: &mut ProjectReadCache,
) -> Result<Arc<DesignModel>, EngineError> {
    let root = root.canonicalize()?;
    let lease = ProjectWriteLease::acquire(&root)?;
    let mut model = cache.resolve_with_lease(&root, &lease)?;
    sync_durable_model(&root, &model, &lease)?;
    if model
        .diagnostics
        .iter()
        .any(|d| d.code == "journal_cursor_behind")
    {
        Arc::make_mut(&mut model)
            .diagnostics
            .retain(|d| d.code != "journal_cursor_behind");
    }
    Ok(model)
}

fn sync_durable_model(
    root: &Path,
    model: &DesignModel,
    lease: &ProjectWriteLease,
) -> Result<(), EngineError> {
    lease.validate(root)?;
    if model.diagnostics.iter().any(|d| {
        d.code == "revision_integrity_unavailable"
            || (d.code.starts_with("journal_") && d.code != "journal_cursor_behind")
    }) {
        return Err(EngineError::Operation(
            "Project durability cannot be confirmed: integrity or journal diagnostics".into(),
        ));
    }
    // Re-issue sync after an earlier fsync may have failed despite visible bytes.
    // Only Project-local canonical shards are owned here; external pools remain
    // outside this bounded writer's authority.
    for shard in &model.source_shards {
        let path = shard.path.canonicalize()?;
        if shard.path.starts_with(&root) && !path.starts_with(&root) {
            return Err(EngineError::Operation(
                "Project durability refused: local shard escapes the guarded root".into(),
            ));
        }
        if path.starts_with(&root) {
            sync_file(&path)?;
        }
    }
    let store = root.join(".datum/revision/v1");
    if store.exists() {
        sync_tree(&store)?;
    }
    let journal = crate::substrate::transaction_journal_path(&root);
    if journal.exists() {
        sync_file(&journal)?;
    }
    crate::substrate::journal_io::write_journal_cursor(&root, &model.journal_cursor)?;
    lease.validate(root)?;
    File::open(root)?.sync_all()?;
    Ok(())
}

fn sync_file(path: &Path) -> Result<(), EngineError> {
    File::open(path)?.sync_all()?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
fn sync_tree(path: &Path) -> Result<(), EngineError> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(EngineError::Operation(
                "Project durability refused: symlink in revision store".into(),
            ));
        }
        if kind.is_dir() {
            sync_tree(&entry.path())?;
        } else if kind.is_file() {
            sync_file(&entry.path())?;
        } else {
            return Err(EngineError::Operation(
                "Project durability refused: nonregular revision entry".into(),
            ));
        }
    }
    File::open(path)?.sync_all()?;
    Ok(())
}
