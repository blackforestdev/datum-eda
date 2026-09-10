"""Commit exact activation and separate installed verification evidence."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

root = Path('/home/bfadmin/Documents/datum-eda')
store = root / '.git/datum-wdq'
proposal = store / 'proposals/terminal-integration-1644'
verification = store / 'logs/terminal-1644-installed-recovery-02.json'
report = json.loads(verification.read_bytes())
assert report['ok'] and report['protected_state_unchanged'] and report['support_unchanged']
files = {}
for path in sorted((store / 'logs').glob('terminal-1644-*')):
    if path.is_file():
        files['logs/' + path.name] = path.read_bytes()
for name in ('activation-request.json', 'activation-request-basis.json', 'prepare_activation_request.py',
             'retire_python_caches.py', 'verify_installed_recovery.py', 'retain_activation.py'):
    files['preparation/' + name] = (proposal / name).read_bytes()
response = store / 'owner-responses/terminal-1644-standing-authorization.json'
files['authorization/' + response.name] = response.read_bytes()
old = Path('/home/bfadmin/Documents/datum-wdq-owner-activation-20260907')
for name in ('activate.py', 'activation-uov7bvnf.log', 'preparation.json', 'promotion.md'):
    files['retired-pilot-support/' + name] = (old / name).read_bytes()
files['retired-pilot-support/pre-commit'] = (old / 'owner-hooks/pre-commit').read_bytes()
inventory = [{'path': name, 'size': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
             for name, raw in sorted(files.items())]
files['inventory.json'] = (json.dumps(inventory, indent=2) + '\n').encode()

def git(*args, data=None, env=None):
    return subprocess.check_output(['git', '--no-replace-objects', '--no-optional-locks', *args], cwd=root, input=data, env=env)

prefix = 'docs/reviews/workflow-delivery-rollout/infrastructure/installed-1644/'
with tempfile.TemporaryDirectory(prefix='retention-index-', dir=proposal) as temporary:
    env = dict(os.environ, GIT_INDEX_FILE=str(Path(temporary) / 'index'))
    git('read-tree', '--empty', env=env)
    for name, raw in sorted(files.items()):
        oid = git('hash-object', '-w', '--stdin', data=raw).decode().strip()
        git('update-index', '--add', '--cacheinfo', '100644', oid, prefix + name, env=env)
    tree = git('write-tree', env=env).decode().strip()
message = '''test(workflow): retain installed activation and recovery verification

Problem:
The installer published exact pins but its legacy final observer rejected allowed Beads runtime files. Its failure must remain visible beside subsequent installed checks.

Change:
Retain exact request, standing-authority encoding, failed activation, source-only installed recovery verification, cache retirement inventory, preparation scripts and obsolete pilot support logs.

Proof:
The installed hook and selector exit zero; authenticated bundle and full protected state remain unchanged. All retained blob bytes are checked after commit. Original installer exit two remains unchanged.

Roadmap:
WDQ-I04, PM042, dat-wdq-rollout-implementation-ffy. Infrastructure verification only; no product acceptance, dependency or licensing change. Observer repair remains a distinct defect.
'''
commit = git('commit-tree', tree, data=message.encode()).decode().strip()
ref = 'refs/datum-wdq/evidence/installed-1644-' + commit
git('update-ref', ref, commit, '0' * 40)
for name, raw in files.items():
    assert git('show', commit + ':' + prefix + name) == raw
print(json.dumps({'commit': commit, 'ref': ref, 'files': len(files), 'verification_sha256': hashlib.sha256(verification.read_bytes()).hexdigest()}))
