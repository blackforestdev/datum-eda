import ctypes as c,time,json,sys
from pathlib import Path
# Existing system XTest/Xlib ABI only; no application or dependency mutation.
ox,oy=map(int,sys.argv[1:3]);path=Path(sys.argv[3]);x=c.CDLL('libX11.so.6');t=c.CDLL('libXtst.so.6')
x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p;x.XFlush.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p]
t.XTestFakeMotionEvent.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_int,c.c_ulong];t.XTestFakeMotionEvent.restype=c.c_int
d=x.XOpenDisplay(None);assert d
rows=[];start=time.monotonic_ns();wall=time.time_ns()
try:
 for i in range(3600):
  deadline=start+round(i*1e9/120);remaining=(deadline-time.monotonic_ns())/1e9
  if remaining>0:time.sleep(remaining)
  edge=i//900;u=(i%900)/900
  lx,ly=[(300+400*u,150),(700,150+240*u),(700-400*u,390),(300,390-240*u)][edge]
  px,py=round(lx),round(ly);actual=time.monotonic_ns();assert t.XTestFakeMotionEvent(d,-1,ox+px,oy+py,0);x.XFlush(d)
  rows.append({'index':i,'scheduled_ns':deadline,'sent_ns':actual,'logical_position':[lx,ly],'physical_position':[px,py]})
 remaining=(start+30_000_000_000-time.monotonic_ns())/1e9
 if remaining>0:time.sleep(remaining)
finally:
 x.XCloseDisplay(d)
 path.write_text(json.dumps({'started_ns':start,'started_realtime_ns':wall,'finished_ns':time.monotonic_ns(),'scheduled_count':3600,'schedule':'30seconds120Hz single400x240physicalpixelrectangle,fourequal7.5secondedges;XTestintegerpositionsroundplannedcoordinates,duplicatesretained','rows':rows},indent=2)+'\n')
