"""Remove only untracked, regular, regenerable scripts bytecode after publication."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path('/home/bfadmin/Documents/datum-eda')
directory = root / 'scripts/__pycache__'
assert directory.is_dir() and not directory.is_symlink()
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root).decode().strip() == '1644e2129f4ff83cec1cdf0e6e13534c3b258e8e'
assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=root)
assert subprocess.run(['lsof', '+D', str(directory)], capture_output=True).returncode == 1
tracked = set(subprocess.check_output(['git', 'ls-files', '-z'], cwd=root).decode().split('\0'))
paths = sorted(directory.glob('*.pyc'))
inventory = []
for path in paths:
    assert path.is_file() and not path.is_symlink() and path.resolve().parent == directory
    assert path.relative_to(root).as_posix() not in tracked
    raw = path.read_bytes()
    inventory.append({'path': path.relative_to(root).as_posix(), 'size': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
output = root / '.git/datum-wdq/logs/terminal-1644-python-cache-retirement.json'
with output.open('x') as stream:
    json.dump({'kind': 'regenerable-python-cache-retirement', 'files': inventory,
        'reason': 'Published source superseded prior local bytecode. Source-only execution does not consume it.',
        'recovery': 'Bytecode deleted permanently; regenerate from retained Python source. No source or Beads records removed.'}, stream, indent=2)
    stream.write('\n')
for path, row in zip(paths, inventory):
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row['sha256']
    path.unlink()
print(json.dumps({'removed_files': len(paths), 'removed_bytes': sum(row['size'] for row in inventory)}))
