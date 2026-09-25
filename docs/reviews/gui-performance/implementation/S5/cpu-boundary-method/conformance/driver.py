"""Isolated Linux CPU accounting conformance; no GUI performance acceptance."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time
import uuid

FIXTURE = r'''
import json, os, resource, threading, time

def burn(seconds):
    start=time.thread_time()
    while time.thread_time()-start < seconds:
        pass

def identity():
    s=open('/proc/self/stat').read().rsplit(')',1)[1].split()
    return {'pid':os.getpid(),'start_ticks':int(s[19])}

parent=identity()
children=[]
for seconds in (0.025,0.065):
    pid=os.fork()
    if pid==0:
        burn(seconds)
        os._exit(0)
    children.append(pid)
workers=[threading.Thread(target=burn,args=(0.015,)) for _ in range(2)]
for thread in workers:thread.start()
for thread in workers:thread.join()
exits=[]
for pid in children:
    got,status,usage=os.wait4(pid,0)
    exits.append({'pid':got,'status':status,'user_seconds':usage.ru_utime,'system_seconds':usage.ru_stime})
usage=resource.getrusage(resource.RUSAGE_SELF)
print(json.dumps({'parent':parent,'parent_pre_exit':{'user_seconds':usage.ru_utime,'system_seconds':usage.ru_stime},'children_exit':exits,'process_cpu_ns':time.process_time_ns()}),flush=True)
'''

def counters(group):
    return {k:int(v) for k,v in (line.split() for line in (group/'cpu.stat').read_text().splitlines())}

def main():
    out=Path(tempfile.mkdtemp(prefix='pm045-cgroup-conformance-'))
    print(out,flush=True)
    parent=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'
    group=parent/('datum-pm045-conformance-'+uuid.uuid4().hex)
    result={'purpose':'CPU observer conformance only; not GUI or PM045 row qualification','kernel':platform.uname()._asdict(),'observer_pid':os.getpid(),'clock_ticks_per_second':os.sysconf('SC_CLK_TCK'),'group':str(group),'cases':[]}
    (out/'fixture.py').write_text(FIXTURE)
    group.mkdir()
    child=None
    try:
        for attempt in range(3):
            assert not (group/'cgroup.procs').read_text().strip()
            before=counters(group)
            def enter():
                (group/'cgroup.procs').write_text(str(os.getpid()))
            child=subprocess.Popen([sys.executable,str(out/'fixture.py')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,preexec_fn=enter)
            started=time.monotonic_ns()
            # wait4 includes the final exit receipt; do not poll/reap via Popen.
            pid,status,exit_usage=os.wait4(child.pid,0)
            child.returncode=os.waitstatus_to_exitcode(status)
            stdout=child.stdout.read();stderr=child.stderr.read()
            ended=time.monotonic_ns()
            after=counters(group)
            assert child.returncode==0,stderr
            reported=json.loads(stdout)
            assert not (group/'cgroup.procs').read_text().strip()
            time.sleep(0.05)
            settled=counters(group)
            elapsed_cpu_us=after['usage_usec']-before['usage_usec']
            wait_cpu_us=round((exit_usage.ru_utime+exit_usage.ru_stime)*1e6)
            sampled_parent_us=round(sum(reported['parent_pre_exit'].values())*1e6)
            children_us=round(sum(c['user_seconds']+c['system_seconds'] for c in reported['children_exit'])*1e6)
            case={'attempt':attempt+1,'root_pid':pid,'window_monotonic_ns':[started,ended],'before':before,'after':after,'settled_after_empty_50ms':settled,'fixture':reported,'root_wait4':{'user_seconds':exit_usage.ru_utime,'system_seconds':exit_usage.ru_stime},'group_cpu_us':elapsed_cpu_us,'wait4_cpu_us':wait_cpu_us,'parent_pre_exit_us':sampled_parent_us,'child_exit_us':children_us,'absolute_group_wait4_delta_us':abs(elapsed_cpu_us-wait_cpu_us),'negative_live_only_endpoint_us':0,'negative_dropped_child_gap_us':elapsed_cpu_us-sampled_parent_us}
            # Kernel process accounting and cgroup accounting differ in update/
            # rounding. The conformance tolerance is 5ms, not a product budget.
            assert abs(elapsed_cpu_us-wait_cpu_us)<=5000,case
            assert children_us>=90000,case
            assert elapsed_cpu_us-sampled_parent_us>=85000,case
            assert settled['usage_usec']==after['usage_usec'],case
            assert elapsed_cpu_us>100000,case
            result['cases'].append(case)
            (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        result['result']='All3 controlled process/thread/short-lived-child cases pass; live-only and omitted-child negatives fail conservation.'
        result['limits']=['Cgroup hierarchy covers only members; externally delegated cooperating services require separate admission/accounting.','This does not observe GPU client lifetime or validate CPU action/readiness boundaries.','Conformance5ms tolerance is not GUI measurement uncertainty or budget relaxation; quantify action-window agreement and observer overhead separately.','No GUI performance or independent replay acceptance.']
    except BaseException as error:
        result['error']=repr(error)
        raise
    finally:
        if child is not None and child.returncode is None:
            child.kill();child.wait()
        members=(group/'cgroup.procs').read_text().strip()
        result['remaining_members']=members
        if not members:
            group.rmdir();result['cleanup']='Owned empty cgroup removed'
        result['source_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(result['result'],flush=True)

if __name__=='__main__':main()
