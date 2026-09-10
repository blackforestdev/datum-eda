"""Retain the independently validated technical review, not owner acceptance."""
import hashlib
import json
from pathlib import Path
import tarfile

ROOT=Path(__file__).resolve().parents[5]
STORE=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910'
DOCS=Path(__file__).parent
OVERLAY=STORE/'typed-review-attempt-02'
J=lambda p:json.loads(p.read_bytes())
H=lambda raw:hashlib.sha256(raw).hexdigest()
result=J(STORE/'typed-review-attempt-02-result.json')
assert J(STORE/'typed-review-attempt-02-process.json')['exit_code']==0
assert J(STORE/'typed-review-attempt-01-process.json')['exit_code']==1
actual=[{'path':p.relative_to(OVERLAY).as_posix(),'sha256':H(p.read_bytes()),'size':p.stat().st_size}
        for p in sorted(OVERLAY.rglob('*')) if p.is_file()]
assert actual==result['inventory']
archive=DOCS/'independent-sequencing-review.tar.xz'
descriptor=DOCS/'independent-sequencing-review.json'
assert not archive.exists() and not descriptor.exists()
files=[(OVERLAY/r['path'],'overlay/'+r['path']) for r in actual]
for name in ('assemble-typed-review.py','assemble-typed-review-v2.py',
    'typed-review-attempt-01-process.json','typed-review-attempt-01-stdout.bin','typed-review-attempt-01-stderr.bin',
    'typed-review-attempt-02-process.json','typed-review-attempt-02-stdout.bin','typed-review-attempt-02-stderr.bin',
    'typed-review-attempt-02-result.json'):
    files.append((STORE/name,'construction/'+name))
members=[{'path':name,'sha256':H(p.read_bytes()),'size':p.stat().st_size} for p,name in sorted(files,key=lambda row:row[1])]
with tarfile.open(archive,'x:xz') as t:
    for p,name in files:t.add(p,arcname=name,recursive=False)
with tarfile.open(archive) as t:
    reread={m.name:{'sha256':H(t.extractfile(m).read()),'size':m.size} for m in t if m.isfile()}
assert reread=={r['path']:{k:r[k] for k in ('sha256','size')} for r in members}
typed=OVERLAY/'docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/independent-sequencing-typed'
validation=J(typed/'validation.json')
assert validation['proof_environment_correlations_independent_review']=='pass'
findings=[
 {'issue_id':'dat-wdq-workspace-inputs-zmt','severity':'blocking','technical_assessment':'Owner-pinned classification and source-only startup were independently exercised across exact172 workspace cases, mixed real24, and current-source focused45. Unknown/changed source, forged caches, policy drift and explicit proof inputs continue to refuse. No blanket ignore filtering or owner-file deletion was used. Installed verification and exact owner resolution remain pending.'},
 {'issue_id':'dat-wdq-index-refresh-ogw','severity':'blocking','technical_assessment':'Exact post-setup replay includes stale index/cache inputs and divergent staged/history surfaces. All1314 required before/after protected states match; the original selector index-byte mutation failure remains historical. Repair uses isolated temporary index and selected-index untracked enumeration; no GIT_OPTIONAL_LOCKS workaround counts as repair proof. Exact owner resolution remains pending.'},
 {'issue_id':'dat-wdq-proof-renewal-cycle-qgb','severity':'blocking','technical_assessment':'Explicit owner-approved reopening, identified prospective baseline and exact paired real fixtures allowed honest producer then independent proof renewal. Historical failed old-manifest captures and prior approvals remain intact. This verifies the authorized sequencing, not an autonomous right to reopen checkpoints; exact owner resolution remains pending.'},
 {'issue_id':'dat-wdq-review-publication-cycle-a1y','severity':'blocking','technical_assessment':'Actual supported --inspect-review ran against clean real main e54fe and candidate ee342 with unchanged protected state and evidence/publication/activation flags false. Producer and independent packets no longer depend on their own later full-evidence inspection. Later candidate delta reconciliation and renewed exact-publication inspection are still required before RECHECK closeout/I04 presentation; strict owner-only I04 checks remain. Exact owner resolution remains pending.'},
 {'issue_id':'dat-wdq-test-default-branch-apx','severity':'nonblocking','technical_assessment':'The first isolated45 run passed44 and correctly refused attached-main preflight because plain git init defaulted to master. A distinct45 run passed with only a pinned reviewer-owned init.defaultBranch=main profile; no runtime/test source changed. This is retained non-hermetic test-setup debt, not a fixed defect. Owner DEFER or other explicit disposition remains required.'},
]
value={'schema_version':1,'step':'WDQ-RECHECK','issue_id':'dat-wdq-rollout-implementation-ffy',
 'reviewer_session':'wdq-independent-review-20260906','source_commit':'1d48f249dc7071fc3718b345a4ab16b366af43be',
 'producer_candidate':validation['candidate'],'producer_packet_sha256':validation['producer_packet_sha256'],
 'authority_sha256':'b3130c9322932a9d26e967d6ecb55c8b1591892636d26ac3cdd98e175475ee39',
 'input_manifest_sha256':'468e860972610aca1b57e152e34f2ca817a74b7412cbd25e0492e362ae771857',
 'technical_disposition':'approve','validation':validation,'overlay':result,
 'evidence_accounting':{'required_fresh_observations':1314,'original':1123,'additional_workspace':191,
    'focused_tests':45,'focused_matrix_credit':0,'actual_main_pre_review_inspection':1,
    'source_modules_built_in_memory':151,'source_inputs':164,
    'binding':'Same exact producer input-manifest/fixture/environment Blob hashes and build command/toolchain/interpreter identity; independent session and all raw event/invocation IDs disjoint. All required scenario assertions pass. Contract N/A dimensions and reasons explicitly enumerated in provenance.'},
 'independence':'This reviewer did not implement production source or create producer proof. Only independently executed captures, construction and read-only inspection observations supply the verdict. Original timestamps are preserved, including captures before formal closeout.',
 'findings':findings,'owner_receipt':None,'owner_disposition_refs':[None]*5,
 'prior_dispositions':'Five earlier owner resolutions remain historical and scoped to their earlier packet/replay; they do not resolve these five findings or accept this review.',
 'failures_preserved':'Expired original ownership lease; original bundle-ref seed failure; actual180s hook timeout of undetermined cause; first44/45 default-branch test result. The first typed assembly also failed before artifacts/verdict on a producer-row schema assumption; v2 reads actual hash-bound raw trace identities. No runtime/source/clock/timeout was changed and no successful capture was rerun to conceal failure.',
 'helper_review':{'commit':'186475d83e967926e6f7b333ff9d200f96bdb5c2',
    'assessment':'No blocking issue found in bounded exact producer/review import. Fixed Git producer pin, complete hashed overlay inventory, lane checks and symlink refusal preserve payload identity. Six tests exercise import only, not independent review or publication; final prepared candidate must separately validate the exact review and undergo independent delta inspection.'},
 'publication_handoff':{'payload':'Import proof.json and paired-real-typed/ only from exact06ba. Import canonical review.json and independent-sequencing-typed/ only from this exact overlay inventory. Four previously committed independent archives are referenced from main; do not replace them with different bytes.',
    'reviewed_earlier_base':'e54fe5562174ef3f3c6748bb2c9c649626e2f652',
    'reviewed_earlier_candidate':'ee3425a23d3c220b9303d26ebfa89f0ae6f1b19d',
    'remaining_before_i04':'Prepare an actual-main descendant preserving all other55 Frontier lanes and canonical tracker changes. Independently reconcile every later evidence/helper/intake/review delta against the inspected identities and run the exact renewed supported source/history inspection. No current-source matrix rerun is implied unless relevant source/input/authority actually changes.',
    'owner_boundary':'Five exact owner dispositions, final full-evidence I04 inspection and owner-controlled promotion/installed verification remain required. This report does not authorize activation or mark RECHECK complete.'},
 'archive':{'path':archive.relative_to(ROOT).as_posix(),'sha256':H(archive.read_bytes()),'size':archive.stat().st_size,
    'member_count':len(members),'every_member_rehashed':True},'archive_members':members,
 'scope_limits':'Infrastructure pipes/protocol and governed snapshot tests only, not native product/GUI acceptance. Raw fixture/object prerequisites remain in retained Git-common stores and producer archives; no standalone portable replay claim.',
 'independent_recheck_complete':False,'publication_authorized':False,'activation_performed':False}
with descriptor.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
print(json.dumps({'archive_sha256':value['archive']['sha256'],'descriptor_sha256':H(descriptor.read_bytes()),
    'overlay_inventory_sha256':result['inventory_sha256'],'review_file_sha256':result['review']['sha256'],
    'review_sha256':result['review_sha256'],'replay_sha256':result['replay']['sha256']}))
