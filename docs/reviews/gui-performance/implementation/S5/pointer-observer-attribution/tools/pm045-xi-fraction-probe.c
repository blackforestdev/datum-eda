#include <X11/Xlib.h>
#include <X11/extensions/XInput2.h>
#include <stdio.h>
#include <unistd.h>
int main(void) {
 Display *d=XOpenDisplay(NULL); if(!d)return 2;
 int opcode,event,error,major=2,minor=0,id=0;
 if(!XQueryExtension(d,"XInputExtension",&opcode,&event,&error)||XIQueryVersion(d,&major,&minor)!=Success)return 3;
 Window w=XCreateSimpleWindow(d,DefaultRootWindow(d),50,50,300,300,0,0,0);
 XStoreName(d,w,"PM045 fractional input diagnostic");
 unsigned char bits[XIMaskLen(XI_Motion)]={0};XISetMask(bits,XI_Motion);
 XIEventMask mask={XIAllMasterDevices,sizeof(bits),bits};XISelectEvents(d,w,&mask,1);
 XMapWindow(d,w);XSync(d,False);usleep(200000);
 if(!XIGetClientPointer(d,w,&id))return 4;
 printf("{\"kind\":\"setup\",\"device\":%d,\"window\":%lu}\n",id,w);
 for(int i=0;i<12;i++) {
  double x=20.0+i*.25;int rc=XIWarpPointer(d,id,None,w,0,0,0,0,x,40.25);XSync(d,False);usleep(30000);
  printf("{\"kind\":\"request\",\"index\":%d,\"x\":%.4f,\"y\":40.25,\"rc\":%d}\n",i,x,rc);
  while(XPending(d)) {XEvent e;XNextEvent(d,&e);
   if(e.type==GenericEvent&&e.xcookie.extension==opcode&&XGetEventData(d,&e.xcookie)) {
    if(e.xcookie.evtype==XI_Motion) {XIDeviceEvent *v=e.xcookie.data;printf("{\"kind\":\"motion\",\"device\":%d,\"x\":%.4f,\"y\":%.4f}\n",v->deviceid,v->event_x,v->event_y);}
    XFreeEventData(d,&e.xcookie);
   }
  }
 }
 XDestroyWindow(d,w);XCloseDisplay(d);return 0;
}
