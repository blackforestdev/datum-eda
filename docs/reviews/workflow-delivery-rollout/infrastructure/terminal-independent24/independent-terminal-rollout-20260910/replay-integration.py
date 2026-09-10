"""Adapt established exact restoration to the separately inspected I04 candidate."""
import hashlib
import json
from pathlib import Path
import sys

STORE=Path(__file__).parent
prior=Path('/home/bfadmin/Documents/datum-eda/.git/datum-wdq/proposals/independent-sequencing-20260910/replay-real-prestates.py')
raw=prior.read_bytes()
code=raw.decode()
changes={
    'sequencing-repair-20260910':'terminal-owner-20260910',
    '1d48f249dc7071fc3718b345a4ab16b366af43be':'9924678c39fe5d176e58ebfd8e25ab6feb213f35',
    'sequencing-roadmap-377323b2':'terminal-real-roadmap-1644',
    '8b9f667fe2d8f406edf2e790060b7329393c2acc939723c69f6954ee4f072f6b':'f6912f89b8bb078b619b85292e5fa1415916eec065c8d45dcce942874f432b78',
    'sequencing-paired-evidence-20260910/owner-reopened':'terminal-integration-1644',
}
for old,new in changes.items():
    assert old in code
    code=code.replace(old,new)
old="git_seed('clone','--no-checkout',str(BUNDLE),str(seed))"
new="""request=J(BUNDLE.with_name('request.json'))
snapshot=J(BUNDLE.with_name('result.json'))
assert request['source_commit']==snapshot['source_commit']=='1644e2129f4ff83cec1cdf0e6e13534c3b258e8e'
assert snapshot['bundle_sha256']==BUNDLE_SHA
heads={line.split()[1]:line.split()[0] for line in git_seed('bundle','list-heads',str(BUNDLE)).decode().splitlines()}
assert heads[request['source_ref']]==request['source_commit']
assert all(heads[ref]==oid for ref,oid in request['evidence_refs'].items())
git_seed('init','--quiet',str(seed))
git_seed('-C',str(seed),'fetch','--quiet','--no-tags',str(BUNDLE),request['source_ref'],*sorted(request['evidence_refs']))
save(own/'explicit-bundle-fetch.json',{'request_sha256':H(BUNDLE.with_name('request.json').read_bytes()),'snapshot_result_sha256':H(BUNDLE.with_name('result.json').read_bytes()),'bundle_sha256':BUNDLE_SHA,'heads':heads,'source_ref':request['source_ref'],'evidence_refs':request['evidence_refs'],'reason':'Fetch explicitly recorded non-head refs from the same independently hashed bundle; no external objects or regenerated history. Current descriptors bind the source bundle path; this retained request/result supplies its exact hash.'})"""
assert code.count(old)==1
code=code.replace(old,new)
old="assert descriptor['source_bundle_sha256']==BUNDLE_SHA and Path(descriptor['source_bundle'])==BUNDLE"
assert code.count(old)==1
code=code.replace(old,"assert Path(descriptor['source_bundle'])==BUNDLE and snapshot['bundle_sha256']==BUNDLE_SHA")
old="save(own/'observations.json',{'candidate':PIN,"
assert code.count(old)==1
code=code.replace(old,"save(own/'observations.json',{'candidate':PIN,'runtime_source':PIN,'integration_candidate':'1644e2129f4ff83cec1cdf0e6e13534c3b258e8e',")
record={'template':str(prior),'template_sha256':hashlib.sha256(raw).hexdigest(),
    'changes':changes,'executed_source_sha256':hashlib.sha256(code.encode()).hexdigest(),
    'bundle_binding':'Exact externally pinned snapshot request/result and bundle hash; missing per-case hash field is not manufactured.',
    'scope':'Only after terminal producer custody release; same-path exact restoration, original timeout and no source/claim/cache regeneration.'}
with (STORE/('integration-'+sys.argv[1]+'-adapter.json')).open('x') as f:json.dump(record,f,indent=2)
exec(compile(code,str(prior)+'[terminal-I04-exact-inputs]','exec'),{'__name__':'__main__','__file__':str(Path(__file__))})
