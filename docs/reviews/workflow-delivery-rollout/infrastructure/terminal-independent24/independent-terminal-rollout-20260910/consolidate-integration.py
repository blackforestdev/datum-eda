import hashlib
import json
from pathlib import Path

ROOT = Path('/home/bfadmin/Documents/datum-eda')
OWN = ROOT / '.git/datum-wdq/proposals/independent-terminal-rollout-20260910'
PRODUCER = ROOT / '.git/datum-wdq/proposals/terminal-integration-1644'
def read(path):
    return json.loads(path.read_bytes())
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

rows, originals, receipts, cleanup_paths = [], [], [], set()
for batch, process_name, count in [('real-roadmap', 'integration-real15', 15), ('current-inventory', 'integration-inventory9', 9)]:
    directory = OWN / (batch + '-exact')
    observation = read(directory / 'observations.json')
    current = observation['observations']
    original = read(PRODUCER / batch / 'observations.json')['invocations']
    report = read(directory / 'independent-verification.json')
    process = read(OWN / (process_name + '-process.json'))
    assert process['exit_code'] == 0
    for stream in ('stdout', 'stderr'):
        assert sha(OWN / (process_name + '-' + stream + '.bin')) == process[stream + '_sha256']
    assert len(current) == len(original) == count
    assert report['integration_candidate'] == '1644e2129f4ff83cec1cdf0e6e13534c3b258e8e'
    for row in current:
        assert row['exact_before_state_equal'] and row['states_unchanged']
        cleanup = read(directory / row['directory'] / 'cleanup.json')
        assert cleanup['restored_owned_fixture_removed'] and cleanup['raw_capture_preserved']
        path = Path(cleanup['path'])
        assert not path.exists() and not path.is_symlink()
        cleanup_paths.add(str(path))
        rows.append(dict(batch=batch, **row))
    originals.extend(row['invocation_id'] for row in original)
    receipts.append({'batch': batch, 'observations_sha256': sha(directory / 'observations.json'),
                     'verification_sha256': sha(directory / 'independent-verification.json'),
                     'process_sha256': sha(OWN / (process_name + '-process.json')), 'verification': report})
ids = [row['invocation_id'] for row in rows]
assert len(ids) == len(set(ids)) == len(originals) == len(set(originals)) == 24
assert not set(ids) & set(originals)
assert sum(row['returncode'] == 0 for row in rows) == 20
assert sum(row['returncode'] == 1 and row['expected_refusal'] for row in rows) == 4
result = {'reviewer_session': 'wdq-independent-review-20260906',
          'integration_candidate': '1644e2129f4ff83cec1cdf0e6e13534c3b258e8e',
          'runtime_source': '9924678c39fe5d176e58ebfd8e25ab6feb213f35',
          'observations': rows, 'batch_receipts': receipts,
          'counts': {'observations': 24, 'success': 20, 'expected_refusal': 4,
                     'artifact_hashes_per_side': 168, 'handler_source_checks_per_side': 900},
          'cross_batch_ids_unique_and_disjoint_from_producer': True,
          'full_restored_prestate_and_within_run_state_equal': True,
          'cleanup_paths_confirmed_absent': sorted(cleanup_paths),
          'failed_replay_attempts': [],
          'scope': 'Separate mandatory I04 integration receipts; typed 1400 evidence unchanged. Prior inspection failures and superseded producer observations remain historical, not counted here.',
          'activation_performed': False, 'installed_verification_claimed': False,
          'product_acceptance_claimed': False}
output = OWN / 'integration-24-complete.json'
with output.open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps({'path': str(output), 'sha256': sha(output), 'counts': result['counts']}))
