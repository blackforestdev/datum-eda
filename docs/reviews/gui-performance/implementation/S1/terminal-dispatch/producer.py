import os,sys,time,json
from pathlib import Path
case=Path(sys.argv[1]);deadline=time.monotonic()+30
while not (case/'go').exists():
 if time.monotonic()>deadline:raise SystemExit(2)
 time.sleep(.01)
count=0
for i in range(24):
 data=(f'PM045-{i:02d} '+ 'x'*4087).encode();os.write(1,data);count+=len(data);time.sleep(.02)
data=b'y'*(128*1024);offset=0
while offset<len(data):offset+=os.write(1,data[offset:])
count+=len(data)
(case/'sent.json').write_text(json.dumps({'bytes':count,'event_log':os.environ['DATUM_TOOL_SESSION_EVENT_LOG']}))
time.sleep(20)
