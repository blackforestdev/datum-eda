"""Retain actual logical bytes before and after isolated Preferences mutations."""
from pathlib import Path
import stat

from .inputs import read_file, require


CATEGORIES = {'generations': 'generation_bytes', 'receipts': 'receipt_bytes',
              'request-index': 'request_index_bytes', 'fixed': 'fixed_overhead_bytes'}


def snapshot(repository, capture, identity):
    """Read every regular file without following links; do not infer byte counts.

    The repository embeds receipts and the request index in generation manifests.
    They therefore count once as generation bytes. Mutable head/lock metadata is
    reported separately; unexpected files are preserved rather than discarded.
    """
    repository = Path(repository).absolute()
    require(repository == repository.resolve(), 'storage root is redirected')
    files = []
    if repository.exists():
        require(repository.is_dir(), 'storage root is not a directory')
        for path in sorted(repository.rglob('*')):
            mode = path.lstat().st_mode
            require(not stat.S_ISLNK(mode), 'storage entry is a symlink')
            if stat.S_ISDIR(mode):
                continue
            require(stat.S_ISREG(mode), 'storage entry is not a regular file')
            name = path.relative_to(repository).as_posix()
            raw = read_file(repository, name)
            reference = capture.put(raw)
            prefix = '' if name.split('/')[0] in CATEGORIES else 'fixed/'
            files.append({'path': prefix + name, 'kind': 'file',
                          'bytes': len(raw), 'sha256': reference['sha256']})
    state = {'schema': 'datum.preferences.state.v2', 'root': identity,
             'files': sorted(files, key=lambda entry: entry['path'])}
    return state, capture.put(state)


def growth(before, after):
    """Compute increments from retained files and refuse lost immutable history."""
    old = {entry['path']: entry for entry in before['files']}
    new = {entry['path']: entry for entry in after['files']}
    require(before['root'] == after['root'], 'storage identity changed')
    for name, entry in old.items():
        if not name.startswith('fixed/'):
            require(new.get(name) == entry, 'immutable storage was removed or rewritten')
    totals = {field: 0 for field in CATEGORIES.values()}
    for direction, state in ((-1, before), (1, after)):
        for entry in state['files']:
            totals[CATEGORIES[entry['path'].split('/')[0]]] += direction * entry['bytes']
    return totals
