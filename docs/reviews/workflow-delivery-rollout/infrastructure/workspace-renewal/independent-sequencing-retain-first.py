"""Retain terminal independent evidence without including live batch directories."""
import hashlib,json,tarfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
H=lambda b:hashlib.sha256(b).hexdigest()
J=lambda p:json.loads(p.read_bytes())
archive=DOCS/'independent-sequencing-first-batch.tar.xz'
report=DOCS/'independent-sequencing-first-batch.json'
assert not archive.exists() and not report.exists()
directories=['s01','s02-coverage','s03','workspace-proof-input-exact','live-review','ownership-expiry-diagnostic']
files=[]
for name in directories:
    base=STORE/name
    # Retained exact fixtures remain at explicit local paths. This archive
    # retains all captured bytes/identities and does not claim portability.
    for p in base.rglob('*'):
        if 'fixture' not in p.relative_to(base).parts and p.is_file():
            assert not p.is_symlink();files.append((p,p.relative_to(STORE).as_posix()))
for name in ['construction-command.json','construction-started.json','construction-process.json',
             'construction-stdout.bin','construction-stderr.bin','construction-verification.json',
             'ownership-expiry-command.json','ownership-expiry-started.json','ownership-expiry-process.json',
             'ownership-expiry-stdout.bin','ownership-expiry-stderr.bin','diagnostic-provenance.json',
             'workspace-proof-exact-started.json','workspace-proof-exact-process.json',
             'workspace-proof-exact-stdout.bin','workspace-proof-exact-stderr.bin',
             'run-exact-batch.py','verify-exact-batch.py','replay-workspace-prestates.py',
             'verify-workspace-exact.py','inspect-live.py','formatter-before.json','observe-formatter.py']:
    p=STORE/name;assert p.is_file();files.append((p,name))
log=ROOT/'.git/datum-wdq/logs/independent-sequencing-recheck-20260910.json'
files.append((log,'live-review/supported-log.json'))
inventory=[{'path':name,'sha256':H(path.read_bytes()),'size':path.stat().st_size} for path,name in sorted(files,key=lambda pair:pair[1])]
with tarfile.open(archive,'x:xz') as t:
    for path,name in sorted(files,key=lambda pair:pair[1]):t.add(path,arcname=name,recursive=False)
with tarfile.open(archive) as t:
    actual={m.name:H(t.extractfile(m).read()) for m in t if m.isfile()}
assert actual=={row['path']:row['sha256'] for row in inventory}
verifications={b:J(STORE/b/'independent-verification.json') for b in ['s01','s02-coverage','s03','workspace-proof-input-exact','live-review']}
value={'schema_version':1,'step':'WDQ-RECHECK','reviewer_session':'wdq-independent-review-20260906',
 'source_commit':'1d48f249dc7071fc3718b345a4ab16b366af43be',
 'source_input_manifest_sha256':'468e860972610aca1b57e152e34f2ca817a74b7412cbd25e0492e362ae771857',
 'authority_sha256':'b3130c9322932a9d26e967d6ecb55c8b1591892636d26ac3cdd98e175475ee39',
 'construction':J(STORE/'construction-verification.json'),'verifications':verifications,
 'required_exact_observations':376,'original_matrix_subset':331,'workspace_subset':45,
 'supported_pre_review_inspections':1,'retained_failed_captures':1,
 'failed_attempt':{'invocation_id':'84747179-b370-4225-8e5d-71e26b54e55e','actual':'WDQ-TRANSITION: NEXT claim expired at 2026-09-10T09:23:05Z',
 'expected_original':'WDQ-COVERAGE: historical completion/landing evidence changed',
 'disposition':'Retained failure; existing contract requires newly identified paired producer and independent observations after expiry. Not counted as passing replay.'},
 'paired_workspace_basis':'45 exact pre-state archives restored at producer original /tmp/tmpmulf3m7z after explicit terminal custody transfer; full protected-state equality checked before replay. Each owned restoration removed afterward, recoverable from preserved producer archives.',
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,'member_count':len(inventory),'all_member_hashes_verified':True},
 'inventory':inventory,'retained_store':str(STORE),
 'remaining':['Other original exact replay batches','Remaining exact workspace batches','Real-roadmap 24 independent observations','Final typed replay/review and exact later publication delta reconciliation','Strict full-evidence I04 inspection and owner activation'],
 'scope':'Partial independent execution only. Runtime/fixtures remain identified local prerequisites; duplicate fixture directories are omitted from this archive. No standalone portability, complete matrix, full review, owner acceptance or activation claim.',
 'independent_recheck_complete':False,'publication_authorized':False,'activation_performed':False}
with report.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'members':len(inventory),'report_sha256':H(report.read_bytes())}))
