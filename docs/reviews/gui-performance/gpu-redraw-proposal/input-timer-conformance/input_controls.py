"""Offline observer-consumer controls; no input injection or timing campaign."""
from copy import deepcopy
from validate import pointer_input, rejected

context = {'final_selection':'None','final_focus':'Editor(PaneId(0))','final_focused_pane':0}
start = 1_000_000_000
schedule = {'started_ns':start,'scheduled_count':3600,'rows':[]}
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
    schedule['rows'].append({'index':i,'scheduled_ns':scheduled,
                             'physical_position':physical,'logical_position':list(logical)})
    if physical != previous:
        records.append(record(physical,scheduled+1))
        previous = physical
for i,r in enumerate(records): r['sequence'] = i
receipt = {'complete':True,'overflow':False,'monotonic_origin_ns':0,
           'records':records,'final_state':deepcopy(records[-1]['after']),**context}
end = start+35_000_000_000
assert pointer_input(schedule,receipt,end,context)['received'] == 1280
for field,value in [('complete',False),('overflow',True)]:
    bad = dict(receipt,**{field:value})
    rejected(lambda: pointer_input(schedule,bad,end,context))
for field,value in [('route','unhandled'),('completed_ns',None),('position',[1,2])]:
    bad = deepcopy(receipt);bad['records'][1][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context))
for field,value in [('cursor',[1,2]),('device_epoch',8),('pan_active',True),('truncated',True)]:
    bad = deepcopy(receipt);bad['records'][1]['after'][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context))
bad = deepcopy(receipt);bad['records'].pop(10)
rejected(lambda: pointer_input(schedule,bad,end,context))
bad_schedule = deepcopy(schedule);bad_schedule['rows'][20]['logical_position'][0]+=1
rejected(lambda: pointer_input(bad_schedule,receipt,end,context))
print('PASS:3600 scheduled /1280 changed /2320 repeated positions reconciled; incomplete, overflow, missing route/completion, altered cursor/path, epoch, pan, truncation and lost-record controls rejected. No native input sent.')

for field,value in [('camera_zoom',2),('device_epoch',8),('native_cursor',[1,2]),('pan_active',True),('hover_utf8','different')]:
    bad = deepcopy(receipt);bad['final_state'][field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context))
for field,value in [('final_selection','AuthoredObject("wrong")'),('final_focus','Terminal'),('final_focused_pane',3)]:
    bad = deepcopy(receipt);bad[field]=value
    rejected(lambda: pointer_input(schedule,bad,end,context))
print('PASS: final camera/epoch/native cursor/pan/hover and selection/focus/pane changes rejected.')
