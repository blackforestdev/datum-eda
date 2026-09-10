"""Retain final independent non-authorizing inspection and exact delta receipt."""
import hashlib,json,tarfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
H=lambda raw:hashlib.sha256(raw).hexdigest()
J=lambda p:json.loads(p.read_bytes())
verification=J(STORE/'final-live-review/independent-verification.json')
trace=J(STORE/'final-live-trace-verification.json')
assert verification['exit_code']==0 and trace['process']['exit_code']==0
assert verification['actual_main_clean_before_after'] and trace['protected_state_equal']
for field in ('evidence_validated','publication_authorized','activation_asserted'):assert verification[field] is False
files=[(p,'final-live-review/'+p.name) for p in sorted((STORE/'final-live-review').iterdir()) if p.is_file()]
for name in ('inspect-live.py','inspect-final-live.py','verify-final-live.py','final-live-request.json',
             'final-delta-reconciliation.json','final-uninstalled-support.json','final-live-trace-verification.json'):
    files.append((STORE/name,name))
log=Path(verification['log_path']);assert H(log.read_bytes())==verification['log_sha256']
files.append((log,'supported-cli-log.json'))
members=[{'path':name,'sha256':H(p.read_bytes()),'size':p.stat().st_size} for p,name in sorted(files,key=lambda r:r[1])]
archive=DOCS/'independent-sequencing-final-inspection.tar.xz'
report=DOCS/'independent-sequencing-final-inspection.json'
assert not archive.exists() and not report.exists()
with tarfile.open(archive,'x:xz') as t:
    for p,name in files:t.add(p,arcname=name,recursive=False)
with tarfile.open(archive) as t:
    actual={m.name:{'sha256':H(t.extractfile(m).read()),'size':m.size} for m in t if m.isfile()}
assert actual=={r['path']:{k:r[k] for k in ('sha256','size')} for r in members}
value={'schema_version':1,'step':'WDQ-RECHECK','issue_id':'dat-wdq-rollout-implementation-ffy',
 'reviewer_session':'wdq-independent-review-20260906','verification':verification,'trace_verification':trace,
 'delta_reconciliation':J(STORE/'final-delta-reconciliation.json'),
 'producer_packet_sha256':'fa0b802e11d5dc266f7f3e78bde8a1ac91ac6c1ae4165df8858b56b94c158fee',
 'review_sha256':'10f3ffee6c48d247bb88ecf1a4124b293daeb2b3f8e5a4bb651bbca28dae843b',
 'review_file_sha256':'b352f9b2fe495ba95f46f468f3ca3c2e34b15cb1464857e2ee339ce1b70d5789',
 'replay_sha256':'bbfdcf2ae7a2a107ba3cbdb3aa2358ec85afec072323ecb9fd5348f83f151f83',
 'technical_review_recommendation':'RECHECK technical criteria are satisfied for exact992ce/FA:1314 fresh exact observations, current-source focused45, independent build, validated typed replay/review, and renewed actual-main source/history inspection with later-delta reconciliation. The coordinating lane may record technical closeout and present the owner-only I04 boundary.',
 'ordering':'This later receipt does not alter the earlier typed replay or pretend its inspection happened earlier. It explicitly reconciles the distinct final evidence candidate. No self-referential proof dependency was introduced.',
 'owner_obligations':{'blocking':['dat-wdq-workspace-inputs-zmt','dat-wdq-index-refresh-ogw','dat-wdq-proof-renewal-cycle-qgb','dat-wdq-review-publication-cycle-a1y'],
    'nonblocking':['dat-wdq-test-default-branch-apx'],'disposition_refs':'All remain null in reviewed review.json. No owner choice or issue closure is inferred.',
    'strict_i04':'992ce selects RECHECK, not claim-free I04, and is not activation-ready. A later real I04 base/candidate must preserve exact reviewed source/proof/replay, explicitly reconcile governance/owner receipt changes, and pass strict full-evidence inspection before any separately authorized promotion/installed verification.'},
 'preservation':'Actual main remained clean at8423 throughout supported capture. All55 other Frontier rows, canonical Beads bytes,164 source inputs and54 imported producer/reviewer payloads were checked. New support bytes are uninstalled at the exact Git-common992ce location; installed legacy hooks/config remained identical.',
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,
    'member_count':len(members),'every_member_hash_and_size_checked':True},'members':members,
 'main_roadmap_changed_by_reviewer':False,'owner_acceptance_asserted':False,'publication_authorized':False,'activation_performed':False}
with report.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'report_sha256':H(report.read_bytes()),'members':len(members)}))
