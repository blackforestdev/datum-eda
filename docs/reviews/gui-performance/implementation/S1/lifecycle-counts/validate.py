import json
from pathlib import Path
report=[]
paths=[Path('/tmp/pm045-counts-host-proof')/h/'native.log' for h in ['GLOBAL','PROJECT','NEW']]
paths += [Path('/tmp/pm045-counts-final-device-proof')/(h+'.log') for h in ['GLOBAL','oom']]
for path in paths:
 rows=[json.loads(line.split('native_surface_lifecycle ',1)[1]) for line in path.read_text().splitlines() if 'native_surface_lifecycle ' in line]
 assert rows, str(path)+' has no lifecycle receipts'
 last={}
 for r in rows:
  key=(r['queue_epoch'],r['host']);prev=last.get(key)
  assert r['acquired']==r['presented']+r['discarded']+r['active'],r
  assert r['active'] in (0,1) and r['acquire_attempts']>=r['acquired'],r
  assert r['configure_attempts']>=r['configuration_generation'],r
  assert r['last_present_receipt']<=r['present_receipts_issued'],r
  assert r['gpu_completed_through_receipt']<=r['present_receipts_issued'],r
  if r['reason']=='present':assert r['active']==0 and r['presented']>0 and r['configuration_generation']>0,r
  if prev:
   assert prev['window']==r['window'],r
   for field in ['acquired','presented','discarded','acquire_attempts','configure_attempts','configuration_generation','last_present_receipt','present_receipts_issued','gpu_completed_through_receipt']:
    assert r[field]>=prev[field],(prev,r)
  last[key]=r
 report.append({'path':str(path),'records':len(rows),'owners':len(last),'queue_epochs':sorted(set(r['queue_epoch'] for r in rows)),'final_owner_records':list(last.values()),'passed':True})
assert len(report[3]['queue_epochs'])==2,'device loss did not create distinct observed queue epochs'
assert len(report[4]['queue_epochs'])==3,'automatic and F5 OOM retries were not separately identified'
Path('/tmp/pm045-lifecycle-counts-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print('Lifecycle reconciliation passed:',sum(r['records'] for r in report),'records in',len(report),'native runs')
