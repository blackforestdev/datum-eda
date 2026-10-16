"""Canonical journal staging and I/O owner checks for the private-writer gate.

The historical single journal.rs allowance belongs to two current owners:
two staging writes in journal.rs and one cursor write in journal_io.rs. The
append/recovery paths use three explicit OpenOptions calls in journal_io.rs.
These are exact per-owner counts, not a transferable aggregate allowance.
"""
from pathlib import Path

import source_pattern_scan


JOURNAL_OWNERS = {
    Path("crates/engine/src/substrate/journal.rs"): {
        "writes": 2,
        "open_options": 0,
        "required": [
            "fn stage_shard_write(",
            "pub(super) fn stage_new_shard_write(",
            'join(".datum/stage")',
            "pub(super) fn promote_staged_shard_writes(",
            "std::fs::rename(staged, &write.destination)?;",
            "pub(super) use super::journal_io::{",
            "append_transaction_journal, read_journal_cursor, read_transaction_journal, write_journal_cursor,",
        ],
    },
    Path("crates/engine/src/substrate/journal_io.rs"): {
        "writes": 1,
        "open_options": 3,
        "required": [
            "pub(crate) fn write_journal_cursor(",
            "journal_cursor_path(project_root)",
            "pub(super) fn append_transaction_journal(",
            "transaction_journal_path(project_root)",
            "pub(super) fn recover_torn_journal_tail(",
            "std::fs::OpenOptions::new()",
            "file.sync_all()?;",
            "sync_directory(parent)?;",
        ],
    },
}


def check_journal_persistence(root: Path) -> list[str]:
    """Require the real owners, delegation, persistence counts and key seams."""
    failures = []
    for relative, config in JOURNAL_OWNERS.items():
        path = root / relative
        if not path.is_file():
            failures.append(f"missing engine journal persistence file: {relative}")
            continue
        source_pattern_scan.require_patterns(
            failures, relative, path.read_text(), config["required"],
            "engine journal persistence owner",
        )
        for field, pattern, label in [
            ("writes", "std::fs::write(", "std::fs::write calls"),
            ("open_options", "std::fs::OpenOptions::new()", "OpenOptions calls"),
        ]:
            actual = source_pattern_scan.count_pattern(path, pattern)
            expected = config[field]
            if actual != expected:
                failures.append(
                    f"{relative}: expected exactly {expected} engine "
                    f"journal-persistence {label}, found {actual}"
                )
    return failures
