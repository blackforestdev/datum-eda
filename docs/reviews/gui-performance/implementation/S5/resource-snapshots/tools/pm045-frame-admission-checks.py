"""Offline parser controls using archived records, never native qualification."""
import copy
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


parser = load('frame_parser', 'pm045-frame-admission-analyze.py')
snapshots = load('snapshot_parser', 'pm045-resource-snapshot-analyze.py')
s5 = Path(__file__).resolve().parents[2]
raw_path = s5 / 'frame-admission-delivery/screen-submission/trace.gz'
raw = [json.loads(line) for line in gzip.decompress(raw_path.read_bytes()).splitlines()]
frames = [row for row in raw if row['phase'] == 'frame']
# The actual component trace deliberately ends in overflow. Its preserved raw
# result must fail; a synthetic clean envelope only exercises parser positives.
positive = copy.deepcopy(frames) + [{'phase': 'end', 'frames': len(frames), 'frame_delivery_failed': False}]
controls = {}


def rejected(label, rows, origins=None):
    try:
        parser.check(rows, origins)
    except (AssertionError, KeyError, IndexError, TypeError):
        controls[label] = 'rejected'
    else:
        raise AssertionError(('invalid input accepted', label))


rejected('actual-overflow-trace', raw)
base = parser.check(positive)
assert base['frames'] == len(frames) and not base['qualification_pass']
controls['synthetic-clean-envelope'] = 'record consistency checked, no acceptance'
for label, mutate in [
    ('sequence-gap', lambda r: r[0].__setitem__('sequence', 2)),
    ('count-mismatch', lambda r: r[-1].__setitem__('frames', len(frames) + 1)),
    ('unavailable-screen', lambda r: next(f for f in r if f.get('submitted_frame') is True).__setitem__('submitted_screen_geometry', None)),
    ('wrong-screen-vertices', lambda r: next(f for f in r if f.get('submitted_screen_geometry'))['submitted_screen_geometry'][0].__setitem__('submitted_vertices', 1)),
    ('wrong-grid-triangles', lambda r: next(f for f in r if f.get('submitted_frame') is True and f.get('prepared_surface_grids'))['prepared_surface_grids'][0].__setitem__('submitted_triangles', -1)),
    ('duplicate-text-origin', lambda r: next(f for f in r if f.get('text_origins'))['text_origins'].append(copy.deepcopy(next(f for f in r if f.get('text_origins'))['text_origins'][0]))),
    ('wrong-text-total', lambda r: next(f for f in r if f.get('text_admission'))['text_admission']['overlay'].__setitem__('runs', 999999)),
]:
    rows = copy.deepcopy(positive)
    mutate(rows)
    rejected(label, rows)
rejected('unknown-renderer', positive, set())
missing = copy.deepcopy(positive)
for frame in missing[:-1]:
    for pane in frame['world_panes']:
        pane['source'] = None
assert parser.check(missing)['admission_gaps']['missing_world_source_identity'] > 0
controls['missing-source-identity'] = 'reported as admission gap, not zero or pass'
archive = s5 / 'resource-snapshots/on-1.tar.gz'
with tarfile.open(archive) as bundle:
    historical = [json.loads(line) for line in bundle.extractfile('resources.jsonl')]
origins = {h['reservations']['renderer_id'] for r in historical if r['phase'] == 'snapshot' for h in r['hosts']}
history = snapshots.check(historical, origins)
assert not history['frame_admission']['available']
controls['historical-snapshot-trace'] = 'existing snapshot checks preserved; frame admission unavailable'
# Synthetic integration control: replay archived frame payloads in an archived
# lifecycle envelope, remapping renderer identity explicitly. This is not a run.
mixed = copy.deepcopy(historical)
index = next(i for i, r in enumerate(mixed) if r['phase'] == 'snapshot' and any(h['present'] for h in r['hosts']))
renderer = next(h['reservations']['renderer_id'] for h in mixed[index]['hosts'] if h['present'])
injected = copy.deepcopy(frames)
for row in injected:
    row['renderer_id'] = renderer
mixed[index + 1:index + 1] = injected
mixed[-1].update(frames=len(injected), frame_delivery_failed=False)
result = snapshots.check(mixed, origins)
assert result['snapshots'] == history['snapshots']
assert result['frame_admission']['frames'] == len(injected)
assert not result['qualification_pass']
controls['synthetic-mixed-envelope'] = 'frame and snapshot checks both exercised; no acceptance'
late = copy.deepcopy(historical)
late[-1].update(frames=1, frame_delivery_failed=False)
row = copy.deepcopy(frames[0])
row.update(renderer_id=renderer, sequence=1)
late.insert(-1, row)
try:
    snapshots.check(late, origins)
except AssertionError:
    controls['frame-after-retirement'] = 'rejected'
else:
    raise AssertionError('frame after renderer retirement accepted')
print(json.dumps({'qualification_pass': False, 'controls': controls,
                  'observed_record_gaps': base['admission_gaps'],
                  'inputs': {str(p.relative_to(s5)): hashlib.sha256(p.read_bytes()).hexdigest() for p in (raw_path, archive)},
                  'limits': 'Offline parser conformance only. Synthetic envelopes/remapped identities are not execution evidence; actual overflow failure preserved.'}, indent=2))
