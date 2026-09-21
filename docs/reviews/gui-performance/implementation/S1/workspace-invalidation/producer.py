import sys,time,json
from pathlib import Path
case=Path(sys.argv[1]); deadline=time.monotonic()+30
while not (case/'go').exists():
 if time.monotonic()>deadline: raise SystemExit(2)
 time.sleep(.01)
payload=b''
for i in range(24):
 row=f'PM045 background output {i:02d}\r\n'.encode();sys.stdout.buffer.write(row);sys.stdout.buffer.flush();payload+=row;time.sleep(.06)
(case/'sent.json').write_text(json.dumps({'bursts':24,'bytes':len(payload)}))
time.sleep(30)
