import pathlib,json,hashlib,subprocess,collections,sys
root=pathlib.Path.cwd();batch=sys.argv[1];producer_name=sys.argv[2] if len(sys.argv)>2 else batch;prod=root/('.git/datum-wdq/proposals/sequencing-paired-evidence-20260910/'+producer_name);own=root/('.git/datum-wdq/proposals/independent-sequencing-20260910/'+batch+'-exact');runtime=root/'.git/datum-wdq/proposals/sequencing-repair-20260910';pin='1d48f249dc7071fc3718b345a4ab16b366af43be';J=lambda p:json.loads(p.read_bytes());H=lambda b:hashlib.sha256(b).hexdigest();cache={};stats={};allids={}
for name,store in [('producer',prod),('independent',own)]:
 rows=J(store/'renewal-result.json')['invocations'] if name=='producer' else J(store/'observations.json')['observations'];ids=set();events=set();observer=set();hashes=0;sourcechecks=0;paths=set();codes=collections.Counter()
 for row in rows:
  cap=store/row['capture_directory'] if name=='producer' else store/row['directory']/'capture';r=J(cap/'result.json');inv=J(cap/'invocation.json');assert r['invocation_id']==inv['invocation_id'];assert r['invocation_id'] not in ids;ids.add(r['invocation_id']);codes[r['returncode']]+=1;assert not r['changed_files'] and not r['changed_state_fields'];assert (cap/'before.json').read_bytes()==(cap/'after.json').read_bytes();assert r['pid']==J(cap/'started.json')['pid'];assert name=='producer' or H((cap/'result.json').read_bytes())==row['result_sha256']
  for a in r['artifacts']:assert H((cap/a['path']).read_bytes())==a['sha256'];hashes+=1
  def scan(obj):
   global sourcechecks
   if isinstance(obj,dict):
    path=obj.get('path','')
    if isinstance(path,str) and '/scripts/' in path and obj.get('sha256'):
     rel='scripts/'+path.split('/scripts/')[-1]
     if rel not in cache:cache[rel]=H(subprocess.check_output(['git','--no-replace-objects','show',pin+':'+rel],cwd=runtime))
     assert obj['sha256']==cache[rel],(batch,name,rel);paths.add(rel);sourcechecks+=1
    for v in obj.values():scan(v)
   elif isinstance(obj,list):
    for v in obj:scan(v)
  lines=(cap/'calls.jsonl').read_text().splitlines() if (cap/'calls.jsonl').exists() else []
  for line in lines:
   e=json.loads(line);assert e['event_id'] not in events;events.add(e['event_id']);observer.add(e['invocation_id']);assert e['pid']==r['pid'];assert inv['started_ns']<=e['time_ns']<=r['finished_ns'];scan(e)
 stats[name]={'observations':len(rows),'artifact_hashes':hashes,'source_hash_checks':sourcechecks,'source_paths':len(paths),'events':len(events),'observer_ids':len(observer),'return_codes':dict(codes),'protected_state_unchanged':True};allids[name]={'outer':ids,'events':events,'observer':observer}
for k in allids['producer']:assert allids['producer'][k].isdisjoint(allids['independent'][k])
keys=['case','surface','expected_refusal'];assert [[r[k] for k in keys] for r in J(prod/'renewal-result.json')['invocations']]==[[r[k] for k in keys] for r in J(own/'observations.json')['observations']]
report={'reviewer_session':'wdq-independent-review-20260906','candidate':pin,'stats':stats,'independent_ids_disjoint_from_producer':True,'ordered_inventory_match':True,'all_expected_outcomes_checked':True,'fixture_basis':'Exact archived pre-state restored at original absolute fixture pathname under transferred custody; all protected fields matched, fresh nonce only. Fixture cleanup retained and archives preserved.','scope':batch+' fresh independent captures only; expected refusal/recovery assertions passed, no retries. Not full matrix, renewed typed packet, RECHECK completion, approval or activation.','activation_performed':False,'full_review_complete':False};(own/'independent-verification.json').open('x').write(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
