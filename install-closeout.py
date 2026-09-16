"""Owner-invoked, exact reviewed closeout maintenance; no lease exception."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/home/bfadmin/Documents/datum-eda')
BASE = '9b26980815a63d65987e1f1b1d7a6ca2ac270599'
CANDIDATE = 'ac33c634092967f71364bf61174119bca6040de8'
OLD = '61f7b2db1730f0dc7911b5a05f623ad6bf2d1000'
ENV = 'specs/workflow_delivery/rollout.environments.json'
PATHS = sorted([
    '.beads/issues.jsonl', 'docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md',
    'docs/reviews/workflow-delivery-rollout/adoption/handoff.json',
    'scripts/test_workflow_delivery_source_scopes.py', 'scripts/workflow_delivery_checkpoints.py',
    'specs/PROGRESS.md', 'specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md',
    'specs/active_frontier.json', 'specs/evidence_traceability_manifest.json'])


def main():
    if sys.argv[1:] != ['--install'] or not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError('requires -I -S -B and explicit --install')
    source = ROOT / '.git/datum-wdq/trusted' / CANDIDATE / 'scripts'
    for name in ('workflow_delivery_bootstrap.py', 'workflow_delivery_source_only.py'):
        path = source / name
        expected = subprocess.check_output(['git', '--no-replace-objects', 'show', CANDIDATE + ':scripts/' + name], cwd=ROOT)
        if path.is_symlink() or path.read_bytes() != expected:
            raise ValueError('preparation bootstrap source changed')
    bootstrap = source / 'workflow_delivery_bootstrap.py'
    bootstrap_namespace = {'__name__': '_verified_bootstrap', '__file__': str(bootstrap)}
    exec(compile(bootstrap.read_bytes(), str(bootstrap), 'exec'), bootstrap_namespace)
    bootstrap_namespace['verify_support'](ROOT, CANDIDATE)
    namespace = {'__name__': '_source_only'}
    path = source / 'workflow_delivery_source_only.py'
    exec(compile(path.read_bytes(), str(path), 'exec'), namespace)
    namespace['install'](source)
    sys.path.insert(0, str(source))
    from workflow_delivery_support_bundle import verify_bundle
    from workflow_delivery_support_store import proposed_local_trust, support_locations
    from workflow_delivery_publication_delta import publication_delta
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_tree import Tree
    from workflow_delivery_shapes import require
    from argparse import Namespace
    from check_workflow_delivery import run
    import workflow_delivery_activation as activation
    root_packet = (ROOT / '.git/datum-wdq/proposals/lease-renewal-activation-20260915/activation-request.json').read_bytes()
    if hashlib.sha256(root_packet).hexdigest() != '681d51e12c7cd0d5e2a363091136e35dd774b82b4d885aad742bd02311f489be':
        raise ValueError('reviewed input-root packet changed')
    roots = json.loads(root_packet)['input_roots']
    prior = proposed_local_trust(ROOT, authority=OLD, base='19e948ba913595ec3aeffb60660167e419642491', environment_path=ENV)
    proposed = proposed_local_trust(ROOT, authority=CANDIDATE, base=BASE, environment_path=ENV)
    delta = publication_delta(ROOT, base=BASE, candidate=CANDIDATE)
    if delta['touched_paths'] != PATHS:
        raise ValueError('closeout delta differs from independently reviewed nine paths')
    locations = support_locations(ROOT, CANDIDATE)
    verify_bundle(ROOT, CANDIDATE)
    request = dict(schema_version=1, kind='owner-directed-closeout-maintenance', base=BASE,
        candidate=CANDIDATE, authority=CANDIDATE, environment_path=ENV, input_roots=roots,
        prior_local_trust=prior, proposed_local_trust=proposed,
        publication_review={'delta_sha256': sha256(canonical_json(delta)), 'paths': PATHS},
        coordination=activation.COORDINATION)
    def inspect(root, *, base, candidate, authority, environment_path, publication_review):
        if (Path(root), base, candidate, authority, environment_path) != (ROOT, BASE, CANDIDATE, CANDIDATE, ENV):
            raise ValueError('maintenance pins changed')
        before = Tree(ROOT, revision=BASE).json('specs/active_frontier.json')['frontier']
        after = Tree(ROOT, revision=CANDIDATE).json('specs/active_frontier.json')['frontier']
        if len(before) != len(after):
            raise ValueError('Frontier inventory changed')
        for old, new in zip(before, after):
            if old['key'] != 'WORKFLOW-DELIVERY-IMPLEMENTATION' and old != new:
                raise ValueError('another lane changed')
        result = run(Namespace(root=ROOT, staged=False, candidate_ref=CANDIDATE, enforce=True,
            authority_ref=CANDIDATE, base_ref=BASE, environment_path=ENV, contract=None))
        return {'checks': result, 'scope': 'owner-approved closeout maintenance; no lease exception or product acceptance'}
    # Use the original guarded mutation/verification transaction. Replace only
    # its fixed historical repair profile with this exact independently reviewed
    # maintenance scope and the ordinary enforcing candidate validator.
    activation.inspect_promotion_candidate = inspect
    log = ROOT / '.git/datum-wdq/logs/guardrail-closeout-install.jsonl'
    with log.open('x') as stream:
        def record(value):
            stream.write(json.dumps(value, sort_keys=True) + '\n')
            stream.flush()
            os.fsync(stream.fileno())
            print(json.dumps(value, sort_keys=True), flush=True)
        record({'stage': 'owner-directed-maintenance', 'request': request,
            'owner_scope_record': 'docs/reviews/workflow-delivery-rollout/scope-correction-owner-20260916.json',
            'independent_review': 'No blocking finding; 17 focused tests, PM025, projection, traceability, governance and source-health passed.',
            'full_suite_repeated': False, 'milestone_completed': False})
        result = activation.activate(ROOT, request, record=record)
        record({'stage': 'maintenance-result', 'result': result})


if __name__ == '__main__':
    main()
