"""One-time owner-directed expired-lease recovery; no general gate exemption."""

import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import stat
import sys
import time

ROOT = Path('/home/bfadmin/Documents/datum-eda')
STORE = ROOT / '.git/datum-wdq'
PACKET = STORE / 'proposals/lease-renewal-activation-20260915'
AUTHORITY = '61f7b2db1730f0dc7911b5a05f623ad6bf2d1000'
REQUEST_HASH = '681d51e12c7cd0d5e2a363091136e35dd774b82b4d885aad742bd02311f489be'
EXPIRED = 'WORKFLOW-DELIVERY-IMPLEMENTATION: claim expired at 2026-09-16T02:58:04Z'
OWNER_INSTRUCTION = 'I dont want to re-run 527 test that already passed. we need to blow past the beads gate keeping and complete milestone 10.'


def main():
    if sys.argv[1:] not in (['--inspect'], ['--install']):
        raise ValueError('explicit --inspect or --install required')
    install = sys.argv[1] == '--install'
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError('requires -I -S -B')
    raw = (PACKET / 'activation-request.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != REQUEST_HASH:
        raise ValueError('original approved request changed')
    request = json.loads(raw)
    if request['candidate'] != AUTHORITY or request['authority'] != AUTHORITY:
        raise ValueError('original approved candidate changed')
    runtime = STORE / 'trusted' / AUTHORITY / 'scripts'
    bootstrap = runtime / 'workflow_delivery_bootstrap.py'
    bootstrap_raw = bootstrap.read_bytes()
    expected = subprocess.check_output(['git', '--no-replace-objects', 'show', AUTHORITY + ':scripts/workflow_delivery_bootstrap.py'], cwd=ROOT)
    if bootstrap.is_symlink() or bootstrap_raw != expected:
        raise ValueError('support bootstrap differs from approved Git source')
    bootstrap_namespace = {'__name__': '_datum_activation_bootstrap', '__file__': str(bootstrap)}
    exec(compile(bootstrap_raw, str(bootstrap), 'exec'), bootstrap_namespace)
    bootstrap_namespace['verify_support'](ROOT, AUTHORITY)
    source_bootstrap = runtime / 'workflow_delivery_source_only.py'
    source_namespace = {'__name__': '_datum_source_bootstrap'}
    exec(compile(source_bootstrap.read_bytes(), str(source_bootstrap), 'exec'), source_namespace)
    source_namespace['install'](runtime)
    sys.path.insert(0, str(runtime))
    from workflow_delivery_support_bundle import verify_bundle
    from workflow_delivery_activation_preflight import preflight, inspect_promotion_candidate
    from workflow_delivery_workspace_authority import promotion_workspace
    from workflow_delivery_publication_delta import inspect_activation_response
    from workflow_delivery_activation import ActivationSignals
    from workflow_delivery_activation_state import inspect_activation_state
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_bootstrap import git
    from workflow_delivery_tree import Tree
    from workflow_delivery_trust import Trust
    from workflow_delivery_review import validate_defect_dispositions
    import project_status

    log = STORE / 'logs' / ('owner-lease-recovery-install.jsonl' if install else f'owner-lease-recovery-inspect-{time.time_ns()}.jsonl')
    lock_path = STORE / 'activation.lock'
    descriptor = os.open(lock_path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    with os.fdopen(descriptor, 'a') as lock, log.open('x') as stream:
        if not stat.S_ISREG(os.fstat(lock.fileno()).st_mode):
            raise ValueError('activation lock must be a regular file')
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if lock_path.stat().st_ino != os.fstat(lock.fileno()).st_ino:
            raise ValueError('activation lock changed during acquisition')
        def record(value):
            stream.write(json.dumps(value, sort_keys=True) + '\n')
            stream.flush()
            os.fsync(stream.fileno())
            print(json.dumps(value, sort_keys=True), flush=True)
        record({'stage': 'owner-exception', 'owner_instruction': OWNER_INSTRUCTION,
                'exception': EXPIRED, 'request_sha256': REQUEST_HASH,
                'activation_asserted': False, 'milestone_completed': False})
        support = verify_bundle(ROOT, AUTHORITY)
        response = (STORE / 'owner-responses/lease-renewal-681d51e1.json').read_bytes()
        inspect_activation_response(response, response_sha256=hashlib.sha256(response).hexdigest(), request_sha256=REQUEST_HASH)
        workspace = promotion_workspace(ROOT, candidate=AUTHORITY, authority=AUTHORITY)
        options = dict(base=request['base'], candidate=AUTHORITY, input_roots=request['input_roots'],
                       expected_local_trust=request['prior_local_trust'], publication_review=request['publication_review'], workspace=workspace)
        before = preflight(ROOT, **options)
        original_validate = project_status.validate
        exceptions = []
        def owner_exception(*args, **kwargs):
            failures, state = original_validate(*args, **kwargs)
            matches = [failure for failure in failures if failure == EXPIRED]
            if len(matches) != 1:
                raise ValueError('expected exactly the original candidate expired-lease diagnostic')
            exceptions.extend(matches)
            return [failure for failure in failures if failure != EXPIRED], state
        project_status.validate = owner_exception
        try:
            inspection = inspect_promotion_candidate(ROOT, base=request['base'], candidate=AUTHORITY,
                authority=AUTHORITY, environment_path=request['environment_path'], publication_review=request['publication_review'])
        finally:
            project_status.validate = original_validate
        if not exceptions:
            raise ValueError('lease exception was not explicitly observed')
        tree = Tree(ROOT, revision=AUTHORITY)
        trust = Trust(tree, AUTHORITY, request['base'])
        item = next(i for i in tree.json('specs/active_frontier.json')['frontier'] if i['key'] == 'WORKFLOW-DELIVERY-IMPLEMENTATION')
        contract = tree.json(item['completion']['delivery']['contract_path'])
        validate_defect_dispositions(tree, tree.json(contract['review_path']), contract['review_path'], trust)
        if preflight(ROOT, **options) != before or verify_bundle(ROOT, AUTHORITY) != support:
            raise ValueError('state changed during inspection')
        record({'stage': 'inspection-complete', 'inspection': inspection, 'lease_exception_applied': exceptions,
                'ordinary_preflight_pass': True, 'ordinary_full_activation_pass': False})
        if not install:
            return
        with ActivationSignals(record) as signals:
            signals.lock_fd = lock.fileno()
            record({'stage': 'publication-starting', 'candidate': AUTHORITY})
            if git(ROOT, 'rev-parse', 'HEAD').decode().strip() != request['base']:
                raise ValueError('HEAD moved before publication')
            signals.write_git(ROOT, 'merge', '--ff-only', AUTHORITY)
            proposed = request['proposed_local_trust']
            expected_trust = dict(request['prior_local_trust'])
            for key in [k for k in proposed if k != 'core.hooksPath'] + ['core.hooksPath']:
                state = protected_state(ROOT, request['input_roots'])
                if state['head'] != AUTHORITY or state['local_trust'] != expected_trust or git(ROOT, 'status', '--porcelain=v1', '--untracked-files=all'):
                    raise ValueError('HEAD, trust or worktree changed during installation')
                record({'stage': 'config-starting', 'key': key, 'value': proposed[key]})
                signals.write_git(ROOT, 'config', '--local', '--replace-all', key, proposed[key][0])
                expected_trust[key] = proposed[key]
            for key, values in proposed.items():
                if git(ROOT, 'config', '--get-all', key).decode().splitlines() != values:
                    raise ValueError('effective trust differs from approved value: ' + key)
        if verify_bundle(ROOT, AUTHORITY) != support:
            raise ValueError('support changed during installation')
        state = inspect_activation_state(ROOT, base=request['base'], candidate=AUTHORITY,
            input_roots=request['input_roots'], prior_trust=request['prior_local_trust'],
            proposed_trust=request['proposed_local_trust'], workspace=workspace)
        if state['state'] != 'candidate_and_configuration_match_unverified':
            raise ValueError('installed state differs from exact approved candidate and configuration')
        record({'stage': 'installed-awaiting-ordinary-lease-renewal-and-verification',
                'activation_asserted': False, 'milestone_completed': False})


if __name__ == '__main__':
    main()
