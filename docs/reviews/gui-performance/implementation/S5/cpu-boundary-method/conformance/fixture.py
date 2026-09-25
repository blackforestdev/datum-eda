
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
