"""Read-only hashes for native QA source/journal preservation checkpoints."""
import hashlib
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_files(project: Path) -> dict[str, str]:
    return {str(p.relative_to(project)): digest(p) for p in project.rglob("*")
            if p.is_file() and (".datum" not in p.relative_to(project).parts
                or p.relative_to(project).parts[:2] in
                {(".datum", "journal"), (".datum", "revision")} )}


def validate_frozen_sources(project: Path, expected: dict[str, str]) -> dict:
    """Bind the selected canonical files to independently tracked hashes.

    Writer admission can leave an advisory lock even without a transaction.
    Account for that exact operational path separately, never as source data.
    """
    actual = source_files(project)
    lock_present = actual.pop('.datum/revision/writer.lock', None) is not None
    if actual != expected:
        raise ValueError('canonical source/journal inventory differs from frozen catalogue')
    return {'canonical_files': actual, 'operational_writer_lock_present': lock_present}
