import ctypes as c,time,json,sys
from pathlib import Path
button=int(sys.argv[1]);count=int(sys.argv[2]);rate=float(sys.argv[3]);path=Path(sys.argv[4])
x=c.CDLL('libX11.so.6');t=c.CDLL('libXtst.so.6');x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p;x.XFlush.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p]
t.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong];t.XTestFakeButtonEvent.restype=c.c_int
d=x.XOpenDisplay(None);assert d;rows=[];start=time.monotonic_ns();wall=time.time_ns()
try:
 for i in range(count):
  due=start+round(i*1e9/rate);remaining=(due-time.monotonic_ns())/1e9
  if remaining>0:time.sleep(remaining)
  sent=time.monotonic_ns();pressed=(i%2==0) if len(sys.argv)<6 else sys.argv[5]=='press';assert t.XTestFakeButtonEvent(d,button,int(pressed),0);x.XFlush(d);rows.append({'index':i,'scheduled_ns':due,'sent_ns':sent,'pressed':pressed})
 remaining=(start+round(count*1e9/rate)-time.monotonic_ns())/1e9
 if remaining>0:time.sleep(remaining)
finally:
 x.XCloseDisplay(d)
 path.write_text(json.dumps({'button':button,'count':count,'rate_hz':rate,'started_ns':start,'started_realtime_ns':wall,'finished_ns':time.monotonic_ns(),'rows':rows},indent=2)+'\n')
