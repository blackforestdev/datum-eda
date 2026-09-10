"""Verify published pins without retrying activation or rewriting its failure."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/home/bfadmin/Documents/datum-eda')
STORE = ROOT / '.git/datum-wdq'
CANDIDATE = '1644e2129f4ff83cec1cdf0e6e13534c3b258e8e'
OUT = STORE / 'logs/terminal-1644-installed-recovery-02.json'

def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    assert not OUT.exists()
    request = json.loads((STORE / 'proposals/terminal-integration-1644/activation-request.json').read_bytes())
    directory = STORE / 'trusted' / CANDIDATE / 'scripts'
    bootstrap = directory / 'workflow_delivery_bootstrap.py'
    expected = subprocess.check_output(['git', 'show', CANDIDATE + ':scripts/workflow_delivery_bootstrap.py'], cwd=ROOT)
    assert bootstrap.read_bytes() == expected
    ns = {'__name__': '_authenticated_bootstrap', '__file__': str(bootstrap)}
    exec(compile(expected, str(bootstrap), 'exec'), ns)
    ns['verify_support'](ROOT, CANDIDATE)
    loader = directory / 'workflow_delivery_source_only.py'
    ns = {}
    exec(compile(loader.read_bytes(), str(loader), 'exec'), ns)
    ns['install'](directory)
    sys.path.insert(0, str(directory))
    from workflow_delivery_activation_preflight import clean_inputs
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_support_bundle import verify_bundle
    from workflow_delivery_workspace_authority import promotion_workspace
    before = protected_state(ROOT, request['input_roots'])
    assert before['head'] == CANDIDATE and before['symbolic_head'] == 'refs/heads/main'
    assert before['local_trust'] == request['proposed_local_trust']
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT)
    bundle = verify_bundle(ROOT, CANDIDATE)
    workspace = promotion_workspace(ROOT, candidate=CANDIDATE, authority=CANDIDATE)
    clean_inputs(ROOT, before, CANDIDATE, workspace=workspace, explicit_paths=request['input_roots'])
    result = {'schema_version': 1, 'candidate': CANDIDATE, 'base': request['base'],
        'kind': 'datum.workflow-delivery.installed-recovery-verification',
        'original_activation_exit_code': 2,
        'original_activation_log_sha256': hashlib.sha256((STORE / 'logs/terminal-1644-activation-01.json').read_bytes()).hexdigest(),
        'scope': 'Read-only verification after partial activation; no activation retry, runtime patch, trust write, or product acceptance.',
        'observer_defect': 'The original observer omits the approved workspace policy when calling clean_inputs and therefore rejects allowed Beads runtime files.',
        'workspace_aware_tracked_input_check': 'pass', 'commands': [], 'ok': False}
    env = {'PATH': os.defpath, 'LC_ALL': 'C', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_GRAFT_FILE': os.devnull}
    commands = [[str(directory.parent / 'owner-hooks/pre-commit')],
        [sys.executable, '-I', '-S', '-B', '-c',
         'import sys; sys.path.insert(0, sys.argv[1]); from project_status import main; raise SystemExit(main(sys.argv[2:]))',
         str(directory), '--root', str(ROOT), 'check']]
    for command in commands:
        completed = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, timeout=240)
        result['commands'].append({'argv': command, 'exit_code': completed.returncode,
            'stdout': completed.stdout.decode(), 'stderr': completed.stderr.decode()})
    after = protected_state(ROOT, request['input_roots'])
    result['protected_state_unchanged'] = before == after
    result['support_unchanged'] = bundle == verify_bundle(ROOT, CANDIDATE)
    result['before'] = before
    result['after'] = after
    result['ok'] = all(row['exit_code'] == 0 for row in result['commands']) and result['protected_state_unchanged'] and result['support_unchanged']
    with OUT.open('x') as stream:
        json.dump(result, stream, sort_keys=True)
        stream.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('before', 'after')}))
    return 0 if result['ok'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
