import copy,json,tempfile
from pathlib import Path
from pm045_kernel_ledger import reconcile
source=Path('/tmp/pm045-kernel-fd-classified-cs1mi03_')
out=Path(tempfile.mkdtemp(prefix='pm045-kernel-ledger-controls-'));print(out)
results={}
for mode in ['0','1','2']:
 result=reconcile(source/(mode+'.ledger.jsonl'))
 assert result['descriptor_continuity_complete'],result['failures']
 assert len(result['clients'])=={'0':4,'1':1,'2':1}[mode]
 results[mode]=result
rows=[json.loads(s) for s in (source/'0.ledger.jsonl').read_text().splitlines()]
for kind in ['missing_final','missing_birth','reset','missing_end']:
 edited=copy.deepcopy(rows)
 if kind=='missing_final':
  index=next(i for i,r in enumerate(edited) if r['kind']=='drm' and r['phase']=='retire_before');edited.pop(index)
 elif kind=='missing_birth':
  index=next(i for i,r in enumerate(edited) if r['kind']=='drm' and r['phase']=='birth_after');edited.pop(index)
 elif kind=='missing_end':edited.pop()
 else:
  # Inject a nonzero birth counter followed by real zero duplicate/retirement.
  row=next(r for r in edited if r['kind']=='drm' and r['phase']=='birth_after')
  raw=bytes.fromhex(row['raw_hex']).decode().replace('drm-engine-render:\t0 ns','drm-engine-render:\t100 ns');row['raw_hex']=raw.encode().hex()
 p=out/(kind+'.jsonl');p.write_text(''.join(json.dumps(r)+'\n' for r in edited))
 result=reconcile(p);assert not result['descriptor_continuity_complete'],kind
 assert result['engine_delta_ns'] is None,kind
 results[kind]=result
(out/'result.json').write_text(json.dumps(results,indent=2)+'\n')
print('3real fixtures and4missing/reset negatives passed')
