"""Retain terminal S06/baseline execution without implying real-roadmap closure."""
import hashlib,json,tarfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
H=lambda b:hashlib.sha256(b).hexdigest()
J=lambda p:json.loads(p.read_bytes())
batches=['s06-selection','s06-headless','s06-trust','baseline']
archive=DOCS/'independent-sequencing-third-batch.tar.xz'
report=DOCS/'independent-sequencing-third-batch.json'
assert not archive.exists() and not report.exists()
files=[]
for batch in batches:
    base=STORE/batch
    for p in base.rglob('*'):
        if 'fixture' not in p.relative_to(base).parts and p.is_file():
            assert not p.is_symlink();files.append((p,p.relative_to(STORE).as_posix()))
for name in ['run-exact-batch.py','verify-exact-batch.py']:
    p=STORE/name;assert p.is_file();files.append((p,name))
inventory=[{'path':n,'sha256':H(p.read_bytes()),'size':p.stat().st_size} for p,n in sorted(files,key=lambda v:v[1])]
with tarfile.open(archive,'x:xz') as t:
    for p,n in sorted(files,key=lambda v:v[1]):t.add(p,arcname=n,recursive=False)
with tarfile.open(archive) as t:
    actual={m.name:H(t.extractfile(m).read()) for m in t if m.isfile()}
assert actual=={r['path']:r['sha256'] for r in inventory}
verifications={b:J(STORE/b/'independent-verification.json') for b in batches}
assert sum(v['stats']['independent']['observations'] for v in verifications.values())==383
value={'schema_version':1,'step':'WDQ-RECHECK','reviewer_session':'wdq-independent-review-20260906',
 'source_commit':'1d48f249dc7071fc3718b345a4ab16b366af43be',
 'source_input_manifest_sha256':'468e860972610aca1b57e152e34f2ca817a74b7412cbd25e0492e362ae771857',
 'authority_sha256':'b3130c9322932a9d26e967d6ecb55c8b1591892636d26ac3cdd98e175475ee39',
 'verifications':verifications,'required_exact_observations_this_archive':383,
 'required_exact_observations_across_three_archives':1290,
 'original_synthetic_observations':1118,'added_workspace_exact_observations':172,
 'supplemental_recipe_observations_exact_credit':0,
 'trust_outcome_precision':'223 outer captures include five deliberately interrupted processes and six pre-observer refusals; they are not described as 223 successful validator executions.',
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,
    'member_count':len(inventory),'all_member_hashes_verified':True},
 'inventory':inventory,'retained_store':str(STORE),
 'remaining':['Real-roadmap 24 independent exact observations','Formal typed replay/review against reconciled producer packet',
    'Later exact publication delta reconciliation','Strict I04 inspection and owner activation'],
 'scope':'Bounded 1290-observation execution is complete across three archives. Supported actual-main pre-review inspection is separately retained in the first archive. Duplicate exact fixture directories remain identified local prerequisites, not a standalone portable replay package. No full RECHECK or activation claim.',
 'independent_recheck_complete':False,'publication_authorized':False,'activation_performed':False}
with report.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'members':len(inventory),'report_sha256':H(report.read_bytes())}))
