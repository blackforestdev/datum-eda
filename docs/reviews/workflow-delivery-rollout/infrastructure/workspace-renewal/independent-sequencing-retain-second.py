"""Retain the next terminal exact batches and separately labeled supplemental runs."""
import hashlib,json,tarfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
H=lambda b:hashlib.sha256(b).hexdigest()
J=lambda p:json.loads(p.read_bytes())
exact=['s02-ownership','s04','s05-snapshots','workspace-policy-legacy-exact','workspace-runtime-exact']
supplemental=['workspace-runtime','workspace-policy-legacy','workspace-proof-input']
archive=DOCS/'independent-sequencing-second-batch.tar.xz'
report=DOCS/'independent-sequencing-second-batch.json'
assert not archive.exists() and not report.exists()
files=[]
for name in exact+supplemental:
    base=STORE/name
    for p in base.rglob('*'):
        if 'fixture' not in p.relative_to(base).parts and p.is_file():
            assert not p.is_symlink();files.append((p,p.relative_to(STORE).as_posix()))
for name in ['run-workspace.py','formatter-before.json','formatter-after.json','observe-formatter.py',
             'workspace-policy-exact-started.json','workspace-policy-exact-process.json',
             'workspace-policy-exact-stdout.bin','workspace-policy-exact-stderr.bin',
             'workspace-runtime-exact-started.json','workspace-runtime-exact-process.json',
             'workspace-runtime-exact-stdout.bin','workspace-runtime-exact-stderr.bin']:
    p=STORE/name;assert p.is_file();files.append((p,name))
for name in supplemental:
    for suffix in ['command.json','started.json','process.json','stdout.bin','stderr.bin']:
        p=STORE/(name+'-'+suffix);assert p.is_file();files.append((p,p.name))
inventory=[{'path':name,'sha256':H(p.read_bytes()),'size':p.stat().st_size} for p,name in sorted(files,key=lambda x:x[1])]
with tarfile.open(archive,'x:xz') as t:
    for p,name in sorted(files,key=lambda x:x[1]):t.add(p,arcname=name,recursive=False)
with tarfile.open(archive) as t:
    actual={m.name:H(t.extractfile(m).read()) for m in t if m.isfile()}
assert actual=={r['path']:r['sha256'] for r in inventory}
verifications={b:J(STORE/b/'independent-verification.json') for b in exact}
before,after=(J(STORE/('formatter-'+name+'.json')) for name in ['before','after'])
assert all(before[k]==after[k] for k in ['path','resolved_path','sha256','size','version_stdout','version_stderr'])
value={'schema_version':1,'step':'WDQ-RECHECK','reviewer_session':'wdq-independent-review-20260906',
 'source_commit':'1d48f249dc7071fc3718b345a4ab16b366af43be',
 'source_input_manifest_sha256':'468e860972610aca1b57e152e34f2ca817a74b7412cbd25e0492e362ae771857',
 'authority_sha256':'b3130c9322932a9d26e967d6ecb55c8b1591892636d26ac3cdd98e175475ee39',
 'verifications':verifications,'required_exact_observations_this_archive':531,
 'required_exact_observations_with_first_archive':907,
 'supplemental_recipe_observations':{'count':172,'required_exact_replay_credit':False,
    'reason':'Fresh synthetic recipes generated distinct fixture histories; retained as supplemental only. Required workspace evidence uses separately paired exact same-path pre-state replay.'},
 'formatter_before_after':{'before':before,'after':after,'byte_and_version_identity_unchanged':True},
 'custody_cleanup':'Policy /tmp/tmpd20fvjpz and runtime /tmp/tmp1044_7x2 were restored atomically only after producer terminal absence and explicit custody transfer; every protected pre-state matched. Each owned restoration removed after successful replay; original pre-state archives remain recoverable.',
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,'member_count':len(inventory),'all_member_hashes_verified':True},
 'inventory':inventory,'retained_store':str(STORE),
 'remaining':['S06 and baseline exact replays','Real-roadmap 24 independent exact observations','Formal typed replay/review and later exact publication delta reconciliation','Strict I04 inspection and owner activation'],
 'scope':'Partial independent execution. Duplicate exact fixture directories remain identified local prerequisites rather than being repeated in the archive. Supplemental final synthetic fixture archives are retained; no standalone portability or full review claim.',
 'independent_recheck_complete':False,'publication_authorized':False,'activation_performed':False}
with report.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'members':len(inventory),'report_sha256':H(report.read_bytes())}))
