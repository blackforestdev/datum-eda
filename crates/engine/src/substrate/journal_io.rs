use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use super::{
    EngineError, JOURNAL_RELATIVE_PATH, JournalCursor, ResolveDiagnostic, TransactionRecord,
};
use crate::ir::serialization::to_json_deterministic;

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

pub fn transaction_journal_path(project_root: &Path) -> PathBuf {
    project_root.join(JOURNAL_RELATIVE_PATH)
}

fn journal_cursor_path(project_root: &Path) -> PathBuf {
    project_root.join(".datum/journal/cursor.json")
}

pub(crate) fn read_transaction_journal(
    project_root: &Path,
) -> (Vec<TransactionRecord>, Vec<ResolveDiagnostic>) {
    let path = transaction_journal_path(project_root);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (Vec::new(), Vec::new());
        }
        Err(error) => {
            return (
                Vec::new(),
                vec![ResolveDiagnostic {
                    code: "journal_read_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };

    let mut seen = BTreeMap::<Uuid, String>::new();
    let mut transactions = Vec::new();
    let mut diagnostics = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let transaction = match serde_json::from_str::<TransactionRecord>(line) {
            Ok(transaction) => transaction,
            Err(error) => {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_parse_error".to_string(),
                    message: format!("line {}: {error}", line_index + 1),
                    path: Some(path.clone()),
                });
                break;
            }
        };
        let canonical = match to_json_deterministic(&transaction) {
            Ok(canonical) => canonical,
            Err(error) => {
                diagnostics.push(ResolveDiagnostic {
                    code: "journal_canonicalization_error".to_string(),
                    message: format!("line {}: {error}", line_index + 1),
                    path: Some(path.clone()),
                });
                break;
            }
        };
        if let Some(existing) = seen.get(&transaction.transaction_id) {
            let code = if existing == &canonical {
                "journal_duplicate_transaction_skipped"
            } else {
                "journal_transaction_id_conflict"
            };
            diagnostics.push(ResolveDiagnostic {
                code: code.to_string(),
                message: format!(
                    "line {}: duplicate transaction {}",
                    line_index + 1,
                    transaction.transaction_id
                ),
                path: Some(path.clone()),
            });
            if existing != &canonical {
                break;
            }
        } else {
            seen.insert(transaction.transaction_id, canonical);
            transactions.push(transaction);
        }
    }
    (transactions, diagnostics)
}

pub(super) fn read_journal_cursor(
    project_root: &Path,
    journal_len: usize,
) -> (JournalCursor, Vec<ResolveDiagnostic>) {
    let path = journal_cursor_path(project_root);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                Vec::new(),
            );
        }
        Err(error) => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                vec![ResolveDiagnostic {
                    code: "journal_cursor_read_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };

    let cursor = match serde_json::from_str::<JournalCursor>(&content) {
        Ok(cursor) => cursor,
        Err(error) => {
            return (
                JournalCursor {
                    applied_transaction_count: journal_len,
                },
                vec![ResolveDiagnostic {
                    code: "journal_cursor_parse_error".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                }],
            );
        }
    };
    if cursor.applied_transaction_count > journal_len {
        return (
            JournalCursor {
                applied_transaction_count: journal_len,
            },
            vec![ResolveDiagnostic {
                code: "journal_cursor_out_of_range".to_string(),
                message: format!(
                    "cursor applied_transaction_count {} exceeds journal length {}",
                    cursor.applied_transaction_count, journal_len
                ),
                path: Some(path),
            }],
        );
    }
    if cursor.applied_transaction_count < journal_len {
        return (
            JournalCursor {
                applied_transaction_count: journal_len,
            },
            vec![ResolveDiagnostic {
                code: "journal_cursor_behind".to_string(),
                message: format!(
                    "cursor applied_transaction_count {} is behind journal length {}",
                    cursor.applied_transaction_count, journal_len
                ),
                path: Some(path),
            }],
        );
    }
    (cursor, Vec::new())
}

pub(crate) fn write_journal_cursor(
    project_root: &Path,
    cursor: &JournalCursor,
) -> Result<(), EngineError> {
    let path = journal_cursor_path(project_root);
    if let Some(parent) = path.parent() {
        crate::revision::create_directory_durable(parent)?;
    }
    let json = to_json_deterministic(cursor)?;
    std::fs::write(&path, format!("{json}\n"))?;
    std::fs::File::open(&path)?.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

pub(super) fn append_transaction_journal(
    project_root: &Path,
    transaction: &TransactionRecord,
) -> Result<(), EngineError> {
    let path = transaction_journal_path(project_root);
    if let Some(parent) = path.parent() {
        crate::revision::create_directory_durable(parent)?;
        sync_directory(parent)?;
    }

    let line = to_json_deterministic(transaction)?;
    if path.exists() {
        let existing = recover_torn_journal_tail(&path)?;
        let mut existing_transactions = Vec::new();
        for (line_index, existing_line) in existing.lines().enumerate() {
            if existing_line.trim().is_empty() {
                continue;
            }
            let existing_transaction = serde_json::from_str::<TransactionRecord>(existing_line)
                .map_err(|error| {
                    EngineError::Operation(format!(
                        "journal append refused: existing journal parse error on line {}: {error}",
                        line_index + 1
                    ))
                })?;
            let existing_canonical = to_json_deterministic(&existing_transaction)?;
            existing_transactions.push((existing_transaction, existing_canonical));
        }
        for (index, (existing_transaction, existing_canonical)) in
            existing_transactions.iter().enumerate()
        {
            if existing_transaction.transaction_id == transaction.transaction_id {
                if existing_canonical == &line && index + 1 == existing_transactions.len() {
                    return Ok(());
                }
                if existing_canonical == &line {
                    return Err(EngineError::Operation(format!(
                        "journal append refused: transaction {} already exists before journal tip",
                        transaction.transaction_id
                    )));
                }
                return Err(EngineError::Operation(format!(
                    "journal transaction id conflict: {}",
                    transaction.transaction_id
                )));
            }
        }
        if let Some((last_transaction, _)) = existing_transactions.last()
            && last_transaction.after_model_revision != transaction.before_model_revision
        {
            return Err(EngineError::Operation(format!(
                "journal append refused: transaction {} before revision {} does not match journal tip {}",
                transaction.transaction_id,
                transaction.before_model_revision.0,
                last_transaction.after_model_revision.0
            )));
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}
