"""Read-only DRM fdinfo snapshots and conservative observed-client continuity.

Polling cannot prove that no client was born and closed between observations.
A complete ACC-03 pass additionally needs production lifecycle coverage.
"""
import os
from pathlib import Path
import time


def snapshot(pids):
    result={'begin_ns':time.monotonic_ns(),'processes':[],'descriptors':[],'errors':[]}
    for pid in pids:
        try:
            stat=Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
            identity={'pid':int(pid),'start_ticks':int(stat[19])}
            result['processes'].append(identity)
            paths=os.scandir(f'/proc/{pid}/fdinfo')
        except (FileNotFoundError,PermissionError) as error:
            result['errors'].append({'pid':str(pid),'error':repr(error)});continue
        for entry in paths:
            path=Path(entry.path)
            try:
                raw=path.read_text()
                fields=dict(line.split(':',1) for line in raw.splitlines() if ':' in line)
                fields={k:v.strip() for k,v in fields.items()}
                if 'drm-client-id' not in fields:continue
                device=os.stat(f'/proc/{pid}/fd/{path.name}').st_rdev
                if not fields.get('drm-pdev') or not fields.get('drm-driver'):
                    raise ValueError('DRM PCI/driver identity unavailable')
                engines={}
                for key,value in fields.items():
                    if key.startswith('drm-engine-'):
                        count,unit=value.split()
                        if unit!='ns' or int(count)<0:raise ValueError('invalid engine counter')
                        engines[key]=int(count)
                if not engines:raise ValueError('missing engine counters')
                result['descriptors'].append({'process':identity,'fd':int(path.name),'device_node':[os.major(device),os.minor(device)],'device':[fields['drm-driver'],fields['drm-pdev']],'client_id':fields['drm-client-id'],'engines_ns':engines,'raw':raw})
            except (FileNotFoundError,PermissionError,ValueError) as error:
                # FD churn has no invented zero or extrapolated final counter.
                result['errors'].append({'pid':str(pid),'fd':path.name,'error':repr(error)})
        paths.close()
    result['end_ns']=time.monotonic_ns()
    return result


def clients(sample):
    unique={}
    for row in sample['descriptors']:
        key=tuple(row['device'])+(row['client_id'],)
        item=unique.setdefault(key,{'engines_ns':{},'descriptor_count':0})
        item['descriptor_count']+=1
        for name,value in row['engines_ns'].items():
            item['engines_ns'][name]=max(item['engines_ns'].get(name,0),value)
    return unique


def interval(samples):
    """Fail continuity when any observed client/counter loses its final value."""
    if len(samples)<2:raise ValueError('two endpoints required')
    first=clients(samples[0]);last={k:dict(v['engines_ns']) for k,v in first.items()}
    baseline={k:dict(v['engines_ns']) for k,v in first.items()}
    active=set(first);epochs={k:1 for k in first};failures=[]
    for sample in samples:
        failures.extend(sample['errors'])
        current=clients(sample)
        for key in active-current.keys():
            failures.append({'client':key,'epoch':epochs[key],'error':'client disappeared without final lifecycle receipt'})
        for key,item in current.items():
            now=item['engines_ns']
            if key not in active and key in epochs:
                epochs[key]+=1
                failures.append({'client':key,'epoch':epochs[key],'error':'client identity reappeared; epoch cannot be bridged'})
            elif key not in epochs:
                epochs[key]=1
                baseline[key]={name:0 for name in now}
            previous=last.get(key,{name:0 for name in now})
            if now.keys()!=previous.keys() or any(now[name]<previous.get(name,0) for name in now):
                failures.append({'client':key,'error':'engine set changed or cumulative counter reset'})
            last[key]=dict(now)
        active=set(current)
    totals={}
    for key,values in last.items():
        for engine,value in values.items():
            totals[engine]=totals.get(engine,0)+value-baseline[key].get(engine,0)
    return {'observed_continuity_complete':not failures,'failures':failures,'engine_delta_ns':totals if not failures else None,'client_epochs':[{'device':list(k[:2]),'client_id':k[2],'epoch':v} for k,v in epochs.items()],'coverage':'Observed clients only. Unobserved short lifetimes require production lifecycle hooks; not complete ACC-03.'}
