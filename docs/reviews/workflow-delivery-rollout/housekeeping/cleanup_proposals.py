"""One-time, recoverable cleanup of the completed WDQ proposal scratch space."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time
import zipfile

ROOT = Path('/home/bfadmin/Documents/datum-eda')
SOURCE = ROOT / '.git/datum-wdq/proposals'
DEST = ROOT / '.git/datum-wdq/retained/proposal-recovery-20260916.zip'
RECEIPT = ROOT / '.git/datum-wdq/retained/proposal-cleanup-20260916.json'


def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()


def in_source(value):
    return value == str(SOURCE) or value.startswith(str(SOURCE) + '/')


def idle():
    blockers = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit() or int(proc.name) == os.getpid():
            continue
        for entry in [proc / 'cwd', proc / 'exe', *list((proc / 'fd').glob('*'))]:
            try:
                target = os.readlink(entry)
            except (OSError, PermissionError):
                continue
            if in_source(target):
                blockers.append((proc.name, str(entry), target))
    if blockers:
        raise RuntimeError(f'Open proposal workspace handles: {blockers[:20]}')


def records():
    for directory, dirs, files in os.walk(SOURCE, followlinks=False):
        dirs.sort()
        files.sort()
        for name in dirs + files:
            path = Path(directory) / name
            info = path.lstat()
            relative = str(path.relative_to(SOURCE))
            yield path, relative, info


def identity(info):
    return [info.st_mode, info.st_size, info.st_mtime_ns]


def archive():
    assert SOURCE.is_dir() and not SOURCE.is_symlink()
    assert not DEST.exists() and not RECEIPT.exists()
    idle()
    original_head = git('rev-parse', 'HEAD')
    assert not git('status', '--porcelain')
    allocated = int(subprocess.check_output(['du', '-s', '-B1', str(SOURCE)]).split()[0])
    entries, seen, inode_cache = {}, set(), {}
    total = 0
    last = time.monotonic()
    os.umask(0o077)
    with zipfile.ZipFile(DEST, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=1) as output:
        for path, relative, info in records():
            item = {'stat': identity(info)}
            if stat.S_ISLNK(info.st_mode):
                item['link'] = os.readlink(path)
            elif stat.S_ISREG(info.st_mode):
                key = (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns)
                digest = inode_cache.get(key)
                if digest is None:
                    digest = hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()
                    inode_cache[key] = digest
                item['sha256'] = digest
                if digest not in seen:
                    output.write(path, 'objects/' + digest)
                    seen.add(digest)
                assert identity(path.lstat()) == item['stat'], relative
                total += info.st_size
            elif not stat.S_ISDIR(info.st_mode):
                raise RuntimeError(f'Unexpected special file: {relative}')
            entries[relative] = item
            if time.monotonic() - last > 15:
                print(f'Archived {len(entries)} paths; {total / 2**30:.1f} GiB scanned; {len(seen)} unique contents', flush=True)
                last = time.monotonic()
        manifest = {'source': str(SOURCE), 'entries': entries}
        output.writestr('manifest.json', json.dumps(manifest, separators=(',', ':')))
    print('Verifying every unique archived object.', flush=True)
    with zipfile.ZipFile(DEST) as saved:
        for name in saved.namelist():
            if name.startswith('objects/'):
                with saved.open(name) as stream:
                    assert hashlib.file_digest(stream, 'sha256').hexdigest() == name[8:]
        assert json.loads(saved.read('manifest.json')) == manifest
    receipt = {
        'source': str(SOURCE), 'archive': str(DEST), 'head_before': original_head,
        'source_allocated_bytes': allocated, 'source_logical_bytes': total,
        'archived_paths': len(entries), 'unique_contents': len(seen),
        'archive_bytes': DEST.stat().st_size,
        'archive_sha256': hashlib.file_digest(DEST.open('rb'), 'sha256').hexdigest(),
        'verification': 'Every unique archived object read back and SHA-256 verified; manifest round-trip exact.',
        'deleted': False,
    }
    RECEIPT.write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2), flush=True)


def clean():
    receipt = json.loads(RECEIPT.read_text())
    assert not receipt['deleted']
    assert git('rev-parse', 'HEAD') == receipt['head_before']
    assert not git('status', '--porcelain')
    assert hashlib.file_digest(DEST.open('rb'), 'sha256').hexdigest() == receipt['archive_sha256']
    with zipfile.ZipFile(DEST) as saved:
        expected = json.loads(saved.read('manifest.json'))['entries']
    actual = {}
    for path, relative, info in records():
        actual[relative] = identity(info)
        assert actual[relative] == expected[relative]['stat'], relative
    assert set(actual) == set(expected)
    idle()
    worktrees = []
    for block in git('worktree', 'list', '--porcelain').split('\n\n'):
        fields = dict(line.split(' ', 1) for line in block.splitlines() if ' ' in line)
        path = fields.get('worktree', '')
        if in_source(path):
            ref = 'refs/datum-wdq/retired-worktrees/' + fields['HEAD']
            git('update-ref', ref, fields['HEAD'])
            subprocess.run(['git', '-C', str(ROOT), 'worktree', 'remove', path], check=True)
            worktrees.append({'path': path, 'retained_ref': ref})
    # SOURCE is a fixed, validated scratch directory, never the repository root.
    shutil.rmtree(SOURCE)
    receipt.update(deleted=True, retired_worktrees=worktrees,
                   net_reclaimed_bytes=receipt['source_allocated_bytes'] - DEST.stat().st_size)
    RECEIPT.write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2), flush=True)


if __name__ == '__main__':
    {'archive': archive, 'clean': clean}[sys.argv[1]]()
