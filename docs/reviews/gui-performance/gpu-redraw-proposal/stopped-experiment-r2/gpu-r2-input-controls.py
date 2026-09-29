"""Offline observer-consumer controls; no input injection or timing campaign."""
from copy import deepcopy
from gpu_r2_validate import pointer_input, rejected

context = {'final_selection':'None','final_focus':'Editor(PaneId(0))','final_focused_pane':0}
start = 1_000_000_000
schedule = {'started_ns':start,'scheduled_count':3600,'finished_ns':start+30_000_100_000,'rows':[]}
state = {'cursor':[300,150],'native_cursor':[300,150],'camera_center_nm':[0,0],
         'camera_zoom':1,'pan_active':False,'device_epoch':7,'truncated':False}
def record(position, received):
    after = dict(state,cursor=position,native_cursor=position)
    return {'sequence':0,'received_ns':received,'completed_ns':received+1,
            'position':position,'button':None,'route':'authoring_hover','after':after}
records = [record([300,150],start-10)]
previous = [300,150]
for i in range(3600):
    edge,u = i//900,(i%900)/900
    logical = [(300+400*u,150),(700,150+240*u),(700-400*u,390),(300,390-240*u)][edge]
    physical = [round(v) for v in logical]
    scheduled = start+round(i*1e9/120)
    schedule['rows'].append({'index':i,'scheduled_ns':scheduled,'sent_ns':scheduled+100_000,
                             'physical_position':physical,'logical_position':list(logical)})
    if physical != previous:
        records.append(record(physical,scheduled+1))
        previous = physical
for i,r in enumerate(records): r['sequence'] = i
receipt = {'complete':True,'overflow':False,'monotonic_origin_ns':0,
           'records':records,'final_state':deepcopy(records[-1]['after']),**context}
end = start+35_000_000_000
assert pointer_input(schedule,receipt,end,context,8_333_334)['received'] == 1280
for field,value in [('complete',False),('overflow',True)]:
    bad = dict(receipt,**{field:value})
    rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
for field,value in [('route','unhandled'),('completed_ns',None),('position',[1,2])]:
    bad = deepcopy(receipt);bad['records'][1][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
for field,value in [('cursor',[1,2]),('device_epoch',8),('pan_active',True),('truncated',True)]:
    bad = deepcopy(receipt);bad['records'][1]['after'][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
bad = deepcopy(receipt);bad['records'].pop(10)
rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
bad_schedule = deepcopy(schedule);bad_schedule['rows'][20]['logical_position'][0]+=1
rejected(lambda: pointer_input(bad_schedule,receipt,end,context,8_333_334))
print('PASS:3600 scheduled /1280 changed /2320 repeated positions reconciled; incomplete, overflow, missing route/completion, altered cursor/path, epoch, pan, truncation and lost-record controls rejected. No native input sent.')

for field,value in [('camera_zoom',2),('device_epoch',8),('native_cursor',[1,2]),('pan_active',True),('hover_utf8','different')]:
    bad = deepcopy(receipt);bad['final_state'][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
for field,value in [('final_selection','AuthoredObject("wrong")'),('final_focus','Terminal'),('final_focused_pane',3)]:
    bad = deepcopy(receipt);bad[field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context,8_333_334))
print('PASS: final camera/epoch/native cursor/pan/hover and selection/focus/pane changes rejected.')

late=deepcopy(schedule);late['rows'][100]['sent_ns']+=10_000_000
rejected(lambda:pointer_input(late,receipt,end,context,8_333_334))
long=deepcopy(schedule);long['finished_ns']+=10_000_000
rejected(lambda:pointer_input(long,receipt,end,context,8_333_334))
early=deepcopy(schedule);early['finished_ns']=early['started_ns']+29_999_999_999
rejected(lambda:pointer_input(early,receipt,end,context,8_333_334))
print('PASS: actual producer deadline, early termination and prolonged duration negatives rejected.')
