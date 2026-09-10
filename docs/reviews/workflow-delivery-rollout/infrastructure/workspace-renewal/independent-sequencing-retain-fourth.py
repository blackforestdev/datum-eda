"""Retain exact real replay and focused regression, including unsuccessful attempts."""
import hashlib,json,os,tarfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
H=lambda b:hashlib.sha256(b).hexdigest()
J=lambda p:json.loads(p.read_bytes())
archive=DOCS/'independent-sequencing-fourth-batch.tar.xz'
report=DOCS/'independent-sequencing-fourth-batch.json'
assert not archive.exists() and not report.exists()
assert J(STORE/'real-roadmap-exact-v3-process.json')['exit_code']==0
assert J(STORE/'current-inventory-exact-v2-process.json')['exit_code']==0
verifications={name:J(STORE/name/'independent-verification.json') for name in
               ['real-roadmap-exact-complete','current-inventory-exact-v2']}
assert sum(v['stats']['independent']['observations'] for v in verifications.values())==24
assert J(STORE/'focused-workspace-v2/process.json')['exit_code']==0
assert J(STORE/'focused-workspace/process.json')['exit_code']==1
files=[]
directories=['real-roadmap-exact-v2','real-roadmap-exact-v3','real-roadmap-exact-complete',
             'current-inventory-exact','current-inventory-exact-v2','focused-workspace','focused-workspace-v2']
for name in directories:
    base=STORE/name
    for directory,dirs,names in os.walk(base,followlinks=False):
        dirs[:]=[d for d in dirs if d not in ('fixture','bundle-seed')]
        for leaf in names:
            p=Path(directory)/leaf;assert p.is_file() and not p.is_symlink()
            files.append((p,p.relative_to(STORE).as_posix()))
helpers=['replay-real-prestates.py','replay-real-prestates-v2.py','replay-real-prestates-v3.py',
    'run-real-replay.py','run-real-replay-v2.py','run-real-replay-v3.py',
    'verify-real-exact.py','verify-real-exact-v2.py','verify-real-complete.py','verify-workspace-exact.py',
    'run-focused-workspace.py','run-focused-workspace-v2.py',
    'verify-producer-target.py','producer-target-independent-verification.json',
    'retire-timeout-restoration.py','real-roadmap-timeout-retirement.json']
for name in helpers:
    p=STORE/name;assert p.is_file();files.append((p,name))
for prefix in ['current-inventory-exact','current-inventory-exact-v2','real-roadmap-exact-v2','real-roadmap-exact-v3']:
    for suffix in ['command.json','started.json','process.json','stdout.bin','stderr.bin']:
        p=STORE/(prefix+'-'+suffix);assert p.is_file();files.append((p,p.name))
inventory=[{'path':name,'sha256':H(p.read_bytes()),'size':p.stat().st_size} for p,name in sorted(files,key=lambda v:v[1])]
assert len({r['path'] for r in inventory})==len(inventory)
with tarfile.open(archive,'x:xz') as t:
    for p,name in sorted(files,key=lambda v:v[1]):t.add(p,arcname=name,recursive=False)
with tarfile.open(archive) as t:
    actual={m.name:H(t.extractfile(m).read()) for m in t if m.isfile()}
assert actual=={r['path']:r['sha256'] for r in inventory}
value={'schema_version':1,'step':'WDQ-RECHECK','issue_id':'dat-wdq-rollout-implementation-ffy',
 'reviewer_session':'wdq-independent-review-20260906','source_commit':'1d48f249dc7071fc3718b345a4ab16b366af43be',
 'source_input_manifest_sha256':'468e860972610aca1b57e152e34f2ca817a74b7412cbd25e0492e362ae771857',
 'authority_sha256':'b3130c9322932a9d26e967d6ecb55c8b1591892636d26ac3cdd98e175475ee39',
 'verifications':verifications,'required_exact_observations_this_archive':24,
 'required_exact_observations_across_four_archives':1314,'original_matrix_observations':1123,'additional_workspace_observations':191,
 'real_replay_accounting':{'inventory':9,'original_real_successes':9,'remaining_real_retry_successes':6,
    'preserved_failed_timeout_invocation':'1331918b-8f21-444e-894a-3b4dcde1f5b1','timeout_seconds':180,'timeout_cause':'Undetermined'},
 'setup_failure':'First inventory seed used plain clone, which omitted non-head bundle refs; checkout refused before fixture restoration or capture. V2 fetches only the original explicit refs from the same hashed bundle. Failed command/process/seed remain retained; no source/claim/history regeneration.',
 'focused_regression':{'initial':J(STORE/'focused-workspace/child-observation.json'),
    'corrected_environment':J(STORE/'focused-workspace-v2/verification.json'),
    'open_finding':'dat-wdq-test-default-branch-apx','test_source_fixed':False,'matrix_observation_credit':0,
    'precondition':'Explicit reviewer-owned mode0600 Git config contains only init.defaultBranch=main. Isolated HOME and no system config retained; runtime Git reads remain independently sanitized.',
    'initial_fixture_limit':'The normal unittest cleanup removed the first failed synthetic fixture; its raw diagnostic/process/source receipts remain, not a claim of retained full fixture.'},
 'producer_target':J(STORE/'producer-target-independent-verification.json'),
 'helper_review':{'commits':['b4a8469b','64d0230e'],
    'finding':'No new paired-real binding blocker found. Both real groups switch together; original1123 multiset/191 additions, explicit mode, distinct namespace/session and packet-pinned committed overlay remain checked.',
    'retention_precision':'First producer real command embeds the older stat-cache-based helper; inventory embeds strengthened literal blob/type/mode checks. No retroactive byte-check claim for the first batch. Independent full restored before-state equality passed for every actual replay.'},
 'commit_message_correction':{'commit':'c55fa057','incorrect_issue':'dat-workflow-gate-implementation-8mx',
    'correct_issue':'dat-wdq-rollout-implementation-ffy','impact':'Commit-message transcription only; evidence/source/counts unchanged. History was not rewritten.'},
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,
    'member_count':len(inventory),'all_member_hashes_verified':True},'inventory':inventory,'retained_store':str(STORE),
 'custody':'Original paired paths restored only after producer terminal custody transfer; pinned bundle plus metadata/payloads and complete pre-state equality. Successful own restorations removed recoverably; failed hook restoration moved intact to real-roadmap-failed-timeout-fixture after absent process-group/open-handle checks. Producer retired fixtures unchanged.',
 'limits':'Duplicate bundle seeds and failed full fixture remain local prerequisites, omitted from this archive. Producer pre-state archives/bundle are separately pinned. No standalone portable replay, native product proof, exhaustive descendant closure or final publication claim.',
 'remaining':['Formal typed independent replay/review','Explicit later final candidate delta reconciliation','Strict I04 inspection and owner dispositions/activation'],
 'independent_recheck_complete':False,'publication_authorized':False,'activation_performed':False}
with report.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'members':len(inventory),'report_sha256':H(report.read_bytes())}))
