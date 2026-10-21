//! Canonical source snapshots and immutable setup-stage archives for S5A fixtures.
use anyhow::{Result, ensure};
use eda_engine::substrate::{DesignModel, ProjectResolver, SourceShardKind};
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

// Match native_project_source_inventory.py before descending or reading bytes.
fn operational(relative: &Path) -> bool {
    let mut suffix = PathBuf::new();
    let mut in_datum = false;
    for part in relative.components() {
        if in_datum {
            suffix.push(part.as_os_str());
        } else if part.as_os_str() == ".datum" {
            in_datum = true;
        }
    }
    in_datum
        && (suffix == Path::new("revision/writer.lock")
            || suffix == Path::new("gui-terminal-context.json")
            || suffix.starts_with("tool-sessions")
            || suffix.starts_with("terminal-contexts"))
}

// Read the complete canonical Project, including journal/revision history.
pub(super) fn source_bytes(root: &Path, directory: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if operational(path.strip_prefix(root)?) {
            continue;
        }
        ensure!(!path.is_symlink(), "fixture source symlink refused");
        if path.is_dir() {
            files.extend(source_bytes(root, &path)?);
        } else {
            files.insert(
                path.strip_prefix(root)?.to_path_buf(),
                std::fs::read(&path)?,
            );
        }
    }
    Ok(files)
}

pub(super) fn preserve_stage(
    root: &Path,
    project: &Path,
    name: &str,
    model: &DesignModel,
) -> Result<serde_json::Value> {
    let before = source_bytes(project, project)?;
    let reopened = ProjectResolver::new(project).resolve()?;
    ensure!(
        reopened.model_revision == model.model_revision,
        "stage reopen revision"
    );
    ensure!(
        source_bytes(project, project)? == before,
        "reopen preserves complete source/history"
    );
    let archive = root.join("stages").join(name);
    ensure!(!archive.exists(), "stage archive must be fresh");
    // Byte copies only to evidence archives, never to active canonical sources.
    for (relative, bytes) in &before {
        let destination = archive.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap())?;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?
            .write_all(bytes)?;
    }
    let board = reopened.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    Ok(serde_json::json!({
        "name":name,"archive":archive,"revision":reopened.model_revision,
        "board_schema_version":board["schema_version"],
        "journal_transactions":reopened.journal.len(),
        "latest_operations":reopened.journal.last().map(|t| &t.operations),
        "files":before.iter().map(|(path,bytes)| (path.to_string_lossy().into_owned(), bytes.len())).collect::<BTreeMap<_,_>>(),
        "reopen_preserved_source_and_history":true
    }))
}
