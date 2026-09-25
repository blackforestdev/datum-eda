import json,struct,sys,subprocess,collections,hashlib
from pathlib import Path
p=Path(sys.argv[1]);data=(p/'cpu-samples.bin').read_bytes();offset=0;windows=[]
from functools import lru_cache
@lru_cache(maxsize=None)
def load_segments(path):
 b=Path(path).read_bytes();assert b[:6]==b'\x7fELF\x02\x01',path
 phoff=struct.unpack_from('<Q',b,32)[0];size,n=struct.unpack_from('<HH',b,54)
 return [struct.unpack_from('<IIQQQQQQ',b,phoff+i*size) for i in range(n)]
for bound in ['minimum','maximum']:
 h=struct.unpack_from('<11Q',data,offset);assert h[0]==0x504d303435435055 and h[1]==1;offset+=88
 samples=[struct.unpack_from('<QQq',data,offset+24*i) for i in range(h[3])];offset+=24*h[3]
 maps=[]
 for line in (p/(bound+'-maps.txt')).read_text().splitlines():
  parts=line.split(maxsplit=5);a,b=parts[0].split('-');maps.append((int(a,16),int(b,16),int(parts[2],16),parts[5] if len(parts)==6 else '',parts[1]))
 raw_counts=collections.Counter(x[0] for x in samples);modules=collections.Counter();addresses={};unknown=[];identities={}
 for pc,n in raw_counts.items():
  m=next((m for m in maps if m[0]<=pc<m[1]),None)
  if not m:unknown.append({'pc':pc,'count':n,'reason':'not mapped'});continue
  modules[m[3]]+=n
  if not m[3].startswith('/') or not Path(m[3]).exists():unknown.append({'pc':pc,'count':n,'reason':'non-file mapping','mapping':m});continue
  seg=next((s for s in load_segments(m[3]) if s[0]==1 and (s[2]&~4095)==m[2] and s[1]&1),None)
  if seg is None:unknown.append({'pc':pc,'count':n,'reason':'executable ELF segment unmatched','mapping':m});continue
  address=pc-(m[0]-(seg[3]&~4095));addresses.setdefault(m[3],[]).append((address,n,pc))
  if m[3] not in identities:identities[m[3]]=hashlib.sha256(Path(m[3]).read_bytes()).hexdigest()
 resolved=[]
 for module,entries in addresses.items():
  lines=subprocess.check_output(['addr2line','-f','-C','-e',module,*[hex(a) for a,n,pc in entries]],text=True).splitlines();assert len(lines)==2*len(entries)
  for i,(a,n,pc) in enumerate(entries):resolved.append({'module':module,'object_address':hex(a),'runtime_pc':hex(pc),'samples':n,'function_hint':lines[2*i],'function_range_verified':False,'location':lines[2*i+1]})
 window={'bound':bound,'samples':h[3],'dropped':h[4],'cpu_ns':h[6]-h[5],'begin_monotonic_ns':h[7],'end_monotonic_ns':h[8],'period_ns':h[9],'tid':h[10],'overruns':sum(x[2] for x in samples),'max_overrun':max((x[2] for x in samples),default=0),'observed_sample_cpu_ratio':h[3]*h[9]/(h[6]-h[5]),'expiration_cpu_ratio':(h[3]+sum(x[2] for x in samples))*h[9]/(h[6]-h[5]),'module_counts':modules,'resolved':sorted(resolved,key=lambda r:-r['samples']),'unknown':unknown,'module_sha256':identities,'limit':'Main thread program counters only, no stacks; samples after interrupted syscalls can reflect return sites rather than kernel cost. Overruns never multiplied into hotspot counts. Function labels are unverified addr2line hints; stripped-library nearest symbols may lie outside the actual function range. Example libc0x9a72e is beyond sem_trywait0x9a6d0+0x2c and must not be attributed to sem_trywait. Native overhead and other-thread coverage remain unqualified.'};windows.append(window)
 print(bound,'N',window['samples'],'CPUms',window['cpu_ns']/1e6,'overruns',window['overruns'],'ratio',window['observed_sample_cpu_ratio']);print(modules)
 for x in window['resolved'][:12]:print(x['samples'],x['function_hint'],x['module'])
assert offset==len(data)
(p/'cpu-symbolized.json').write_text(json.dumps({'windows':windows},indent=2)+'\n')
