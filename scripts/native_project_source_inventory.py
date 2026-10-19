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
    actual, operational = {}, {}
    lock_present = False
    for path in project.rglob('*'):
        relative = path.relative_to(project)
        parts = relative.parts
        suffix = parts[parts.index('.datum') + 1:] if '.datum' in parts else ()
        role = None
        if suffix == ('revision', 'writer.lock'):
            role = 'revision/writer.lock'
            lock_present |= relative.as_posix() == '.datum/revision/writer.lock'
        elif suffix == ('gui-terminal-context.json',):
            role = 'gui-terminal-context.json'
        elif suffix[:1] in [('tool-sessions',), ('terminal-contexts',)]:
            role = suffix[0]
        if role:
            operational[role] = operational.get(role, 0) + 1
            continue  # Operational context/credential bytes are never read.
        if path.is_symlink():
            raise ValueError('symlink canonical source refused')
        if path.is_file():
            actual[relative.as_posix()] = digest(path)
    if actual != expected:
        raise ValueError('canonical source/journal inventory differs from frozen catalogue')
    return {'canonical_files': actual, 'operational_writer_lock_present': lock_present,
            'operational_role_counts': operational}
