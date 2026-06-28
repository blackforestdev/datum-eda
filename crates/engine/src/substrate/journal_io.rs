use std::path::Path;

use super::EngineError;

pub(super) fn recover_torn_journal_tail(path: &Path) -> Result<String, EngineError> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8(bytes).map_err(|error| {
        EngineError::Operation(format!(
            "journal append refused: journal is not UTF-8: {error}"
        ))
    })?;
    let Some(last_newline) = text.rfind('\n') else {
        if text.is_empty() {
            return Ok(text);
        }
        truncate_journal_to(path, 0)?;
        return Ok(String::new());
    };
    if last_newline + 1 == text.len() {
        return Ok(text);
    }
    let repaired = text[..=last_newline].to_string();
    truncate_journal_to(path, repaired.len() as u64)?;
    Ok(repaired)
}

fn truncate_journal_to(path: &Path, len: u64) -> Result<(), EngineError> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_len(len)?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

pub(super) fn sync_directory(path: &Path) -> Result<(), EngineError> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}
