import json,subprocess
from pathlib import Path
from PIL import Image
campaigns=['dggmrpnm','r_pji_1k','skt9h5e4','a7fhaisg','efqab64f'];out={'qualification_pass':False,'runs':[],'limits':['Method pilots, not30-cycle qualification, numerical caps or continuous presentation proof. All four failures retained.','Window readbacks and later compositor captures can disagree; they are not synchronized. This proves a capture/observation limitation, not its precise cause or physical latency.','Thumb-scan scratch image was overwritten on each scan; durable named pre-action captures are used for independent grab-fraction checks.','Automatic focus return failed in both final seeded hosts; manual Main activation was cleanup only. No focus restoration/complete teardown qualification. Compound input CPU not normalized into fabricated semantic-action costs.']}
def ae(a,b):
 r=subprocess.run(['compare','-metric','AE',str(a),str(b),'null:'],capture_output=True,text=True);assert r.returncode in(0,1);return int(r.stderr.strip())
def thumb(path):
 im=Image.open(path).convert('RGB');ys=[y for y in range(100,720) if im.getpixel((954,y))==(178,184,195)];runs=[]
 for y in ys:
  if not runs or y!=runs[-1][-1]+1:runs.append([])
  runs[-1].append(y)
 runs=[r for r in runs if len(r)>=32];assert len(runs)==1
 return min(runs[0]),max(runs[0])+1
for idx,suffix in enumerate(campaigns,1):
 c=Path('/tmp/pm045-controls-campaign-'+suffix);declaration=json.loads((c/'result.json').read_text());assert declaration['display_restored']
 for run in declaration['runs']:
  p=Path(run['path']);r=json.loads((p/'result.json').read_text());row={'generation':idx,**run,'error':r.get('error'),'normal_exit_code':r.get('normal_exit_code'),'controls':r.get('controls',[]),'process_ledger':(p/'process.jsonl').read_text().splitlines()}
  if idx==5:
   assert r['normal_exit_code']==0 and not r.get('error');control=r['controls'][0];host=control['host'];actions={a['name']:a for a in control['actions']};assert len(actions)==17
   checks={}
   for fraction,prior in [(.25,'search-cleared'),(.75,'grab-0.25-release')]:
    name='grab-'+str(fraction);top,bottom=thumb(p/(host+'-'+prior+'.png'));actual=actions[name]['input'][4];expected=round(top+(bottom-top)*fraction);assert actual==expected,(host,name,actual,expected)
    checks[name]={'thumb_bounds':[top,bottom],'fraction':fraction,'grab_y':actual,'press_AE':ae(p/(host+'-'+prior+'.png'),p/(host+'-'+name+'.png')),'drag_AE':ae(p/(host+'-'+name+'.png'),p/(host+'-'+name+'-drag.png')),'release_AE':ae(p/(host+'-'+name+'-drag.png'),p/(host+'-'+name+'-release.png'))}
    assert checks[name]['press_AE']==0 and checks[name]['drag_AE']>0 and checks[name]['release_AE']==0
   checks['track_page_AE']=ae(p/(host+'-grab-0.75-release.png'),p/(host+'-track-page.png'));assert checks['track_page_AE']>0
   checks['cancel_AE']=ae(p/(host+'-cancel-before-motion.png'),p/(host+'-cancel-after-motion.png'));assert checks['cancel_AE']==0
   checks['capture_pairs']={name:ae(p/(host+'-'+name+'-window.png'),p/(host+'-'+name+'.png')) for name in actions}
   row['checks']=checks
  out['runs'].append(row)
Path('/tmp/pm045-controls-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
for row in out['runs']:print(row['generation'],row['role'],row['returncode'],row['error'],row.get('checks',{}))
