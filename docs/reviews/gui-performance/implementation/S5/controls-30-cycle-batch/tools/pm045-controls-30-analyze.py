import json,hashlib
from pathlib import Path
from PIL import Image,ImageChops

def pixels(path):return Image.open(path).convert('RGB')
def ae(a,b):
 assert a.size==b.size
 return sum(pixel!=(0,0,0) for pixel in ImageChops.difference(a,b).getdata())
def thumb(path):
 im=pixels(path);ys=[y for y in range(100,720) if im.getpixel((954,y))==(178,184,195)];runs=[]
 for y in ys:
  if not runs or y!=runs[-1][-1]+1:runs.append([])
  runs[-1].append(y)
 runs=[r for r in runs if len(r)>=32];assert len(runs)==1
 return min(runs[0]),max(runs[0])+1

def analyze(path):
 p=Path(path);r=json.loads((p/'result.json').read_text());assert r['normal_exit_code']==0 and not r.get('error');c=r['controls'][0];host=c['host'];wid=c['window'];assert c['completed_cycles']==30 and c['automatic_focus_restored'];log=(p/'native.log').read_bytes();rows=[]
 for cycle in range(1,31):
  actions={a['name']:a for a in c['actions'] if a['cycle']==cycle};assert len(actions)==17
  records={'cycle':cycle,'motion_receipts':0,'grabs':[]}
  for name in ['grab-0.25-drag','grab-0.75-drag','cancel-after-motion']:
   a=actions[name];x,y=a['input'][3:5];text=log[a['start_log_offset']:a['end_log_offset']].decode();needle=f'window event WindowId({wid}) cursor moved {x:.2f},{y:.2f}'
   assert needle in text,(host,cycle,name,needle);records['motion_receipts']+=1
  for fraction,prior in [(.25,'search-cleared'),(.75,'grab-0.25-release')]:
   name='grab-'+str(fraction);before=p/actions[prior]['capture'];top,bottom=thumb(before);actual=actions[name]['input'][4];assert actual==round(top+(bottom-top)*fraction)
   press=ae(pixels(before),pixels(p/actions[name]['capture']));drag=ae(pixels(p/actions[name]['capture']),pixels(p/actions[name+'-drag']['capture']));release=ae(pixels(p/actions[name+'-drag']['capture']),pixels(p/actions[name+'-release']['capture']));assert press==0 and drag>0 and release==0
   records['grabs'].append({'fraction':fraction,'thumb':[top,bottom],'grab_y':actual,'press_AE':press,'drag_AE':drag,'release_AE':release})
  before=p/actions['grab-0.75-release']['capture'];top,bottom=thumb(before);page=actions['track-page']['input'][4];assert page<top or page>=bottom
  records['page_AE']=ae(pixels(before),pixels(p/actions['track-page']['capture']));assert records['page_AE']>0
  records['cancel_AE']=ae(pixels(p/f'{host}-{cycle}-cancel-before-motion.png'),pixels(p/actions['cancel-after-motion']['capture']));assert records['cancel_AE']==0
  probe=next(v for v in c['clip_probes'] if v['cycle']==cycle);segment=log[probe['begin_log_offset']:probe['end_log_offset']].decode()
  assert f'window event WindowId({wid}) mouse input Left Released' in segment
  records['probe_AE']=ae(pixels(p/actions['search-refocus']['capture']),pixels(p/f'{host}-{cycle}-clip-probe.png'));assert records['probe_AE']==0
  # First Global name focus has no announcement: check its complete image too,
  # even if the live harness used a notice mask before that notice existed.
  if host=='GLOBAL' and cycle==1:
   ref=Path('/tmp/pm045-controls-30-references')/host/(host+'-last-name.png')
   records['first_name_unmasked_AE']=ae(pixels(ref),pixels(p/actions['last-name']['capture']));assert records['first_name_unmasked_AE']==0
  rows.append(records)
 out={'host':host,'path':str(p),'cycles':30,'actions':len(c['actions']),'motion_receipts':sum(x['motion_receipts'] for x in rows),'cycles_verified':rows,'visual_checks':len(c['visual_checks']),'normal_exit':r['normal_exit_code'],'focus_restored':c['automatic_focus_restored'],'limits':'One X11/1x profile; exact endpoints and received motion, not physical timing, native numerical telemetry or full resource/cap/other backend-scale qualification. Project header probes require separately recorded actual hidden-control overlap supplement.'}
 (p/'cycle-analysis.json').write_text(json.dumps(out,indent=2)+'\n');return out
if __name__=='__main__':
 import sys
 for p in sys.argv[1:]:
  r=analyze(p);print({k:v for k,v in r.items() if k!='cycles_verified'})
