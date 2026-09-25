import json,sys,collections
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads((p/'result.json').read_text());rows=json.loads((p/'callback-cpu.json').read_text());assert len(rows)<=65536
rows.sort(key=lambda a:a[1]);assert all(a[1]<=a[2] and a[3]<=a[4] for a in rows)
overlaps=[(a,b) for a,b in zip(rows,rows[1:]) if a[2]>b[1]];assert not overlaps
names={1:'wheel',2:'modifiers',3:'other_window',4:'about_to_wait'};out={'rows':len(rows),'overlaps':len(overlaps),'windows':[],'limits':['Instrumented candidate, not normal-mode acceptance. Main-thread callback CPU includes callees invoked synchronously from these callbacks; other threads and event-loop work outside callbacks are not included.','Scope clock/read/record overhead remains in family CPU; record push is outside scoped duration. No overhead subtraction or synthetic hotspot inference.','Only callbacks fully within the family CPU interval are summed; crossing boundaries are reported separately.']}
for trial in r['clamp_trials']:
 start,end=trial['started_ns'],trial['active_finished_ns'];inside=[a for a in rows if start<=a[1] and a[2]<=end];cross=[a for a in rows if a[1]<end and a[2]>start and a not in inside];totals=collections.defaultdict(lambda:{'calls':0,'cpu_ms':0,'wall_ms':0})
 for tag,m0,m1,c0,c1 in inside:totals[names[tag]]['calls']+=1;totals[names[tag]]['cpu_ms']+=(c1-c0)/1e6;totals[names[tag]]['wall_ms']+=(m1-m0)/1e6
 callback_cpu=sum(x['cpu_ms'] for x in totals.values());a={'bound':trial['bound'],'family_cpu_ms':trial['active_cpu_ms'],'callback_cpu_ms':callback_cpu,'callback_fraction_of_family':callback_cpu/trial['active_cpu_ms'],'by_callback':dict(totals),'crossing':cross,'endpoint_AE':trial['endpoint_AE'],'pilot_endpoint_comparisons':trial['pilot_endpoint_comparisons']};out['windows'].append(a);print(a)
(p/'callback-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
