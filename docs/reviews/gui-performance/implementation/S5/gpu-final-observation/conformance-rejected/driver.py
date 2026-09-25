import copy,json,os,tempfile
from pathlib import Path
from pm045_client_ledger import snapshot,clients,interval
out=Path(tempfile.mkdtemp(prefix='pm045-drm-conformance-'));print(out,flush=True)
fd=os.open('/dev/dri/renderD128',os.O_RDWR|os.O_CLOEXEC);duplicate=None
try:
 one=snapshot([os.getpid()]);duplicate=os.dup(fd);two=snapshot([os.getpid()])
 assert not one['errors'] and not two['errors']
 assert len(clients(one))==len(clients(two))==1
 assert next(iter(clients(two).values()))['descriptor_count']==2
 os.close(duplicate);duplicate=None
 retained=snapshot([os.getpid()]);assert interval([one,two,retained])['observed_continuity_complete']
 os.close(fd);fd=None
 closed=snapshot([os.getpid()]);lost=interval([one,two,retained,closed]);assert not lost['observed_continuity_complete'] and lost['engine_delta_ns'] is None
 # Known nonzero accounting fixture proves duplicates cannot double the total.
 start=copy.deepcopy(two);end=copy.deepcopy(two)
 for row in start['descriptors']:
  row['engines_ns']['drm-engine-render']=100
 for row in end['descriptors']:
  row['engines_ns']['drm-engine-render']=400
 counted=interval([start,end]);assert counted['engine_delta_ns']['drm-engine-render']==300
 reset=interval([end,start]);assert not reset['observed_continuity_complete']
 reappeared=interval([one,closed,one]);assert not reappeared['observed_continuity_complete']
 (out/'result.json').write_text(json.dumps({'scope':'Real fd/client identity and deliberate accounting negatives; no GPU work, native performance or full lifetime qualification','raw':[one,two,retained,closed],'lost_final':lost,'synthetic_nonzero_dedup':counted,'synthetic_reset':reset,'reappeared_epoch':reappeared,'result':'pass'},indent=2)+'\n')
 print('Real duplicate/client continuity plus missing-final, reset and epoch negatives pass',flush=True)
finally:
 if duplicate is not None:os.close(duplicate)
 if fd is not None:os.close(fd)
